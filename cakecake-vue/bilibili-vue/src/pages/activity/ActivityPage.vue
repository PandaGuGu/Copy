<template>
  <div class="activity-page">
    <div class="page-header">
      <div class="header-title">
        <span>🚩</span>
        <span>活动列表</span>
      </div>
      <div class="tab-group">
        <div
          v-for="t in tabs"
          :key="t.key"
          :class="['tab-item', { active: activeTab === t.key }]"
          @click="activeTab = t.key"
        >
          {{ t.label }}
        </div>
      </div>
    </div>

    <div class="activity-wrap">
      <div v-for="item in items" :key="item.id" class="activity-card">
        <router-link :to="`/special/${item.slug}`" class="activity-cover-wrap">
          <img v-if="item.cover" :src="item.cover" class="activity-cover" loading="lazy" />
          <div v-else class="activity-cover skeleton"><div class="skeleton-shimmer"></div></div>
        </router-link>
        <div class="activity-info">
          <router-link :to="`/special/${item.slug}`" class="activity-name">{{ item.title }}</router-link>
          <div class="tag-list">
            <span v-for="t in item.tags" :key="t" class="tag-item">{{ t }}</span>
          </div>
          <div class="activity-time">{{ item.time }}</div>
        </div>
      </div>

      <!-- skeleton placeholders -->
      <div v-for="i in skeletonCount" :key="'sk-' + i" class="activity-card">
        <div class="activity-cover skeleton"><div class="skeleton-shimmer"></div></div>
        <div class="activity-info">
          <div class="skeleton-text" style="width:70%;height:20px;"></div>
          <div class="skeleton-text" style="width:50%;height:14px;margin-top:8px;"></div>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
import http from "@/utils/http";

export default {
  name: "ActivityPage",
  data() {
    return {
      items: [],
      activeTab: "active",
      tabs: [
        { key: "active", label: "进行中" },
        { key: "ended", label: "已结束" },
      ],
    };
  },
  computed: {
    skeletonCount() {
      return Math.max(0, 4 - this.items.length);
    },
  },
  created() {
    this.fetch();
  },
  methods: {
    async fetch() {
      try {
        const res = await http.get("/api/v1/specials");
        if (res && res.code === 0 && res.data) {
          const list = (res.data.items || res.data || []).map((s) => ({
            id: s.id,
            slug: s.slug || s.id,
            title: s.title || "",
            cover: s.cover_url || s.banner_url || "",
            tags: s.tags_json ? JSON.parse(s.tags_json) : [],
            time: this.fmtRange(s.start_at, s.end_at),
          }));
          this.items = list;
        }
      } catch (e) {
        console.warn("ActivityPage fetch:", e);
      }
    },
    fmtRange(start, end) {
      const fmt = (t) => t ? new Date(t).toLocaleDateString("zh-CN") : "";
      if (fmt(start) && fmt(end)) return fmt(start) + " — " + fmt(end);
      return fmt(start) || fmt(end);
    },
  },
};
</script>

<style scoped>
.activity-page {
  max-width: 1160px;
  margin: 0 auto;
  padding: 32px 60px 60px;
  min-height: 60vh;
}
.page-header {
  display: flex;
  align-items: center;
  gap: 32px;
  margin-bottom: 36px;
  padding-bottom: 12px;
  border-bottom: 1px solid #eee;
}
.header-title {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 26px;
  font-weight: 600;
}
.tab-group {
  display: flex;
  gap: 40px;
  font-size: 20px;
}
.tab-item {
  color: #999;
  cursor: pointer;
  padding-bottom: 8px;
}
.tab-item.active {
  color: #ff5682;
  border-bottom: 3px solid #ff5682;
}
.activity-wrap {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 32px 24px;
}
.activity-card {
  display: flex;
  gap: 16px;
}
.activity-cover-wrap {
  flex-shrink: 0;
  text-decoration: none;
}
.activity-cover {
  width: 140px;
  height: 140px;
  border-radius: 6px;
  background: #e5e7eb;
  object-fit: cover;
  display: block;
}
.activity-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.activity-name {
  font-size: 20px;
  font-weight: 500;
  color: #222;
  text-decoration: none;
}
.activity-name:hover { color: #ff5682; }
.tag-list {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}
.tag-item {
  padding: 3px 10px;
  background: #f0f1f3;
  color: #666;
  font-size: 13px;
  border-radius: 4px;
}
.activity-time {
  color: #888;
  font-size: 15px;
}
.skeleton {
  position: relative;
  background: #e8e8e8 !important;
  overflow: hidden;
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
  background: #e8e8e8;
  border-radius: 4px;
}
</style>
