// Package usercap is the single source of truth for user capabilities
// (细粒度能力限制). Each capability maps to the write endpoints that consume it.
package usercap

import "strings"

// Cap describes one user capability.
type Cap struct {
	Name  string `json:"name"`
	Label string `json:"label"`
}

// All lists the enabled capabilities. Init order matters: append new ones.
var All = []Cap{
	{Name: "publish", Label: "发布内容"},
	{Name: "comment", Label: "评论"},
	{Name: "danmaku", Label: "弹幕"},
	{Name: "dm", Label: "私信"},
	{Name: "live", Label: "直播"},
	{Name: "coin", Label: "投币"},
}

// NameCode maps a capability name to a stable numeric code.
var NameCode = func() map[string]int {
	m := map[string]int{}
	for i, c := range All {
		m[c.Name] = i + 1
	}
	return m
}()

// CodeName is the inverse of NameCode.
var CodeName = func() map[int]string {
	m := map[int]string{}
	for name, code := range NameCode {
		m[code] = name
	}
	return m
}()

// Valid reports whether name is a known capability.
func Valid(name string) bool {
	_, ok := NameCode[name]
	return ok
}

// Label returns the human label for a capability name, or the raw name.
func Label(name string) string {
	for _, c := range All {
		if c.Name == name {
			return c.Label
		}
	}
	return name
}

// Code returns the stable numeric code for a capability name, or 0.
func Code(name string) int { return NameCode[name] }

// Name returns the capability name for a stable code, or "".
func Name(code int) string { return CodeName[code] }

// CapabilityOf resolves which capability, if any, a writable request consumes.
// Reads (non-POST) return no capability, so restricted users can still read.
func CapabilityOf(method, path string) (string, bool) {
	m := strings.ToUpper(strings.TrimSpace(method))
	low := strings.ToLower(strings.TrimSpace(path))
	if m != "POST" || low == "" {
		return "", false
	}
	switch {
	case strings.Contains(low, "/coin"):
		return "coin", true
	case strings.Contains(low, "/comments"):
		return "comment", true
	case strings.Contains(low, "/danmaku"):
		return "danmaku", true
	case strings.HasPrefix(low, "/api/v1/dm/"):
		return "dm", true
	case strings.HasPrefix(low, "/api/v1/live/"):
		return "live", true
	}
	// Content creation endpoints (exact match to avoid sub-resources like like/coin).
	switch low {
	case "/api/v1/videos", "/api/v1/articles", "/api/v1/user-dynamics":
		return "publish", true
	}
	return "", false
}
