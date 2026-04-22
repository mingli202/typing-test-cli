package hub

import (
	"encoding/json"
	"log"
	"math/rand/v2"
	"net/http"
	"sync"
	"tui/backend/models"

	"github.com/gorilla/websocket"
)

var upgrader = websocket.Upgrader{}

type Hub struct {
	mu     sync.Mutex
	groups map[string]map[*websocket.Conn]bool
}

// Handles websocket message
// Maps the message function to its own function (the client "calls" a function on the hub)
func (hub *Hub) HandleMessage(p []byte, conn *websocket.Conn) ([]byte, error) {
	readMessage := models.ReadMessage{}

	err := json.Unmarshal(p, &readMessage)

	if err != nil {
		return []byte{}, err
	}

	switch readMessage.Type {
	case "NewGroup":
		id := hub.NewGroup(conn)
		return json.Marshal(models.NewGroupResponse{Id: id})
	case "Join":
		joinGroup := models.JoinGroup{}
		err = json.Unmarshal([]byte(readMessage.Payload), &joinGroup)
		if err != nil {
			return []byte{}, err
		}

		success := hub.Join(joinGroup.Id, conn)
		return json.Marshal(models.JoinResponse{Success: success})

	case "Exit":
		exitGroup := models.ExitGroup{}
		err = json.Unmarshal([]byte(readMessage.Payload), &exitGroup)

		if err != nil {
			return []byte{}, err
		}

		success := hub.Exit(exitGroup.Id, conn)

		return json.Marshal(models.JoinResponse{Success: success})
	default:
		return []byte{}, TypeNotFoundError{}
	}
}

// Makes a new group with the given conn
// Returns the newly created group id
func (hub *Hub) NewGroup(conn *websocket.Conn) string {
	hub.mu.Lock()
	defer hub.mu.Unlock()

	id := newGroupId()
	_, ok := hub.groups[id]

	for ok {
		id = newGroupId()
		_, ok = hub.groups[id]
	}

	hub.groups[id] = make(map[*websocket.Conn]bool)
	hub.groups[id][conn] = true

	return id
}

// Appends the given conn to the group with the given id
// Return whether the conn was added to the group
func (hub *Hub) Join(id string, conn *websocket.Conn) bool {
	hub.mu.Lock()
	defer hub.mu.Unlock()

	group, ok := hub.groups[id]

	if ok {
		group[conn] = true
	}

	return ok
}

// Removes the given conn from the group with the given id
// Returns whether the remove was successful or not
func (hub *Hub) Exit(id string, conn *websocket.Conn) bool {
	hub.mu.Lock()
	defer hub.mu.Unlock()

	group, ok := hub.groups[id]

	if ok {
		delete(group, conn)
	}

	return ok
}

func (hub *Hub) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	conn, err := upgrader.Upgrade(w, r, nil)

	if err != nil {
		log.Println(err)
		return
	}

	defer func() {
		conn.Close()
	}()

	for {
		messageType, p, err := conn.ReadMessage()

		if err != nil {
			log.Println(err)
			return
		}

		if messageType != websocket.TextMessage {
			continue
		}

		returnMessage, err := hub.HandleMessage(p, conn)

		if err != nil {
			errBytes, errErr := json.Marshal(err)

			if errErr == nil {
				err = conn.WriteMessage(websocket.TextMessage, errBytes)
			}

		} else {
			err = conn.WriteMessage(websocket.TextMessage, []byte(returnMessage))
		}

		if err != nil {
			log.Println(err)
		}
	}
}

func Handler() http.Handler {
	hub := Hub{}

	return &hub
}

func newGroupId() string {
	s := ""

	for i := 0; i < 6; i += 1 {
		randomChar := rand.IntN('z'-'a') + 'a'
		s = s + string(rune(randomChar))
	}

	return s
}
