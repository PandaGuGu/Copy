<template>
  <div class="zone-page">
    <!-- 标题栏：参照 HTML 设计 -->
    <div class="zone-page-header">
      <div class="zone-icon-wrap">
        <span class="zone-icon">{{ icon }}</span>
      </div>
      <span class="zone-page-title">{{ zoneName }}</span>
    </div>

    <!-- 第一行：左侧大卡片 + 右侧三张小卡片（参照 HTML row1） -->
    <div class="zone-row1">
      <div
        v-for="i in 4"
        :key="'r1-' + i"
        class="video-card"
        :class="{ 'is-big': i === 1 }"
      >
        <template v-if="items[i - 1]">
          <router-link :to="`/video/BV${items[i - 1].aid}`" class="card-cover-wrap">
            <img :src="items[i - 1].pic" :alt="items[i - 1].title" class="card-cover" loading="lazy" />
            <span class="duration-tag">{{ fmtDuration(items[i - 1].duration) }}</span>
          </router-link>
          <router-link :to="`/video/BV${items[i - 1].aid}`" class="card-title">{{ items[i - 1].title }}</router-link>
          <div class="card-meta">
            <span class="card-up">{{ items[i - 1].author }}</span>
            <span class="card-play">{{ fmtCount(items[i - 1].play) }} 播放</span>
          </div>
        </template>
        <template v-else>
          <div class="card-cover-wrap skeleton">
            <div class="skeleton-shimmer"></div>
          </div>
          <div class="card-title skeleton-text"></div>
          <div class="card-meta skeleton-text-short"></div>
        </template>
      </div>
    </div>

    <!-- 第二行：5 列等宽（参照 HTML row2） -->
    <div class="zone-row2">
      <div
        v-for="i in 5"
        :key="'r2-' + i"
        class="video-card"
      >
        <template v-if="items[3 + i]">
          <router-link :to="`/video/BV${items[3 + i].aid}`" class="card-cover-wrap">
            <img :src="items[3 + i].pic" :alt="items[3 + i].title" class="card-cover" loading="lazy" />
            <span class="duration-tag">{{ fmtDuration(items[3 + i].duration) }}</span>
          </router-link>
          <router-link :to="`/video/BV${items[3 + i].aid}`" class="card-title">{{ items[3 + i].title }}</router-link>
          <div class="card-meta">
            <span class="card-up">{{ items[3 + i].author }}</span>
            <span class="card-play">{{ fmtCount(items[3 + i].play) }} 播放</span>
          </div>
        </template>
        <template v-else>
          <div class="card-cover-wrap skeleton">
            <div class="skeleton-shimmer"></div>
          </div>
          <div class="card-title skeleton-text"></div>
          <div class="card-meta skeleton-text-short"></div>
        </template>
      </div>
    </div>

    <!-- 加载更多 -->
    <div v-if="hasMore && items.length >= 9" class="zone-more" @click="loadMore">
      加载更多
    </div>

    <!-- 3×5 无限滚动视频流（同首页 VideoFeed） -->
    <div class="zone-feed">
      <div class="zone-feed-grid">
        <div
          v-for="(item, idx) in paddedFeedItems"
          :key="'feed-' + idx"
          class="feed-card"
        >
            <template v-if="item">
              <router-link :to="`/video/BV${item.aid}`" class="feed-cover-wrap">
                <img :src="item.pic" :alt="item.title" class="feed-cover" loading="lazy" />
                <span class="feed-duration">{{ fmtDuration(item.duration) }}</span>
              </router-link>
              <router-link :to="`/video/BV${item.aid}`" class="feed-title">{{ item.title }}</router-link>
              <div class="feed-meta">{{ fmtCount(item.play) }}播放 · {{ item.author }}</div>
            </template>
            <template v-else>
              <div class="feed-cover-wrap skeleton"><div class="skeleton-shimmer"></div></div>
              <div class="feed-title skeleton-text"></div>
              <div class="feed-meta skeleton-text-short"></div>
            </template>
          </div>
      </div>
      <div v-if="feedLoading" class="zone-feed-loading">加载中...</div>
    </div>
  </div>
</template>

<script>
import http from "@/utils/http";

const ZONE_ICONS = {
  "番剧": "🎬", "国创": "🏮", "动画": "✨", "游戏": "🎮",
  "科技": "💻", "生活": "🏠", "音乐": "🎵", "影视": "🎞️",
  "鬼畜": "👻", "舞蹈": "💃", "娱乐": "🎤", "时尚": "💄",
  "纪录片": "📖", "电影": "🍿", "电视剧": "📺",
};
const DEFAULT_ICON = "📺";

