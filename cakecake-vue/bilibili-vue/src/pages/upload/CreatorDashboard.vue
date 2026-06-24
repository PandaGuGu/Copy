<template>
  <CreatorShell>
    <div class="cd-page" v-loading="loading">
      <header class="cd-head">
        <h2 class="cd-title">数据中心</h2>
        <p class="cd-sub">实时掌握你的稿件表现</p>
      </header>

      <!-- Stats cards -->
      <section class="cd-cards">
        <div class="cd-card">
          <span class="cd-card-icon cd-icon--video">▶</span>
          <div class="cd-card-body">
            <span class="cd-card-val">{{ fmtNum(stats.total_videos) }}</span>
            <span class="cd-card-label">视频总数</span>
          </div>
        </div>
        <div class="cd-card">
          <span class="cd-card-icon cd-icon--play">▶</span>
          <div class="cd-card-body">
            <span class="cd-card-val">{{ fmtNum(stats.total_plays) }}</span>
            <span class="cd-card-label">总播放量</span>
          </div>
        </div>
        <div class="cd-card">
          <span class="cd-card-icon cd-icon--coin">⛁</span>
          <div class="cd-card-body">
            <span class="cd-card-val">{{ fmtNum(stats.total_coins) }}</span>
            <span class="cd-card-label">总硬币</span>
          </div>
        </div>
        <div class="cd-card">
          <span class="cd-card-icon cd-icon--fan">♥</span>
          <div class="cd-card-body">
            <span class="cd-card-val">{{ fmtNum(stats.total_fans) }}</span>
            <span class="cd-card-label">粉丝数</span>
          </div>
        </div>
      </section>

      <!-- 7-Day trend chart -->
      <section class="cd-section" v-if="stats.trend_7d && stats.trend_7d.length > 0">
        <h3 class="cd-section-title">近 7 日播放趋势</h3>
        <div class="cd-chart-wrap">
          <svg class="cd-line-chart" :viewBox="'0 0 ' + chartWidth + ' ' + chartHeight">
            <line v-for="(_, gi) in 4" :key="'g'+gi"
              :x1="chartPadL" :y1="chartPadT + gi * chartStepY"
              :x2="chartW" :y2="chartPadT + gi * chartStepY"
              stroke="#eee" stroke-width="1" />
            <polyline :points="chartPoints"
              fill="none" stroke="#00a1d6" stroke-width="2.5" />
            <circle v-for="(p, di) in chartDots" :key="'d'+di"
              :cx="p.x" :cy="p.y" r="4"
              fill="#fff" stroke="#00a1d6" stroke-width="2" />
            <text v-for="(d, di) in stats.trend_7d" :key="'xl'+di"
              :x="chartPadL + di * chartStepX" :y="chartH + 18"
              text-anchor="middle" font-size="11" fill="#999">{{ d.date.slice(5) }}</text>
            <text v-for="(v, yi) in chartYLabels" :key="'yl'+yi"
              :x="chartPadL - 6" :y="chartPadT + yi * chartStepY + 4"
              text-anchor="end" font-size="10" fill="#999">{{ v }}</text>
          </svg>
        </div>
      </section>

      <!-- Per-video stats -->
      <section class="cd-section">
        <h3 class="cd-section-title">稿件表现 ({{ videoStats.length }})</h3>
        <div v-if="videoStats.length > 0" class="cd-video-table">
          <div class="cd-video-row cd-video-row--head">
            <span class="cd-video-title-col">视频</span>
            <span class="cd-video-stat-col">播放</span>
            <span class="cd-video-stat-col">弹幕</span>
            <span class="cd-video-stat-col">评论</span>
            <span class="cd-video-stat-col">点赞</span>
            <span class="cd-video-stat-col">硬币</span>
            <span class="cd-video-stat-col">收藏</span>
          </div>
          <div v-for="v in videoStats" :key="v.video_id" class="cd-video-row">
            <span class="cd-video-title-col">
              <router-link :to="`/video/av${v.video_id}`" class="cd-video-link">{{ v.title }}</router-link>
            </span>
            <span class="cd-video-stat-col">{{ fmtNum(v.play_count) }}</span>
            <span class="cd-video-stat-col">{{ fmtNum(v.danmaku_count) }}</span>
            <span class="cd-video-stat-col">{{ fmtNum(v.comment_count) }}</span>
            <span class="cd-video-stat-col">{{ fmtNum(v.like_count) }}</span>
            <span class="cd-video-stat-col">{{ fmtNum(v.coin_count) }}</span>
            <span class="cd-video-stat-col">{{ fmtNum(v.fav_count) }}</span>
          </div>
        </div>
        <p v-else-if="!loading" class="cd-empty">暂无稿件数据</p>
      </section>
    </div>
  </CreatorShell>
</template>

<script>
import CreatorShell from "@/components/creator/CreatorShell.vue";
import { mbGetCreatorStats, mbGetCreatorVideoStats } from "@/api/minibili";
import { ElMessage } from "element-plus";

