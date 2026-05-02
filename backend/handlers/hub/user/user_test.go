package user_test

import (
	"testing"
	"tui/backend/handlers/hub/user"
)

func TestNewUser(t *testing.T) {
	user1 := user.NewUser(nil)

	if user1.GroupId != nil {
		t.Fatalf("User should not belong in any group for now")
	}
}

// Issue: sending on a cleaned-up user should not panic due to channel close races.
// If this test fails with a panic, the implementation is still vulnerable.
func TestSendMsgAfterCleanupDoesNotPanic(t *testing.T) {
	u := user.NewUser(nil)
	ch := make(chan []byte)
	u.SetCh(ch)
	u.Cleanup()

	defer func() {
		if recovered := recover(); recovered != nil {
			t.Fatalf("SendMsg panicked after Cleanup: %v", recovered)
		}
	}()

	u.SendMsg("hello")
}
