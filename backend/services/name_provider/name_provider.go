package nameprovider

import (
	"encoding/json"
	"tui/backend/assets"
	"tui/backend/models"
)

type NameProvider struct {
	repository models.NamesRepo
}

// Makes a new name provider
func NewNameProvider() (NameProvider, error) {
	var repository models.NamesRepo

	if err := json.Unmarshal(assets.Names, &repository); err != nil {
	}

	return NameProvider{
		repository: repository,
	}, nil
}

// Returns a default provider in cases of errors
func defaultNameProvider() NameProvider {
	return NameProvider{
		repository: models.NamesRepo{
			Names:      []string{"Guest"},
			Adjectives: []string{"Handsome"},
		},
	}
}
