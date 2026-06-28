package model

import "time"

// ──────────────────────────────────────────────
// Module 2: Player Advanced Features
// ──────────────────────────────────────────────

// VideoChapter marks a chapter point inside a video (multi-P / chapter navigation).
type VideoChapter struct {
	ID        uint64 `gorm:"primaryKey"`
	VideoID   uint64 `gorm:"index:idx_chapter_video;not null"`
	Title     string `gorm:"size:80;not null"`
	TimeSec   float64 `gorm:"not null"` // chapter start time in seconds
	CreatedAt time.Time
}

func (VideoChapter) TableName() string { return "video_chapters" }

// VideoBitrate stores alternative transcoded bitrate variants for a video.
type VideoBitrate struct {
	ID        uint64 `gorm:"primaryKey"`
	VideoID   uint64 `gorm:"index:idx_bitrate_video;not null"`
	Label     string `gorm:"size:32;not null"` // e.g. "360p", "720p", "1080p"
	Width     int    `gorm:"not null;default:0"`
	Height    int    `gorm:"not null;default:0"`
	Kbps      int    `gorm:"not null;default:0"` // bitrate in kbps
	URL       string `gorm:"size:1024;not null"`  // OSS URL for this variant
	CreatedAt time.Time
}

func (VideoBitrate) TableName() string { return "video_bitrates" }

// ──────────────────────────────────────────────
// Module 3: Subtitle Management
// ──────────────────────────────────────────────

// Subtitle stores a subtitle track for a video (multi-language support).
type Subtitle struct {
	ID        uint64    `gorm:"primaryKey" json:"id"`
	VideoID   uint64    `gorm:"index:idx_subtitle_video;not null" json:"video_id"`
	Lang      string    `gorm:"size:16;not null;default:zh" json:"lang"`
	Title     string    `gorm:"size:80;not null" json:"title"`
	Content   string    `gorm:"type:longtext;not null" json:"content"`
	Format    string    `gorm:"size:8;not null;default:vtt" json:"format"`
	AutoGen   bool      `gorm:"not null;default:0" json:"auto_gen"`
	CreatedAt time.Time `json:"created_at"`
	UpdatedAt time.Time `json:"updated_at"`
}

func (Subtitle) TableName() string { return "subtitles" }

// ──────────────────────────────────────────────
// Module 4: Comment Enhancement (image in comment)
// ──────────────────────────────────────────────

// CommentImage stores images attached to a comment.
type CommentImage struct {
	ID        uint64 `gorm:"primaryKey"`
	CommentID uint64 `gorm:"index:idx_comment_image_comment;not null"`
	URL       string `gorm:"size:1024;not null"`
	CreatedAt time.Time
}

func (CommentImage) TableName() string { return "comment_images" }

// ──────────────────────────────────────────────
// Module 5: Creator Center — Scheduled Publish
// ──────────────────────────────────────────────

// ScheduledPublish records a future publish time for a draft video.
type ScheduledPublish struct {
	ID           uint64    `gorm:"primaryKey"`
	VideoID      uint64    `gorm:"uniqueIndex:idx_scheduled_publish_video;not null"`
	PublishAt    time.Time `gorm:"index;not null"`
	Published    bool      `gorm:"not null;default:0"`
	CreatedAt    time.Time
	UpdatedAt    time.Time
}

func (ScheduledPublish) TableName() string { return "scheduled_publishes" }

// ──────────────────────────────────────────────
// Module 13: Ticket / Work Order System
// ──────────────────────────────────────────────

// Ticket is a support/operations work order.
type Ticket struct {
	ID            uint64    `gorm:"primaryKey"`
	ReporterID    uint64    `gorm:"index;not null"` // user who created the ticket
	AssigneeID    *uint64   `gorm:"index"`           // admin assigned to handle
	Category      string    `gorm:"size:32;not null"` // report / copyright / appeal / general
	Subject       string    `gorm:"size:200;not null"`
	Description   string    `gorm:"type:text;not null"`
	Status        string    `gorm:"size:32;not null;default:open;index"` // open / assigned / processing / resolved / closed / reopened
	Priority      string    `gorm:"size:16;not null;default:normal"` // low / normal / high / urgent
	RelatedID     uint64    `gorm:"index"` // related entity (video/article/dynamic id)
	RelatedType   string    `gorm:"size:32"` // video / article / dynamic / user / danmaku
	SLADeadline   *time.Time `gorm:"index"` // computed SLA deadline
	ResolvedAt    *time.Time
	ClosedAt      *time.Time
	CreatedAt     time.Time `gorm:"index"`
	UpdatedAt     time.Time
}

