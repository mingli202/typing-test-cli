package models

import (
	"encoding/json"
	"fmt"
)

type LobbyInfo struct {
	LobbyId string `json:"lobby_id"`
	Data    Data   `json:"data"`
}

type PlayerInfo struct {
	IsLeader bool `json:"is_leader"`
	// The current wpm of the user, calculated by the tui client
	Wpm float64 `json:"wpm"`
	// At which character the user is at
	ProgressPercent uint8 `json:"progress_percent"`
}

type PlayerInfoSnapshot struct {
	LobbyId string                `json:"lobby_id"`
	Version uint64                `json:"version"`
	Players map[string]PlayerInfo `json:"players"`
}

type NewGame struct {
	Data        Data               `json:"data"`
	PlayersInfo PlayerInfoSnapshot `json:"players_info"`
}

type ErrorMessage struct {
	Msg string
}

func (err ErrorMessage) ToMsg() string {
	return fmt.Sprintf("Error %s", err.Msg)
}

func (lobbyInfo LobbyInfo) ToMsg() (string, error) {
	lobbyInfoStr, err := json.Marshal(lobbyInfo)

	if err != nil {
		return "", err
	}

	return "LobbyInfo " + string(lobbyInfoStr), nil
}

func (newGame NewGame) ToMsg() (string, error) {
	p, err := json.Marshal(newGame)

	if err != nil {
		return "", err
	}

	return "NewGame " + string(p), nil
}
