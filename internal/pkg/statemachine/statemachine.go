// Package statemachine provides a lightweight, dependency-free state machine
// for business-domain status transitions (ADR-018).
//
// Design goals:
//   - Centralize allowed transitions per domain instead of scattered if-checks.
//   - Validate BEFORE writing to DB: callers use Transition() then persist.
//   - Optional OnChange hook for audit logging (ADR-006 integration).
//   - Zero third-party deps; ~150 lines. If a domain later grows complex
//     enough for an event-driven FSM (looplab/fsm), the transition tables
//     here map 1:1 onto it.
package statemachine

import (
	"fmt"
)

// Machine defines the allowed transitions for one business domain.
// Transitions is a map of current-state -> allowed next-states.
type Machine struct {
	// Name identifies the domain (e.g. "video", "ticket").
	Name string
	// Transitions holds the state graph.
	Transitions map[string][]string
	// OnChange, when set, is invoked after a legal transition is validated.
	// Signature: OnChange(domain, from, to, actor string).
	OnChange func(domain, from, to, actor string)
}

// New creates an empty machine for a domain.
func New(name string) *Machine {
	return &Machine{Name: name, Transitions: map[string][]string{}}
}

// Allow registers a legal transition from -> to.
func (m *Machine) Allow(from, to string) *Machine {
	m.Transitions[from] = append(m.Transitions[from], to)
	return m
}

// AllowMany registers all transitions from one current state.
func (m *Machine) AllowMany(from string, to ...string) *Machine {
	for _, t := range to {
		m.Allow(from, t)
	}
	return m
}

// Can reports whether the transition from -> to is legal.
func (m *Machine) Can(from, to string) bool {
	for _, t := range m.Transitions[from] {
		if t == to {
			return true
		}
	}
	return false
}

// Transition validates from -> to. On success it fires OnChange (if set)
// and returns nil. On an illegal transition it returns a descriptive error.
// Callers should perform the actual DB write AFTER this returns nil.
func (m *Machine) Transition(from, to, actor string) error {
	if !m.Can(from, to) {
		return &IllegalTransitionError{Domain: m.Name, From: from, To: to}
	}
	if m.OnChange != nil {
		m.OnChange(m.Name, from, to, actor)
	}
	return nil
}

// IllegalTransitionError describes a forbidden status jump.
type IllegalTransitionError struct {
	Domain string
	From   string
	To     string
}

func (e *IllegalTransitionError) Error() string {
	return fmt.Sprintf("statemachine[%s]: illegal transition %q -> %q", e.Domain, e.From, e.To)
}

// MustValidate panics on illegal transition — use in tests or startup paths.
func (m *Machine) MustValidate(from, to string) {
	if !m.Can(from, to) {
		panic(fmt.Sprintf("statemachine[%s]: illegal transition %q -> %q", m.Name, from, to))
	}
}

// AllStates returns the union of all known states (for docs / validation).
func (m *Machine) AllStates() []string {
	seen := map[string]bool{}
	var out []string
	for from := range m.Transitions {
		if !seen[from] {
			seen[from] = true
			out = append(out, from)
		}
		for _, to := range m.Transitions[from] {
			if !seen[to] {
				seen[to] = true
				out = append(out, to)
			}
		}
	}
	return out
}
