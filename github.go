package main

import (
	"context"
	"strings"
	"sync"
	"time"

	"github.com/google/go-github/v27/github"
	"go.uber.org/zap"
	"golang.org/x/oauth2"
)

const githubProviderName = "github.com"

type Github struct {
	APIKey            string
	SkipOwned         bool
	SkipStarred       bool
	SkipOrganisations bool
	SkipCollaborator  bool
}

func (g Github) Valid() error {
	if g.APIKey == "" {
		return ErrMissingGithubApiKey
	}

	return nil
}

func (g Github) affiliations() string {
	var affiliations []string
	if !g.SkipOwned {
		affiliations = append(affiliations, "owner")
	}
	if !g.SkipOrganisations {
		affiliations = append(affiliations, "organization_member")
	}
	if !g.SkipCollaborator {
		affiliations = append(affiliations, "collaborator")
	}

	if len(affiliations) == 0 {
		return ""
	}

	return strings.Join(affiliations, ",")
}

func FetchGithubRepos(ctx context.Context, cfg Github, logger *zap.Logger) <-chan Repo {
	token := oauth2.StaticTokenSource(&oauth2.Token{AccessToken: cfg.APIKey})
	client := github.NewClient(oauth2.NewClient(ctx, token))

	var repoChans []<-chan []Repo

	affiliations := cfg.affiliations()
	if affiliations != "" {
		ch := getPaginated(ctx, getOwnedAndOrgs(ctx, client, affiliations, logger), logger)
		repoChans = append(repoChans, ch)
	}

	if !cfg.SkipStarred {
		ch := getPaginated(ctx, getStarredRepos(ctx, client, logger), logger)
		repoChans = append(repoChans, ch)
	}

	return merge(repoChans...)
}

func merge(cs ...<-chan []Repo) <-chan Repo {
	var wg sync.WaitGroup
	out := make(chan Repo)

	output := func(c <-chan []Repo) {
		for items := range c {
			for _, item := range items {
				out <- item
			}
		}
		wg.Done()
	}
	wg.Add(len(cs))
	for _, c := range cs {
		go output(c)
	}

	go func() {
		wg.Wait()
		close(out)
	}()
	return out
}

type getPageFunc func(ctx context.Context, page int) ([]Repo, response, error)

func getPaginated(ctx context.Context, getPage getPageFunc, logger *zap.Logger) <-chan []Repo {
	start := time.Now()
	page := 0
	numRepos := 0
	repos := make(chan []Repo)

	go func() {
		defer close(repos)

		for {
			pageStart := time.Now()
			got, response, err := getPage(ctx, page)
			if err != nil {
				logger.Warn("Unable to retrieve the page",
					zap.Error(err),
					zap.Int("page", page))
				break
			}

			select {
			case <-ctx.Done():
				break
			case repos <- got:
			}

			numRepos += len(got)

			logger.Debug("Fetched page",
				zap.Int("page", page),
				zap.Duration("duration", time.Since(pageStart)),
				zap.Any("rate-limit", response.GetRate()),
				zap.Any("next-page", response.NextPage))

			if ctx.Err() != nil || page >= response.LastPage {
				break
			}
			page++
		}

		logger.Debug("Fetched all pages",
			zap.Int("num-pages", page),
			zap.Int("num-repos", numRepos),
			zap.Duration("duration", time.Since(start)))
	}()

	return repos
}

func getStarredRepos(ctx context.Context, client *github.Client, logger *zap.Logger) getPageFunc {
	return func(ctx context.Context, page int) ([]Repo, response, error) {
		options := &github.ActivityListStarredOptions{
			ListOptions: github.ListOptions{
				Page:    page,
				PerPage: 100,
			},
		}
		got, resp, err := client.Activity.ListStarred(ctx, "", options)
		if err != nil {
			return nil, response{}, err
		}

		repos := make([]Repo, 0, len(got))
		for _, r := range got {
			repos = append(repos, Repo{
				Provider: githubProviderName,
				Name:     r.GetRepository().GetFullName(),
				URL:      r.GetRepository().GetGitURL(),
			})
		}

		return repos, githubResponse(resp), nil
	}
}

func getOwnedAndOrgs(ctx context.Context, client *github.Client, affiliations string, logger *zap.Logger) getPageFunc {
	return func(ctx context.Context, page int) ([]Repo, response, error) {
		options := &github.RepositoryListOptions{
			Affiliation: affiliations,
			ListOptions: github.ListOptions{
				Page:    page,
				PerPage: 100,
			},
		}
		got, resp, err := client.Repositories.List(ctx, "", options)
		if err != nil {
			return nil, response{}, err
		}

		repos := make([]Repo, 0, len(got))
		for _, r := range got {
			repos = append(repos, Repo{
				Provider: githubProviderName,
				Name:     r.GetFullName(),
				URL:      r.GetGitURL(),
			})
		}

		return repos, githubResponse(resp), nil
	}
}

func githubResponse(r *github.Response) response {
	return response{
		NextPage: r.NextPage,
		LastPage: r.LastPage,
		getRate:  func() interface{} { return r.Rate },
	}
}
