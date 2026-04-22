package models

type ReadMessage struct {
	Type    string
	Payload string
}

type JoinGroup struct {
	Id string
}

type ExitGroup struct {
	Id string
}

type WriteMessage struct {
	Type    string
	Payload string
}

type NewGroupResponse struct {
	Id string
}

type JoinResponse struct {
	Success bool
}

type ExitResponse struct {
	Success bool
}
