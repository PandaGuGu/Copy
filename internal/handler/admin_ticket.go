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
	assigneeBriefs := loadAdminBriefs(a.DB, aidSet)

	type item struct {
		ID          uint64     `json:"id"`
		UserID      uint64     `json:"user_id"`
		User        gin.H      `json:"user"`
		AssigneeID  *uint64    `json:"assignee_id"`
		Assignee    gin.H      `json:"assignee"`
		Category    string     `json:"category"`
		Title       string     `json:"title"`
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
			UserID:      t.ReporterID,
			User:        userBriefs[t.ReporterID],
			AssigneeID:  t.AssigneeID,
			Category:    t.Category,
			Title:       t.Subject,
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
	var openCount, processingCount, resolvedCount, closedCount int64
	a.DB.Model(&model.Ticket{}).Where("status = 'open'").Count(&openCount)
	a.DB.Model(&model.Ticket{}).Where("status IN ('assigned','processing')").Count(&processingCount)
	a.DB.Model(&model.Ticket{}).Where("status = 'resolved'").Count(&resolvedCount)
	a.DB.Model(&model.Ticket{}).Where("status = 'closed'").Count(&closedCount)

	resp.OK(c, gin.H{
		"items":            items,
		"total":            total,
		"page":             page,
		"page_size":        pageSize,
		"open_count":       openCount,
		"processing_count": processingCount,
		"resolved_count":   resolvedCount,
		"closed_count":     closedCount,
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

	// Load satisfaction if exists
	var sat model.TicketSatisfaction
	satData := gin.H(nil)
	if err := a.DB.Where("ticket_id = ?", id).First(&sat).Error; err == nil {
		satData = gin.H{
			"id":         sat.ID,
			"score":      sat.Score,
			"comment":    sat.Comment,
			"created_at": sat.CreatedAt,
		}
	}

	resp.OK(c, gin.H{
		"id":           t.ID,
		"user_id":      t.ReporterID,
		"user":         loadUserBrief(a.DB, t.ReporterID),
		"assignee_id":  t.AssigneeID,
		"assignee":     loadUserBriefNull(a.DB, t.AssigneeID),
		"category":     t.Category,
		"title":        t.Subject,
		"description":  t.Description,
		"status":       t.Status,
		"priority":     t.Priority,
		"related_id":   t.RelatedID,
		"related_type": t.RelatedType,
		"sla_deadline": t.SLADeadline,
		"resolved_at":  t.ResolvedAt,
		"closed_at":    t.ClosedAt,
		"created_at":   t.CreatedAt,
		"updated_at":   t.UpdatedAt,
		"satisfaction": satData,
		"messages":     msgItems,
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
		AssigneeID interface{} `json:"assignee_id"`
	}
	if err := c.ShouldBindJSON(&req); err != nil {
		a.Log.Error("assign bind failed", zap.Error(err))
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	var aid uint64
	switch v := req.AssigneeID.(type) {
	case float64:
		aid = uint64(v)
	case string:
		aid, err = strconv.ParseUint(v, 10, 64)
		if err != nil {
			resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
			return
		}
	default:
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	if aid == 0 {
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

	slaDeadline := time.Now().Add(2 * time.Hour) // default SLA: 2h from assignment
	updates := map[string]interface{}{
		"assignee_id":  aid,
		"status":       "assigned",
		"sla_deadline": slaDeadline,
		"updated_at":   time.Now(),
	}
	if err := a.DB.Model(&t).Updates(updates).Error; err != nil {
		a.Log.Error("assign ticket", zap.Error(err))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	a.Log.Info("admin assigned ticket",
		zap.Uint64("ticket_id", id),
		zap.Uint64("assignee_id", aid),
		zap.Uint64("operator_id", adminID),
	)

	a.pushTicketNotification(t.ReporterID, id, "assigned", "您的工单已分配给客服处理")

	resp.OK(c, gin.H{"ticket_id": id, "assignee_id": aid, "status": "assigned", "sla_deadline": slaDeadline})
}

// AdminGetSatisfactionStats GET /admin/tickets/satisfaction-stats
// Returns aggregate satisfaction scores and trends.
func (a *API) AdminGetSatisfactionStats(c *gin.Context) {
	type row struct {
		AvgScore  float64 `json:"avg_score"`
		TotalRated int    `json:"total_rated"`
		AdminID    uint64  `json:"admin_id"`
		AdminName  string  `json:"admin_name"`
		Month      string  `json:"month"`
	}

	var monthFilter string
	if m := c.Query("months"); m != "" {
		monthFilter = m // e.g. "3" = last 3 months
	}
	if monthFilter == "" {
		monthFilter = "3"
	}

	var overall struct {
		AvgScore  float64 `json:"avg_score"`
		Total     int64   `json:"total"`
		DistScore map[int]int `json:"distribution"`
	}
	a.DB.Raw(`SELECT COALESCE(AVG(score),0) as avg_score, COUNT(*) as total FROM ticket_satisfactions`).Scan(&overall)
	
	var dist []struct {
		Score int `json:"score"`
		Count int `json:"count"`
	}
	a.DB.Raw(`SELECT score, COUNT(*) as count FROM ticket_satisfactions GROUP BY score ORDER BY score`).Scan(&dist)
	overall.DistScore = make(map[int]int)
	for _, d := range dist {
		overall.DistScore[d.Score] = d.Count
	}

	// By admin (per assigned ticket)
	var byAdmin []gin.H
	rows, _ := a.DB.Raw(`
		SELECT ts.admin_id, a.display_name as admin_name,
			ROUND(AVG(ts.score),2) as avg_score, COUNT(*) as total_rated
		FROM ticket_satisfactions ts
		JOIN tickets t ON t.id = ts.ticket_id
		LEFT JOIN admins a ON a.id = ts.admin_id
		GROUP BY ts.admin_id, a.display_name
		ORDER BY avg_score DESC
		LIMIT 20
	`).Rows()
	if rows != nil {
		for rows.Next() {
			var adminID uint64
			var adminName string
			var avg float64
			var cnt int
			rows.Scan(&adminID, &adminName, &avg, &cnt)
			byAdmin = append(byAdmin, gin.H{
				"admin_id": adminID, "admin_name": adminName,
				"avg_score": avg, "total_rated": cnt,
			})
		}
		rows.Close()
	}

	resp.OK(c, gin.H{
		"overall":  overall,
		"by_admin": byAdmin,
	})
}

// AdminGetTicketStats GET /admin/tickets/stats
// Returns dashboard counters for the ticket system.
func (a *API) AdminGetTicketStats(c *gin.Context) {
	var open, assigned, processing, resolved, closed, overdue int64
	a.DB.Model(&model.Ticket{}).Where("status = 'open'").Count(&open)
	a.DB.Model(&model.Ticket{}).Where("status = 'assigned'").Count(&assigned)
	a.DB.Model(&model.Ticket{}).Where("status = 'processing'").Count(&processing)
	a.DB.Model(&model.Ticket{}).Where("status = 'resolved'").Count(&resolved)
	a.DB.Model(&model.Ticket{}).Where("status = 'closed'").Count(&closed)
	a.DB.Model(&model.Ticket{}).Where("sla_deadline IS NOT NULL AND sla_deadline < NOW() AND status NOT IN ('closed','resolved')").Count(&overdue)

	// Average satisfaction this month
	var avgSat struct{ Avg float64 }
	a.DB.Raw(`SELECT COALESCE(AVG(score),0) as avg FROM ticket_satisfactions WHERE created_at >= DATE_FORMAT(NOW(),'%Y-%m-01')`).Scan(&avgSat)

	// Avg response time (minutes) for tickets resolved this week
	var avgResp struct{ Min float64 }
	a.DB.Raw(`SELECT COALESCE(AVG(TIMESTAMPDIFF(MINUTE, created_at, resolved_at)),0) as min FROM tickets WHERE status = 'resolved' AND resolved_at >= DATE_SUB(NOW(), INTERVAL 7 DAY)`).Scan(&avgResp)

	resp.OK(c, gin.H{
		"counts": gin.H{
			"open": open, "assigned": assigned, "processing": processing,
			"resolved": resolved, "closed": closed, "overdue": overdue,
		},
		"avg_satisfaction_this_month": avgSat.Avg,
		"avg_response_minutes_this_week": int(avgResp.Min),
	})
}

// AdminAutoAssignTicket POST /admin/tickets/:id/auto-assign
// Auto-assigns a ticket to the admin with the least open assignments.
func (a *API) AdminAutoAssignTicket(c *gin.Context) {
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

	// Find admin with fewest open tickets (round-robin via workload)
	var bestAdmin struct {
		AdminID uint64
		Count   int64
	}
	a.DB.Raw(`
		SELECT admins.id as admin_id, COALESCE(COUNT(tickets.id),0) as count
		FROM admins
		LEFT JOIN tickets ON tickets.assignee_id = admins.id AND tickets.status IN ('assigned','processing')
		WHERE admins.status = 'active'
		GROUP BY admins.id
		ORDER BY count ASC
		LIMIT 1
	`).Scan(&bestAdmin)

	if bestAdmin.AdminID == 0 {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}

	slaDeadline := time.Now().Add(2 * time.Hour)
	a.DB.Model(&t).Updates(map[string]interface{}{
		"assignee_id":  bestAdmin.AdminID,
		"status":       "assigned",
		"sla_deadline": slaDeadline,
		"updated_at":   time.Now(),
	})

	a.pushTicketNotification(t.ReporterID, id, "assigned", "工单已自动分配，请等待客服处理")
	resp.OK(c, gin.H{"ticket_id": id, "assignee_id": bestAdmin.AdminID, "status": "assigned"})
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

	a.pushTicketNotification(t.ReporterID, id, "status_"+req.Status, "工单状态已更新为"+statusLabel(req.Status))

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

	a.pushTicketNotification(t.ReporterID, id, "new_reply", truncateText(req.Content, 60))

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

	a.pushTicketNotification(t.ReporterID, id, "closed", "工单已关闭")

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

// PostTicketSatisfaction POST /api/v1/tickets/:id/satisfaction
func (a *API) PostTicketSatisfaction(c *gin.Context) {
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
	if t.Status != "resolved" {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	var req struct {
		Score   int    `json:"score"`
		Comment string `json:"comment"`
	}
	if err := c.ShouldBindJSON(&req); err != nil || req.Score < 1 || req.Score > 5 {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	s := model.TicketSatisfaction{TicketID: id, UserID: userID, Score: req.Score, Comment: req.Comment}
	if err := a.DB.Create(&s).Error; err != nil {
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}
	a.DB.Model(&t).Update("status", "closed")
	resp.OK(c, gin.H{"id": s.ID})
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

// loadAdminBriefs loads basic admin info (from admins table, not users table).
func loadAdminBriefs(db *gorm.DB, idSet map[uint64]bool) map[uint64]gin.H {
	result := make(map[uint64]gin.H, len(idSet))
	if len(idSet) == 0 {
		return result
	}
	ids := make([]uint64, 0, len(idSet))
	for id := range idSet {
		ids = append(ids, id)
	}
	type brief struct {
		ID          uint64
		Username    string
		DisplayName string
	}
	var admins []brief
	if err := db.Model(&model.Admin{}).Select("id, username, display_name").Where("id IN ?", ids).Find(&admins).Error; err == nil {
		for _, a := range admins {
			name := a.DisplayName
			if name == "" {
				name = a.Username
			}
			result[a.ID] = gin.H{
				"id":       a.ID,
				"username": a.Username,
				"nickname": name,
			}
		}
	}
	return result
}

// pushTicketNotification sends a real-time notification to a ticket owner via WebSocket.
func (a *API) pushTicketNotification(userID uint64, ticketID uint64, eventType string, summary string) {
	if a.ChatHub == nil || userID == 0 {
		return
	}
	a.ChatHub.PushJSON(userID, gin.H{
		"type": "ticket_" + eventType,
		"data": gin.H{
			"ticket_id": ticketID,
			"summary":   summary,
		},
	})
}

// statusLabel converts a ticket status to Chinese label.
func statusLabel(s string) string {
	switch s {
	case "open": return "待处理"
	case "assigned": return "已分配"
	case "processing": return "处理中"
	case "resolved": return "已解决"
	case "closed": return "已关闭"
	case "reopened": return "已重开"
	default: return s
	}
}
