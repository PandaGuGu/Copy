<template>
  <div class="live-page">
    <!-- 顶部开播按钮 -->
    <div class="live-top-bar">
      <div class="live-top-bar-inner">
        <h2 class="live-section-title">直播广场</h2>
        <router-link to="/minibili/live/create" class="go-live-btn">
          <span class="go-live-icon">📹</span>
          我要开播
        </router-link>
      </div>
    </div>

    <!-- 顶部大图区 -->
    <div class="top-banner">
      <div class="banner-main">
        <div v-if="topRoom" class="banner-main-inner">
          <router-link :to="`/minibili/live/${topRoom.id}`" class="banner-link">
            <img :src="topRoom.cover" class="banner-cover" />
            <div class="banner-overlay">
              <div class="banner-title">{{ topRoom.title }}</div>
              <div class="banner-info">
                <span>{{ topRoom.host }}</span>
                <span>👁 {{ fmtCount(topRoom.viewers) }}</span>
              </div>
            </div>
            <span class="banner-live-badge">● 直播中</span>
          </router-link>
        </div>
        <div v-else class="banner-main-inner skeleton"><div class="skeleton-shimmer"></div></div>
      </div>
      <div class="banner-side-list">
        <router-link
          v-for="item in sideItems"
          :key="item.id"
          :to="`/minibili/live/${item.id}`"
          class="side-item"
        >
          <img v-if="item.cover" :src="item.cover" class="side-cover" />
          <div v-else class="side-item skeleton"><div class="skeleton-shimmer"></div></div>
          <span class="side-title">{{ item.title }}</span>
        </router-link>
        <div v-for="i in (5 - sideItems.length)" :key="'side-sk-'+i" class="side-item skeleton">
          <div class="skeleton-shimmer"></div>
        </div>
      </div>
    </div>

    <!-- 关注与分类导航 -->
    <div class="nav-row">
      <div class="nav-title-bar">
        <span>正在直播</span>
        <span>查看全部 &gt;</span>
      </div>
      <div class="avatar-scroll">
        <router-link
          v-for="r in liveRooms.slice(0, 14)"
          :key="r.id"
          :to="`/minibili/live/${r.id}`"
          class="avatar-item"
        >
          <img :src="r.avatar" class="avatar-img" />
          <span class="avatar-name">{{ r.host }}</span>
        </router-link>
        <div v-for="i in (14 - Math.min(liveRooms.length, 14))" :key="'av-sk-'+i" class="avatar-item skeleton">
          <div class="skeleton-shimmer"></div>
        </div>
      </div>
    </div>

    <!-- 中部三栏 -->
    <div class="mid-container">
      <div class="col-left">
        <div
          v-for="item in leftBigCards"
          :key="item.id"
          class="big-card"
        >
          <router-link :to="`/minibili/live/${item.id}`" class="big-card-inner">
            <img :src="item.cover" class="big-card-img" />
            <div class="big-card-label">{{ item.title }}</div>
          </router-link>
        </div>
        <div class="two-small-row">
          <div v-for="item in leftSmallCards" :key="item.id" class="small-card">
            <router-link :to="`/minibili/live/${item.id}`" class="small-card-inner">
              <img :src="item.cover" class="small-card-img" />
              <div class="small-card-label">{{ item.title }}</div>
            </router-link>
          </div>
          <div v-for="i in (2 - leftSmallCards.length)" :key="'lsc-sk-'+i" class="small-card skeleton">
            <div class="skeleton-shimmer"></div>
          </div>
        </div>
      </div>
      <div class="col-mid">
        <div class="mid-block">
          <div class="mid-block-title">热门分类</div>
          <div class="mid-tag-list">
            <span v-for="z in ['游戏','音乐','娱乐','舞蹈','动画','生活']" :key="z" class="mid-tag">{{ z }}</span>
          </div>
        </div>
        <div class="mid-block">
          <div class="mid-block-title">热门直播</div>
          <div v-for="r in liveRooms.slice(14, 19)" :key="r.id" class="mid-live-row">
            <router-link :to="`/minibili/live/${r.id}`" class="mid-live-link">
              {{ r.title }}
              <span class="mid-live-count">👁 {{ fmtCount(r.viewers) }}</span>
            </router-link>
          </div>
        </div>
      </div>
      <div class="rank-block">
        <div class="rank-title">直播排行</div>
        <div
          v-for="(r, idx) in rankedRooms"
          :key="r.id"
          class="rank-item"
        >
          <span :class="['rank-num', { top3: idx < 3 }]">{{ idx + 1 }}</span>
          <router-link :to="`/minibili/live/${r.id}`" class="rank-link">
            {{ r.title }}
            <span v-if="r.status !== 'live'" class="rank-idle-tag">未开播</span>
          </router-link>
          <span class="rank-count">👁 {{ fmtCount(r.viewers) }}</span>
        </div>
      </div>
    </div>

    <!-- 全部直播 5×N 网格 -->
    <div class="recommend-wrap">
      <div class="recommend-title">
        <span>全部直播</span>
      </div>
      <div class="live-grid">
        <div v-for="item in allRooms" :key="item.id" class="live-card">
          <router-link :to="`/minibili/live/${item.id}`" class="lc-cover-wrap">
            <img :src="item.cover" class="lc-cover" loading="lazy" />
            <span v-if="item.status === 'live'" class="lc-badge lc-badge--live">● 直播中 {{ fmtCount(item.viewers) }} 观看</span>
            <span v-else class="lc-badge lc-badge--idle">未开播</span>
          </router-link>
          <router-link :to="`/minibili/live/${item.id}`" class="lc-title">{{ item.title }}</router-link>
          <div class="lc-meta">{{ item.host }}</div>
        </div>
        <div v-for="i in gridSkeleton" :key="'g-sk-'+i" class="live-card">
          <div class="lc-cover skeleton"><div class="skeleton-shimmer"></div></div>
          <div class="skeleton-text" style="width:70%;height:14px;"></div>
          <div class="skeleton-text" style="width:40%;height:12px;"></div>
        </div>
      </div>
    </div>

    <!-- 底部通栏 -->
    <div class="bottom-banner">
      <div class="bottom-banner-inner">更多直播即将开播，敬请期待</div>
    </div>
  </div>
