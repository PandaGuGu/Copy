<template>
  <div class="zone-section">
    <!-- 分区标题栏（参照 HTML .page-title） -->
    <div class="zone-header">
      <span class="zone-icon">{{ icon }}</span>
      <span class="zone-name">{{ name }}</span>
      <router-link :to="`/zone/${name}`" class="zone-more">查看更多 &gt;</router-link>
    </div>

    <!-- 第一行：1大+3小（参照 HTML .row1） -->
    <div class="zone-row1">
      <div v-for="i in 4" :key="'r1-' + i" class="video-card" :class="{ 'is-big': i === 1 }">
        <template v-if="items[i - 1]">
          <router-link :to="videoLink(items[i - 1].aid)" class="card-cover-wrap">
            <img :src="items[i - 1].pic" :alt="items[i - 1].title" class="card-cover" loading="lazy" />
            <span class="duration-tag">{{ fmtDuration(items[i - 1].duration) }}</span>
          </router-link>
          <router-link :to="videoLink(items[i - 1].aid)" class="card-title">{{ items[i - 1].title }}</router-link>
          <div class="card-meta">
            <span class="card-up">{{ items[i - 1].author }}</span>
            <span class="card-play">{{ fmtCount(items[i - 1].play) }} 播放</span>
          </div>
        </template>
        <template v-else>
          <div class="card-cover-wrap skeleton"><div class="skeleton-shimmer"></div></div>
          <div class="card-title skeleton-text"></div>
          <div class="card-meta skeleton-text-short"></div>
        </template>
      </div>
    </div>

    <!-- 第二行：5 列等宽（参照 HTML .row2） -->
    <div class="zone-row2">
      <div v-for="i in 5" :key="'r2-' + i" class="video-card">
        <template v-if="items[3 + i]">
          <router-link :to="videoLink(items[3 + i].aid)" class="card-cover-wrap">
            <img :src="items[3 + i].pic" :alt="items[3 + i].title" class="card-cover" loading="lazy" />
            <span class="duration-tag">{{ fmtDuration(items[3 + i].duration) }}</span>
          </router-link>
          <router-link :to="videoLink(items[3 + i].aid)" class="card-title">{{ items[3 + i].title }}</router-link>
          <div class="card-meta">
            <span class="card-up">{{ items[3 + i].author }}</span>
            <span class="card-play">{{ fmtCount(items[3 + i].play) }} 播放</span>
          </div>
        </template>
        <template v-else>
          <div class="card-cover-wrap skeleton"><div class="skeleton-shimmer"></div></div>
          <div class="card-title skeleton-text"></div>
          <div class="card-meta skeleton-text-short"></div>
        </template>
      </div>
    </div>
  </div>
</template>

<script>
import http from "@/utils/http";

export default {
  name: "ZoneSection",
  props: {
    name: { type: String, required: true },
    icon: { type: String, default: "📺" },
    zoneParent: { type: String, required: true },
  },
  data() {
    return { items: [], loaded: false };
  },
  created() {
    this.fetch();
  },
  methods: {
    async fetch() {
      try {
        const res = await http.get("/api/v1/videos", {
          params: { zone_parent: this.zoneParent, limit: 9, sort: "hot" },
        });
        if (res && res.code === 0 && res.data) {
          this.items = (res.data.items || []).map((it) => ({
            aid: it.id,
            title: it.title || "",
            pic: it.cover_url || "",
            duration: it.duration || 0,
            play: it.play_count || 0,
            author: it.uploader || "",
          }));
        }
      } catch (e) {
        console.warn("ZoneSection fetch error:", this.zoneParent, e);
      } finally {
        this.loaded = true;
      }
    },
    videoLink(aid) {
      return { name: "video", params: { aid: "BV" + aid } };
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
  },
};
</script>

<style scoped>
.zone-section {
  margin-bottom: 32px;
}

/* ── 标题栏 ── */
.zone-header {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 20px;
}
.zone-icon { font-size: 24px; }
.zone-name {
  font-size: 22px;
  font-weight: 700;
  color: #222;
}
.zone-more {
  margin-left: auto;
  font-size: 13px;
  color: #999;
  text-decoration: none;
  border: 1px solid #e0e0e0;
  border-radius: 4px;
  padding: 4px 12px;
  transition: all 0.2s;
}
.zone-more:hover { color: #00a1d6; border-color: #00a1d6; }

/* ── Row 1: 1大+3小（3fr+2fr+2fr+2fr） ── */
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

/* ── Row 2: 5 列等宽 ── */
.zone-row2 {
  display: grid;
  grid-template-columns: repeat(5, 1fr);
  gap: 16px;
}

/* ── Card ── */
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
  color: #aaa;
}
.card-play { margin-left: auto; }

/* ── Skeleton ── */
.skeleton {
  background: #e8e8e8 !important;
  position: relative;
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

/* ── 响应式 ── */
@media (max-width: 1200px) {
  .zone-row1 { grid-template-columns: repeat(4, 1fr); }
  .zone-row1 .is-big .card-title { font-size: 15px; font-weight: 500; }
  .zone-row2 { grid-template-columns: repeat(3, 1fr); }
}
</style>
