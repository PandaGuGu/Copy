// Command seed_appeals is a one-off development seeder for the 治理申诉 page.
// It inserts a demo user + video + sample appeals so the admin page has rows to show.
//
// Usage: go run ./scripts/seed_appeals
// Idempotent: skips a demo appeal if the same user+target already has it.
package main

import (
	"fmt"
	"log"
	"os"
	"time"

	"gorm.io/driver/mysql"
	"gorm.io/gorm"

	"minibili/internal/model"
)

func main() {
	dsn := os.Getenv("MYSQL_DSN")
	if dsn == "" {
		dsn = "root:123456@tcp(127.0.0.1:3306)/minibili?charset=utf8mb4&parseTime=True&loc=Local"
	}
	db, err := gorm.Open(mysql.Open(dsn), &gorm.Config{})
	if err != nil {
		log.Fatalf("connect: %v", err)
	}

	// 1. Demo user (active, so applicant shows a nickname on the admin page).
	user := model.User{Username: "seed_appeal_user", PasswordHash: "x", Nickname: "举个栗子", Status: "active"}
	if err := db.Where("username = ?", user.Username).FirstOrCreate(&user).Error; err != nil {
		log.Fatalf("user: %v", err)
	}
	if user.CakeID == "" {
		user.CakeID = model.FormatCakeID(user.ID)
		db.Model(&user).Update("cake_id", user.CakeID)
	}

	// 2. Demo video owned by the user (a pending-ish published video to appeal).
	video := model.Video{UserID: user.ID, Title: "演示视频（被误下架）", Status: "takedown", PlayCount: 120}
	if err := db.Where("user_id = ? AND title = ?", user.ID, video.Title).FirstOrCreate(&video).Error; err != nil {
		log.Fatalf("video: %v", err)
	}

	seeds := []model.Appeal{
		{UserID: user.ID, TargetType: "video", TargetID: video.ID, ReasonType: "takedown", Content: "我的视频《演示视频》被误下架了，这是我的原创内容，请求复查恢复。", Status: "pending"},
		{UserID: user.ID, TargetType: "user", TargetID: user.ID, ReasonType: "ban", Content: "账号被系统多次举报自动标记，但我没有违规，请求解封。", Status: "approved", AdminNote: "经复核系统误判，予以解封。", HandledAt: nowPtr(time.Now().Add(-2 * time.Hour))},
		{UserID: user.ID, TargetType: "video", TargetID: video.ID, ReasonType: "takedown", Content: "希望人工重新审核我的投稿是否符合社区规范。", Status: "rejected", AdminNote: "经核实确属搬运内容，维持下架。", HandledAt: nowPtr(time.Now().Add(-5 * time.Hour))},
	}

	inserted := 0
	for _, s := range seeds {
		var dup int64
		db.Model(&model.Appeal{}).
			Where("user_id = ? AND target_type = ? AND target_id = ? AND reason_type = ? AND content = ?",
				s.UserID, s.TargetType, s.TargetID, s.ReasonType, s.Content).
			Count(&dup)
		if dup > 0 {
			continue
		}
		s.CreatedAt = time.Now()
		if err := db.Create(&s).Error; err != nil {
			log.Printf("seed appeal failed: %v", err)
			continue
		}
		inserted++
	}

	var total int64
	db.Model(&model.Appeal{}).Count(&total)
	fmt.Printf("demo user=%d video=%d appeal_inserted=%d appeal_total=%d\n", user.ID, video.ID, inserted, total)
}

func nowPtr(t time.Time) *time.Time { return &t }