export default {
  name: "ZonePage",
  data() {
    return {
      items: [],
      loading: true,
      page: 1,
      hasMore: true,
      // Infinite scroll feed
      feedItems: [],
      feedLoading: false,
      feedPage: 1,
      feedHasMore: true,
      feedCursor: "",
    };
  },
  computed: {
    zoneName() {
      return this.$route.params.zoneName || "未知";
    },
    icon() {
      return ZONE_ICONS[this.zoneName] || DEFAULT_ICON;
    },
    paddedFeedItems() {
      const total = this.feedItems.length;
      const min = total > 0 ? total + (15 - (total % 15)) % 15 : 15;
      const out = this.feedItems.slice();
      while (out.length < min) out.push(null);
      return out;
    },
  },
  watch: {
    "$route.params.zoneName"() {
      this.items = [];
      this.page = 1;
      this.hasMore = true;
      this.feedItems = [];
      this.feedPage = 1;
      this.feedHasMore = true;
      this.feedCursor = "";
      this.fetch();
      this.fetchFeed();
    },
  },
  created() {
    this.fetch();
    this.fetchFeed();
    window.addEventListener("scroll", this.onScroll);
  },
  beforeDestroy() {
    window.removeEventListener("scroll", this.onScroll);
  },
  methods: {
    async fetch() {
      this.loading = true;
      try {
        const res = await http.get("/api/v1/videos", {
          params: {
            zone_parent: this.zoneName,
            limit: 20,
            sort: "hot",
          },
        });
        if (res && res.code === 0 && res.data) {
          const list = (res.data.items || []).map((it) => ({
            aid: it.id,
            title: it.title || "",
            pic: it.cover_url || "",
            duration: it.duration || 0,
            play: it.play_count || 0,
            author: it.uploader || "",
          }));
          this.items = this.items.concat(list);
          this.hasMore = list.length >= 20;
        }
      } catch (e) {
        this.items = []; // keep skeleton visible
      } finally {
        this.loading = false;
      }
    },
    loadMore() {
      this.page++;
      this.fetch();
    },
    fmtDuration(sec) {
      const d = Number(sec) || 0;
      const m = Math.floor(d / 60);
      const s = Math.floor(d % 60);
      return m + ":" + String(s).padStart(2, "0");
    },
    fmtCount(n) {
      const v = Number(n) || 0;
      if (v >= 10000) return (v / 10000).toFixed(v >= 100000 ? 0 : 1) + "万";
      if (v >= 1000) return (v / 1000).toFixed(1) + "k";
      return String(v);
    },
    // ── Infinite scroll feed ──
    async fetchFeed() {
      if (this.feedLoading || !this.feedHasMore) return;
      this.feedLoading = true;
      try {
        const params = {
          zone_parent: this.zoneName,
          limit: 15,
          sort: "hot",
        };
        if (this.feedCursor) params.cursor = this.feedCursor;
        const res = await http.get("/api/v1/videos", { params });
        if (res && res.code === 0 && res.data) {
          const list = (res.data.items || []).map((it) => ({
            aid: it.id,
            title: it.title || "",
            pic: it.cover_url || "",
            duration: it.duration || 0,
            play: it.play_count || 0,
            author: it.uploader || "",
          }));
          this.feedItems = this.feedItems.concat(list);
          this.feedCursor = String(res.data.next_cursor || "");
          this.feedHasMore = list.length >= 15;
        } else {
          this.feedHasMore = false;
        }
      } catch (e) {
        this.feedHasMore = false;
      } finally {
        this.feedLoading = false;
      }
    },
    onScroll() {
      if (this.feedLoading) return;
      const bottom = window.innerHeight + window.scrollY;
      const doc = document.documentElement;
      if (bottom >= doc.scrollHeight - 300) {
        if (this.feedHasMore) {
          this.feedPage++;
          this.fetchFeed();
        } else {
          // Keep adding a full row of skeletons forever.
          for (let i = 0; i < 15; i++) this.feedItems.push(null);
        }
      }
    },
  },
};
</script>

<style scoped>
.zone-page {
  margin: 0 auto;
  max-width: 1400px;
  padding: 20px 56px 40px;
  min-height: 60vh;
  box-sizing: border-box;
}

