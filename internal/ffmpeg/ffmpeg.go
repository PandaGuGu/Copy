package ffmpeg

import (
	"bytes"
	"fmt"
	"os/exec"
	"strconv"
	"strings"
)

var ffprobeExe = "ffprobe"
var ffmpegExe = "ffmpeg"

// Init sets ffprobe/ffmpeg executable names or absolute paths (from FFPROBE_PATH / FFMPEG_PATH).
func Init(ffprobePath, ffmpegPath string) {
	if strings.TrimSpace(ffprobePath) != "" {
		ffprobeExe = strings.TrimSpace(ffprobePath)
	}
	if strings.TrimSpace(ffmpegPath) != "" {
		ffmpegExe = strings.TrimSpace(ffmpegPath)
	}
}

// FFprobeExe returns the configured ffprobe command (for logs).
func FFprobeExe() string { return ffprobeExe }

// FFmpegExe returns the configured ffmpeg command (for logs).
func FFmpegExe() string { return ffmpegExe }

// CheckFFprobe runs ffprobe -version once at startup to surface PATH / 安装问题。
func CheckFFprobe() error {
	return exec.Command(ffprobeExe, "-version").Run()
}

// ProbeDurationSeconds returns media duration using ffprobe.
func ProbeDurationSeconds(path string) (float64, error) {
	cmd := exec.Command(ffprobeExe,
		"-v", "error",
		"-show_entries", "format=duration",
		"-of", "default=noprint_wrappers=1:nokey=1",
		path,
	)
	var out bytes.Buffer
	cmd.Stdout = &out
	cmd.Stderr = &out
	if err := cmd.Run(); err != nil {
		return 0, fmt.Errorf("ffprobe: %w: %s", err, out.String())
	}
	s := strings.TrimSpace(out.String())
	if s == "" {
		return 0, fmt.Errorf("empty ffprobe output")
	}
	return strconv.ParseFloat(s, 64)
}

// TranscodeToH264MP4 converts input to H.264 + AAC MP4.
func TranscodeToH264MP4(inputPath, outputPath string) (stderr string, err error) {
	cmd := exec.Command(ffmpegExe,
		"-y",
		"-i", inputPath,
		"-c:v", "libx264",
		"-preset", "fast",
		"-crf", "23",
		"-c:a", "aac",
		"-movflags", "+faststart",
		outputPath,
	)
	var eb bytes.Buffer
	cmd.Stderr = &eb
	if err := cmd.Run(); err != nil {
		return eb.String(), fmt.Errorf("ffmpeg: %w", err)
	}
	return eb.String(), nil
}

// ScreenshotJPEG captures a frame at t seconds as JPEG.
func ScreenshotJPEG(inputPath, outputPath string, atSeconds float64) (stderr string, err error) {
	cmd := exec.Command(ffmpegExe,
		"-y",
		"-ss", fmt.Sprintf("%.3f", atSeconds),
		"-i", inputPath,
		"-vframes", "1",
		"-q:v", "2",
		outputPath,
	)
	var eb bytes.Buffer
	cmd.Stderr = &eb
	if err := cmd.Run(); err != nil {
		return eb.String(), fmt.Errorf("ffmpeg screenshot: %w", err)
	}
	return eb.String(), nil
}

// IsPermanentTranscodeFailure classifies FFmpeg errors (Skill S-004).
func IsPermanentTranscodeFailure(stderr string) bool {
	patterns := []string{
		"Invalid data found when processing input",
		"Unsupported codec",
		"No such file or directory",
		"Permission denied",
	}
	for _, p := range patterns {
		if strings.Contains(stderr, p) {
			return true
		}
	}
	return false
}

// Variant describes one ABR (adaptive-bitrate) rendition to transcode.
// Width==0/Height==0 means "keep source resolution" (the master rendition).
type Variant struct {
	Label  string // e.g. "720p", "480p", "360p", "原始"
	Width  int
	Height int
	Kbps   int
	CRF    string
}

// ABRLadder returns the downscale ladder for a given source height (px).
// It yields only renditions strictly smaller than the source, ordered from
// highest to lowest. For unknown/small sources it returns nil (single master).
func ABRLadder(sourceHeight int) []Variant {
	// {label, height, bitrateKbps, crf}
	tiers := []Variant{
		{Label: "720p", Height: 720, Kbps: 2800, CRF: "20"},
		{Label: "480p", Height: 480, Kbps: 1300, CRF: "23"},
		{Label: "360p", Height: 360, Kbps: 750, CRF: "26"},
	}
	if sourceHeight <= 0 {
		return nil
	}
	out := make([]Variant, 0, len(tiers))
	for _, t := range tiers {
		if sourceHeight > t.Height {
			out = append(out, t)
		}
	}
	return out
}

// ProbeVideoHeight returns the video stream height using ffprobe.
func ProbeVideoHeight(path string) (int, error) {
	cmd := exec.Command(ffprobeExe,
		"-v", "error",
		"-select_streams", "v:0",
		"-show_entries", "stream=height",
		"-of", "default=noprint_wrappers=1:nokey=1",
		path,
	)
	var out bytes.Buffer
	cmd.Stdout = &out
	cmd.Stderr = &out
	if err := cmd.Run(); err != nil {
		return 0, fmt.Errorf("ffprobe height: %w: %s", err, out.String())
	}
	h, err := strconv.Atoi(strings.TrimSpace(out.String()))
	if err != nil || h <= 0 {
		return 0, fmt.Errorf("invalid ffprobe height: %q", out.String())
	}
	return h, nil
}

// TranscodeVariant renders a single ABR rendition (H.264 + AAC MP4).
// Variants with Height==0 keep the source resolution.
func TranscodeVariant(inputPath, outputPath string, v Variant) (stderr string, err error) {
	args := []string{
		"-y",
		"-i", inputPath,
		"-c:v", "libx264",
		"-preset", "veryslow",
		"-crf", v.CRF,
	}
	if v.Height > 0 {
		args = append(args, "-vf",
			fmt.Sprintf("scale=-2:%d:force_original_aspect_ratio=decrease", v.Height))
	}
	args = append(args,
		"-c:a", "aac",
		"-movflags", "+faststart",
		outputPath,
	)
	cmd := exec.Command(ffmpegExe, args...)
	var eb bytes.Buffer
	cmd.Stderr = &eb
	if err := cmd.Run(); err != nil {
		return eb.String(), fmt.Errorf("ffmpeg variant %s: %w", v.Label, err)
	}
	return eb.String(), nil
}
