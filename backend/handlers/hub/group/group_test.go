package group_test

import (
	"slices"
	"testing"
	"tui/backend/handlers/hub/group"
	"tui/backend/handlers/hub/user"
	"tui/backend/services/data_provider"
)

var dataProvider, _ = data_provider.NewDataProvider()

func newGroup() *group.Group {
	data, _ := dataProvider.NewData()
	group := group.NewGroup("asdf", data)

	return &group
}

func TestNewGroup(t *testing.T) {
	gr := newGroup()

	users := gr.GetUsersSnapshot()

	if users == nil {
		t.Fatal("group.users should not be nil")
	}

	if len(users) != 0 {
		t.Fatal("There should be no users")
	}
}

func TestGroupAddUser(t *testing.T) {
	u := user.NewUser(nil)

	gr := newGroup()

	gr.AddUser(&u)

	users := gr.GetUsersSnapshot()

	if len(users) != 1 || !slices.Contains(users, &u) {
		t.Fatal("It should have added the added user")
	}

	if *u.GroupId != gr.Id() {
		t.Fatal("user group should have been set to the group's id")
	}

	gr.AddUser(&u)

	users = gr.GetUsersSnapshot()

	if len(users) != 1 {
		t.Fatal("Duplicate user tf")
	}
}

func TestRemoverUser(t *testing.T) {
	u1 := user.NewUser(nil)
	u2 := user.NewUser(nil)

	gr := newGroup()

	gr.AddUser(&u1)
	gr.AddUser(&u2)

	isEmpty := gr.RemoveUser(&u2)

	if isEmpty {
		t.Fatal("Group is still not empty")
	}

	users := gr.GetUsersSnapshot()

	if len(users) != 1 {
		t.Fatal("User did not get removed")
	}

	if slices.Contains(users, &u2) {
		t.Fatal("Group removed the wrong user vro")
	}

	if !slices.Contains(users, &u1) {
		t.Fatal("Where tf is the first user")
	}

	isEmpty = gr.RemoveUser(&u1)

	if !isEmpty {
		t.Fatal("There should be no more users in the group")
	}

}