</template>

<script>
import http from "@/utils/http";

export default {
  name: "LivePage",
  data() {
    return { items: [] };
  },
  computed: {
    liveRooms() {
      return this.items
        .filter((r) => r.status === "live")
        .map((r) => this.mapRoom(r));
    },
    topRoom() {
      return this.liveRooms[0] || null;
    },
    sideItems() {
      return this.liveRooms.slice(1, 6);
    },
    leftBigCards() {
      return this.liveRooms.slice(6, 8);
    },
    leftSmallCards() {
      return this.liveRooms.slice(8, 10);
    },
    allRooms() {
      return this.items.map((r) => this.mapRoom(r));
    },
    rankedRooms() {
      return [...this.items]
        .sort((a, b) => (b.viewer_count || 0) - (a.viewer_count || 0))
        .slice(0, 10)
        .map((r) => this.mapRoom(r));
    },
    gridSkeleton() {
      return Math.max(0, 10 - this.allRooms.length);
    },
  },
  created() {
    this.fetch();
  },
  methods: {
    mapRoom(r) {
      return {
        id: r.id,
        title: r.title || "直播间",
        cover: r.cover_url || "",
        host: r.host_name || r.uploader || "主播",
        avatar: r.avatar_url || `https://api.dicebear.com/7.x/initials/svg?seed=${r.id}`,
        viewers: r.viewer_count || 0,
        status: r.status || "idle",
        time: r.started_at || "",
      };
    },
    async fetch() {
      try {
        const res = await http.get("/api/v1/live/rooms");
        if (res && res.code === 0 && res.data) {
          this.items = res.data.rooms || res.data.items || res.data || [];
        }
      } catch (e) {
        console.warn("LivePage fetch:", e);
      }
    },
    fmtCount(n) {
      const v = Number(n) || 0;
      if (v >= 10000) return (v / 10000).toFixed(v >= 100000 ? 0 : 1) + "万";
      return String(v);
    },
  },
};
</script>

<style scoped>
.live-page {
  background: #f5f5f7;
  min-height: 100vh;
}

