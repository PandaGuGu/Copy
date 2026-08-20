package statemachine

import "testing"

func TestVideoTransitions(t *testing.T) {
	cases := []struct {
		from, to string
		ok       bool
	}{
		{"draft", "processing", true},
		{"draft", "takedown", true},
		{"draft", "published", false},
		{"processing", "pending_review", true},
		{"processing", "published", true},
		{"processing", "failed", true},
		{"pending_review", "published", true},
		{"pending_review", "rejected", true},
		{"failed", "processing", true},   // media replace
		{"rejected", "processing", true}, // media replace
		{"published", "takedown", true},  // copyright
		{"takedown", "published", true},  // restore
		{"failed", "published", false},   // the old bug: failed → published must be illegal
		{"draft", "rejected", false},
		{"pending_review", "failed", false},
	}
	for _, c := range cases {
		if got := Video.Can(c.from, c.to); got != c.ok {
			t.Errorf("video %q -> %q = %v, want %v", c.from, c.to, got, c.ok)
		}
	}
}

func TestTicketTransitions(t *testing.T) {
	cases := []struct {
		from, to string
		ok       bool
	}{
		{"open", "assigning", true},
		{"open", "assigned", true},
		{"open", "resolved", false}, // tightening: no direct jump from open
		{"open", "closed", false},
		{"assigning", "assigned", true},
		{"assigned", "processing", true},
		{"processing", "resolved", true},
		{"resolved", "closed", true},
		{"resolved", "reopened", true},
		{"closed", "reopened", true},
		{"reopened", "processing", true},
		{"reopened", "resolved", true},
	}
	for _, c := range cases {
		if got := Ticket.Can(c.from, c.to); got != c.ok {
			t.Errorf("ticket %q -> %q = %v, want %v", c.from, c.to, got, c.ok)
		}
	}
}

func TestIllegalTransitionError(t *testing.T) {
	err := Video.Transition("failed", "published", "test")
	if err == nil {
		t.Fatal("expected error for failed -> published")
	}
	if _, ok := err.(*IllegalTransitionError); !ok {
		t.Fatalf("expected IllegalTransitionError, got %T", err)
	}
	// Legal transition should fire OnChange.
	called := false
	m := New("probe").Allow("a", "b")
	m.OnChange = func(domain, from, to, actor string) {
		called = true
		if domain != "probe" || from != "a" || to != "b" || actor != "x" {
			t.Errorf("bad hook args: %s %s %s %s", domain, from, to, actor)
		}
	}
	if err := m.Transition("a", "b", "x"); err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if !called {
		t.Error("OnChange not called")
	}
}
