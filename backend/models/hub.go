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

type PlayerInfoSnapshot struct {
	Version uint64
	Players map[string]PlayerInfo
}

type NewGame struct {
	Data        Data
	PlayersInfo PlayerInfoSnapshot
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
