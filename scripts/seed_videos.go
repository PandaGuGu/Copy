// Seed videos for development/demo purposes.
//go:build ignore
// +build ignore

package main

import (
	"encoding/json"
	"fmt"
	"log"
	"math/rand"
	"time"

	"gorm.io/driver/mysql"
	"gorm.io/gorm"
	"gorm.io/gorm/logger"

	"minibili/internal/model"
)

type seedVideo struct {
	Title    string
	Zone     string
	Tags     []string
	Duration float64
	UserID   uint64
	PlayCnt  uint64
	LikeCnt  uint64
	CoinCnt  uint64
	FavCnt   uint64
	DanCnt   uint64
}

var seedData = []seedVideo{
	// === 游戏区 ===
	{Title: "【艾尔登法环】无伤通关女武神玛莲妮亚", Zone: "游戏", Tags: []string{"实况", "魂系", "攻略"}, Duration: 480, UserID: 1, PlayCnt: 52300, LikeCnt: 3200, CoinCnt: 850, FavCnt: 1200, DanCnt: 280},
	{Title: "【黑神话悟空】虎先锋15秒速杀打法教学", Zone: "游戏", Tags: []string{"攻略", "速通", "教学"}, Duration: 180, UserID: 1, PlayCnt: 89000, LikeCnt: 5600, CoinCnt: 1400, FavCnt: 2300, DanCnt: 520},
	{Title: "原神4.6新地图探索实况", Zone: "游戏", Tags: []string{"实况", "开放世界", "RPG"}, Duration: 2100, UserID: 2, PlayCnt: 34500, LikeCnt: 2100, CoinCnt: 500, FavCnt: 800, DanCnt: 190},
	{Title: "DOTA2 Ti13精彩集锦TOP10", Zone: "游戏", Tags: []string{"集锦", "MOBA", "电竞"}, Duration: 420, UserID: 2, PlayCnt: 67800, LikeCnt: 4300, CoinCnt: 1100, FavCnt: 1600, DanCnt: 350},

	// === 动画区 ===
	{Title: "【MAD/AMV】进击的巨人 × Attack on Titan", Zone: "动画", Tags: []string{"MAD", "剪辑", "热血"}, Duration: 240, UserID: 2, PlayCnt: 120000, LikeCnt: 8900, CoinCnt: 3200, FavCnt: 4500, DanCnt: 1200},
	{Title: "2024年7月新番导视", Zone: "动画", Tags: []string{"新番", "导视", "盘点"}, Duration: 600, UserID: 1, PlayCnt: 45600, LikeCnt: 2800, CoinCnt: 700, FavCnt: 900, DanCnt: 150},
	{Title: "【动画杂谈】为什么EVA至今无法被超越", Zone: "动画", Tags: []string{"杂谈", "EVA", "分析"}, Duration: 900, UserID: 2, PlayCnt: 78000, LikeCnt: 5200, CoinCnt: 1800, FavCnt: 2800, DanCnt: 680},
	{Title: "【补番推荐】10部冷门但超好看的动画", Zone: "动画", Tags: []string{"推荐", "盘点", "冷门"}, Duration: 720, UserID: 3, PlayCnt: 34000, LikeCnt: 1900, CoinCnt: 450, FavCnt: 750, DanCnt: 110},

	// === 科技区 ===
	{Title: "RTX 5090首发评测：性能翻倍？", Zone: "科技", Tags: []string{"评测", "显卡", "硬件"}, Duration: 1500, UserID: 3, PlayCnt: 156000, LikeCnt: 10200, CoinCnt: 2800, FavCnt: 3500, DanCnt: 890},
	{Title: "Python入门到精通 Day1：环境搭建", Zone: "科技", Tags: []string{"教程", "编程", "Python"}, Duration: 1800, UserID: 3, PlayCnt: 23000, LikeCnt: 1500, CoinCnt: 380, FavCnt: 600, DanCnt: 80},
	{Title: "AI绘画Stable Diffusion保姆级教程", Zone: "科技", Tags: []string{"教程", "AI", "绘画"}, Duration: 1200, UserID: 4, PlayCnt: 89000, LikeCnt: 6700, CoinCnt: 2100, FavCnt: 3100, DanCnt: 450},
	{Title: "【装机】5000元预算能配什么配置？", Zone: "科技", Tags: []string{"装机", "硬件", "教程"}, Duration: 960, UserID: 1, PlayCnt: 41000, LikeCnt: 2400, CoinCnt: 550, FavCnt: 850, DanCnt: 140},

	// === 生活区 ===
	{Title: "独居男生的周末日常Vlog", Zone: "生活", Tags: []string{"Vlog", "日常", "独居"}, Duration: 540, UserID: 4, PlayCnt: 12300, LikeCnt: 850, CoinCnt: 180, FavCnt: 300, DanCnt: 45},
	{Title: "【美食】红烧肉终极做法，肥而不腻", Zone: "生活", Tags: []string{"美食", "教程", "家常菜"}, Duration: 300, UserID: 1, PlayCnt: 42100, LikeCnt: 2600, CoinCnt: 700, FavCnt: 1200, DanCnt: 210},
	{Title: "搬家整理收纳大法", Zone: "生活", Tags: []string{"收纳", "搬家", "家居"}, Duration: 420, UserID: 4, PlayCnt: 18900, LikeCnt: 1200, CoinCnt: 250, FavCnt: 400, DanCnt: 65},

	// === 音乐区 ===
	{Title: "【钢琴】久石让《Summer》完整版", Zone: "音乐", Tags: []string{"钢琴", "演奏", "久石让"}, Duration: 360, UserID: 2, PlayCnt: 56000, LikeCnt: 4100, CoinCnt: 1200, FavCnt: 2100, DanCnt: 320},
	{Title: "【原创曲】夏天的风 | Chill Beat", Zone: "音乐", Tags: []string{"原创", "电子", "Lo-Fi"}, Duration: 240, UserID: 3, PlayCnt: 26700, LikeCnt: 1800, CoinCnt: 420, FavCnt: 650, DanCnt: 95},

	// === 科技/国创（无独立知识区）===
	{Title: "宇宙有多大？从地球到可观测宇宙边缘", Zone: "科技", Tags: []string{"科普", "宇宙", "天文"}, Duration: 720, UserID: 1, PlayCnt: 134000, LikeCnt: 9400, CoinCnt: 3500, FavCnt: 5200, DanCnt: 1100},
	{Title: "五分钟看懂量子力学", Zone: "科技", Tags: []string{"科普", "物理", "量子"}, Duration: 300, UserID: 4, PlayCnt: 98000, LikeCnt: 7200, CoinCnt: 2600, FavCnt: 3800, DanCnt: 750},
	{Title: "中国古代史脉络：从夏商周到清朝", Zone: "国创", Tags: []string{"历史", "中国", "科普"}, Duration: 1500, UserID: 3, PlayCnt: 45000, LikeCnt: 3000, CoinCnt: 900, FavCnt: 1500, DanCnt: 280},

	// === 影视区 ===
	{Title: "【电影解说】《肖申克的救赎》深度解析", Zone: "影视", Tags: []string{"解说", "电影", "经典"}, Duration: 900, UserID: 2, PlayCnt: 72000, LikeCnt: 4800, CoinCnt: 1500, FavCnt: 2200, DanCnt: 420},
	{Title: "2024年度烂片盘点", Zone: "影视", Tags: []string{"盘点", "烂片", "吐槽"}, Duration: 660, UserID: 4, PlayCnt: 38000, LikeCnt: 2200, CoinCnt: 520, FavCnt: 700, DanCnt: 160},
}

