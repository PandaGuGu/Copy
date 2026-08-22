package usercap

import "testing"

func TestCapabilityOf_Publish(t *testing.T) {
	// Reads are never restricted.
	if _, ok := CapabilityOf("GET", "/api/v1/videos/5"); ok {
		t.Error("GET must not resolve a capability")
	}
	for _, p := range []string{"/api/v1/videos", "/api/v1/articles", "/api/v1/user-dynamics"} {
		if got, ok := CapabilityOf("POST", p); !ok || got != "publish" {
			t.Errorf("POST %s => publish, got %q ok=%v", p, got, ok)
		}
	}
}

func TestCapabilityOf_Subresources(t *testing.T) {
	cases := []struct {
		method, path, want string
	}{
		{"POST", "/api/v1/videos/5/coin", "coin"},
		{"POST", "/api/v1/articles/5/coin", "coin"},
		{"POST", "/api/v1/videos/5/comments", "comment"},
		{"POST", "/api/v1/videos/5/danmaku", "danmaku"},
		{"POST", "/api/v1/dm/3/messages", "dm"},
		{"POST", "/api/v1/live/rooms", "live"},
	}
	for _, c := range cases {
		got, ok := CapabilityOf(c.method, c.path)
		if !ok || got != c.want {
			t.Errorf("%s %s => %s, got %q ok=%v", c.method, c.path, c.want, got, ok)
		}
	}
}

func TestCapabilityOf_None(t *testing.T) {
	for _, c := range [][2]string{
		{"POST", "/api/v1/videos/5/like"},
		{"POST", "/api/v1/users/5/follow"},
		{"POST", "/api/v1/users/me/avatar"},
		{"POST", "/api/v1/reports"},
	} {
		if _, ok := CapabilityOf(c[0], c[1]); ok {
			t.Errorf("expected no capability for %s %s", c[0], c[1])
		}
	}
}

func TestCodeRoundTrip(t *testing.T) {
	for _, c := range All {
		code := Code(c.Name)
		if code <= 0 {
			t.Errorf("no code for %s", c.Name)
		}
		if Name(code) != c.Name {
			t.Errorf("code %d != %s", code, c.Name)
		}
	}
	if !Valid("comment") || Valid("nope") {
		t.Error("Valid mismatch")
	}
}
