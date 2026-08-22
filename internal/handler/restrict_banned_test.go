package handler

import (
	"bytes"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/stretchr/testify/require"
	"golang.org/x/crypto/bcrypt"

	"minibili/internal/model"
)

// TestBannedUserRestrictedSession verifies 受限登录: a banned user can log in
// and reach appeal/status endpoints, but authenticated platform calls are denied,
// while a normal user is unaffected.
func TestBannedUserRestrictedSession(t *testing.T) {
	api, r, _ := newTestAPI(t)

	hash, err := bcrypt.GenerateFromPassword([]byte("password12"), bcrypt.DefaultCost)
	require.NoError(t, err)

	normal := model.User{Username: "normal_user", PasswordHash: string(hash), Status: "active"}
	require.NoError(t, api.DB.Create(&normal).Error)
	banned := model.User{Username: "banned_user", PasswordHash: string(hash), Status: "banned", BannedReason: "违规"}
	require.NoError(t, api.DB.Create(&banned).Error)

	login := func(username string) string {
		var buf bytes.Buffer
		_ = json.NewEncoder(&buf).Encode(map[string]string{"username": username, "password": "password12"})
		req := httptest.NewRequest(http.MethodPost, "/api/v1/auth/login", &buf)
		req.Header.Set("Content-Type", "application/json")
		w := httptest.NewRecorder()
		r.ServeHTTP(w, req)
		require.Equal(t, http.StatusOK, w.Code, w.Body.String())
		var out struct {
			Data struct {
				AccessToken string `json:"access_token"`
			} `json:"data"`
		}
		require.NoError(t, json.Unmarshal(w.Body.Bytes(), &out))
		return out.Data.AccessToken
	}

	normalTok := login("normal_user")
	bannedTok := login("banned_user")
	require.NotEmpty(t, normalTok)
	require.NotEmpty(t, bannedTok)

	get := func(path, token string) (int, []byte) {
		req := httptest.NewRequest(http.MethodGet, path, nil)
		req.Header.Set("Authorization", "Bearer "+token)
		w := httptest.NewRecorder()
		r.ServeHTTP(w, req)
		return w.Code, w.Body.Bytes()
	}

	// Normal user: unrestricted.
	code, _ := get("/api/v1/users/me/coin-ledger", normalTok)
	require.Equal(t, http.StatusOK, code)

	// Banned user may read own status (users/me).
	code, body := get("/api/v1/users/me", bannedTok)
	require.Equal(t, http.StatusOK, code, string(body))
	var me struct {
		Data struct {
			Status       string `json:"status"`
			BannedReason string `json:"banned_reason"`
		} `json:"data"`
	}
	require.NoError(t, json.Unmarshal(body, &me))
	require.Equal(t, "banned", me.Data.Status)
	require.Equal(t, "违规", me.Data.BannedReason)

	// Banned user may reach appeal endpoints.
	code, _ = get("/api/v1/appeals/me", bannedTok)
	require.Equal(t, http.StatusOK, code)

	// Banned user denied on a normal platform endpoint.
	code, _ = get("/api/v1/users/me/coin-ledger", bannedTok)
	require.Equal(t, http.StatusForbidden, code)
}
