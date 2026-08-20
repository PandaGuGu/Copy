package service

import (
	"fmt"

	"go.uber.org/zap"
	"gorm.io/gorm"

	"minibili/internal/pkg/statemachine"
)

// Domain machines exposed to workers / handlers (ADR-018).
var (
	VideoMachine = statemachine.Video
	UserMachine  = statemachine.User
	TicketMachine = statemachine.Ticket
)

// TransitionGuard wraps a domain state machine with a DB-backed audit hook
// (ADR-018 + ADR-006 integration). All business status writes should flow
// through Guard.Transition before persisting.
type TransitionGuard struct {
	Machine *statemachine.Machine
	DB      *gorm.DB
	Log     *zap.Logger
}

// NewTransitionGuard wires a machine with an audit sink.
func NewTransitionGuard(m *statemachine.Machine, db *gorm.DB, log *zap.Logger) *TransitionGuard {
	return &TransitionGuard{Machine: m, DB: db, Log: log}
}

// Can reports whether from -> to is legal for this domain.
func (g *TransitionGuard) Can(from, to string) bool { return g.Machine.Can(from, to) }

// Check validates from -> to and returns a descriptive error if illegal.
// Use it BEFORE any DB write.
func (g *TransitionGuard) Check(from, to, actor string) error {
	return g.Machine.Transition(from, to, actor)
}

// Transit validates and persists a status change on the given model.
//   - model: the GORM model (e.g. &model.Video{})
//   - id: primary key
//   - from: current status value (caller must load it, or pass "" to skip)
//   - to: target status
//   - actor: who performed the change (e.g. "admin:1" or "worker:transcode")
//
// It performs the state-machine check, then Updates(status=to). The audit
// hook is fired on success. Returns a statemachine error on illegal jumps.
func (g *TransitionGuard) Transit(model interface{}, id uint64, from, to, actor string) error {
	if from != "" && !g.Can(from, to) {
		return fmt.Errorf("statemachine[%s]: illegal transition %q -> %q", g.Machine.Name, from, to)
	}
	if err := g.DB.Model(model).Where("id = ?", id).Update("status", to).Error; err != nil {
		return fmt.Errorf("transition[%s] persist: %w", g.Machine.Name, err)
	}
	if g.Machine.OnChange != nil {
		g.Machine.OnChange(g.Machine.Name, from, to, actor)
	}
	return nil
}
