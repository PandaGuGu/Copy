<template>
  <div class="db-page" v-loading="loading">
    <header class="db-page__head">
      <h2 class="db-page__title">数据仪表盘</h2>
      <p class="db-page__desc">运营数据总览</p>
    </header>

    <!-- 概览卡片 -->
    <div class="db-cards">
      <div class="db-card">
        <div class="db-card__value">{{ fmtNum(data.total_users) }}</div>
        <div class="db-card__label">总用户</div>
        <div class="db-card__sub">今日 +{{ fmtNum(data.today_users) }}</div>
      </div>
      <div class="db-card">
        <div class="db-card__value">{{ fmtNum(data.total_videos) }}</div>
        <div class="db-card__label">总视频</div>
        <div class="db-card__sub">今日 +{{ fmtNum(data.today_videos) }}</div>
      </div>
      <div class="db-card">
        <div class="db-card__value">{{ fmtNum(data.total_articles) }}</div>
        <div class="db-card__label">总文章</div>
        <div class="db-card__sub">今日 +{{ fmtNum(data.today_articles) }}</div>
      </div>
      <div class="db-card">
        <div class="db-card__value">{{ fmtNum(data.total_comments) }}</div>
        <div class="db-card__label">总评论</div>
        <div class="db-card__sub">&nbsp;</div>
      </div>
    </div>

    <!-- 审核队列 -->
    <div class="db-cards db-cards--review">
      <div class="db-card db-card--warn">
        <div class="db-card__value">{{ fmtNum(data.pending_videos) }}</div>
        <div class="db-card__label">待审视频</div>
      </div>
      <div class="db-card db-card--warn">
        <div class="db-card__value">{{ fmtNum(data.pending_articles) }}</div>
        <div class="db-card__label">待审文章</div>
      </div>
    </div>

    <!-- 7 日趋势 -->
    <div class="db-chart-section">
      <h3 class="db-chart__title">近 7 日新增趋势</h3>
      <div class="db-chart" ref="chartBox">
        <svg :viewBox="`0 0 ${chartW} ${chartH}`" class="db-chart__svg">
          <!-- grid lines -->
          <line v-for="i in 4" :key="'gl'+i" :x1="0" :y1="chartPad + (i-1)*chartStepH" :x2="chartW" :y2="chartPad + (i-1)*chartStepH" stroke="#e8e9eb" stroke-width="1" />
          <!-- bars: users -->
          <g v-for="(pt, idx) in chartTrend" :key="'u'+idx">
            <rect
              :x="chartPad + idx * chartStepW + chartStepW * 0.15"
              :y="chartY(pt.users)"
              :width="chartStepW * 0.3"
              :height="chartH - chartPad - chartY(pt.users)"
              fill="#00a1d6"
              rx="2"
            />
          </g>
          <!-- bars: videos -->
          <g v-for="(pt, idx) in chartTrend" :key="'v'+idx">
            <rect
              :x="chartPad + idx * chartStepW + chartStepW * 0.52"
              :y="chartY(pt.videos)"
              :width="chartStepW * 0.3"
              :height="chartH - chartPad - chartY(pt.videos)"
              fill="#fb7299"
              rx="2"
            />
          </g>
          <!-- x labels -->
          <text v-for="(pt, idx) in chartTrend" :key="'xl'+idx"
            :x="chartPad + idx * chartStepW + chartStepW * 0.5"
            :y="chartH - 4"
            text-anchor="middle" font-size="11" fill="#9499a0"
          >{{ pt.date }}</text>
        </svg>
        <div class="db-chart__legend">
          <span class="db-chart__leg"><i style="background:#00a1d6"></i> 新增用户</span>
          <span class="db-chart__leg"><i style="background:#fb7299"></i> 新增视频</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
import { ElMessage } from "element-plus";
import { adminGetDashboard } from "@/api/admin";

export default {
  name: "Dashboard",
  data() {
    return {
      loading: false,
      data: {
        total_users: 0,
        today_users: 0,
        total_videos: 0,
        total_articles: 0,
        total_comments: 0,
        today_videos: 0,
        today_articles: 0,
        pending_videos: 0,
        pending_articles: 0,
        trend: [],
      },
      chartW: 680,
      chartH: 240,
      chartPad: 24,
    };
  },
  computed: {
    chartTrend() {
      return (this.data && this.data.trend) || [];
    },
    chartMax() {
      const arr = this.chartTrend;
      let m = 1;
      for (const p of arr) {
        if (p.users > m) m = p.users;
        if (p.videos > m) m = p.videos;
      }
      return m;
    },
    chartStepW() {
      const n = this.chartTrend.length || 7;
      return (this.chartW - this.chartPad * 2) / n;
    },
    chartStepH() {
      return (this.chartH - this.chartPad - 20) / 4;
    },
  },
  created() {
    this.load();
  },
  methods: {
    async load() {
      this.loading = true;
      try {
        const body = await adminGetDashboard();
        this.data = (body && body.data) || {};
        if (!this.data.trend) this.data.trend = [];
      } catch (e) {
        ElMessage.error((e && e.message) || "加载失败");
      } finally {
        this.loading = false;
      }
    },
    fmtNum(n) {
      if (n == null) return "0";
      if (n >= 10000) return (n / 10000).toFixed(1) + "万";
      return String(n);
    },
    chartY(val) {
      const max = this.chartMax || 1;
      const ratio = val / max;
      return this.chartPad + (this.chartH - this.chartPad - 20) * (1 - ratio);
    },
  },
};
</script>

<style scoped>
.db-page { padding: 20px 24px; max-width: 900px; }
.db-page__head { margin-bottom: 20px; }
.db-page__title { margin: 0 0 4px; font-size: 18px; font-weight: 600; color: #18191c; }
.db-page__desc { margin: 0; font-size: 13px; color: #9499a0; }
.db-cards { display: grid; grid-template-columns: repeat(4, 1fr); gap: 14px; margin-bottom: 14px; }
.db-cards--review { grid-template-columns: repeat(4, 1fr); }
.db-card { background: #fff; border: 1px solid #e3e5e7; border-radius: 8px; padding: 16px 20px; }
.db-card--warn { border-color: #ffe0b0; background: #fffaf3; }
.db-card__value { font-size: 28px; font-weight: 700; color: #18191c; line-height: 1.2; }
.db-card--warn .db-card__value { color: #e6a23c; }
.db-card__label { font-size: 13px; color: #61666d; margin-top: 4px; }
.db-card__sub { font-size: 12px; color: #9499a0; margin-top: 2px; }
.db-chart-section { margin-top: 6px; background: #fff; border: 1px solid #e3e5e7; border-radius: 8px; padding: 16px 20px; }
.db-chart__title { margin: 0 0 12px; font-size: 14px; font-weight: 600; color: #18191c; }
.db-chart__svg { width: 100%; height: auto; }
.db-chart__legend { display: flex; gap: 20px; margin-top: 8px; font-size: 12px; color: #61666d; }
.db-chart__leg i { display: inline-block; width: 10px; height: 10px; border-radius: 2px; margin-right: 4px; vertical-align: -1px; }
</style>
