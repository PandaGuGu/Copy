package main

import (
	"fmt"
	"math/rand"
	"time"

	"gorm.io/driver/mysql"
	"gorm.io/gorm"
)

type Danmaku struct {
	ID        uint64 `gorm:"primaryKey"`
	VideoID   uint64 `gorm:"index"`
	UserID    uint64
	Content   string
	VideoTime float64
	Color     string
	Type      int
	FontSize  float64
	CreatedAt time.Time
}

type VideoLike struct {
	ID        uint64 `gorm:"primaryKey"`
	VideoID   uint64 `gorm:"uniqueIndex:idx_vl_vu"`
	UserID    uint64 `gorm:"uniqueIndex:idx_vl_vu"`
	CreatedAt time.Time
}

type VideoFavorite struct {
	ID        uint64 `gorm:"primaryKey"`
	VideoID   uint64 `gorm:"uniqueIndex:idx_vf_vu"`
	UserID    uint64 `gorm:"uniqueIndex:idx_vf_vu"`
	CreatedAt time.Time
}

type VideoCoin struct {
	ID        uint64 `gorm:"primaryKey"`
	VideoID   uint64 `gorm:"uniqueIndex:idx_vc_vu"`
	UserID    uint64 `gorm:"uniqueIndex:idx_vc_vu"`
	Amount    int
	CreatedAt time.Time
}

func main() {
	dsn := "root:123456@tcp(127.0.0.1:3306)/minibili?charset=utf8mb4&parseTime=True&loc=Local"
	db, err := gorm.Open(mysql.Open(dsn), &gorm.Config{})
	if err != nil {
		panic(err)
	}

	var vids []uint64
	db.Model(&struct{ ID uint64 }{}).Table("videos").Where("status = ?", "published").Pluck("id", &vids)

	var uids []uint64
	db.Model(&struct{ ID uint64 }{}).Table("users").Limit(10).Pluck("id", &uids)

	if len(vids) == 0 || len(uids) == 0 {
		fmt.Println("No videos or users found, skipping")
		return
	}

	rng := rand.New(rand.NewSource(time.Now().UnixNano()))
	now := time.Now()

	fmt.Println("Seeding danmaku...")
	for i := 0; i < 300; i++ {
		offset := rng.Intn(30)
		ts := now.AddDate(0, 0, -offset).Add(time.Duration(rng.Intn(86400)) * time.Second)
		d := Danmaku{
			VideoID: vids[rng.Intn(len(vids))], UserID: uids[rng.Intn(len(uids))],
			Content: fmt.Sprintf("弹幕测试 %d", i), VideoTime: float64(rng.Intn(600)),
			Color: "#ffffff", CreatedAt: ts,
		}
		db.Create(&d)
	}

	fmt.Println("Seeding video likes...")
	usedLikes := make(map[string]bool)
	for i := 0; i < 200; i++ {
		vid := vids[rng.Intn(len(vids))]
		uid := uids[rng.Intn(len(uids))]
		key := fmt.Sprintf("%d-%d", vid, uid)
		if usedLikes[key] { continue }
		usedLikes[key] = true
		offset := rng.Intn(30)
		ts := now.AddDate(0, 0, -offset).Add(time.Duration(rng.Intn(86400)) * time.Second)
		db.Create(&VideoLike{VideoID: vid, UserID: uid, CreatedAt: ts})
	}

	fmt.Println("Seeding video favorites...")
	usedFavs := make(map[string]bool)
	for i := 0; i < 150; i++ {
		vid := vids[rng.Intn(len(vids))]
		uid := uids[rng.Intn(len(uids))]
		key := fmt.Sprintf("%d-%d", vid, uid)
		if usedFavs[key] { continue }
		usedFavs[key] = true
		offset := rng.Intn(30)
		ts := now.AddDate(0, 0, -offset).Add(time.Duration(rng.Intn(86400)) * time.Second)
		db.Create(&VideoFavorite{VideoID: vid, UserID: uid, CreatedAt: ts})
	}

	fmt.Println("Seeding video coins...")
	usedCoins := make(map[string]bool)
	for i := 0; i < 100; i++ {
		vid := vids[rng.Intn(len(vids))]
		uid := uids[rng.Intn(len(uids))]
		key := fmt.Sprintf("%d-%d", vid, uid)
		if usedCoins[key] { continue }
		usedCoins[key] = true
		offset := rng.Intn(30)
		ts := now.AddDate(0, 0, -offset).Add(time.Duration(rng.Intn(86400)) * time.Second)
		db.Create(&VideoCoin{VideoID: vid, UserID: uid, Amount: 1 + rng.Intn(2), CreatedAt: ts})
	}

	fmt.Println("Done! Seeded ~750 engagement records over 30 days.")
}
