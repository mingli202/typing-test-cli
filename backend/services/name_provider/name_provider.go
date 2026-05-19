package name_provider

import (
	"encoding/json"
	"fmt"
	"log"
	"math/rand/v2"
	"tui/backend/assets"
	"tui/backend/models"
)

type NameProvider struct {
	nouns      []string
	adjectives []string
}

// Makes a new name provider
func NewNameProvider() (NameProvider, error) {
	var repository models.NamesRepo

	if err := json.Unmarshal(assets.Names, &repository); err != nil {
		log.Printf("Could no decode into Names: %v", err)
		return defaultNameProvider(), nil
	}

	return NameProvider{
		nouns:      repository.Nouns,
		adjectives: repository.Adjectives,
	}, nil
}

// Gets a new name
func (provider *NameProvider) NewName() (string, error) {
	nounsLen := len(provider.nouns)
	adLen := len(provider.adjectives)

	if nounsLen == 0 || adLen == 0 {
		return "Handsome Guest", fmt.Errorf("Oops, no nouns or adjectives")
	}

	randomNounIndex := rand.IntN(nounsLen)
	randomAdIndex := rand.IntN(adLen)

	return provider.adjectives[randomAdIndex] + " " + provider.nouns[randomNounIndex], nil
}

// Returns whether the nouns or adjectives repo is empty or only 1
func (provider *NameProvider) LessThan2NounsOrAdjectives() bool {
	return len(provider.nouns) < 2 || len(provider.adjectives) < 2
}

// Returns a default provider in cases of errors
func defaultNameProvider() NameProvider {
	return NameProvider{
		nouns:      []string{"Guest"},
		adjectives: []string{"Handsome"},
	}
}
