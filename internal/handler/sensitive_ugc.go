package handler

import (
	"net/http"

	"github.com/gin-gonic/gin"
	"go.uber.org/zap"

	"minibili/internal/errcode"
	"minibili/internal/pkg/resp"
	"minibili/internal/pkg/sensitive"
)

// rejectIfCommentSensitive blocks UGC comment/reply text that hits the configured word list.
func (a *API) rejectIfCommentSensitive(c *gin.Context, content string) bool {
	return a.rejectIfSensitive(c, content, errcode.CodeCommentSensitive)
}

func (a *API) rejectIfSensitive(c *gin.Context, content string, code int) bool {
	if a.Sens == nil {
		return false
	}
	if err := a.Sens.Check(content); err != nil {
		if _, ok := err.(sensitive.ErrBlocked); ok {
			resp.Err(c, http.StatusBadRequest, code)
			return true
		}
		if a.Log != nil {
			a.Log.Error("sensitive check", zap.Error(err))
		}
		resp.Err(c, http.StatusInternalServerError, errcode.CodeInternalError)
		return true
	}

	// AI semantic review (optional; fail-open on LLM errors).
	if a.AISens != nil && a.AISens.Enabled() {
		blocked, aiErr := a.AISens.Review(c.Request.Context(), content)
		if aiErr != nil {
			if a.Log != nil {
				a.Log.Warn("ai ugc review error (fail-open)", zap.Error(aiErr))
			}
		} else if blocked {
			resp.Err(c, http.StatusBadRequest, code)
			return true
		}
	}
	return false
}