func (Ticket) TableName() string { return "tickets" }

// TicketMessage is a message in a ticket thread (between user and admin).
type TicketMessage struct {
	ID         uint64    `gorm:"primaryKey"`
	TicketID   uint64    `gorm:"index:idx_ticket_msg_ticket;not null"`
	SenderID   uint64    `gorm:"not null"` // user_id or admin_id
	SenderType string    `gorm:"size:16;not null;default:user"` // user / admin
	Content    string    `gorm:"type:text;not null"`
	CreatedAt  time.Time
}

func (TicketMessage) TableName() string { return "ticket_messages" }

// TicketSatisfaction records user rating after ticket resolution.
type TicketSatisfaction struct {
	ID        uint64    `gorm:"primaryKey" json:"id"`
	TicketID  uint64    `gorm:"uniqueIndex;not null" json:"ticket_id"`
	UserID    uint64    `gorm:"index;not null" json:"user_id"`
	Score     int       `gorm:"not null" json:"score"`
	Comment   string    `gorm:"type:text" json:"comment,omitempty"`
	CreatedAt time.Time `json:"created_at"`
}

func (TicketSatisfaction) TableName() string { return "ticket_satisfactions" }

// ──────────────────────────────────────────────
// Module 14: Risk Control & Ban Management
// ──────────────────────────────────────────────

// RiskRule is a configurable rule for automatic risk detection.
type RiskRule struct {
	ID          uint64    `gorm:"primaryKey"`
	Name        string    `gorm:"size:80;not null"`
	Category    string    `gorm:"size:32;not null;index"` // keyword / rate_limit / device_fingerprint / behavior
	RuleType    string    `gorm:"size:32;not null"` // block / flag / throttle
	Pattern     string    `gorm:"type:text;not null"` // regex / threshold / config JSON
	Action      string    `gorm:"size:32;not null"` // reject / quarantine / notify_admin / auto_ban
	DurationSec int       `gorm:"not null;default:0"` // ban duration in seconds (0 = permanent)
	Enabled     bool      `gorm:"not null;default:1;index"`
	Priority    int       `gorm:"not null;default:0"` // higher = first match
	CreatedAt   time.Time
	UpdatedAt   time.Time
}

func (RiskRule) TableName() string { return "risk_rules" }

// BlackWhiteList stores black/white list entries for risk control.
type BlackWhiteList struct {
	ID        uint64    `gorm:"primaryKey"`
	ListType  string    `gorm:"size:16;not null;index"` // blacklist / whitelist
	Target    string    `gorm:"size:200;not null"` // user_id / ip / device_id / keyword
	Reason    string    `gorm:"size:200"`
	ExpiresAt *time.Time `gorm:"index"` // null = permanent
	CreatedBy uint64    `gorm:"not null"`
	CreatedAt time.Time
}

func (BlackWhiteList) TableName() string { return "black_white_lists" }

// RiskHitLog records each time a risk rule triggers an action.
type RiskHitLog struct {
	ID        uint64    `gorm:"primaryKey" json:"id"`
	RuleID    uint64    `gorm:"index;not null" json:"rule_id"`
	RuleName  string    `gorm:"size:100" json:"rule_name"`
	TargetID  uint64    `gorm:"index;not null" json:"target_id"`     // video_id / user_id
	TargetType string   `gorm:"size:32;not null" json:"target_type"` // video / comment / user
	MatchText string    `gorm:"size:500" json:"match_text"`          // the matched content snippet
	Action    string    `gorm:"size:32;not null" json:"action"`      // delete / hide / limit / warn
	CreatedAt time.Time `json:"created_at"`
}

func (RiskHitLog) TableName() string { return "risk_hit_logs" }

// AdminLoginLog records every admin login attempt.
type AdminLoginLog struct {
	ID        uint64    `gorm:"primaryKey" json:"id"`
	AdminID   uint64    `gorm:"index;not null" json:"admin_id"`
	Username  string    `gorm:"size:64" json:"username"`
	IP        string    `gorm:"size:64" json:"ip"`
	Success   bool      `gorm:"not null" json:"success"`
	FailReason string   `gorm:"size:200" json:"fail_reason,omitempty"`
	CreatedAt time.Time `json:"created_at"`
}

