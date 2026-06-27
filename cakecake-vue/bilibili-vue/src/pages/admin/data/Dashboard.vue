<template>
  <div class="db-page" v-loading="loading">
    <header class="db-page__head">
      <h2 class="db-page__title">数据仪表盘</h2>
      <p class="db-page__desc">运营数据总览</p>
    </header>

    <!-- 概览卡片 -->
    <div class="db-cards">
      <div class="db-card db-card--accent">
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
      <div class="db-card db-card--pending">
        <div class="db-card__value">{{ fmtNum(data.pending_videos) }}</div>
        <div class="db-card__label">待审视频</div>
        <div class="db-card__indicator" v-if="data.pending_videos > 0">
          <span class="db-card__dot"></span>需要处理
        </div>
      </div>
      <div class="db-card db-card--pending">
        <div class="db-card__value">{{ fmtNum(data.pending_articles) }}</div>
        <div class="db-card__label">待审文章</div>
        <div class="db-card__indicator" v-if="data.pending_articles > 0">
          <span class="db-card__dot"></span>需要处理
        </div>
      </div>
    </div>

    <!-- 7 日趋势 -->
    <div class="db-chart-section">
      <h3 class="db-chart__title">近 7 日新增趋势</h3>
      <div class="db-chart" ref="chartBox">
        <svg :viewBox="`0 0 ${chartW} ${chartH}`" class="db-chart__svg">
          <!-- grid lines -->
          <line v-for="i in 4" :key="'gl'+i"
            :x1="0" :y1="chartPad + (i-1)*chartStepH"
            :x2="chartW" :y2="chartPad + (i-1)*chartStepH"
            stroke="#ecf0f4" stroke-width="1" />
          <!-- bars: users -->
          <g v-for="(pt, idx) in chartTrend" :key="'u'+idx">
            <rect
              :x="chartPad + idx * chartStepW + chartStepW * 0.15"
              :y="chartY(pt.users)"
              :width="chartStepW * 0.3"
              :height="chartH - chartPad - chartY(pt.users)"
              fill="url(#barGradA)"
              rx="3"
            />
          </g>
          <!-- bars: videos -->
          <g v-for="(pt, idx) in chartTrend" :key="'v'+idx">
            <rect
              :x="chartPad + idx * chartStepW + chartStepW * 0.52"
              :y="chartY(pt.videos)"
              :width="chartStepW * 0.3"
              :height="chartH - chartPad - chartY(pt.videos)"
              fill="url(#barGradB)"
              rx="3"
            />
          </g>
          <!-- x labels -->
          <text v-for="(pt, idx) in chartTrend" :key="'xl'+idx"
            :x="chartPad + idx * chartStepW + chartStepW * 0.5"
            :y="chartH - 4"
            text-anchor="middle" font-size="11" fill="#9499a0"
          >{{ pt.date }}</text>
          <!-- gradient defs -->
          <defs>
            <linearGradient id="barGradA" x1="0" y1="0" x2="0" y2="1">
              <stop offset="0%" stop-color="#00a1d6" />
              <stop offset="100%" stop-color="#4fc3f7" />
            </linearGradient>
            <linearGradient id="barGradB" x1="0" y1="0" x2="0" y2="1">
              <stop offset="0%" stop-color="#008fc5" />
              <stop offset="100%" stop-color="#00b5e5" />
            </linearGradient>
          </defs>
        </svg>
        <div class="db-chart__legend">
          <span class="db-chart__leg"><i class="db-chart__leg--a"></i> 新增用户</span>
          <span class="db-chart__leg"><i class="db-chart__leg--b"></i> 新增视频</span>
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
/* ── 整体 ── */
.db-page {
  padding: 24px 28px;
  max-width: 920px;
}

.db-page__head {
  margin-bottom: 24px;
}
.db-page__title {
  margin: 0 0 4px;
  font-size: 20px; font-weight: 700;
  color: #0d2b45;
  letter-spacing: 0.3px;
}
.db-page__desc {
  margin: 0;
  font-size: 13px; color: #8c97a6;
}

/* ── 卡片网格 ── */
.db-cards {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 14px;
  margin-bottom: 14px;
}
.db-cards--review {
  grid-template-columns: repeat(4, 1fr);
}

/* ── 通用卡片 ── */
.db-card {
  position: relative;
  background: linear-gradient(135deg, #ffffff 0%, #f4f9fd 100%);
  border: 1px solid #dde7f0;
  border-radius: 10px;
  padding: 18px 22px;
  transition: box-shadow 0.2s, transform 0.2s;
}
.db-card:hover {
  box-shadow: 0 2px 12px rgba(0, 161, 214, 0.08);
  transform: translateY(-1px);
}

/* 首卡强调（总用户）使用蓝色渐变顶边 */
.db-card--accent {
  background: linear-gradient(135deg, #f0f7ff 0%, #ffffff 100%);
  border-color: #c4dff5;
}
.db-card--accent::before {
  content: "";
  position: absolute; top: 0; left: 16px; right: 16px;
  height: 3px;
  background: linear-gradient(90deg, #00a1d6, #4fc3f7);
  border-radius: 0 0 3px 3px;
}

/* 预警卡片 */
.db-card--pending {
  background: linear-gradient(135deg, #fefaf3 0%, #fff8ec 100%);
  border-color: #f0d8a4;
}
.db-card--pending .db-card__value {
  color: #c7802d;
}

.db-card__value {
  font-size: 30px; font-weight: 700;
  color: #0d2b45;
  line-height: 1.1;
}
.db-card__label {
  font-size: 13px; color: #5f6b7a;
  margin-top: 6px;
}
.db-card__sub {
  font-size: 12px; color: #8c97a6;
  margin-top: 4px;
}
.db-card__indicator {
  font-size: 12px; color: #b8862d;
  display: flex; align-items: center; gap: 5px;
  margin-top: 6px;
}
.db-card__dot {
  width: 6px; height: 6px;
  border-radius: 50%;
  background: #e6a23c;
  animation: db-dot-pulse 2s infinite;
}
@keyframes db-dot-pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.35; }
}

/* ── 图表 ── */
.db-chart-section {
  margin-top: 8px;
  background: linear-gradient(135deg, #ffffff 0%, #f7fafd 100%);
  border: 1px solid #dde7f0;
  border-radius: 10px;
  padding: 20px 24px;
}
.db-chart__title {
  margin: 0 0 16px;
  font-size: 14px; font-weight: 600;
  color: #0d2b45;
}
.db-chart__svg {
  width: 100%; height: auto;
}
.db-chart__legend {
  display: flex; gap: 24px;
  margin-top: 12px;
  font-size: 12px; color: #5f6b7a;
}
.db-chart__leg i {
  display: inline-block;
  width: 12px; height: 12px;
  border-radius: 3px;
  margin-right: 6px;
  vertical-align: -2px;
}
.db-chart__leg--a {
  background: linear-gradient(180deg, #00a1d6, #4fc3f7);
}
.db-chart__leg--b {
  background: linear-gradient(180deg, #008fc5, #00b5e5);
}
</style>
