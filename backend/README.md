# Typing test tui backend

The backend that manage the games written in Go

# Schema specification

The websocket server communicates with the client via strings that consists of a command and optional payloads after, separated by spaces. Parenthesis indicate positional payloads (Name: type).

## client -> server schema (see hub.go handleMessage)
```
- NewGroup
- JoinGroup (GroupId: string)
- LeaveGroup
- UpdateStats (Wpm: float64; Wpm > 0) (Progress: uint8; 0 < Progress < 100)
- StartGame
```

## server -> client schema (see models.go)
some type definition:
```
Data = { text: string, source: string }
PlayerInfo = { is_leader: bool, wpm: float64, progress_percent: uint8 }
PlayerInfoSnapshot = {
  lobby_id: string,
  version: uint64,
  players: { [userId: string]: PlayerInfo }
}
```

all the possible responses:
```
- LobbyInfo (LobbyInfo)
- NewGame (NewGame: { data: Data, player_info: PlayerInfoSnapshot })
- EndGame (FinalPlayersInfo: PlayerInfoSnapshot)
- Error (Msg: string)
- UserId (UserId: string)
- LeaveGroup (DidSucceed: bool)
- PlayersInfo (PlayerInfoSnapshot)
- Countdown (Coundown: int)
```
