package handler

import (
	"net/http"
	"strconv"
	"strings"
	"time"

	"github.com/gin-gonic/gin"
	"go.uber.org/zap"

	"minibili/internal/errcode"
	"minibili/internal/middleware"
	"minibili/internal/model"
	"minibili/internal/pkg/resp"
)

// ──────────────────────────────────────────────
// CS conversation admin handlers
// ──────────────────────────────────────────────

// AdminListCSConversations GET /admin/cs/conversations
func (a *API) AdminListCSConversations(c *gin.Context) {
	page, _ := strconv.Atoi(c.DefaultQuery("page", "1"))
	pageSize, _ := strconv.Atoi(c.DefaultQuery("page_size", "20"))
	if page < 1 {
		page = 1
	}
	if pageSize < 1 || pageSize > 50 {
		pageSize = 20
	}
	statusQ := strings.TrimSpace(c.Query("status"))

	q := a.DB.Model(&model.CSConversation{})
	if statusQ != "" {
		q = q.Where("status = ?", statusQ)
	}

	var total int64
	if err := q.Count(&total).Error; err != nil {
		a.Log.Error("count cs conversations failed", zap.Error(err))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	offset := (page - 1) * pageSize
	var rows []model.CSConversation
	if err := q.Order("updated_at DESC").Offset(offset).Limit(pageSize).Find(&rows).Error; err != nil {
		a.Log.Error("list cs conversations failed", zap.Error(err))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	// Batch load user & admin info
	uidSet := make(map[uint64]bool)
	aidSet := make(map[uint64]bool)
	for i := range rows {
		uidSet[rows[i].UserID] = true
		if rows[i].AdminID != nil {
			aidSet[*rows[i].AdminID] = true
		}
	}

	type brief struct {
		ID       uint64
		Username string
		Nickname string
	}
	var uList []brief
	userBriefs := make(map[uint64]gin.H)
	if len(uidSet) > 0 {
		uidList := make([]uint64, 0, len(uidSet))
		for id := range uidSet { uidList = append(uidList, id) }
		_ = a.DB.Model(&model.User{}).Select("id, username, nickname").Where("id IN ?", uidList).Find(&uList).Error
		for _, u := range uList {
			userBriefs[u.ID] = gin.H{"id": u.ID, "username": u.Username, "nickname": u.Nickname}
		}
	}
	var aList []brief
	adminBriefs := make(map[uint64]gin.H)
	if len(aidSet) > 0 {
		aidList := make([]uint64, 0, len(aidSet))
		for id := range aidSet { aidList = append(aidList, id) }
		_ = a.DB.Model(&model.Admin{}).Select("id, username, display_name").Where("id IN ?", aidList).Find(&aList).Error
		for _, a := range aList {
			adminBriefs[a.ID] = gin.H{"id": a.ID, "username": a.Username, "nickname": a.Nickname}
		}
	}

	items := make([]gin.H, 0, len(rows))
	for i := range rows {
		conv := &rows[i]
		h := gin.H{
			"id":         conv.ID,
			"user_id":    conv.UserID,
			"user":       userBriefs[conv.UserID],
			"admin_id":   conv.AdminID,
			"admin":      convAdminBrief(adminBriefs, conv.AdminID),
			"ticket_id":  conv.TicketID,
			"status":     conv.Status,
			"created_at": conv.CreatedAt,
			"updated_at": conv.UpdatedAt,
		}
		// Message count
		var msgCount int64
		var lastMsg struct{ Content string }
		_ = a.DB.Model(&model.CSMessage{}).Where("conversation_id = ?", conv.ID).Count(&msgCount).Error
		_ = a.DB.Model(&model.CSMessage{}).Select("content").Where("conversation_id = ?", conv.ID).Order("created_at DESC").Limit(1).Scan(&lastMsg).Error
		h["message_count"] = msgCount
		h["last_message"] = lastMsg.Content
		items = append(items, h)
	}

	resp.OK(c, gin.H{
		"items":     items,
		"total":     total,
		"page":      page,
		"page_size": pageSize,
	})
}

// AdminGetCSConversation GET /admin/cs/conversations/:id
func (a *API) AdminGetCSConversation(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil || id == 0 {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	var conv model.CSConversation
	if err := a.DB.First(&conv, id).Error; err != nil {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}

	// Load user info
	var u model.User
	_ = a.DB.Select("id, username, nickname, avatar_url").First(&u, conv.UserID).Error

	// Load admin info if assigned
	adminBrief := gin.H(nil)
	if conv.AdminID != nil {
		var adm model.Admin
		if a.DB.Select("id, username, display_name").First(&adm, *conv.AdminID).Error == nil {
			adminBrief = gin.H{"id": adm.ID, "username": adm.Username, "nickname": adm.DisplayName}
		}
	}

	// Load messages
	var msgs []model.CSMessage
	_ = a.DB.Where("conversation_id = ?", id).Order("created_at ASC").Find(&msgs).Error
	msgItems := make([]gin.H, 0, len(msgs))
	for i := range msgs {
		msgItems = append(msgItems, gin.H{
			"id":          msgs[i].ID,
			"sender_id":   msgs[i].SenderID,
			"sender_type": msgs[i].SenderType,
			"is_admin":    msgs[i].SenderType == "admin",
			"content":     msgs[i].Content,
			"created_at":  msgs[i].CreatedAt,
		})
	}

	resp.OK(c, gin.H{
		"id": conv.ID,
		"user": gin.H{
			"id":         u.ID,
			"username":   model.DisplayUsername(&u),
			"nickname":   u.Nickname,
			"avatar_url": u.AvatarURL,
		},
		"admin_id":  conv.AdminID,
		"admin":     adminBrief,
		"ticket_id": conv.TicketID,
		"status":    conv.Status,
		"messages":  msgItems,
		"created_at": conv.CreatedAt,
		"updated_at": conv.UpdatedAt,
	})
}

// AdminAssignCSConversation POST /admin/cs/conversations/:id/assign
func (a *API) AdminAssignCSConversation(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil || id == 0 {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	adminID, _ := middleware.AdminID(c)

	var conv model.CSConversation
	if err := a.DB.First(&conv, id).Error; err != nil {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}

	now := time.Now()
	updates := map[string]interface{}{
		"admin_id":   adminID,
		"status":     "active",
		"updated_at": now,
	}
	if err := a.DB.Model(&conv).Updates(updates).Error; err != nil {
		a.Log.Error("assign cs conversation failed", zap.Error(err))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	a.Log.Info("cs conversation assigned",
		zap.Uint64("conversation_id", id),
		zap.Uint64("admin_id", adminID))
	resp.OK(c, gin.H{"status": "active", "admin_id": adminID})
}

// AdminSendCSMessage POST /admin/cs/conversations/:id/messages
func (a *API) AdminSendCSMessage(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil || id == 0 {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	adminID, _ := middleware.AdminID(c)

	var body struct {
		Content string `json:"content" binding:"required"`
	}
	if err := c.ShouldBindJSON(&body); err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	body.Content = strings.TrimSpace(body.Content)
	if body.Content == "" {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	var conv model.CSConversation
	if err := a.DB.First(&conv, id).Error; err != nil {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}
	if conv.Status == "closed" {
		resp.Err(c, http.StatusForbidden, errcode.CodeForbidden)
		return
	}

	msg := model.CSMessage{
		ConversationID: id,
		SenderID:       adminID,
		SenderType:     "admin",
		Content:        body.Content,
		CreatedAt:      time.Now(),
	}
	if err := a.DB.Create(&msg).Error; err != nil {
		a.Log.Error("create cs admin message failed", zap.Error(err))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	// Update conversation timestamp and set to active if not already.
	// Auto-assign to the responding admin if unassigned.
	updates := map[string]interface{}{
		"updated_at": time.Now(),
		"status":     "active",
	}
	if conv.AdminID == nil {
		updates["admin_id"] = adminID
	}
	a.DB.Model(&conv).Updates(updates)

	a.Log.Info("cs admin message sent",
		zap.Uint64("conversation_id", id),
		zap.Uint64("message_id", msg.ID))

	// Push real-time notification to user
	a.pushCSNotification(conv.UserID, "new_message", gin.H{
		"conversation_id": id,
		"message":         gin.H{"id": msg.ID, "content": truncateText(body.Content, 60), "sender_type": "admin", "created_at": msg.CreatedAt},
	})

	resp.OK(c, gin.H{"id": msg.ID, "created_at": msg.CreatedAt})
}

// AdminCloseCSConversation POST /admin/cs/conversations/:id/close
func (a *API) AdminCloseCSConversation(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil || id == 0 {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	var conv model.CSConversation
	if err := a.DB.First(&conv, id).Error; err != nil {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}

	now := time.Now()
	if err := a.DB.Model(&conv).Updates(map[string]interface{}{
		"status":     "closed",
		"updated_at": now,
	}).Error; err != nil {
		a.Log.Error("close cs conversation failed", zap.Error(err))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	a.Log.Info("cs conversation closed", zap.Uint64("conversation_id", id))

	// Push real-time notification
	a.pushCSNotification(conv.UserID, "conversation_closed", gin.H{
		"conversation_id": id,
	})

	resp.OK(c, gin.H{"status": "closed"})
}

// pushCSNotification sends a real-time notification to a user via ChatHub.
func (a *API) pushCSNotification(userID uint64, eventType string, data gin.H) {
	if a.ChatHub == nil || userID == 0 {
		return
	}
	a.ChatHub.PushJSON(userID, gin.H{
		"type": "cs_" + eventType,
		"data": data,
	})
}

// truncateText truncates text to maxLen characters, adding "..." if truncated.
func truncateText(s string, maxLen int) string {
	runes := []rune(s)
	if len(runes) <= maxLen {
		return s
	}
	return string(runes[:maxLen]) + "..."
}

// convAdminBrief safely extracts admin brief from map, handling nil AdminID.
func convAdminBrief(m map[uint64]gin.H, adminID *uint64) gin.H {
	if adminID == nil {
		return nil
	}
	return m[*adminID]
}

// AdminListCSTemplates GET /admin/cs/templates
func (a *API) AdminListCSTemplates(c *gin.Context) {
	categoryQ := strings.TrimSpace(c.Query("category"))

	q := a.DB.Model(&model.CSTemplate{})
	if categoryQ != "" {
		q = q.Where("category = ?", categoryQ)
	}

	var rows []model.CSTemplate
	if err := q.Order("category ASC, created_at DESC").Find(&rows).Error; err != nil {
		a.Log.Error("list cs templates failed", zap.Error(err))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	items := make([]gin.H, 0, len(rows))
	for i := range rows {
		items = append(items, gin.H{
			"id":         rows[i].ID,
			"name":       rows[i].Name,
			"category":   rows[i].Category,
			"content":    rows[i].Content,
			"created_by": rows[i].CreatedBy,
			"created_at": rows[i].CreatedAt,
			"updated_at": rows[i].UpdatedAt,
		})
	}
	resp.OK(c, gin.H{"templates": items})
}

// AdminCreateCSTemplate POST /admin/cs/templates
func (a *API) AdminCreateCSTemplate(c *gin.Context) {
	adminID, _ := middleware.AdminID(c)

	var body struct {
		Name     string `json:"name"     binding:"required"`
		Category string `json:"category" binding:"required"`
		Content  string `json:"content"  binding:"required"`
	}
	if err := c.ShouldBindJSON(&body); err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	tmpl := model.CSTemplate{
		Name:      strings.TrimSpace(body.Name),
		Category:  strings.TrimSpace(body.Category),
		Content:   strings.TrimSpace(body.Content),
		CreatedBy: adminID,
		CreatedAt: time.Now(),
		UpdatedAt: time.Now(),
	}
	if err := a.DB.Create(&tmpl).Error; err != nil {
		a.Log.Error("create cs template failed", zap.Error(err))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	a.Log.Info("cs template created", zap.Uint64("template_id", tmpl.ID))
	resp.OK(c, gin.H{"id": tmpl.ID})
}

// AdminUpdateCSTemplate PUT /admin/cs/templates/:id
func (a *API) AdminUpdateCSTemplate(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil || id == 0 {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	var tmpl model.CSTemplate
	if err := a.DB.First(&tmpl, id).Error; err != nil {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}

	var body struct {
		Name     *string `json:"name"`
		Category *string `json:"category"`
		Content  *string `json:"content"`
	}
	if err := c.ShouldBindJSON(&body); err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	updates := map[string]interface{}{"updated_at": time.Now()}
	if body.Name != nil {
		updates["name"] = strings.TrimSpace(*body.Name)
	}
	if body.Category != nil {
		updates["category"] = strings.TrimSpace(*body.Category)
	}
	if body.Content != nil {
		updates["content"] = strings.TrimSpace(*body.Content)
	}

	if err := a.DB.Model(&tmpl).Updates(updates).Error; err != nil {
		a.Log.Error("update cs template failed", zap.Error(err))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	a.Log.Info("cs template updated", zap.Uint64("template_id", id))
	resp.OK(c, gin.H{"status": "updated"})
}

// AdminDeleteCSTemplate DELETE /admin/cs/templates/:id
func (a *API) AdminDeleteCSTemplate(c *gin.Context) {
	id, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil || id == 0 {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	var tmpl model.CSTemplate
	if err := a.DB.First(&tmpl, id).Error; err != nil {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}

	if err := a.DB.Delete(&tmpl).Error; err != nil {
		a.Log.Error("delete cs template failed", zap.Error(err))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	a.Log.Info("cs template deleted", zap.Uint64("template_id", id))
	resp.OK(c, gin.H{"status": "deleted"})
}

// ──────────────────────────────────────────────
// User-facing CS handlers
// ──────────────────────────────────────────────

// PostCSConversation POST /cs/conversations
func (a *API) PostCSConversation(c *gin.Context) {
	userID, ok := middleware.UserID(c)
	if !ok {
		resp.Err(c, http.StatusUnauthorized, errcode.CodeUnauthorized)
		return
	}

	var body struct {
		Message string `json:"message" binding:"required"`
	}
	_ = c.ShouldBindJSON(&body)
	body.Message = strings.TrimSpace(body.Message)
	if body.Message == "" {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	now := time.Now()
	conv := model.CSConversation{
		UserID:    userID,
		Status:    "waiting",
		CreatedAt: now,
		UpdatedAt: now,
	}
	if err := a.DB.Create(&conv).Error; err != nil {
		a.Log.Error("create cs conversation failed", zap.Error(err))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	msg := model.CSMessage{
		ConversationID: conv.ID,
		SenderID:       userID,
		SenderType:     "user",
		Content:        body.Message,
		CreatedAt:      now,
	}
	if err := a.DB.Create(&msg).Error; err != nil {
		a.Log.Error("create cs first message failed", zap.Error(err))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	a.Log.Info("cs conversation started",
		zap.Uint64("conversation_id", conv.ID),
		zap.Uint64("user_id", userID))
	resp.OK(c, gin.H{
		"id":         conv.ID,
		"message_id": msg.ID,
		"status":     "waiting",
		"created_at": now,
	})
}

// ListMyCSConversations GET /users/me/cs/conversations
func (a *API) ListMyCSConversations(c *gin.Context) {
	userID, ok := middleware.UserID(c)
	if !ok {
		resp.Err(c, http.StatusUnauthorized, errcode.CodeUnauthorized)
		return
	}

	page, _ := strconv.Atoi(c.DefaultQuery("page", "1"))
	pageSize, _ := strconv.Atoi(c.DefaultQuery("page_size", "20"))
	if page < 1 {
		page = 1
	}
	if pageSize < 1 || pageSize > 50 {
		pageSize = 20
	}

	q := a.DB.Model(&model.CSConversation{}).Where("user_id = ?", userID)

	var total int64
	if err := q.Count(&total).Error; err != nil {
		a.Log.Error("count user cs conversations failed", zap.Error(err))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	offset := (page - 1) * pageSize
	var rows []model.CSConversation
	if err := q.Order("updated_at DESC").Offset(offset).Limit(pageSize).Find(&rows).Error; err != nil {
		a.Log.Error("list user cs conversations failed", zap.Error(err))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	items := make([]gin.H, 0, len(rows))
	for i := range rows {
		conv := &rows[i]
		var lastMsg model.CSMessage
		_ = a.DB.Where("conversation_id = ?", conv.ID).Order("created_at DESC").Limit(1).First(&lastMsg).Error

		h := gin.H{
			"id":         conv.ID,
			"status":     conv.Status,
			"created_at": conv.CreatedAt,
			"updated_at": conv.UpdatedAt,
		}
		if lastMsg.ID != 0 {
			h["last_message"] = gin.H{
				"id":          lastMsg.ID,
				"content":     truncateStr(lastMsg.Content, 80),
				"sender_type": lastMsg.SenderType,
				"created_at":  lastMsg.CreatedAt,
			}
		}
		items = append(items, h)
	}

	resp.OK(c, gin.H{
		"items":     items,
		"total":     total,
		"page":      page,
		"page_size": pageSize,
	})
}

// GetMyCSConversation GET /users/me/cs/conversations/:id
func (a *API) GetMyCSConversation(c *gin.Context) {
	userID, ok := middleware.UserID(c)
	if !ok {
		resp.Err(c, http.StatusUnauthorized, errcode.CodeUnauthorized)
		return
	}

	id, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil || id == 0 {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	var conv model.CSConversation
	if err := a.DB.Where("id = ? AND user_id = ?", id, userID).First(&conv).Error; err != nil {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}

	var msgs []model.CSMessage
	_ = a.DB.Where("conversation_id = ?", id).Order("created_at ASC").Find(&msgs).Error
	msgItems := make([]gin.H, 0, len(msgs))
	for i := range msgs {
		msgItems = append(msgItems, gin.H{
			"id":          msgs[i].ID,
			"sender_type": msgs[i].SenderType,
			"content":     msgs[i].Content,
			"created_at":  msgs[i].CreatedAt,
		})
	}

	resp.OK(c, gin.H{
		"id":         conv.ID,
		"status":     conv.Status,
		"messages":   msgItems,
		"created_at": conv.CreatedAt,
		"updated_at": conv.UpdatedAt,
	})
}

// SendCSMessageByUser POST /users/me/cs/conversations/:id/messages
func (a *API) SendCSMessageByUser(c *gin.Context) {
	userID, ok := middleware.UserID(c)
	if !ok {
		resp.Err(c, http.StatusUnauthorized, errcode.CodeUnauthorized)
		return
	}

	id, err := strconv.ParseUint(c.Param("id"), 10, 64)
	if err != nil || id == 0 {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	var conv model.CSConversation
	if err := a.DB.Where("id = ? AND user_id = ?", id, userID).First(&conv).Error; err != nil {
		resp.Err(c, http.StatusNotFound, errcode.CodeNotFound)
		return
	}
	if conv.Status == "closed" {
		resp.Err(c, http.StatusForbidden, errcode.CodeForbidden)
		return
	}

	var body struct {
		Content string `json:"content" binding:"required"`
	}
	if err := c.ShouldBindJSON(&body); err != nil {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}
	body.Content = strings.TrimSpace(body.Content)
	if body.Content == "" {
		resp.Err(c, http.StatusBadRequest, errcode.CodeParamError)
		return
	}

	now := time.Now()
	msg := model.CSMessage{
		ConversationID: id,
		SenderID:       userID,
		SenderType:     "user",
		Content:        body.Content,
		CreatedAt:      now,
	}
	if err := a.DB.Create(&msg).Error; err != nil {
		a.Log.Error("create cs user message failed", zap.Error(err))
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return
	}

	a.DB.Model(&conv).Updates(map[string]interface{}{
		"updated_at": now,
	})

	a.Log.Info("cs user message sent",
		zap.Uint64("conversation_id", id),
		zap.Uint64("user_id", userID))
	resp.OK(c, gin.H{"id": msg.ID, "created_at": now})
}
