package main

import (
	"context"
	"net/http"

	gitlab "github.com/xanzy/go-gitlab"
	"go.uber.org/zap"
)

const gitlabProviderName = "gitlab.com"

type Gitlab struct {
	Secret            string `json:"api-key",toml:"api-key"`
	Host              string `json:"host",toml:"host"`
	SkipStarred       bool
	SkipOwned         bool
	SkipOrganisations bool
}

func (g Gitlab) Valid() error {
	if g.Secret == "" {
		return ErrMissingGitlabApiKey
	}

	return nil
}

func (g Gitlab) Provider() string {
	if g.Host != "" {
		return g.Host
	}

	return gitlabProviderName
}

func (g Gitlab) newClient() *gitlab.Client {
	return gitlab.NewOAuthClient(&http.Client{}, g.Secret)
}

func FetchGitlabRepos(ctx context.Context, cfg Gitlab, logger *zap.Logger) <-chan Repo {
	client := cfg.newClient()

	if cfg.Host != "" {
		client.SetBaseURL(cfg.Host)
	}

	get := func(ctx context.Context, page int) ([]Repo, response, error) {
		options := gitlab.ListProjectsOptions{
			Starred:    gitlab.Bool(!cfg.SkipStarred),
			Membership: gitlab.Bool(!cfg.SkipOrganisations),
			Owned:      gitlab.Bool(!cfg.SkipOwned),
			ListOptions: gitlab.ListOptions{
				Page:    page,
				PerPage: 100,
			},
		}

		got, resp, err := client.Projects.ListProjects(&options)
		if err != nil {
			return nil, response{}, err
		}

		var repos []Repo

		for _, proj := range got {
			repos = append(repos, Repo{
				Provider: cfg.Provider(),
				Name:     proj.NameWithNamespace,
				URL:      proj.SSHURLToRepo,
			})
		}

		return repos, gitlabResponse(resp), nil
	}

	return merge(getPaginated(ctx, get, logger))
}

type response struct {
	NextPage int
	LastPage int
	getRate  func() interface{}
}

func gitlabResponse(r *gitlab.Response) response {
	return response{
		NextPage: r.NextPage,
		LastPage: r.TotalPages,
	}
}

func (r response) GetRate() interface{} {
	if r.getRate != nil {
		return r.getRate()
	}

	return nil
}