/* ── 标题栏（参照 HTML .page-title） ── */
.zone-page-header {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 20px;
}
.zone-icon-wrap {
  width: 36px;
  height: 36px;
  background: #409EFF;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
}
.zone-icon {
  font-size: 20px;
  line-height: 1;
}
.zone-page-title {
  font-size: 24px;
  font-weight: 700;
  color: #222;
}

/* ── Row 1: 1大+3小（参照 HTML .row1） ── */
.zone-row1 {
  display: grid;
  grid-template-columns: 3fr 2fr 2fr 2fr;
  gap: 16px;
  margin-bottom: 16px;
}
.zone-row1 .video-card.is-big .card-title {
  font-size: 18px;
  font-weight: 600;
}

/* ── Row 2: 5 列等宽（参照 HTML .row2） ── */
.zone-row2 {
  display: grid;
  grid-template-columns: repeat(5, 1fr);
  gap: 16px;
}

/* ── Card（参照 HTML .video-card） ── */
.video-card {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.card-cover-wrap {
  position: relative;
  display: block;
  width: 100%;
  aspect-ratio: 16 / 9;
  background: #e8e8e8;
  border-radius: 4px;
  overflow: hidden;
}
.card-cover {
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block;
}
.duration-tag {
  position: absolute;
  bottom: 6px;
  right: 6px;
  background: rgba(0, 0, 0, 0.6);
  color: #fff;
  font-size: 12px;
  padding: 2px 6px;
  border-radius: 2px;
  line-height: 1.4;
}
.card-title {
  font-size: 15px;
  line-height: 1.4;
  color: #222;
  text-decoration: none;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
  word-break: break-all;
}
.card-title:hover { color: #00a1d6; }
.card-meta {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  color: #999;
}
.card-play { margin-left: auto; }

/* ── Skeleton 占位 ── */
.skeleton {
  position: relative;
  background: #e8e8e8 !important;
}
.skeleton-shimmer {
  position: absolute;
  inset: 0;
  background: linear-gradient(90deg, transparent 0%, rgba(255,255,255,0.5) 50%, transparent 100%);
  animation: shimmer 1.5s infinite;
}
@keyframes shimmer {
  0% { transform: translateX(-100%); }
  100% { transform: translateX(100%); }
}
.skeleton-text {
  height: 18px;
  background: #e8e8e8;
  border-radius: 3px;
  width: 80%;
}
.skeleton-text-short {
  height: 14px;
  background: #e8e8e8;
  border-radius: 3px;
  width: 50%;
}

/* ── "加载更多" ── */
.zone-more {
  text-align: center;
  color: #00a1d6;
  padding: 24px 0;
  cursor: pointer;
  font-size: 14px;
}

/* ── 响应式 ── */
@media (max-width: 1200px) {
  .zone-row1 { grid-template-columns: repeat(4, 1fr); }
  .zone-row1 .is-big .card-title { font-size: 15px; font-weight: 500; }
  .zone-row2 { grid-template-columns: repeat(3, 1fr); }
}
@media (max-width: 768px) {
  .zone-row1 { grid-template-columns: repeat(2, 1fr); }
  .zone-row2 { grid-template-columns: repeat(2, 1fr); }
}

/* ── Infinite scroll feed grid（同首页 5 列） ── */
.zone-feed {
  margin-top: 32px;
  padding-top: 24px;
  border-top: 1px solid #eee;
}
.zone-feed-grid {
  display: grid;
  grid-template-columns: repeat(5, 1fr);
  gap: 16px;
}
.feed-card {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.feed-cover-wrap {
  position: relative;
  display: block;
  aspect-ratio: 16 / 9;
  border-radius: 4px;
  overflow: hidden;
  background: #e8e8e8;
}
.feed-cover {
  width: 100%;
  height: 100%;
  object-fit: cover;
}
.feed-duration {
  position: absolute;
  bottom: 6px;
  right: 6px;
  background: rgba(0, 0, 0, 0.6);
  color: #fff;
  font-size: 12px;
  padding: 2px 6px;
  border-radius: 2px;
}
.feed-title {
  font-size: 14px;
  line-height: 1.4;
  color: #222;
  text-decoration: none;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
.feed-title:hover { color: #00a1d6; }
.feed-meta {
  font-size: 12px;
  color: #999;
}
.zone-feed-loading,
.zone-feed-end {
  text-align: center;
  color: #999;
  padding: 20px 0;
  font-size: 13px;
}

@media (max-width: 1200px) {
  .zone-feed-grid { grid-template-columns: repeat(4, 1fr); }
}
@media (max-width: 768px) {
  .zone-feed-grid { grid-template-columns: repeat(2, 1fr); }
}
</style>
