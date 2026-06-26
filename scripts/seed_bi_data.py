"""Seed test data for BI tabs: articles, likes, coins, follows."""
import requests
import json
from datetime import datetime, timedelta

BASE = "http://localhost:8888/api/v1"

# Login admin
r = requests.post(f"{BASE}/admin/auth/login", json={
    "username": "admin",
    "password": "admin123"
})
token = r.json()["data"]["access_token"]
headers = {"Authorization": f"Bearer {token}"}

print("Login OK, token:", token[:30] + "...")

# Get existing users for seeding
r = requests.get(f"{BASE}/auth/users", headers=headers)
users = r.json().get("data", {}).get("users", [])
if not users:
    # Try public endpoint
    r = requests.get(f"{BASE}/public/users")
    users = r.json().get("data", {}).get("users", []) or r.json().get("data", [])
    
print(f"Found {len(users)} users")

# Get existing videos for seeding
r = requests.get(f"{BASE}/public/videos?limit=5")
videos = r.json().get("data", {}).get("videos", []) or r.json().get("data", {}).get("items", [])
print(f"Found {len(videos)} videos")

if not users:
    print("No users found, skipping seed")
    exit(1)
if not videos:
    print("No videos found, skipping seed")
    exit(1)

# Pick user and video
user = users[0]
uid = user.get("id")
video = videos[0]
vid = video.get("id")
print(f"Using user ID={uid}, video ID={vid}")

# --- Seed articles ---
today = datetime.now()
articles = []
for i in range(5):
    title = f"测试文章 {i+1} - {today.strftime('%Y-%m-%d')}"
    r = requests.post(f"{BASE}/articles", headers=headers, json={
        "user_id": uid,
        "title": title,
        "content": f"这是第{i+1}篇BI测试专栏文章。用于验证文章统计功能。",
        "category": ["技术", "生活", "游戏", "动画", "音乐"][i % 5],
        "status": "published",
    })
    if r.status_code == 200 or r.status_code == 201:
        d = r.json().get("data", {})
        aid = d.get("id") or d.get("article_id")
        articles.append(aid)
        print(f"  Created article #{aid}: {title}")
    else:
        print(f"  Article create failed: {r.status_code} {r.text[:100]}")

# --- Seed video likes (多条不同日期) ---
for i in range(8):
    day = today - timedelta(days=i*3)
    r = requests.post(f"{BASE}/videos/{vid}/like", headers=headers)
    print(f"  Like video {vid} day-{day.strftime('%m-%d')}: {r.status_code}")

# --- Seed video favorites ---
for i in range(3):
    r = requests.post(f"{BASE}/videos/{vid}/favorite", headers=headers)
    print(f"  Fav video {vid}: {r.status_code}")

# --- Seed coins ---
for i in range(5):
    r = requests.post(f"{BASE}/videos/{vid}/coin", headers=headers, json={"count": 1})
    print(f"  Coin video {vid}: {r.status_code}")

# --- Seed follows ---
for i in range(3):
    other_uid = 1  # follow self or a specific user
    r = requests.post(f"{BASE}/users/{other_uid}/follow", headers=headers)
    print(f"  Follow user {other_uid}: {r.status_code}")

print("\nSeed complete. Verify at http://localhost:8888/#/admin/bi")
