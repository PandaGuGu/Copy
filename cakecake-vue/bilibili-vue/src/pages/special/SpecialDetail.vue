<template>
  <div class="special-detail">
    <div v-if="loading" class="sd-loading">加载中...</div>
    <div v-else-if="!page" class="sd-empty">
      <h2>专题不存在</h2>
      <router-link to="/activity">返回活动列表</router-link>
    </div>
    <template v-else>
      <div class="sd-hero" :style="page.cover_url ? { backgroundImage: `url(${page.cover_url})` } : {}">
        <div class="sd-hero-overlay">
          <h1 class="sd-title">{{ page.title }}</h1>
          <p v-if="page.description" class="sd-desc">{{ page.description }}</p>
        </div>
      </div>
      <div class="sd-body">
        <div v-for="(block, idx) in blocks" :key="idx" class="sd-block">
          <template v-if="block.type === 'banner'">
            <div v-if="block.image_url" class="sd-banner">
              <img :src="block.image_url" :alt="block.title || ''" />
            </div>
            <h3 v-if="block.title" class="sd-block-title">{{ block.title }}</h3>
          </template>
          <template v-else-if="block.type === 'text'">
            <div class="sd-text" v-html="block.content"></div>
          </template>
          <template v-else-if="block.type === 'videos'">
            <h3 v-if="block.title" class="sd-block-title">{{ block.title }}</h3>
            <div class="sd-video-grid">
              <a v-for="v in block.video_ids" :key="v" :href="`#/video/av${v}`" class="sd-video-card">
                <span class="sd-video-icon">▶</span>
                <span>视频 #{{ v }}</span>
              </a>
            </div>
          </template>
        </div>
        <div v-if="blocks.length === 0" class="sd-empty-hint">
          <p>暂无详细内容</p>
        </div>
      </div>
    </template>
  </div>
</template>

<script>
import http from "@/utils/http";

export default {
  name: "SpecialDetail",
  data() {
    return { page: null, loading: true };
  },
  computed: {
    blocks() {
      if (!this.page || !this.page.blocks) return [];
      try {
        return typeof this.page.blocks === "string"
          ? JSON.parse(this.page.blocks)
          : this.page.blocks;
      } catch {
        return [];
      }
    },
  },
  created() {
    this.fetch();
  },
  methods: {
    async fetch() {
      this.loading = true;
      try {
        const slug = this.$route.params.slug;
        const res = await http.get(`/api/v1/specials/${slug}`);
        if (res && res.code === 0 && res.data) {
          this.page = res.data;
        }
      } catch (e) {
        console.warn("SpecialDetail:", e);
      } finally {
        this.loading = false;
      }
    },
  },
};
</script>

<style scoped>
.special-detail { min-height: 60vh; background: #f5f5f7; }
.sd-loading, .sd-empty { text-align: center; padding: 80px 20px; color: #999; }
.sd-empty a { color: #fb7299; }
.sd-hero {
  height: 260px; background: linear-gradient(135deg, #667eea, #764ba2); background-size: cover;
  background-position: center; position: relative; display: flex; align-items: flex-end;
}
.sd-hero-overlay {
  width: 100%; max-width: 1160px; margin: 0 auto; padding: 40px 60px;
  background: linear-gradient(transparent, rgba(0,0,0,0.6));
}
.sd-title { color: #fff; font-size: 28px; font-weight: 700; margin: 0; }
.sd-desc { color: rgba(255,255,255,0.85); margin-top: 8px; font-size: 15px; }
.sd-body { max-width: 900px; margin: 0 auto; padding: 32px 60px 60px; }
.sd-block { margin-bottom: 28px; }
.sd-block-title { font-size: 20px; font-weight: 600; color: #222; margin: 0 0 12px; }
.sd-text { font-size: 15px; line-height: 1.8; color: #333; }
.sd-banner img { width: 100%; max-height: 400px; object-fit: cover; border-radius: 8px; }
.sd-video-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(140px, 1fr)); gap: 12px; }
.sd-video-card {
  display: flex; flex-direction: column; align-items: center; gap: 8px;
  padding: 20px 12px; background: #fff; border-radius: 8px; text-decoration: none;
  color: #666; font-size: 13px; box-shadow: 0 1px 4px rgba(0,0,0,0.06);
}
.sd-video-icon { font-size: 24px; }
.sd-empty-hint { text-align: center; color: #aaa; padding: 40px; }
</style>
