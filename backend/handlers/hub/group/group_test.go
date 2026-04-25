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
		t.Error("group.users should not be nil")
	}

	if len(users) != 0 {
		t.Error("There should be no users")
	}
}

func TestGroupAddUser(t *testing.T) {
	u := user.NewUser(nil)

	gr := newGroup()

	gr.AddUser(&u)

	users := gr.GetUsersSnapshot()

	if len(users) != 1 || !slices.Contains(users, &u) {
		t.Error("It should have added the added user")
	}

	if *u.GroupId != gr.Id() {
		t.Error("user group should have been set to the group's id")
	}
}
