package main

import (
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"strings"

	"github.com/BurntSushi/toml"
)

func parseTomlConfig(body string) (Config, error) {
	var cfg Config

	if _, err := toml.Decode(body, &cfg); err != nil {
		return Config{}, err
	}

	if err := cfg.Valid(); err != nil {
		return Config{}, err
	}

	return cfg, nil
}

func parseJSONConfig(r io.Reader) (Config, error) {
	var cfg Config

	if err := json.NewDecoder(r).Decode(&cfg); err != nil {
		return Config{}, err
	}

	if err := cfg.Valid(); err != nil {
		return Config{}, err
	}

	return cfg, nil
}

var ErrMissingRoot = errors.New("No root provided")
var ErrMissingGithubApiKey = errors.New("Missing GitHub API key")
var ErrMissingGitlabApiKey = errors.New("Missing GitLab API key")

type Validatable interface {
	Valid() error
}

type manyErrors struct {
	Errors []error
}

func (m manyErrors) Error() string {
	switch len(m.Errors) {
	case 0:
		return "No errors occurred"
	case 1:
		return m.Errors[0].Error()
	default:
		sb := strings.Builder{}
		sb.WriteString(fmt.Sprintf("%d errors occurred: ", len(m.Errors)))
		for i, err := range m.Errors {
			if i == len(m.Errors)-1 {
				sb.WriteString(" and ")
			} else if i != 0 {
				sb.WriteString(",")
			}
			sb.WriteString(err.Error())
		}
		return sb.String()
	}
}

type Config struct {
	General General `json:"general",toml:"general"`
	Gitlab  *Gitlab `json:"gitlab,omitempty",toml:"gitlab"`
	Github  *Github `json:"github,omitempty",toml:"github"`
}

func (c Config) Valid() error {
	m := manyErrors{}
	check := func(v Validatable) {
		if v != nil {
			if err := v.Valid(); err != nil {
				m.Errors = append(m.Errors, err)
			}
		}
	}

	check(c.General)
	check(c.Gitlab)
	check(c.Github)

	if len(m.Errors) != 0 {
		return m
	}

	return nil
}

type General struct {
	Root      string   `json:"root",toml:"root"`
	Blacklist []string `json:"blacklist",toml:"blacklist,omitempty"`
}

func (g General) Valid() error {
	if g.Root == "" {
		return ErrMissingRoot
	}

	return nil
}

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

type Gitlab struct {
	Secret string `json:"api-key",toml:"api-key"`
	Host   string `json:"host",toml:"host"`
}

func (g Gitlab) Valid() error {
	if g.Secret == "" {
		return ErrMissingGitlabApiKey
	}

	return nil
}