func main() {
	dsn := "root:123456@tcp(127.0.0.1:3306)/minibili?charset=utf8mb4&parseTime=True&loc=Local"
	db, err := gorm.Open(mysql.Open(dsn), &gorm.Config{Logger: logger.Default.LogMode(logger.Silent)})
	if err != nil {
		log.Fatal(err)
	}

	rand.Seed(time.Now().UnixNano())
	now := time.Now()
	fmt.Printf("Seeding %d videos...\n", len(seedData))

	for i, v := range seedData {
		tagsJSON, _ := json.Marshal(v.Tags)

		// Scatter creation dates across last 30 days.
		daysAgo := rand.Intn(30)
		hoursAgo := rand.Intn(24)
		createdAt := now.Add(-time.Duration(daysAgo)*24*time.Hour - time.Duration(hoursAgo)*time.Hour)

		video := &model.Video{
			UserID:       v.UserID,
			Title:        v.Title,
			Status:       "published",
			Zone:         v.Zone,
			TagsJSON:     string(tagsJSON),
			DurationSec:  v.Duration,
			PlayCount:    v.PlayCnt,
			LikeCount:    v.LikeCnt,
			CoinCount:    v.CoinCnt,
			FavCount:     v.FavCnt,
			DanmakuCount: v.DanCnt,
			CommentCount: uint64(rand.Intn(50)),
			VideoURL:     "/uploads/placeholder.mp4",
			CoverURL:     fmt.Sprintf("https://picsum.photos/seed/%d/320/180", i+100),
			CreatedAt:    createdAt,
			UpdatedAt:    createdAt,
		}

		if err := db.Create(video).Error; err != nil {
			log.Printf("FAIL id=%d: %v", i+1, err)
		} else {
			short := []rune(v.Title)
			if len(short) > 25 {
				short = short[:25]
			}
			fmt.Printf("  ✓ id=%-5d  %s\n", video.ID, string(short))
		}
	}

	// Verify
	var count int64
	db.Model(&model.Video{}).Where("status = ?", "published").Count(&count)
	fmt.Printf("\nTotal published videos: %d\n", count)
}
