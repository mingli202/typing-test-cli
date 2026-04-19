package data_provider

import (
	"bytes"
	"encoding/json"
	"log"
	"os"
	"tui/backend/models"
)

type DataProvider struct {
	repository []models.Data
}

func New() DataProvider {
	filepath := "../assets/english.json"

	quotes_bytes, err := os.ReadFile(filepath)

	if err != nil {
		log.Printf("Could not load from %v: %v\n", filepath, err)
		return default_provider()
	}

	decoded := json.NewDecoder(bytes.NewReader(quotes_bytes))

	var repository []models.Data

	for decoded.More() {
		var d models.Data
		err := decoded.Decode(&d)

		if err != nil {
			log.Printf("Could no decode into Data: %v", err)
		} else {
			repository = append(repository, d)
		}
	}

	return DataProvider{repository}
}

func default_provider() DataProvider {
	return DataProvider{repository: []models.Data{}}
}
