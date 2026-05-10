# Typing test tui backend

The backend that manage the games written in Go

# Schema specification

The websocket server communicates with the client via strings that consists of a command and optional payloads after, separated by spaces. Parenthesis indicate positional payloads (Name: type).

## client -> server schema (see hub.go handleMessage)
```
# Makes a new group and joins it. If the user is already in a group, send an error
- NewGroup

# Joins the group with the given id. If the user is already in a group or the group doesn't exist, send an error 
- JoinGroup (GroupId: string)

# Leaves the group the user is in. If the user is not in a group, send an an error
- LeaveGroup

# Updates the stats of the user. If the game is not running or the user is not part of the game, send an error
- UpdateStats (Wpm: float64; Wpm > 0) (Progress: uint8; 0 < Progress < 100)

# Starts the game, only the leader can start the game, otherwise send an error
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
- Countdown (Countdown: int)
- StartGame
```