/* Top bar with go-live button */
.live-top-bar {
  max-width: 1400px; margin: 0 auto;
  padding: 16px 20px 8px;
}
.live-top-bar-inner {
  display: flex; align-items: center; justify-content: space-between;
}
.live-section-title {
  font-size: 22px; font-weight: 700; color: #222; margin: 0;
}
.go-live-btn {
  display: inline-flex; align-items: center; gap: 6px;
  padding: 10px 24px;
  background: linear-gradient(135deg, #fb7299, #ff5682);
  color: #fff;
  font-size: 15px; font-weight: 600;
  border-radius: 8px;
  text-decoration: none;
  box-shadow: 0 2px 8px rgba(251,114,153,0.35);
  transition: transform 0.15s, box-shadow 0.15s;
}
.go-live-btn:hover {
  transform: translateY(-1px);
  box-shadow: 0 4px 14px rgba(251,114,153,0.5);
  color: #fff;
}
.go-live-icon { font-size: 18px; }

/* Top banner */
.top-banner {
  width: 100%;
  max-width: 1400px;
  margin: 0 auto;
  height: 320px;
  display: flex;
}
.banner-main {
  flex: 1;
  position: relative;
}
.banner-main-inner {
  width: 100%;
  height: 100%;
  background: #c9c9c9;
  overflow: hidden;
}
.banner-link {
  display: block;
  width: 100%;
  height: 100%;
  position: relative;
  text-decoration: none;
}
.banner-cover {
  width: 100%;
  height: 100%;
  object-fit: cover;
}
.banner-overlay {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  padding: 16px 20px;
  background: linear-gradient(transparent, rgba(0,0,0,0.7));
  color: #fff;
}
.banner-title { font-size: 22px; font-weight: 600; margin-bottom: 6px; }
.banner-info { font-size: 14px; display: flex; gap: 16px; }
.banner-live-badge {
  position: absolute; top: 12px; left: 12px;
  background: #ff5682; color: #fff;
  font-size: 13px; padding: 4px 12px; border-radius: 4px;
}
.banner-side-list {
  width: 170px;
  display: flex; flex-direction: column; gap: 6px;
  padding: 8px; background: #eee;
}
.side-item {
  height: 56px; background: #ccc; border-radius: 4px;
  overflow: hidden; position: relative;
  display: flex; align-items: flex-end;
  text-decoration: none;
}
.side-cover {
  position: absolute; inset: 0;
  width: 100%; height: 100%; object-fit: cover;
}
.side-title {
  position: relative; z-index: 1;
  padding: 2px 6px; font-size: 11px;
  color: #fff; background: rgba(0,0,0,0.5);
  width: 100%; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
}

/* Nav row */
.nav-row {
  max-width: 1400px; margin: 0 auto;
  background: #fff; padding: 16px 20px;
  display: flex; flex-direction: column; gap: 12px;
}
.nav-title-bar {
  display: flex; justify-content: space-between;
  font-size: 14px; color: #666;
}
.avatar-scroll {
  display: flex; gap: 16px; overflow-x: auto; padding-bottom: 4px;
}
.avatar-item {
  width: 56px; flex-shrink: 0; text-align: center;
  text-decoration: none; color: #666;
}
.avatar-img {
  width: 48px; height: 48px; border-radius: 50%; background: #ddd;
  object-fit: cover; display: block; margin: 0 auto 4px;
}
.avatar-name { font-size: 11px; display: block; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

/* Mid container */
.mid-container {
  max-width: 1400px; margin: 0 auto;
  display: grid; grid-template-columns: 1fr 1fr 300px;
  gap: 16px; padding: 16px 20px;
}
.col-left, .col-mid { display: flex; flex-direction: column; gap: 12px; }
.big-card {
  height: 140px; background: #ddd; border-radius: 6px;
  overflow: hidden; position: relative;
}
.big-card-inner {
  display: block; width: 100%; height: 100%;
  position: relative; text-decoration: none;
}
.big-card-img { width: 100%; height: 100%; object-fit: cover; }
.big-card-label {
  position: absolute; bottom: 0; left: 0; right: 0;
  padding: 8px 12px; background: linear-gradient(transparent, rgba(0,0,0,0.6));
  color: #fff; font-size: 14px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
}
.two-small-row { display: flex; gap: 10px; }
.small-card {
  flex: 1; height: 90px; background: #ddd; border-radius: 6px;
  overflow: hidden; position: relative;
}
.small-card-inner { display: block; width: 100%; height: 100%; position: relative; text-decoration: none; }
.small-card-img { width: 100%; height: 100%; object-fit: cover; }
.small-card-label {
  position: absolute; bottom: 0; left: 0; right: 0;
  padding: 4px 8px; background: linear-gradient(transparent, rgba(0,0,0,0.5));
  color: #fff; font-size: 12px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
}
.mid-block {
  background: #fff; border-radius: 6px; padding: 14px;
}
.mid-block-title { font-size: 15px; font-weight: 600; margin-bottom: 10px; }
.mid-tag-list { display: flex; flex-wrap: wrap; gap: 8px; }
.mid-tag {
  padding: 4px 12px; background: #f0f1f3; color: #666;
  font-size: 13px; border-radius: 4px; cursor: pointer;
}
.mid-live-row { padding: 6px 0; border-bottom: 1px solid #f0f0f0; }
.mid-live-row:last-child { border: none; }
.mid-live-link {
  display: flex; justify-content: space-between; align-items: center;
  text-decoration: none; color: #222; font-size: 13px;
}
.mid-live-link:hover { color: #ff5682; }
.mid-live-count { color: #999; font-size: 12px; flex-shrink: 0; }
.rank-block {
  background: #fff; border-radius: 6px; padding: 14px;
}
.rank-title { font-size: 15px; font-weight: 600; margin-bottom: 10px; }
.rank-item {
  display: flex; align-items: center; gap: 8px;
  padding: 5px 0; border-bottom: 1px solid #f0f0f0;
}
.rank-item:last-child { border: none; }
.rank-num { font-size: 14px; color: #999; width: 20px; text-align: center; flex-shrink: 0; }
.rank-num.top3 { color: #ff5682; font-weight: 600; }
.rank-link {
  flex: 1; font-size: 13px; color: #222; text-decoration: none;
  white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
}
.rank-link:hover { color: #ff5682; }
.rank-count { color: #999; font-size: 12px; flex-shrink: 0; }
.rank-idle-tag {
  font-size: 11px; color: #999; margin-left: 4px;
  background: #f0f0f0; padding: 0 4px; border-radius: 2px;
}

/* Recommend grid */
.recommend-wrap { max-width: 1400px; margin: 0 auto; padding: 0 20px 20px; }
.recommend-title { display: flex; justify-content: space-between; padding: 12px 0; font-size: 18px; font-weight: 600; }
.live-grid {
  display: grid; grid-template-columns: repeat(5, 1fr); gap: 12px;
}
.live-card { display: flex; flex-direction: column; gap: 6px; }
.lc-cover-wrap { position: relative; display: block; }
.lc-cover {
  width: 100%; height: 130px; object-fit: cover;
  background: #ddd; border-radius: 4px; display: block;
}
.lc-badge {
  position: absolute; bottom: 6px; left: 6px;
  background: rgba(0,0,0,0.6); color: #fff;
  font-size: 11px; padding: 2px 6px; border-radius: 2px;
}
.lc-badge--live { background: rgba(251,114,153,0.85); }
.lc-badge--idle { background: rgba(0,0,0,0.45); }
.lc-title {
  font-size: 14px; color: #222; text-decoration: none;
  display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical; overflow: hidden;
}
.lc-title:hover { color: #ff5682; }
.lc-meta { font-size: 12px; color: #999; }

/* Bottom */
.bottom-banner {
  max-width: 1400px; margin: 0 auto; padding: 0 20px 20px;
}
.bottom-banner-inner {
  height: 100px; background: #e8e8e8; border-radius: 6px;
  display: flex; align-items: center; justify-content: center;
  color: #999; font-size: 16px;
}

/* skeleton */
.skeleton { position: relative; background: #e8e8e8 !important; overflow: hidden; }
.skeleton-shimmer {
  position: absolute; inset: 0;
  background: linear-gradient(90deg, transparent 0%, rgba(255,255,255,0.5) 50%, transparent 100%);
  animation: shimmer 1.5s infinite;
}
@keyframes shimmer { 0%{transform:translateX(-100%)} 100%{transform:translateX(100%)} }
.skeleton-text { background: #e8e8e8; border-radius: 4px; }

@media (max-width: 1100px) {
  .mid-container { grid-template-columns: 1fr 1fr; }
  .rank-block { display: none; }
  .live-grid { grid-template-columns: repeat(3, 1fr); }
  .top-banner { height: 240px; }
  .banner-side-list { display: none; }
}
</style>
