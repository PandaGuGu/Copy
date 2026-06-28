<template>
  <div class="trending-page">
    <div class="tab-nav">
      <div
        v-for="t in tabs"
        :key="t.key"
        :class="['tab-item', { active: activeTab === t.key }]"
        @click="activeTab = t.key"
      >
        {{ t.icon }} {{ t.label }}
      </div>
    </div>

    <div class="desc-row">
      <span>实时热门内容，每30分钟更新</span>
      <span>共 {{ items.length }} 条</span>
    </div>

    <div class="video-grid">
      <div v-for="item in displayItems" :key="item.aid" class="video-card">
        <router-link :to="`/video/BV${item.aid}`" class="video-cover-wrap">
          <img :src="item.pic" :alt="item.title" class="video-cover" loading="lazy" />
          <span class="duration-tag">{{ item.duration }}</span>
        </router-link>
        <div class="video-info">
          <router-link :to="`/video/BV${item.aid}`" class="video-title">{{ item.title }}</router-link>
          <span v-if="item.tags[0]" class="hot-tag">{{ item.tags[0] }}</span>
          <div class="meta-row">
            <div class="meta-item">
              <span>UP：{{ item.author }}</span>
            </div>
            <div class="meta-item">
              <span>播放 {{ fmtCount(item.play) }}</span>
              <span>评论 {{ fmtCount(item.danmaku) }}</span>
            </div>
          </div>
        </div>
      </div>

      <div v-for="i in skeletonCount" :key="'t-sk-'+i" class="video-card">
        <div class="video-cover skeleton"><div class="skeleton-shimmer"></div></div>
        <div class="video-info">
          <div class="skeleton-text" style="width:70%;height:16px;"></div>
          <div class="skeleton-text" style="width:30%;height:12px;"></div>
          <div class="skeleton-text" style="width:50%;height:12px;"></div>
        </div>
      </div>
    </div>

    <div v-if="loading" class="load-more">加载中...</div>
  </div>
</template>

<script>
import http from "@/utils/http";

export default {
  name: "TrendingPage",
  data() {
    return {
      items: [],
      loading: true,
      activeTab: "all",
      tabs: [
        { key: "all", label: "综合", icon: "🔥" },
        { key: "游戏", label: "游戏", icon: "🎮" },
        { key: "科技", label: "科技", icon: "💻" },
        { key: "生活", label: "生活", icon: "🏠" },
        { key: "影视", label: "影视", icon: "🎞️" },
      ],
    };
  },
  computed: {
    displayItems() {
      return this.items
        .filter((v) => this.activeTab === "all" || v.zone_parent === this.activeTab || v.zone === this.activeTab)
        .slice(0, 20)
        .map((v) => ({
          aid: v.id,
          title: v.title || "",
          pic: v.cover_url || "",
          duration: this.fmtDuration(v.duration),
          play: v.play_count || 0,
          danmaku: v.danmaku_count || 0,
          author: v.uploader || "",
          zone: v.zone || "",
          zone_parent: v.zone_parent || "",
          tags: this.parseTags(v.tags_json),
        }));
    },
    skeletonCount() {
      return Math.max(0, 8 - this.displayItems.length);
    },
  },
  created() {
    this.fetch();
  },
  methods: {
    async fetch() {
      this.loading = true;
      try {
        const res = await http.get("/api/v1/videos", { params: { limit: 50, sort: "hot" } });
        if (res && res.code === 0 && res.data) {
          this.items = res.data.items || [];
        }
      } catch (e) {
        console.warn("TrendingPage:", e);
      } finally {
        this.loading = false;
      }
    },
    parseTags(json) {
      try { return JSON.parse(json || "[]"); } catch { return []; }
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
      return String(v);
    },
  },
};
</script>

<style scoped>
.trending-page {
  background: #fff;
  min-height: 100vh;
  max-width: 1400px;
  margin: 0 auto;
}
.top-banner {
  width: 100%; height: 140px; background: linear-gradient(135deg, #f97316, #fb7299);
  display: flex; align-items: center; justify-content: center;
}
.banner-content { font-size: 28px; color: #fff; font-weight: 700; }
.tab-nav {
  display: flex; gap: 40px; padding: 24px 30px;
  border-bottom: 1px solid #eee;
}
.tab-item {
  display: flex; align-items: center; gap: 8px;
  font-size: 18px; color: #666; cursor: pointer; position: relative;
}
.tab-item.active { color: #fb7299; }
.tab-item.active::after {
  content: ""; position: absolute; left: 0; bottom: -24px;
  width: 100%; height: 3px; background: #fb7299;
}
.desc-row {
  display: flex; justify-content: space-between;
  padding: 20px 30px; font-size: 14px; color: #777;
}
.video-grid {
  display: grid; grid-template-columns: 1fr 1fr;
  gap: 30px; padding: 0 30px 40px;
}
.video-card { display: flex; gap: 16px; }
.video-cover-wrap {
  flex-shrink: 0; display: block; position: relative;
}
.video-cover {
  width: 220px; height: 130px; background: #ddd;
  border-radius: 4px; object-fit: cover; display: block;
}
.duration-tag {
  position: absolute; bottom: 6px; right: 6px;
  background: rgba(0,0,0,0.6); color: #fff;
  font-size: 12px; padding: 2px 6px; border-radius: 2px;
}
.video-info {
  flex: 1; display: flex; flex-direction: column; gap: 10px;
}
.video-title {
  font-size: 16px; font-weight: 500; color: #111;
  text-decoration: none; display: -webkit-box;
  -webkit-line-clamp: 2; -webkit-box-orient: vertical; overflow: hidden;
}
.video-title:hover { color: #fb7299; }
.hot-tag {
  width: fit-content; padding: 2px 6px;
  background: #ffd390; font-size: 12px;
  color: #d36c00; border-radius: 2px;
}
.meta-row { font-size: 13px; color: #888; display: flex; flex-direction: column; gap: 4px; }
.meta-item { display: flex; gap: 14px; }
.load-more { text-align: center; color: #999; padding: 20px; }
.skeleton { position: relative; background: #e8e8e8 !important; overflow: hidden; }
.skeleton-shimmer {
  position: absolute; inset: 0;
  background: linear-gradient(90deg, transparent 0%, rgba(255,255,255,0.5) 50%, transparent 100%);
  animation: shimmer 1.5s infinite;
}
@keyframes shimmer { 0%{transform:translateX(-100%)} 100%{transform:translateX(100%)} }
.skeleton-text { background: #e8e8e8; border-radius: 4px; }

@media (max-width: 900px) {
  .video-grid { grid-template-columns: 1fr; }
  .tab-nav { gap: 20px; padding: 16px 16px; }
}
</style>
