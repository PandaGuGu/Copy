<template>
  <div class="sp-page">
    <!-- 加载状态 -->
    <div v-if="loading" class="sp-loading">
      <div class="sp-loading-spinner" />
      <p>加载中...</p>
    </div>

    <!-- 不存在 -->
    <div v-else-if="!page" class="sp-empty">
      <p class="sp-empty-icon">🔍</p>
      <h2>专题不存在</h2>
      <p>该专题已被删除或暂未发布</p>
      <router-link to="/" class="sp-back-link">返回首页</router-link>
    </div>

    <!-- 专题内容 -->
    <template v-else>
      <div class="sp-hero">
        <img v-if="page.cover_url" :src="page.cover_url" :alt="page.title" class="sp-cover" />
        <div class="sp-hero-overlay">
          <h1 class="sp-title">{{ page.title }}</h1>
          <p v-if="page.description" class="sp-desc">{{ page.description }}</p>
        </div>
      </div>

      <div class="sp-body">
        <!-- 内容区块渲染 -->
        <div v-for="(block, idx) in blocks" :key="idx" class="sp-block">
          <!-- 视频区块 -->
          <template v-if="block.type === 'videos'">
            <h3 v-if="block.title" class="sp-block-title">{{ block.title }}</h3>
            <div class="sp-video-grid">
              <router-link
                v-for="v in block.video_ids"
                :key="v"
                :to="'/video/av' + v"
                class="sp-video-card"
              >
                <div class="sp-video-placeholder">
                  <span class="sp-video-icon">▶</span>
                </div>
                <p class="sp-video-label">视频 #{{ v }}</p>
              </router-link>
            </div>
          </template>

          <!-- Banner 区块 -->
          <template v-else-if="block.type === 'banner'">
            <a
              v-if="block.image_url"
              :href="block.link_url || '#'"
              class="sp-banner-link"
              :target="block.link_url ? '_blank' : '_self'"
            >
              <img :src="block.image_url" :alt="block.title || ''" class="sp-banner-img" />
            </a>
          </template>

          <!-- 文本/富文本区块 -->
          <template v-else-if="block.type === 'text'">
            <h3 v-if="block.title" class="sp-block-title">{{ block.title }}</h3>
            <div class="sp-text-block" v-html="block.content_html || block.content || ''" />
          </template>

          <!-- 分区视频区块 -->
          <template v-else-if="block.type === 'zone'">
            <h3 v-if="block.title" class="sp-block-title">{{ block.title }}</h3>
            <p v-if="block.zone_name" class="sp-zone-hint">
              来自「{{ block.zone_name }}」分区的最新视频
            </p>
            <div class="sp-zone-placeholder">
              <span class="sp-zone-await">分区视频加载中...</span>
            </div>
          </template>

          <!-- 未知类型 -->
          <div v-else class="sp-unknown-block">
            <p>未知内容类型: {{ block.type }}</p>
          </div>
        </div>

        <!-- 空区块提示 -->
        <div v-if="blocks.length === 0" class="sp-no-blocks">
          <p>该专题暂无内容区块，敬请期待。</p>
        </div>
      </div>
    </template>
  </div>
</template>

<script>
import http from "@/utils/http";

export default {
  name: "SpecialPage",
  data() {
    return {
      loading: true,
      page: null,
      blocks: []
    };
  },
  computed: {
    slug() {
      return this.$route.params.slug || "";
    }
  },
  watch: {
    slug: {
      immediate: true,
      handler(val) {
        if (val) this.fetchPage(val);
      }
    }
  },
  methods: {
    async fetchPage(slug) {
      this.loading = true;
      this.page = null;
      this.blocks = [];
      try {
        const res = await http.get("/api/v1/specials/" + encodeURIComponent(slug));
        const body = res && res.data ? res.data : res;
        if (!body || body.code !== 0) {
          throw new Error((body && body.msg) || "加载失败");
        }
        this.page = body.data || null;
        if (this.page) {
          this.blocks = this.parseBlocks(this.page.blocks);
        }
      } catch (e) {
        console.warn("专题页加载失败:", slug, e);
        this.page = null;
        this.blocks = [];
      } finally {
        this.loading = false;
      }
    },
    parseBlocks(blocksRaw) {
      if (!blocksRaw) return [];
      try {
        const parsed = typeof blocksRaw === "string" ? JSON.parse(blocksRaw) : blocksRaw;
        return Array.isArray(parsed) ? parsed : [];
      } catch {
        return [];
      }
    }
  }
};
</script>

