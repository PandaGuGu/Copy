package handler

import (
	"net/http"
	"strconv"
	"strings"
	"time"

	"github.com/gin-gonic/gin"
	"go.uber.org/zap"
	"gorm.io/gorm"

	"minibili/internal/errcode"
	"minibili/internal/middleware"
	"minibili/internal/model"
	"minibili/internal/pkg/resp"
)

// ---------- Admin Ticket Endpoints ----------

// AdminListTickets GET /api/v1/admin/tickets
func (a *API) AdminListTickets(c *gin.Context) {
	page, _ := strconv.Atoi(c.DefaultQuery("page", "1"))
	pageSize, _ := strconv.Atoi(c.DefaultQuery("page_size", "20"))
	status := strings.TrimSpace(c.Query("status"))
	category := strings.TrimSpace(c.Query("category"))
	priority := strings.TrimSpace(c.Query("priority"))
	assigneeStr := strings.TrimSpace(c.Query("assignee_id"))

	if page < 1 {
		page = 1
	}
	if pageSize < 1 || pageSize > 100 {
		pageSize = 20
	}

	tx := a.DB.Model(&model.Ticket{})
	if status != "" {
		tx = tx.Where("status = ?", status)
	}
	if category != "" {
		tx = tx.Where("category = ?", category)
	}
	if priority != "" {
		tx = tx.Where("priority = ?", priority)
	}
	if assigneeStr != "" {
		if aid, err := strconv.ParseUint(assigneeStr, 10, 64); err == nil {
			tx = tx.Where("assignee_id = ?", aid)
		}
	}

	var total int64
	tx.Count(&total)

	var tickets []model.Ticket
	tx.Order("created_at DESC").Offset((page - 1) * pageSize).Limit(pageSize).Find(&tickets)

	// Batch load reporter and assignee info
	uidSet := make(map[uint64]bool)
	aidSet := make(map[uint64]bool)
	for _, t := range tickets {
		uidSet[t.ReporterID] = true
		if t.AssigneeID != nil {
			aidSet[*t.AssigneeID] = true
		}
	}
	userBriefs := loadUserBriefs(a.DB, uidSet)
	assigneeBriefs := loadUserBriefs(a.DB, aidSet)

	type item struct {
		ID          uint64     `json:"id"`
		ReporterID  uint64     `json:"reporter_id"`
		Reporter    gin.H      `json:"reporter"`
		AssigneeID  *uint64    `json:"assignee_id"`
		Assignee    gin.H      `json:"assignee"`
		Category    string     `json:"category"`
		Subject     string     `json:"subject"`
		Description string     `json:"description"`
		Status      string     `json:"status"`
		Priority    string     `json:"priority"`
		RelatedID   uint64     `json:"related_id"`
		RelatedType string     `json:"related_type"`
		SLADeadline *time.Time `json:"sla_deadline"`
		ResolvedAt  *time.Time `json:"resolved_at"`
		ClosedAt    *time.Time `json:"closed_at"`
		CreatedAt   time.Time  `json:"created_at"`
		UpdatedAt   time.Time  `json:"updated_at"`
	}
	items := make([]item, 0, len(tickets))
	for _, t := range tickets {
		it := item{
			ID:          t.ID,
			ReporterID:  t.ReporterID,
			Reporter:    userBriefs[t.ReporterID],
			AssigneeID:  t.AssigneeID,
			Category:    t.Category,
			Subject:     t.Subject,
			Description: t.Description,
			Status:      t.Status,
			Priority:    t.Priority,
			RelatedID:   t.RelatedID,
			RelatedType: t.RelatedType,
			SLADeadline: t.SLADeadline,
			ResolvedAt:  t.ResolvedAt,
			ClosedAt:    t.ClosedAt,
			CreatedAt:   t.CreatedAt,
			UpdatedAt:   t.UpdatedAt,
		}
		if t.AssigneeID != nil {
			it.Assignee = assigneeBriefs[*t.AssigneeID]
		}
		items = append(items, it)
	}

	// Stats
	var openCount, assignedCount, resolvedCount, closedCount int64
	a.DB.Model(&model.Ticket{}).Where("status = 'open'").Count(&openCount)
	a.DB.Model(&model.Ticket{}).Where("status IN ('assigned','processing')").Count(&assignedCount)
	a.DB.Model(&model.Ticket{}).Where("status = 'resolved'").Count(&resolvedCount)
	a.DB.Model(&model.Ticket{}).Where("status = 'closed'").Count(&closedCount)

	resp.OK(c, gin.H{
		"items":           items,
		"total":           total,
		"page":            page,
		"page_size":       pageSize,
		"open_count":      openCount,
		"assigned_count":  assignedCount,
		"resolved_count":  resolvedCount,
		"closed_count":    closedCount,
	})
}

