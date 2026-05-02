package models

import "encoding/json"

type LobbyInfo struct {
	LobbyId string
	Data    Data
}

type PlayerInfo struct {
	IsLeader bool
	// The current wpm of the user, calculated by the tui client
	Wpm float64
	// At which character the user is at
	ProgressPercent uint8
}

type NewGame struct {
	Data    Data
	Players map[string]PlayerInfo
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