func (AdminLoginLog) TableName() string { return "admin_login_logs" }

// ──────────────────────────────────────────────
// Module 15: Copyright & Takedown Management
// ──────────────────────────────────────────────

// CopyrightComplaint records a copyright infringement complaint.
type CopyrightComplaint struct {
	ID              uint64    `gorm:"primaryKey"`
	ComplainantID   uint64    `gorm:"index;not null"` // user who filed the complaint
	RelatedID       uint64    `gorm:"index;not null"` // video/article id being complained
	RelatedType     string    `gorm:"size:32;not null"` // video / article
	Description     string    `gorm:"type:text;not null"`
	EvidenceURLs    string    `gorm:"type:text"` // JSON array of evidence file URLs
	Status          string    `gorm:"size:32;not null;default:pending;index"` // pending / accepted / rejected / takedown / restored
	HandlerID       *uint64   `gorm:"index"` // admin who processed
	HandlerComment  string    `gorm:"size:500"`
	TakedownAt      *time.Time
	RestoredAt      *time.Time
	CreatedAt       time.Time `gorm:"index"`
	UpdatedAt       time.Time
}

func (CopyrightComplaint) TableName() string { return "copyright_complaints" }

// CounterNotice is filed by the accused party to dispute a copyright complaint.
type CounterNotice struct {
	ID           uint64    `gorm:"primaryKey" json:"id"`
	ComplaintID  uint64    `gorm:"index;not null" json:"complaint_id"`
	UserID       uint64    `gorm:"index;not null" json:"user_id"`
	Statement    string    `gorm:"type:text;not null" json:"statement"`
	EvidenceURLs string    `gorm:"type:text" json:"evidence_urls,omitempty"`
	Contact      string    `gorm:"size:200" json:"contact,omitempty"`
	Status       string    `gorm:"size:32;not null;default:pending" json:"status"` // pending/accepted/rejected
	HandlerNote  string    `gorm:"size:500" json:"handler_note,omitempty"`
	CreatedAt    time.Time `json:"created_at"`
	UpdatedAt    time.Time `json:"updated_at"`
}

func (CounterNotice) TableName() string { return "counter_notices" }

// ──────────────────────────────────────────────
// Notification system (admin → user / admin → admin)
// ──────────────────────────────────────────────

// NotificationRecord tracks admin-originated notifications across modules.
type NotificationRecord struct {
	ID            uint64     `gorm:"primaryKey" json:"id"`
	RecipientID   uint64     `gorm:"index;not null" json:"recipient_id"`
	RecipientType string     `gorm:"size:16;not null;default:user" json:"recipient_type"` // user / admin
	Channel       string     `gorm:"size:16;not null;default:in_app" json:"channel"`      // in_app / email
	Title         string     `gorm:"size:200;not null" json:"title"`
	Content       string     `gorm:"type:text;not null" json:"content"`
	RelatedType   string     `gorm:"size:32" json:"related_type,omitempty"` // copyright / ticket / report / ban
	RelatedID     uint64     `gorm:"index" json:"related_id,omitempty"`
	Status        string     `gorm:"size:16;not null;default:pending" json:"status"` // pending / sent / read
	SentAt        *time.Time `json:"sent_at,omitempty"`
	ReadAt        *time.Time `json:"read_at,omitempty"`
	CreatedAt     time.Time  `json:"created_at"`
}

func (NotificationRecord) TableName() string { return "notification_records" }

// ──────────────────────────────────────────────
// Module 16: BI / Statistics Report
// ──────────────────────────────────────────────

// SavedReport is a user-saved BI report configuration.
type SavedReport struct {
	ID          uint64    `gorm:"primaryKey"`
	CreatorID   uint64    `gorm:"index;not null"` // admin who created this report config
	Name        string    `gorm:"size:80;not null"`
	Description string    `gorm:"size:200"`
	QueryConfig string    `gorm:"type:text;not null"` // JSON: dimensions, filters, metrics
	ChartType   string    `gorm:"size:32;not null;default:table"` // table / bar / line / pie
	IsPublic    bool      `gorm:"not null;default:0"`
	CreatedAt   time.Time
	UpdatedAt   time.Time
}