// AdminGetTicket GET /api/v1/admin/tickets/:id
func (a *API) AdminGetTicket(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	var t model.Ticket
	if err := a.DB.First(&t, id).Error; err != nil {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}

	var messages []model.TicketMessage
	a.DB.Where("ticket_id = ?", id).Order("created_at ASC").Find(&messages)

	senderIDs := make(map[uint64]bool)
	for _, m := range messages {
		senderIDs[m.SenderID] = true
	}
	senderBriefs := loadUserBriefs(a.DB, senderIDs)

	type msgItem struct {
		ID         uint64    `json:"id"`
		SenderID   uint64    `json:"sender_id"`
		SenderType string    `json:"sender_type"`
		Sender     gin.H     `json:"sender"`
		Content    string    `json:"content"`
		CreatedAt  time.Time `json:"created_at"`
	}
	msgItems := make([]msgItem, 0, len(messages))
	for _, m := range messages {
		msgItems = append(msgItems, msgItem{
			ID:         m.ID,
			SenderID:   m.SenderID,
			SenderType: m.SenderType,
			Sender:     senderBriefs[m.SenderID],
			Content:    m.Content,
			CreatedAt:  m.CreatedAt,
		})
	}

	resp.OK(c, gin.H{
		"ticket": gin.H{
			"id":          t.ID,
			"reporter_id": t.ReporterID,
			"reporter":    loadUserBrief(a.DB, t.ReporterID),
			"assignee_id": t.AssigneeID,
			"assignee":    loadUserBriefNull(a.DB, t.AssigneeID),
			"category":    t.Category,
			"subject":     t.Subject,
			"description": t.Description,
			"status":      t.Status,
			"priority":    t.Priority,
			"related_id":  t.RelatedID,
			"related_type": t.RelatedType,
			"sla_deadline": t.SLADeadline,
			"resolved_at":  t.ResolvedAt,
			"closed_at":    t.ClosedAt,
			"created_at":   t.CreatedAt,
			"updated_at":   t.UpdatedAt,
		},
		"messages": msgItems,
	})
}