<style scoped>
.sp-page {
  max-width: 960px;
  margin: 0 auto;
  padding: 0 16px 40px;
  min-height: 60vh;
}

/* loading */
.sp-loading {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 120px 0;
  color: #999;
}
.sp-loading-spinner {
  width: 40px;
  height: 40px;
  border: 3px solid #e8e8e8;
  border-top-color: #00a1d6;
  border-radius: 50%;
  animation: sp-spin 0.8s linear infinite;
  margin-bottom: 16px;
}
@keyframes sp-spin {
  to { transform: rotate(360deg); }
}

/* empty */
.sp-empty {
  text-align: center;
  padding: 120px 0;
  color: #999;
}
.sp-empty-icon {
  font-size: 64px;
  margin: 0 0 16px;
}
.sp-empty h2 {
  font-size: 20px;
  color: #333;
  margin: 0 0 8px;
}
.sp-back-link {
  display: inline-block;
  margin-top: 16px;
  color: #00a1d6;
  text-decoration: none;
  font-size: 14px;
}
.sp-back-link:hover {
  text-decoration: underline;
}

/* hero */
.sp-hero {
  position: relative;
  margin: 0 -16px 24px;
  border-radius: 0 0 12px 12px;
  overflow: hidden;
  background: linear-gradient(135deg, #1a1a2e 0%, #16213e 50%, #0f3460 100%);
  min-height: 200px;
}
.sp-cover {
  width: 100%;
  height: 240px;
  object-fit: cover;
  display: block;
  opacity: 0.6;
}
.sp-hero-overlay {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  padding: 32px 24px;
  background: linear-gradient(transparent, rgba(0,0,0,0.6));
}
.sp-title {
  font-size: 28px;
  font-weight: 600;
  color: #fff;
  margin: 0 0 8px;
  text-shadow: 0 1px 4px rgba(0,0,0,0.4);
}
.sp-desc {
  font-size: 14px;
  color: rgba(255,255,255,0.85);
  margin: 0;
  line-height: 1.6;
}

/* body */
.sp-body {
  padding: 0;
}

/* block title */
.sp-block-title {
  font-size: 18px;
  font-weight: 600;
  color: #222;
  margin: 28px 0 12px;
  padding-left: 8px;
  border-left: 4px solid #00a1d6;
}

/* video grid */
.sp-video-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
  gap: 12px;
}
.sp-video-card {
  display: flex;
  flex-direction: column;
  text-decoration: none;
  color: #333;
  border-radius: 8px;
  overflow: hidden;
  background: #f5f5f5;
  transition: transform 0.15s ease, box-shadow 0.15s ease;
}
.sp-video-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0,0,0,0.1);
}
.sp-video-placeholder {
  aspect-ratio: 16/9;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  display: flex;
  align-items: center;
  justify-content: center;
}
.sp-video-icon {
  font-size: 32px;
  color: rgba(255,255,255,0.7);
}
.sp-video-label {
  padding: 8px 10px;
  font-size: 13px;
  color: #666;
  margin: 0;
  text-align: center;
}

/* banner */
.sp-banner-link {
  display: block;
  margin: 12px 0;
  border-radius: 8px;
  overflow: hidden;
}
.sp-banner-img {
  width: 100%;
  display: block;
  border-radius: 8px;
}

/* text */
.sp-text-block {
  font-size: 14px;
  line-height: 1.8;
  color: #444;
  max-width: 720px;
}
.sp-text-block :deep(p) {
  margin: 0 0 12px;
}
.sp-text-block :deep(img) {
  max-width: 100%;
  border-radius: 8px;
}

/* zone placeholder */
.sp-zone-hint {
  font-size: 13px;
  color: #999;
  margin: 4px 0 12px;
}
.sp-zone-placeholder {
  padding: 40px;
  text-align: center;
  background: #fafafa;
  border-radius: 8px;
  border: 1px dashed #ddd;
}
.sp-zone-await {
  color: #999;
  font-size: 14px;
}

/* unknown */
.sp-unknown-block {
  padding: 24px;
  background: #fffbe6;
  border: 1px solid #ffe58f;
  border-radius: 8px;
  color: #ad8b00;
  font-size: 13px;
}
.sp-unknown-block p {
  margin: 0;
}

/* no blocks */
.sp-no-blocks {
  padding: 60px 0;
  text-align: center;
  color: #999;
  font-size: 14px;
}
</style>