func (SavedReport) TableName() string { return "saved_reports" }

// ──────────────────────────────────────────────
// Module 17: Customer Service Backend
// ──────────────────────────────────────────────

// CSTemplate is a pre-defined customer service response template.
type CSTemplate struct {
	ID        uint64    `gorm:"primaryKey"`
	Name      string    `gorm:"size:80;not null"`
	Category  string    `gorm:"size:32;not null;index"` // greeting / faq / resolve / escalation
	Content   string    `gorm:"type:text;not null"`
	CreatedBy uint64    `gorm:"not null"`
	CreatedAt time.Time
	UpdatedAt time.Time
}

func (CSTemplate) TableName() string { return "cs_templates" }

// LiveWarnTemplate stores pre-defined admin warning messages for live rooms.
type LiveWarnTemplate struct {
	ID        uint64    `gorm:"primaryKey" json:"id"`
	Name      string    `gorm:"size:80;not null" json:"name"`
	Content   string    `gorm:"type:text;not null" json:"content"`
	SortOrder int       `gorm:"not null;default:0" json:"sort_order"`
	CreatedAt time.Time `json:"created_at"`
	UpdatedAt time.Time `json:"updated_at"`
}

func (LiveWarnTemplate) TableName() string { return "live_warn_templates" }

// CSConversation records a customer service conversation session.
type CSConversation struct {
	ID          uint64    `gorm:"primaryKey"`
	UserID      uint64    `gorm:"index;not null"` // the user seeking help
	AdminID     *uint64   `gorm:"index"`          // assigned CS admin
	TicketID    *uint64   `gorm:"index"`          // linked ticket
	Status      string    `gorm:"size:32;not null;default:waiting;index"` // waiting / active / resolved / closed
	CreatedAt   time.Time `gorm:"index"`
	UpdatedAt   time.Time
}

func (CSConversation) TableName() string { return "cs_conversations" }

// CSMessage is a message in a customer service conversation.
type CSMessage struct {
	ID             uint64    `gorm:"primaryKey"`
	ConversationID uint64    `gorm:"index:idx_cs_msg_conv;not null"`
	SenderID       uint64    `gorm:"not null"`
	SenderType     string    `gorm:"size:16;not null;default:user"` // user / admin / bot
	Content        string    `gorm:"type:text;not null"`
	CreatedAt      time.Time
}

func (CSMessage) TableName() string { return "cs_messages" }

// ──────────────────────────────────────────────
// Module 18: Queue & Task Visualization
// ──────────────────────────────────────────────

// TaskLog records transcode/async task execution for visualization.
type TaskLog struct {
	ID         uint64    `gorm:"primaryKey"`
	TaskType   string    `gorm:"size:32;not null;index"` // transcode / subtitle_asr / report_export / sync
	TargetID   uint64    `gorm:"index;not null"` // video_id / article_id etc.
	Status     string    `gorm:"size:32;not null;index"` // pending / running / success / failed / retrying
	RetryCount int       `gorm:"not null;default:0"`
	ErrorMsg   string    `gorm:"size:2000"`
	StartedAt  *time.Time `gorm:"index"`
	FinishedAt *time.Time
	CreatedAt  time.Time `gorm:"index"`
}

func (TaskLog) TableName() string { return "task_logs" }

// ──────────────────────────────────────────────
// Module 19: Monitoring & Alerting
// ──────────────────────────────────────────────

// AlertRule defines a monitoring alert rule.
type AlertRule struct {
	ID          uint64    `gorm:"primaryKey"`
	Name        string    `gorm:"size:80;not null"`
	Metric      string    `gorm:"size:64;not null;index"` // cpu / memory / disk / bandwidth / error_rate / queue_depth
	Threshold   float64   `gorm:"not null"`
	Operator    string    `gorm:"size:8;not null"` // gt / lt / eq / gte / lte
	DurationSec int       `gorm:"not null;default:0"` // alert fires after condition persists this long
	Channel     string    `gorm:"size:32;not null;default:log"` // log / dingtalk / wecom / email
	ChannelConf string    `gorm:"type:text"` // webhook URL, email list etc.
	Enabled     bool      `gorm:"not null;default:1;index"`
	CreatedAt   time.Time
	UpdatedAt   time.Time
}