export default {
  name: "CreatorDashboard",
  components: { CreatorShell },
  data() {
    return {
      loading: false,
      stats: { total_videos: 0, total_plays: 0, total_coins: 0, total_fans: 0, trend_7d: [] },
      videoStats: []
    };
  },
  mounted() { this.fetchAll(); },
  computed: {
    chartWidth() { return 700 },
    chartHeight() { return 220 },
    chartPadL() { return 50 },
    chartPadT() { return 20 },
    chartW() { return this.chartWidth - this.chartPadL - 10 },
    chartH() { return this.chartHeight - this.chartPadT - 30 },
    chartStepX() { const n = Math.max(1, (this.stats.trend_7d.length - 1)); return this.chartW / n },
    chartStepY() { return this.chartH / 3 },
    chartMax() { return Math.max(...this.stats.trend_7d.map(d => d.play_count), 1) },
    chartDots() {
      return (this.stats.trend_7d || []).map((d, i) => ({
        x: this.chartPadL + i * this.chartStepX,
        y: this.chartPadT + this.chartH - (d.play_count / this.chartMax) * this.chartH
      }));
    },
    chartPoints() { return this.chartDots.map(p => p.x + ',' + p.y).join(' ') },
    chartYLabels() {
      return [0,1,2,3].map(i => this.fmtNumShort(Math.round(this.chartMax * (3-i) / 3)));
    }
  },
  methods: {
    async fetchAll() {
      this.loading = true;
      try {
        const [sRes, vRes] = await Promise.all([
          mbGetCreatorStats(),
          mbGetCreatorVideoStats()
        ]);
        this.stats = { total_videos: 0, total_plays: 0, total_coins: 0, total_fans: 0, trend_7d: [], ...sRes };
        this.videoStats = Array.isArray(vRes) ? vRes : [];
      } catch (e) {
        console.error("[CreatorDashboard] fetchAll error:", e);
        ElMessage.error("加载数据失败：" + (e && e.message ? e.message : "请确认已登录"));
      }
      finally { this.loading = false; }
    },
    // ── Formatting ──
    fmtNum(n) {
      const v = Number(n);
      if (!Number.isFinite(v) || v < 0) return "0";
      return v >= 10000 ? (v / 1e4).toFixed(1).replace(/\.0$/, "") + "万" : String(Math.round(v));
    },
    fmtNumShort(n) {
      const v = Number(n);
      if (!Number.isFinite(v) || v < 0) return "0";
      if (v >= 1e8) return (v / 1e8).toFixed(1) + "亿";
      if (v >= 1e4) return (v / 1e4).toFixed(1) + "万";
      return String(Math.round(v));
    }
  }
};
</script>

<style scoped>
.cd-page { max-width: 920px; padding: 0 0 80px; }
.cd-head { margin-bottom: 24px; }
.cd-title { margin: 0 0 4px; font-size: 22px; font-weight: 700; color: #18191c; }
.cd-sub { margin: 0; font-size: 13px; color: #999; }

.cd-cards { display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 14px; margin-bottom: 28px; }
.cd-card { display: flex; align-items: center; gap: 14px; padding: 16px 18px; background: #fff; border: 1px solid #e3e5e7; border-radius: 10px; }
.cd-card-icon { width: 40px; height: 40px; border-radius: 10px; display: flex; align-items: center; justify-content: center; font-size: 18px; flex-shrink: 0; }
.cd-icon--video { background: #e6f7ff; color: #1890ff; }
.cd-icon--play { background: #fff0e6; color: #fa8c16; }
.cd-icon--coin { background: #fff7e6; color: #faad14; }
.cd-icon--fan { background: #fce4ec; color: #e91e63; }
.cd-card-body { display: flex; flex-direction: column; gap: 2px; min-width: 0; }
.cd-card-val { font-size: 20px; font-weight: 700; color: #18191c; }
.cd-card-label { font-size: 12px; color: #999; }

.cd-section { margin-bottom: 28px; }
.cd-section-title { margin: 0 0 14px; font-size: 16px; font-weight: 600; color: #18191c; }

.cd-chart-wrap { background: #fff; border: 1px solid #e3e5e7; border-radius: 10px; padding: 0 12px 10px; }
.cd-line-chart { width: 100%; display: block; }

/* (remove old bar chart styles below) */
.cd-chart-bars { display: none; }

.cd-video-table { background: #fff; border: 1px solid #e3e5e7; border-radius: 10px; overflow: hidden; }
.cd-video-row { display: flex; align-items: center; padding: 10px 16px; border-bottom: 1px solid #f0f0f0; font-size: 13px; }
.cd-video-row:last-child { border-bottom: none; }
.cd-video-row--head { background: #fafbfc; font-weight: 600; color: #666; font-size: 12px; }
.cd-video-title-col { flex: 1; min-width: 0; padding-right: 8px; }
.cd-video-stat-col { width: 64px; text-align: right; flex-shrink: 0; }
.cd-video-link { color: #18191c; text-decoration: none; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; display: block; }
.cd-video-link:hover { color: #00a1d6; }

.cd-empty { text-align: center; padding: 48px 0; color: #999; font-size: 14px; }
.cd-empty a { color: #00a1d6; }
</style>
