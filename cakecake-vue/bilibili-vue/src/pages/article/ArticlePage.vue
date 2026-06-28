<template>
  <div class="article-page">
    <div class="header-wrap">
      <h1 class="title">推荐文章</h1>
      <input
        class="search-input"
        v-model="searchKeyword"
        placeholder="搜索专栏文章"
        @keyup.enter="doSearch"
      />
    </div>

    <div class="article-list">
      <div
        v-for="(item, idx) in displayItems"
        :key="item.id"
        class="article-item"
      >
        <div class="article-main">
          <div style="display: flex; align-items: center; gap: 8px;">
            <h2 :class="idx === 0 ? 'art-title-blue' : 'art-title-black'">
              {{ item.title }}
            </h2>
            <span v-if="item.category" class="tag-gray">{{ item.category }}</span>
          </div>
          <p class="art-desc">{{ item.desc }}</p>
          <div class="art-meta-row">
            <div class="author-wrap">
              <img
                :src="item.avatar || `https://api.dicebear.com/7.x/initials/svg?seed=${item.author}`"
                class="author-avatar"
              />
              <span>{{ item.author }}</span>
            </div>
            <span>{{ item.time }}</span>
            <div class="stat-item">
              <span>👁</span>
              <span>{{ fmtCount(item.views) }}</span>
            </div>
            <div class="stat-item">
              <span>👍</span>
              <span>{{ fmtCount(item.likes) }}</span>
            </div>
            <div class="stat-item">
              <span>💬</span>
              <span>{{ fmtCount(item.comments) }}</span>
            </div>
          </div>
        </div>
        <router-link :to="`/article/${item.id}`" class="article-thumb-wrap">
          <img
            v-if="item.cover"
            :src="item.cover"
            class="article-thumb"
            loading="lazy"
          />
          <div v-else class="article-thumb skeleton">
            <div class="skeleton-shimmer"></div>
          </div>
        </router-link>
      </div>

      <!-- 骨架占位（始终至少 4 条） -->
      <div
        v-for="i in skeletonCount"
        :key="'sk-' + i"
        class="article-item"
      >
        <div class="article-main">
          <div class="skeleton-text" style="width:60%; height:26px; margin-bottom:8px;"></div>
          <div class="skeleton-text" style="width:90%; height:15px; margin-bottom:6px;"></div>
          <div class="skeleton-text" style="width:40%; height:14px;"></div>
        </div>
        <div class="article-thumb skeleton">
          <div class="skeleton-shimmer"></div>
        </div>
      </div>
    </div>

    <div v-if="loading" class="load-more">加载中...</div>
    <div v-if="!hasMore && items.length" class="load-more">— 已经到底了 —</div>
  </div>
</template>

<script>
import http from "@/utils/http";

export default {
  name: "ArticlePage",
  data() {
    return {
      items: [],
      loading: false,
      hasMore: true,
      page: 1,
      searchKeyword: "",
    };
  },
  computed: {
    displayItems() {
      return this.items.map((a) => ({
        id: a.id,
        title: a.title || "无标题",
        desc: this.stripHtml(a.body_md || a.summary || ""),
        author: a.author || a.uploader || "作者",
        avatar: a.avatar_url || "",
        cover: a.cover_url || "",
        time: this.fmtTime(a.published_at || a.created_at),
        views: a.view_count || 0,
        likes: a.like_count || 0,
        comments: a.comment_count || 0,
        category: a.category || "",
      }));
    },
    skeletonCount() {
      const min = Math.max(4, this.items.length + (5 - (this.items.length % 5)));
      return Math.max(0, min - this.items.length);
    },
  },
  created() {
    this.fetch();
    window.addEventListener("scroll", this.onScroll);
  },
  beforeDestroy() {
    window.removeEventListener("scroll", this.onScroll);
  },
  methods: {
    async fetch() {
      if (this.loading) return;
      this.loading = true;
      try {
        const params = { limit: 20, page: this.page };
        if (this.searchKeyword) params.keyword = this.searchKeyword;
        const res = await http.get("/api/v1/articles", { params });
        if (res && res.code === 0 && res.data) {
          const list = res.data.items || res.data || [];
          this.items = this.items.concat(list);
          this.hasMore = list.length >= 20;
        }
      } catch (e) {
        console.warn("ArticlePage: fetch error", e);
      } finally {
        this.loading = false;
      }
    },
    doSearch() {
      this.items = [];
      this.page = 1;
      this.hasMore = true;
      this.fetch();
    },
    onScroll() {
      if (!this.hasMore || this.loading) return;
      if (window.innerHeight + window.scrollY >= document.documentElement.scrollHeight - 300) {
        this.page++;
        this.fetch();
      }
    },
    stripHtml(md) {
      return String(md || "")
        .replace(/<[^>]+>/g, "")
        .replace(/[#*_~`>\[\]()!|\\]/g, "")
        .trim()
        .slice(0, 120);
    },
    fmtTime(t) {
      if (!t) return "";
      const d = new Date(t);
      if (isNaN(d)) return String(t).slice(0, 10);
      return d.toLocaleDateString("zh-CN");
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
.article-page {
  max-width: 1160px;
  margin: 0 auto;
  padding: 30px 60px 60px;
  min-height: 60vh;
}

.header-wrap {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 30px;
}
.title {
  font-size: 28px;
  font-weight: bold;
  color: #111;
}
.search-input {
  width: 360px;
  height: 46px;
  border: 1px solid #ddd;
  border-radius: 8px;
  padding: 0 16px;
  font-size: 16px;
  background: #f8f9fa;
  outline: none;
}
.search-input:focus { border-color: #00a1d6; }

.article-list {
  display: flex;
  flex-direction: column;
  gap: 32px;
}
.article-item {
  display: flex;
  gap: 20px;
}
.article-main {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.article-thumb-wrap {
  flex-shrink: 0;
}
.article-thumb {
  width: 160px;
  height: 120px;
  object-fit: cover;
  border-radius: 4px;
  background: #eee;
  display: block;
}

.art-title-blue {
  font-size: 26px;
  color: #0088ee;
  font-weight: 500;
  margin: 0;
}
.art-title-black {
  font-size: 22px;
  color: #000;
  font-weight: bold;
  margin: 0;
}
.art-desc {
  font-size: 15px;
  color: #666;
  line-height: 1.5;
  margin: 0;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
.art-meta-row {
  display: flex;
  align-items: center;
  gap: 16px;
  font-size: 14px;
  color: #888;
}
.author-wrap {
  display: flex;
  align-items: center;
  gap: 6px;
}
.author-avatar {
  width: 24px;
  height: 24px;
  border-radius: 50%;
  background: #ddd;
}
.tag-gray {
  color: #999;
  background: #f0f1f3;
  padding: 2px 8px;
  border-radius: 4px;
  font-size: 12px;
}
.stat-item {
  display: flex;
  align-items: center;
  gap: 4px;
}

/* Skeleton */
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
.load-more {
  text-align: center;
  color: #999;
  padding: 24px 0;
  font-size: 14px;
}

@media (max-width: 900px) {
  .article-page { padding: 20px 20px 40px; }
  .article-item { flex-direction: column-reverse; }
  .article-thumb { width: 100%; height: 180px; }
}
</style>