func (AlertRule) TableName() string { return "alert_rules" }

// AlertRecord logs a fired alert instance.
type AlertRecord struct {
	ID        uint64    `gorm:"primaryKey"`
	RuleID    uint64    `gorm:"index;not null"`
	Value     float64   `gorm:"not null"` // actual metric value at alert time
	Status    string    `gorm:"size:16;not null;default:firing;index"` // firing / resolved
	AckedBy   *uint64   `gorm:"index"` // admin who acknowledged
	AckedAt   *time.Time
	CreatedAt time.Time `gorm:"index"`
}

func (AlertRecord) TableName() string { return "alert_records" }

// ──────────────────────────────────────────────
// Module 20: Trace & Log Retrieval
// ──────────────────────────────────────────────

// TraceRecord stores a distributed trace record for request tracing.
type TraceRecord struct {
	ID         uint64    `gorm:"primaryKey"`
	TraceID    string    `gorm:"size:64;uniqueIndex;not null"`
	RequestID  string    `gorm:"size:64;index"`
	UserID     *uint64   `gorm:"index"`
	Path       string    `gorm:"size:256;not null"`
	Method     string    `gorm:"size:8;not null"`
	Status     int       `gorm:"not null"` // HTTP status code
	DurationMs int64     `gorm:"not null"` // response time in ms
	ErrorMsg   string    `gorm:"size:1000"`
	CreatedAt  time.Time `gorm:"index"`
}

func (TraceRecord) TableName() string { return "trace_records" }

// ──────────────────────────────────────────────
// Module 21: Release & Config Management
// ──────────────────────────────────────────────

// FeatureFlag controls feature availability via toggle.
type FeatureFlag struct {
	ID          uint64    `gorm:"primaryKey"`
	Key         string    `gorm:"size:64;uniqueIndex;not null"` // e.g. "live_stream", "hls_player"
	Description string    `gorm:"size:200"`
	Enabled     bool      `gorm:"not null;default:0;index"`
	RolloutPct  int       `gorm:"not null;default:0"` // 0-100 rollout percentage for gradual enable
	Whitelist   string    `gorm:"type:text"` // JSON array of whitelisted user_ids
	UpdatedAt   time.Time
	CreatedAt   time.Time
}

func (FeatureFlag) TableName() string { return "feature_flags" }

// ReleaseRecord tracks a deployment release for rollback/audit.
type ReleaseRecord struct {
	ID           uint64     `gorm:"primaryKey"`
	Version      string     `gorm:"size:32;uniqueIndex;not null"`
	Title        string     `gorm:"size:200"`                        // human-readable title
	Type         string     `gorm:"size:16;default:canary"`          // canary / full / hotfix
	Notes        string     `gorm:"type:text"`                       // release notes
	Snapshot     string     `gorm:"type:longtext"`                   // JSON snapshot of all configs at release time
	Status       string     `gorm:"size:32;not null;default:deploying;index"` // deploying / released / rolled_out / rolled_back
	Description  string     `gorm:"size:500"`
	DeployedBy   uint64     `gorm:"not null"`
	PushedBy     *uint64    `gorm:"index"` // admin who triggered publish
	RolledBackBy *uint64    `gorm:"index"`
	ReleasedAt   *time.Time // when pushed to remote
	CreatedAt    time.Time  `gorm:"index"`
}

func (ReleaseRecord) TableName() string { return "release_records" }

// ──────────────────────────────────────────────
// Module 22: CDN & Storage Ops
// ──────────────────────────────────────────────

// CDNRefreshTask records a CDN cache purge / refresh request.
type CDNRefreshTask struct {
	ID          uint64    `gorm:"primaryKey"`
	RefreshType string    `gorm:"size:16;not null"`  // url / directory
	Urls        string    `gorm:"type:text;not null"` // JSON array of URLs
	Status      string    `gorm:"size:32;not null;default:pending;index"` // pending / processing / success / failed
	RequestedBy uint64    `gorm:"not null"`
	FinishedAt  *time.Time
	CreatedAt   time.Time `gorm:"index"`
}

func (CDNRefreshTask) TableName() string { return "cdn_refresh_tasks" }

