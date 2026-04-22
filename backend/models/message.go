package models

type ReadMessage struct {
	Function string
	Payload  string
}

type JoinGroup struct {
	Id string
}

type ExitGroup struct {
	Id string
}

type WriteMessage struct {
	Type string
}
