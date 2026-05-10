package models

import "testing"

func TestLobbyInfoToMsg(t *testing.T) {
	lobbyInfo := LobbyInfo{
		LobbyId: "asdf",
		Data: Data{
			Text:   "qwer",
			Source: "zxcv",
		},
	}

	msg, err := lobbyInfo.ToMsg()

	if err != nil {
		t.Fatal(err)
	}

	expected := `LobbyInfo {"lobby_id":"asdf","data":{"text":"qwer","source":"zxcv"}}`

	if msg != expected {
		t.Fatalf("expected %v, got %v", expected, msg)
	}
}

func TestEndGameToMsg(t *testing.T) {
	players := make(map[string]PlayerInfo)

	players["asdf"] = PlayerInfo{
		IsLeader:        false,
		Wpm:             10.1,
		ProgressPercent: 100,
	}

	endGame := EndGame{
		FinalPlayersInfo: PlayersInfoSnapshot{
			LobbyId: "asdf",
			Version: 1,
			Players: players,
		},
	}

	msg, err := endGame.ToMsg()

	if err != nil {
		t.Fatal(err)
	}

	expected := `EndGame {"lobby_id":"asdf","version":1,"players":{"asdf":{"is_leader":false,"wpm":10.1,"progress_percent":100}}}`

	if msg != expected {
		t.Fatalf("expected %v, got %v", expected, msg)
	}
}