// OSSLifecycleRule configures OSS object lifecycle management.
type OSSLifecycleRule struct {
	ID          uint64    `gorm:"primaryKey"`
	Name        string    `gorm:"size:80;not null"`  // human-readable rule name
	Bucket      string    `gorm:"size:128;not null"` // OSS bucket name
	Prefix      string    `gorm:"size:256;not null"` // e.g. "temp/", "originals/"
	IADays      int       `gorm:"not null;default:0"`     // days before transition to IA
	ArchiveDays int       `gorm:"not null;default:0"`     // days before transition to Archive
	DeleteDays  int       `gorm:"not null;default:0"`     // days before deletion
	Enabled     bool      `gorm:"not null;default:1;index"`
	CreatedBy   uint64    `gorm:"not null"`
	CreatedAt   time.Time
	UpdatedAt   time.Time
}

func (OSSLifecycleRule) TableName() string { return "oss_lifecycle_rules" }

// ──────────────────────────────────────────────
// Module 23: RBAC & Audit
// ──────────────────────────────────────────────

// AdminRole defines an admin role for RBAC.
type AdminRole struct {
	ID          uint64    `gorm:"primaryKey"`
	Name        string    `gorm:"size:32;uniqueIndex;not null"` // super_admin / content_admin / cs_admin / risk_admin / stats_admin
	Description string    `gorm:"size:200"`
	CreatedAt   time.Time
	UpdatedAt   time.Time
}

func (AdminRole) TableName() string { return "admin_roles" }

// AdminPermission defines a fine-grained permission point.
type AdminPermission struct {
	ID       uint64 `gorm:"primaryKey"`
	Code     string `gorm:"size:64;uniqueIndex;not null"` // e.g. "video:review", "user:ban", "report:handle"
	Resource string `gorm:"size:32;not null;index"` // video / article / user / report / ticket / copyright / setting / dashboard
	Action   string `gorm:"size:16;not null"` // view / create / update / delete / approve / reject / ban / export
}

func (AdminPermission) TableName() string { return "admin_permissions" }

// RolePermission links a role to its permissions.
type RolePermission struct {
	ID           uint64 `gorm:"primaryKey"`
	RoleID       uint64 `gorm:"uniqueIndex:idx_role_perm_pair,priority:1;not null"`
	PermissionID uint64 `gorm:"uniqueIndex:idx_role_perm_pair,priority:2;not null"`
}

func (RolePermission) TableName() string { return "role_permissions" }

// AdminRoleAssignment links an admin user to a role.
type AdminRoleAssignment struct {
	ID      uint64 `gorm:"primaryKey"`
	AdminID uint64 `gorm:"uniqueIndex:idx_admin_role_assign;not null"`
	RoleID  uint64 `gorm:"index;not null"`
}

func (AdminRoleAssignment) TableName() string { return "admin_role_assignments" }

// AuditLog records admin operations for audit trail.
type AuditLog struct {
	ID         uint64    `gorm:"primaryKey"`
	AdminID    uint64    `gorm:"index;not null"`
	Action     string    `gorm:"size:64;not null;index"` // e.g. "ban_user", "approve_video", "delete_comment"
	Resource   string    `gorm:"size:32;not null;index"` // user / video / article / comment / report / ticket / copyright / setting / role
	TargetID   uint64    `gorm:"index"` // ID of affected entity
	Detail     string    `gorm:"type:text"` // JSON detail of what changed
	IPAddress  string    `gorm:"size:64"`
	CreatedAt  time.Time `gorm:"index"`
}

func (AuditLog) TableName() string { return "audit_logs" }

// ApprovalFlow tracks multi-step approval processes.
type ApprovalFlow struct {
	ID           uint64    `gorm:"primaryKey"`
	ResourceType string    `gorm:"size:32;not null;index"` // video / article / ban / copyright
	ResourceID   uint64    `gorm:"index;not null"`
	Status       string    `gorm:"size:32;not null;default:pending;index"` // pending / approved / rejected / cancelled
	CurrentStep  int       `gorm:"not null;default:1"` // current approval step
	TotalSteps   int       `gorm:"not null;default:2"` // total required steps
	RequestorID  uint64    `gorm:"not null"`
	CreatedAt    time.Time
	UpdatedAt    time.Time
}

func (ApprovalFlow) TableName() string { return "approval_flows" }

