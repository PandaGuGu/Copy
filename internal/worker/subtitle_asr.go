package worker

import (
	"context"
	"encoding/json"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"time"

	"go.uber.org/zap"
	"gorm.io/gorm"

	"minibili/internal/config"
	"minibili/internal/model"
)

// SubtitleASRWorker handles automatic speech recognition for video subtitles.
type SubtitleASRWorker struct {
	Cfg        *config.C
	DB         *gorm.DB
	Log        *zap.Logger
	Transcribe func(audioPath string, lang string) (string, error) // pluggable ASR backend
}

// StartSubtitleASRConsumer watches for pending ASR tasks and processes them.
func (w *SubtitleASRWorker) Start(ctx context.Context) {
	w.Log.Info("subtitle ASR worker started")
	ticker := time.NewTicker(15 * time.Second)
	defer ticker.Stop()

	for {
		select {
		case <-ctx.Done():
			w.Log.Info("subtitle ASR worker stopped")
			return
		case <-ticker.C:
			w.processOne(ctx)
		}
	}
}

func (w *SubtitleASRWorker) processOne(ctx context.Context) {
	var sub model.Subtitle
	err := w.DB.Where("auto_gen = 1 AND content = ''").Order("created_at ASC").First(&sub).Error
	if err != nil {
		if err != gorm.ErrRecordNotFound {
			w.Log.Error("query pending ASR subtitle failed", zap.Error(err))
		}
		return
	}

	// Get video info for audio extraction
	var video model.Video
	if err := w.DB.First(&video, sub.VideoID).Error; err != nil {
		w.Log.Error("get video for ASR failed", zap.Uint64("video_id", sub.VideoID), zap.Error(err))
		return
	}

	if video.Status != "published" {
		return
	}

	w.Log.Info("processing ASR for subtitle",
		zap.Uint64("subtitle_id", sub.ID),
		zap.Uint64("video_id", sub.VideoID),
		zap.String("lang", sub.Lang),
	)

	audioPath, err := w.extractAudio(ctx, video.VideoURL)
	if err != nil {
		w.Log.Error("extract audio failed", zap.Error(err))
		return
	}
	defer os.Remove(audioPath)

	transcript, err := w.Transcribe(audioPath, sub.Lang)
	if err != nil {
		w.Log.Error("transcribe failed", zap.Error(err))
		return
	}

	content := buildVTTContent(transcript, sub.Lang)

	if err := w.DB.Model(&sub).Updates(map[string]interface{}{
		"content": content,
		"format":  "vtt",
	}).Error; err != nil {
		w.Log.Error("save ASR transcript failed", zap.Error(err))
		return
	}

	w.Log.Info("ASR completed",
		zap.Uint64("subtitle_id", sub.ID),
		zap.Int("content_len", len(content)),
	)
}

// extractAudio uses ffmpeg to convert video to 16kHz mono WAV.
func (w *SubtitleASRWorker) extractAudio(ctx context.Context, videoURL string) (string, error) {
	tmpDir := filepath.Join(w.Cfg.TempUploadDir, "asr")
	if err := os.MkdirAll(tmpDir, 0o755); err != nil {
		return "", fmt.Errorf("mkdir asr temp: %w", err)
	}
	outPath := filepath.Join(tmpDir, fmt.Sprintf("asr_%d.wav", time.Now().UnixNano()))
	cmd := exec.CommandContext(ctx,
		w.Cfg.FFmpegPath, "-y",
		"-i", videoURL,
		"-vn",                // no video
		"-acodec", "pcm_s16le", // 16-bit PCM
		"-ar", "16000",       // 16kHz
		"-ac", "1",           // mono
		"-f", "wav",
		outPath,
	)
	output, err := cmd.CombinedOutput()
	if err != nil {
		return "", fmt.Errorf("ffmpeg extract audio: %w\n%s", err, string(output))
	}
	return outPath, nil
}

// buildVTTContent builds a WebVTT file from a plain transcript.
// Splits on sentences and assigns rough timestamps.
func buildVTTContent(transcript, lang string) string {
	header := "WEBVTT\n\n"
	if transcript == "" {
		return header
	}
	// Simple: treat entire transcript as one cue block,
	// or split by newlines/periods for rough segmentation.
	lines := splitTranscript(transcript)
	var cues string
	for i, line := range lines {
		if line == "" {
			continue
		}
		start := formatVTTTime(i * 5)        // rough: 5s per segment
		end := formatVTTTime((i + 1) * 5)
		cues += fmt.Sprintf("%s --> %s\n%s\n\n", start, end, line)
	}
	return header + cues
}

func splitTranscript(text string) []string {
	// Simple sentence splitting
	var result []string
	current := ""
	for _, ch := range text {
		current += string(ch)
		if ch == '.' || ch == '!' || ch == '?' || ch == '\n' {
			if len(current) > 1 {
				result = append(result, current)
				current = ""
			}
		}
	}
	if len(current) > 0 {
		result = append(result, current)
	}
	if len(result) == 0 {
		result = []string{text}
	}
	return result
}

func formatVTTTime(seconds int) string {
	h := seconds / 3600
	m := (seconds % 3600) / 60
	s := seconds % 60
	return fmt.Sprintf("%02d:%02d:%02d.000", h, m, s)
}

// ─── Pluggable Transcribe Backend ───

// NewMockTranscriber returns a simple placeholder transcriber for testing.
func NewMockTranscriber(log *zap.Logger) func(audioPath string, lang string) (string, error) {
	return func(audioPath string, lang string) (string, error) {
		log.Warn("ASR mock transcriber used — whisper not configured",
			zap.String("audio", audioPath),
			zap.String("lang", lang),
		)
		// Return a placeholder transcript so the worker doesn't loop forever
		return fmt.Sprintf("[自动转写] 音频文件 %s 已处理，请配置 Whisper 以获取真实转录。", filepath.Base(audioPath)), nil
	}
}

// NewSubtitleASRRequest is the JSON body for POST /api/v1/videos/:id/subtitles/asr-request
type NewSubtitleASRRequest struct {
	Lang  string `json:"lang" binding:"required"`
	Title string `json:"title"`
}

// RequestASR creates a subtitle placeholder and queues it for ASR processing.
// This is called from the API handler.
func RequestASR(db *gorm.DB, videoID uint64, req NewSubtitleASRRequest) (*model.Subtitle, error) {
	sub := model.Subtitle{
		VideoID: videoID,
		Lang:    req.Lang,
		Title:   firstNonEmpty(req.Title, "自动转写"),
		Format:  "vtt",
		AutoGen: true,
		Content: "", // empty = pending ASR
	}
	if err := db.Create(&sub).Error; err != nil {
		return nil, err
	}
	return &sub, nil
}

func firstNonEmpty(a, b string) string {
	if a != "" {
		return a
	}
	return b
}

// Ensure SubtitleASRWorker implements interface
var _ interface {
	Start(context.Context)
} = (*SubtitleASRWorker)(nil)

// MarshalJSON helper (unused, for potential task logging)
var _ = json.Marshal
