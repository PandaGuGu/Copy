package statemachine

// Domain state machines — single source of truth for allowed transitions.
// Every business status write should go through the matching machine below
// (see ADR-018). Add new states here first, then wire callers.

var (
	// Video: draft → processing → (pending_review | published | failed)
	// pending_review → (published | rejected)
	// takedown is the copyright-takedown state (report / copyright module).
	Video = New("video").
		AllowMany("draft", "processing", "takedown").
		AllowMany("processing", "pending_review", "published", "failed", "takedown").
		AllowMany("pending_review", "published", "rejected", "takedown").
		AllowMany("published", "takedown").
		AllowMany("failed", "processing", "takedown").
		AllowMany("rejected", "processing", "takedown").
		AllowMany("takedown", "published", "processing")

	// Article: draft → (pending_review | published)
	// pending_review → (published | rejected) ; takedown is copyright state.
	Article = New("article").
		AllowMany("draft", "pending_review", "published", "takedown").
		AllowMany("pending_review", "published", "rejected", "takedown").
		AllowMany("published", "takedown").
		AllowMany("rejected", "pending_review", "takedown").
		AllowMany("takedown", "published")

	// Ticket: open → assigning → assigned → processing → resolved → closed
	// reopened allows a closed/resolved ticket to cycle back to processing.
	// NOTE: open may only enter the assignment path — no direct resolved/closed
	// jump from open (ADR-018 tightening).
	Ticket = New("ticket").
		AllowMany("open", "assigning", "assigned").
		AllowMany("assigning", "assigned", "processing", "resolved", "closed").
		AllowMany("assigned", "processing", "resolved", "closed", "reopened").
		AllowMany("processing", "resolved", "closed", "reopened").
		AllowMany("resolved", "closed", "reopened").
		AllowMany("closed", "reopened").
		AllowMany("reopened", "processing", "assigned", "resolved", "closed")

	// Report: pending → (resolved | dismissed) ; resolved/dismissed can be reverted.
	Report = New("report").
		AllowMany("pending", "resolved", "dismissed").
		AllowMany("resolved", "pending").
		AllowMany("dismissed", "pending")

	// CopyrightComplaint: pending → (accepted | rejected)
	// accepted → takedown → restored (counter-notice flow).
	Copyright = New("copyright").
			AllowMany("pending", "accepted", "rejected").
			AllowMany("accepted", "takedown").
			AllowMany("takedown", "restored").
			AllowMany("restored", "takedown")

	// ApprovalFlow: multi-step serial approval (ADR-009).
	// Flow: pending → (approved | rejected); steps: pending → (approved | rejected).
	ApprovalFlow = New("approval_flow").
			AllowMany("pending", "approved", "rejected").
			AllowMany("approved", "pending").
			AllowMany("rejected", "pending")

	ApprovalStep = New("approval_step").
			AllowMany("pending", "approved", "rejected").
			AllowMany("approved", "rejected").
			AllowMany("rejected", "approved")

	// User: account lifecycle (ban / unban / delete).
	User = New("user").
		AllowMany("active", "banned", "deleted").
		AllowMany("banned", "active", "deleted").
		AllowMany("deleted", "active")

	// Appeal: user-side governance appeal.
	// pending → (approved | rejected) ; decided appeals can be reopened.
	Appeal = New("appeal").
		AllowMany("pending", "approved", "rejected").
		AllowMany("approved", "pending").
		AllowMany("rejected", "pending")
)

// All returns every registered machine for introspection / docs.
func All() []*Machine {
	return []*Machine{Video, Article, Ticket, Report, Copyright, ApprovalFlow, ApprovalStep, User, Appeal}
}