// ApprovalStep records each step in an approval flow.
type ApprovalStep struct {
	ID          uint64    `gorm:"primaryKey"`
	FlowID      uint64    `gorm:"index:idx_approval_step_flow;not null"`
	StepNumber  int       `gorm:"not null"`
	ApproverID  uint64    `gorm:"not null"` // admin assigned to this step
	Decision    string    `gorm:"size:16"` // approved / rejected / pending
	Comment     string    `gorm:"size:500"`
	DecidedAt   *time.Time
	CreatedAt   time.Time
}

func (ApprovalStep) TableName() string { return "approval_steps" }

// ──────────────────────────────────────────────
// Module 9: Special Pages & Campaigns
// ──────────────────────────────────────────────

// SpecialPage represents a curated content aggregation page (e.g. "2026 夏季新番专题").
type SpecialPage struct {
	ID          uint64    `gorm:"primaryKey" json:"id"`
	Title       string    `gorm:"size:100;not null" json:"title"`
	Slug        string    `gorm:"size:60;uniqueIndex;not null" json:"slug"`
	CoverURL    string    `gorm:"size:1024" json:"cover_url"`
	Description string    `gorm:"size:500" json:"description"`
	Blocks      string    `gorm:"type:longtext" json:"blocks"`
	Status      string    `gorm:"size:16;not null;default:draft" json:"status"`
	CreatedAt   time.Time `json:"created_at"`
	UpdatedAt   time.Time `json:"updated_at"`
}

func (SpecialPage) TableName() string { return "special_pages" }

// Campaign represents a time-limited event with rules and rewards.
type Campaign struct {
	ID          uint64     `gorm:"primaryKey" json:"id"`
	Title       string     `gorm:"size:100;not null" json:"title"`
	Slug        string     `gorm:"size:60;uniqueIndex;not null" json:"slug"`
	CoverURL    string     `gorm:"size:1024" json:"cover_url"`
	Description string     `gorm:"size:500" json:"description"`
	Rules       string     `gorm:"type:text" json:"rules"`
	Rewards     string     `gorm:"type:text" json:"rewards"`
	StartTime   *time.Time `gorm:"index" json:"start_time"`
	EndTime     *time.Time `gorm:"index" json:"end_time"`
	Status      string     `gorm:"size:16;not null;default:draft" json:"status"`
	CreatedAt   time.Time  `json:"created_at"`
	UpdatedAt   time.Time  `json:"updated_at"`
}

func (Campaign) TableName() string { return "campaigns" }

// ──────────────────────────────────────────────
// Daily play stats for creator dashboard charts
// ──────────────────────────────────────────────

// VideoDailyStat records per-video play counts by date.
type VideoDailyStat struct {
	ID        uint64    `gorm:"primaryKey" json:"id"`
	VideoID   uint64    `gorm:"uniqueIndex:idx_vds_vid_date;not null" json:"video_id"`
	Date      string    `gorm:"size:10;uniqueIndex:idx_vds_vid_date;not null" json:"date"` // "2006-01-02"
	PlayCount int64     `gorm:"not null;default:0" json:"play_count"`
	CreatedAt time.Time `json:"created_at"`
	UpdatedAt time.Time `json:"updated_at"`
}

func (VideoDailyStat) TableName() string { return "video_daily_stats" }

// ──────────────────────────────────────────────
// Module: Live Streaming
// ──────────────────────────────────────────────

// LiveRoom represents a streaming room.
type LiveRoom struct {
	ID          uint64     `gorm:"primaryKey" json:"id"`
	UserID      uint64     `gorm:"index:idx_liveroom_user;not null" json:"user_id"`
	Title       string     `gorm:"size:60;not null" json:"title"`
	CoverURL    string     `gorm:"size:1024" json:"cover_url"`
	StreamKey   string     `gorm:"size:64;uniqueIndex;not null" json:"stream_key"`
	Status      string     `gorm:"size:16;not null;default:idle" json:"status"` // idle / live / ended / banned
	ViewerCount int64      `gorm:"not null;default:0" json:"viewer_count"`
	HostName    string     `gorm:"size:40" json:"host_name"`
	AvatarURL   string     `gorm:"size:1024" json:"avatar_url"`
	StartedAt   *time.Time `json:"started_at"`
	EndedAt     *time.Time `json:"ended_at"`
	CreatedAt   time.Time  `json:"created_at"`
	UpdatedAt   time.Time  `json:"updated_at"`
}

func (LiveRoom) TableName() string { return "live_rooms" }