// AdminAssignTicket POST /api/v1/admin/tickets/:id/assign
func (a *API) AdminAssignTicket(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	var req struct {
		AssigneeID uint64 `json:"assignee_id"`
	}
	if err := c.ShouldBindJSON(&req); err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	if req.AssigneeID == 0 {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	adminID, _ := middleware.AdminID(c)

	var t model.Ticket
	if err := a.DB.First(&t, id).Error; err != nil {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}
	if t.Status == "closed" {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	aid := req.AssigneeID
	if err := a.DB.Model(&t).Updates(map[string]interface{}{
		"assignee_id": aid,
		"status":      "assigned",
		"updated_at":  time.Now(),
	}).Error; err != nil {
		a.Log.Error("assign ticket", zap.Error(err))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	a.Log.Info("admin assigned ticket",
		zap.Uint64("ticket_id", id),
		zap.Uint64("assignee_id", aid),
		zap.Uint64("operator_id", adminID),
	)

	resp.OK(c, gin.H{"ticket_id": id, "assignee_id": aid, "status": "assigned"})
}

// AdminUpdateTicketStatus POST /api/v1/admin/tickets/:id/status
func (a *API) AdminUpdateTicketStatus(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	var req struct {
		Status string `json:"status"`
	}
	if err := c.ShouldBindJSON(&req); err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	req.Status = strings.TrimSpace(req.Status)
	validStatuses := map[string]bool{
		"assigning":  true,
		"processing": true,
		"resolved":   true,
	}
	if !validStatuses[req.Status] {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	adminID, _ := middleware.AdminID(c)

	var t model.Ticket
	if err := a.DB.First(&t, id).Error; err != nil {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}
	if t.Status == "closed" {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	now := time.Now()
	updates := map[string]interface{}{
		"status":     req.Status,
		"updated_at": now,
	}
	if req.Status == "resolved" {
		updates["resolved_at"] = now
	}

	if err := a.DB.Model(&t).Updates(updates).Error; err != nil {
		a.Log.Error("update ticket status", zap.Error(err))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	a.Log.Info("admin updated ticket status",
		zap.Uint64("ticket_id", id),
		zap.String("status", req.Status),
		zap.Uint64("operator_id", adminID),
	)

	resp.OK(c, gin.H{"ticket_id": id, "status": req.Status})
}

// AdminAddTicketMessage POST /api/v1/admin/tickets/:id/messages
func (a *API) AdminAddTicketMessage(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	var req struct {
		Content string `json:"content"`
	}
	if err := c.ShouldBindJSON(&req); err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	req.Content = strings.TrimSpace(req.Content)
	if req.Content == "" || len([]rune(req.Content)) > 2000 {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	adminID, _ := middleware.AdminID(c)

	var t model.Ticket
	if err := a.DB.First(&t, id).Error; err != nil {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}
	if t.Status == "closed" {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	msg := model.TicketMessage{
		TicketID:   id,
		SenderID:   adminID,
		SenderType: "admin",
		Content:    req.Content,
	}
	if err := a.DB.Create(&msg).Error; err != nil {
		a.Log.Error("create ticket message", zap.Error(err))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	// Transition from assigned to processing on first admin reply
	if t.Status == "assigned" {
		a.DB.Model(&t).Updates(map[string]interface{}{
			"status":     "processing",
			"updated_at": time.Now(),
		})
	}

	a.Log.Info("admin replied to ticket",
		zap.Uint64("ticket_id", id),
		zap.Uint64("message_id", msg.ID),
		zap.Uint64("admin_id", adminID),
	)

	resp.OK(c, gin.H{
		"id":         msg.ID,
		"ticket_id":  id,
		"content":    msg.Content,
		"created_at": msg.CreatedAt,
	})
}

// AdminCloseTicket POST /api/v1/admin/tickets/:id/close
func (a *API) AdminCloseTicket(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	adminID, _ := middleware.AdminID(c)

	var t model.Ticket
	if err := a.DB.First(&t, id).Error; err != nil {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}
	if t.Status == "closed" {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	now := time.Now()
	if err := a.DB.Model(&t).Updates(map[string]interface{}{
		"status":     "closed",
		"closed_at":  now,
		"updated_at": now,
	}).Error; err != nil {
		a.Log.Error("close ticket", zap.Error(err))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	a.Log.Info("admin closed ticket",
		zap.Uint64("ticket_id", id),
		zap.Uint64("admin_id", adminID),
	)

	resp.OK(c, gin.H{"ticket_id": id, "status": "closed"})
}

// AdminReopenTicket POST /api/v1/admin/tickets/:id/reopen
func (a *API) AdminReopenTicket(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	adminID, _ := middleware.AdminID(c)

	var t model.Ticket
	if err := a.DB.First(&t, id).Error; err != nil {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}
	if t.Status != "closed" && t.Status != "resolved" {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	now := time.Now()
	if err := a.DB.Model(&t).Updates(map[string]interface{}{
		"status":      "reopened",
		"resolved_at": nil,
		"closed_at":   nil,
		"updated_at":  now,
	}).Error; err != nil {
		a.Log.Error("reopen ticket", zap.Error(err))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	a.Log.Info("admin reopened ticket",
		zap.Uint64("ticket_id", id),
		zap.Uint64("admin_id", adminID),
	)

	resp.OK(c, gin.H{"ticket_id": id, "status": "reopened"})
}

// ---------- User Ticket Endpoints ----------

// PostTicket POST /api/v1/tickets
func (a *API) PostTicket(c *gin.Context) {
	userID, _ := middleware.UserID(c)
	if userID == 0 {
		resp.Err(c, http.StatusUnauthorized, errcode.CodeUnauthorized)
		return
	}

	var req struct {
		Category    string `json:"category"`
		Subject     string `json:"subject"`
		Description string `json:"description"`
		RelatedID   uint64 `json:"related_id"`
		RelatedType string `json:"related_type"`
	}
	if err := c.ShouldBindJSON(&req); err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	req.Category = strings.TrimSpace(req.Category)
	req.Subject = strings.TrimSpace(req.Subject)
	req.Description = strings.TrimSpace(req.Description)
	req.RelatedType = strings.TrimSpace(req.RelatedType)

	if req.Category == "" || req.Subject == "" || req.Description == "" {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	if len([]rune(req.Subject)) > 200 || len([]rune(req.Description)) > 5000 {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	categoryValid := map[string]bool{
		"report":    true,
		"copyright": true,
		"appeal":    true,
		"general":   true,
	}
	if !categoryValid[req.Category] {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	t := model.Ticket{
		ReporterID:  userID,
		Category:    req.Category,
		Subject:     req.Subject,
		Description: req.Description,
		Status:      "open",
		Priority:    "normal",
		RelatedID:   req.RelatedID,
		RelatedType: req.RelatedType,
	}
	if err := a.DB.Create(&t).Error; err != nil {
		a.Log.Error("create ticket", zap.Error(err))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	a.Log.Info("user created ticket",
		zap.Uint64("ticket_id", t.ID),
		zap.Uint64("user_id", userID),
		zap.String("category", t.Category),
	)

	resp.OK(c, gin.H{"id": t.ID})
}

// ListMyTickets GET /api/v1/users/me/tickets
func (a *API) ListMyTickets(c *gin.Context) {
	userID, _ := middleware.UserID(c)
	if userID == 0 {
		resp.Err(c, http.StatusUnauthorized, errcode.CodeUnauthorized)
		return
	}

	page, _ := strconv.Atoi(c.DefaultQuery("page", "1"))
	pageSize, _ := strconv.Atoi(c.DefaultQuery("page_size", "20"))
	status := strings.TrimSpace(c.Query("status"))

	if page < 1 {
		page = 1
	}
	if pageSize < 1 || pageSize > 100 {
		pageSize = 20
	}

	tx := a.DB.Model(&model.Ticket{}).Where("reporter_id = ?", userID)
	if status != "" {
		tx = tx.Where("status = ?", status)
	}

	var total int64
	tx.Count(&total)

	var tickets []model.Ticket
	tx.Order("created_at DESC").Offset((page - 1) * pageSize).Limit(pageSize).Find(&tickets)

	type item struct {
		ID          uint64     `json:"id"`
		Category    string     `json:"category"`
		Subject     string     `json:"subject"`
		Description string     `json:"description"`
		Status      string     `json:"status"`
		Priority    string     `json:"priority"`
		RelatedID   uint64     `json:"related_id"`
		RelatedType string     `json:"related_type"`
		SLADeadline *time.Time `json:"sla_deadline"`
		ResolvedAt  *time.Time `json:"resolved_at"`
		ClosedAt    *time.Time `json:"closed_at"`
		CreatedAt   time.Time  `json:"created_at"`
	}
	items := make([]item, 0, len(tickets))
	for _, t := range tickets {
		items = append(items, item{
			ID:          t.ID,
			Category:    t.Category,
			Subject:     t.Subject,
			Description: t.Description,
			Status:      t.Status,
			Priority:    t.Priority,
			RelatedID:   t.RelatedID,
			RelatedType: t.RelatedType,
			SLADeadline: t.SLADeadline,
			ResolvedAt:  t.ResolvedAt,
			ClosedAt:    t.ClosedAt,
			CreatedAt:   t.CreatedAt,
		})
	}

	resp.OK(c, gin.H{
		"items":     items,
		"total":     total,
		"page":      page,
		"page_size": pageSize,
	})
}

// GetMyTicket GET /api/v1/users/me/tickets/:id
func (a *API) GetMyTicket(c *gin.Context) {
	userID, _ := middleware.UserID(c)
	if userID == 0 {
		resp.Err(c, http.StatusUnauthorized, errcode.CodeUnauthorized)
		return
	}

	id, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	var t model.Ticket
	if err := a.DB.Where("id = ? AND reporter_id = ?", id, userID).First(&t).Error; err != nil {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}

	var messages []model.TicketMessage
	a.DB.Where("ticket_id = ?", id).Order("created_at ASC").Find(&messages)

	type msgItem struct {
		ID         uint64    `json:"id"`
		SenderType string    `json:"sender_type"`
		Content    string    `json:"content"`
		CreatedAt  time.Time `json:"created_at"`
	}
	msgItems := make([]msgItem, 0, len(messages))
	for _, m := range messages {
		msgItems = append(msgItems, msgItem{
			ID:         m.ID,
			SenderType: m.SenderType,
			Content:    m.Content,
			CreatedAt:  m.CreatedAt,
		})
	}

	resp.OK(c, gin.H{
		"ticket": gin.H{
			"id":          t.ID,
			"category":    t.Category,
			"subject":     t.Subject,
			"description": t.Description,
			"status":      t.Status,
			"priority":    t.Priority,
			"related_id":  t.RelatedID,
			"related_type": t.RelatedType,
			"sla_deadline": t.SLADeadline,
			"resolved_at":  t.ResolvedAt,
			"closed_at":    t.ClosedAt,
			"created_at":   t.CreatedAt,
		},
		"messages": msgItems,
	})
}

// AddTicketMessageByUser POST /api/v1/users/me/tickets/:id/messages
func (a *API) AddTicketMessageByUser(c *gin.Context) {
	userID, _ := middleware.UserID(c)
	if userID == 0 {
		resp.Err(c, http.StatusUnauthorized, errcode.CodeUnauthorized)
		return
	}

	id, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	var req struct {
		Content string `json:"content"`
	}
	if err := c.ShouldBindJSON(&req); err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	req.Content = strings.TrimSpace(req.Content)
	if req.Content == "" || len([]rune(req.Content)) > 2000 {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	var t model.Ticket
	if err := a.DB.Where("id = ? AND reporter_id = ?", id, userID).First(&t).Error; err != nil {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}
	if t.Status == "closed" {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	msg := model.TicketMessage{
		TicketID:   id,
		SenderID:   userID,
		SenderType: "user",
		Content:    req.Content,
	}
	if err := a.DB.Create(&msg).Error; err != nil {
		a.Log.Error("create ticket message by user", zap.Error(err))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	resp.OK(c, gin.H{
		"id":         msg.ID,
		"ticket_id":  id,
		"content":    msg.Content,
		"created_at": msg.CreatedAt,
	})
}

// AppealTicket POST /api/v1/users/me/tickets/:id/appeal
func (a *API) AppealTicket(c *gin.Context) {
	userID, _ := middleware.UserID(c)
	if userID == 0 {
		resp.Err(c, http.StatusUnauthorized, errcode.CodeUnauthorized)
		return
	}

	id, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	var t model.Ticket
	if err := a.DB.Where("id = ? AND reporter_id = ?", id, userID).First(&t).Error; err != nil {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}
	if t.Status != "resolved" && t.Status != "closed" {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	now := time.Now()
	if err := a.DB.Model(&t).Updates(map[string]interface{}{
		"status":      "reopened",
		"resolved_at": nil,
		"closed_at":   nil,
		"updated_at":  now,
	}).Error; err != nil {
		a.Log.Error("appeal ticket", zap.Error(err))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	a.Log.Info("user appealed ticket",
		zap.Uint64("ticket_id", id),
		zap.Uint64("user_id", userID),
	)

	resp.OK(c, gin.H{"ticket_id": id, "status": "reopened"})
}

// ---------- shared helpers ----------

func loadUserBrief(db *gorm.DB, userID uint64) gin.H {
	if userID == 0 {
		return nil
	}
	type brief struct {
		ID        uint64
		Username  string
		Nickname  string
		AvatarURL string
	}
	var b brief
	if err := db.Model(&model.User{}).Select("id, username, nickname, avatar_url").First(&b, userID).Error; err == nil {
		return gin.H{
			"id":         b.ID,
			"username":   b.Username,
			"nickname":   b.Nickname,
			"avatar_url": b.AvatarURL,
		}
	}
	return gin.H{"id": userID}
}

func loadUserBriefNull(db *gorm.DB, userID *uint64) gin.H {
	if userID == nil || *userID == 0 {
		return nil
	}
	return loadUserBrief(db, *userID)
}

func loadUserBriefs(db *gorm.DB, idSet map[uint64]bool) map[uint64]gin.H {
	result := make(map[uint64]gin.H, len(idSet))
	if len(idSet) == 0 {
		return result
	}
	ids := make([]uint64, 0, len(idSet))
	for id := range idSet {
		ids = append(ids, id)
	}
	type brief struct {
		ID        uint64
		Username  string
		Nickname  string
		AvatarURL string
	}
	var users []brief
	if err := db.Model(&model.User{}).Select("id, username, nickname, avatar_url").Where("id IN ?", ids).Find(&users).Error; err == nil {
		for _, u := range users {
			result[u.ID] = gin.H{
				"id":         u.ID,
				"username":   u.Username,
				"nickname":   u.Nickname,
				"avatar_url": u.AvatarURL,
			}
		}
	}
	return result
}
