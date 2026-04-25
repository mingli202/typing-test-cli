package hub

import (
	"errors"
	"fmt"
	"strconv"
	"sync"
	"testing"
	"tui/backend/services/data_provider"
)

var dataProvider, _ = data_provider.NewDataProvider()

func TestNewGroupId(t *testing.T) {
	groupId := newGroupId()
	anotherGroupId := newGroupId()

	if groupId == "" || anotherGroupId == "" {
		t.Fatalf("One of the group id was empty: groupId (%v) anotherGroupId (%v)", groupId, anotherGroupId)
	}

	if groupId == anotherGroupId {
		t.Fatalf("Got same random id twice in a row")
	}
	fmt.Printf("groupId: %+v\n", groupId)
	fmt.Printf("anotherGroupId: %+v\n", anotherGroupId)
}

func TestNewUser(t *testing.T) {
	hub := newHub(dataProvider)

	user1 := newUser(nil)

	if len(hub.groups) != 0 {
		t.Fatalf("How could a group been made?")
	}

	if user1.group != nil {
		t.Fatalf("User should not belong in any group for now")
	}
}

func TestHandleNewGroup(t *testing.T) {
	hub := newHub(dataProvider)

	user := newUser(nil)

	groupId := hub.handleNewGroup(&user)

	if len(hub.groups) != 1 {
		t.Fatalf("Should have added a group")
	}

	group, ok := hub.groups[groupId]

	if !ok {
		t.Fatalf("Why has the group not been added")
	}

	if len(group.users) != 1 {
		t.Fatalf("User not been added")
	}

	groupId = hub.handleNewGroup(&user)

	if len(hub.groups) != 1 {
		t.Fatalf("Should have added a new group but old group is gone")
	}

	group2 := hub.groups[groupId]

	if group2.id == group.id {
		t.Fatalf("Impossible same group id")
	}

	if len(group2.users) != 1 {
		t.Fatalf("User should have been added to the new group")
	}
}

func TestJoin(t *testing.T) {
	hub := newHub(dataProvider)

	user1 := newUser(nil)
	user2 := newUser(nil)

	groupId1 := hub.handleNewGroup(&user1)
	group1, ok := hub.getGroup(groupId1)

	if !ok {
		t.Fatal("Where is the group??")
	}

	// user joins itself
	ok = hub.handleJoin(groupId1, &user1)

	if !ok {
		t.Fatal("Technically the user can in fact join its own group")
	}

	// user 2 joins valid group
	ok = hub.handleJoin(groupId1, &user2)

	if !ok {
		t.Fatalf("Join unsuccessful")
	}

	// user 1 joins invalid group
	ok = hub.handleJoin("ramdom groupId", &user1)

	if ok {
		t.Fatalf("Group id not found, impossible")
	}

	if len(group1.users) != 2 {
		t.Fatalf("Group should still have 2 after invalid join")
	}

	// user1 makes another group
	groupId2 := hub.handleNewGroup(&user1)
	group2 := hub.groups[groupId2]

	if len(group1.users) != 1 {
		t.Fatalf("User 1 should have left the first group")
	}

	hub.handleJoin(groupId2, &user2)

	if len(hub.groups) != 1 {
		t.Fatal("Old group should have been removed")
	}

	if len(group2.users) != 2 {
		t.Fatalf("User 2 should have joined the second group")
	}

	if len(group1.users) != 0 {
		t.Fatalf("Group1 should no longer have any users")
	}

	_, ok = hub.groups[group1.id]

	if ok {
		t.Fatalf("Group1 should have been deleted")
	}
}

func TestHandleMessageNewGroup(t *testing.T) {
	hub := newHub(dataProvider)

	user := newUser(nil)

	msg := "NewGroup"

	res, err := hub.handleMessage([]byte(msg), &user)

	if err != nil {
		t.Fatal(err)
	}

	id := res

	group, ok := hub.groups[id]

	if !ok {
		t.Fatal("Id returned an invalid group")
	}

	_, ok = group.users[user.id]

	if !ok {
		t.Fatal("Could not find user in returned group")
	}
}

func TestHandleMessageJoinGroup(t *testing.T) {
	hub := newHub(dataProvider)

	user1 := newUser(nil)

	groupId := hub.handleNewGroup(&user1)

	user2 := newUser(nil)

	msg := "JoinGroup " + groupId

	res, err := hub.handleMessage([]byte(msg), &user2)

	if err != nil {
		t.Fatal(err)
	}

	success, err := strconv.ParseBool(res)

	if err != nil {
		t.Fatal(err)
	}

	if success == false {
		t.Fatal("Unsuccessful join")
	}

	group := hub.groups[groupId]

	if len(group.users) != 2 {
		t.Fatal("Group does not have 2 users")
	}
}

func TestHandleMessageLeaveGroup(t *testing.T) {
	hub := newHub(dataProvider)

	user1 := newUser(nil)

	groupId := hub.handleNewGroup(&user1)

	user2 := newUser(nil)

	hub.handleJoin(groupId, &user2)

	msg := "LeaveGroup"

	res, err := hub.handleMessage([]byte(msg), &user2)

	if err != nil {
		t.Fatal(err)
	}

	success, err := strconv.ParseBool(res)

	if err != nil {
		t.Fatal(err)
	}

	if success == false {
		t.Fatal("Unsuccessful leave")
	}

	group := hub.groups[groupId]

	if len(group.users) != 1 {
		t.Fatal("Group does not have 1 user")
	}

	res, _ = hub.handleMessage([]byte(msg), &user1)
	success, _ = strconv.ParseBool(res)
	if success == false {
		t.Fatal("Unsuccessful leave")
	}

	if len(group.users) != 0 {
		t.Fatal("Group does not have 0 user")
	}

	if _, ok := hub.groups[groupId]; ok {
		t.Fatal("Group did not get removed")
	}
}

func TestRemoveUser(t *testing.T) {
	hub := newHub(dataProvider)

	user1 := newUser(nil)
	user2 := newUser(nil)
	user3 := newUser(nil)

	groupId1 := hub.handleNewGroup(&user1)
	hub.handleJoin(groupId1, &user2)

	hub.removeUser(&user1)

	group1, ok := hub.getGroup(groupId1)

	if !ok {
		t.Fatal("Where tf is the group??")
	}

	if len(group1.users) != 1 {
		t.Fatal("User1 did not get removed from its group")
	}

	hub.removeUser(&user2)

	if _, ok = hub.getGroup(groupId1); ok {
		t.Fatal("Group1 should have been removed")
	}

	hub.removeUser(&user3) // don't crash pls
}

func TestHandleLeaveWithoutGroup(t *testing.T) {
	hub := newHub(dataProvider)
	user := newUser(nil)

	if hub.handleLeave(&user) {
		t.Fatal("leave should fail for user that is not in a group")
	}
}

func TestLeaveHelperMatchesHandleLeaveBehavior(t *testing.T) {
	hub := newHub(dataProvider)

	user1 := newUser(nil)
	user2 := newUser(nil)

	groupId := hub.handleNewGroup(&user1)
	hub.handleJoin(groupId, &user2)

	hub.mu.Lock()
	success := hub.leave(&user2)
	hub.mu.Unlock()

	if !success {
		t.Fatal("leave helper should remove a user that belongs to a group")
	}

	group, ok := hub.getGroup(groupId)
	if !ok {
		t.Fatal("group should still exist because user1 is still in it")
	}

	if len(group.users) != 1 {
		t.Fatal("group should keep a single user after helper leave")
	}
}

func TestHandleMessageJoinGroupBadFormat(t *testing.T) {
	hub := newHub(dataProvider)
	user := newUser(nil)

	cases := []string{
		"JoinGroup",
		"JoinGroup too many args",
	}

	for _, msg := range cases {
		_, err := hub.handleMessage([]byte(msg), &user)
		if err == nil {
			t.Fatalf("expected error for %q", msg)
		}

		var errMsg ErrorMessage
		if !errors.As(err, &errMsg) {
			t.Fatalf("expected ErrorMessage for %q, got %T", msg, err)
		}
	}
}

func TestHandleMessageUnknownFunction(t *testing.T) {
	hub := newHub(dataProvider)
	user := newUser(nil)

	_, err := hub.handleMessage([]byte("DoesNotExist"), &user)
	if err == nil {
		t.Fatal("expected FunctionNotFoundError")
	}

	var fnErr FunctionNotFoundError
	if !errors.As(err, &fnErr) {
		t.Fatalf("expected FunctionNotFoundError, got %T", err)
	}
}

func TestHandleMessageEmptyInput(t *testing.T) {
	hub := newHub(dataProvider)
	user := newUser(nil)

	_, err := hub.handleMessage([]byte(""), &user)
	if err == nil {
		t.Fatal("expected error for empty message")
	}

	var fnErr FunctionNotFoundError
	if !errors.As(err, &fnErr) {
		t.Fatalf("expected FunctionNotFoundError, got %T", err)
	}

	if fnErr.Fn != "" {
		t.Fatalf("expected empty function name, got %q", fnErr.Fn)
	}
}

func TestConcurrentJoinStability(t *testing.T) {
	hub := newHub(dataProvider)

	anchor1 := newUser(nil)
	groupId1 := hub.handleNewGroup(&anchor1)

	anchor2 := newUser(nil)
	groupId2 := hub.handleNewGroup(&anchor2)

	const nUsers = 24
	const nIterations = 200

	users := make([]*User, 0, nUsers)
	for i := 0; i < nUsers; i++ {
		user := newUser(nil)
		users = append(users, &user)
	}

	var wg sync.WaitGroup

	for _, user := range users {
		wg.Add(1)
		go func(u *User) {
			defer wg.Done()

			for i := 0; i < nIterations; i++ {
				targetGroupID := groupId1
				if i%2 == 1 {
					targetGroupID = groupId2
				}

				ok := hub.handleJoin(targetGroupID, u)
				if !ok {
					t.Errorf("join should succeed for valid group %s", targetGroupID)
					return
				}
			}
		}(user)
	}

	wg.Wait()

	group1, ok := hub.getGroup(groupId1)
	if !ok {
		t.Fatal("group1 should still exist")
	}

	group2, ok := hub.getGroup(groupId2)
	if !ok {
		t.Fatal("group2 should still exist")
	}

	totalUsers := len(group1.users) + len(group2.users)
	if totalUsers != nUsers+2 {
		t.Fatalf("expected %d users across both groups (including anchors), got %d", nUsers+2, totalUsers)
	}
}
