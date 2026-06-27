<template>
  <div class="bi-page" v-loading="loading">
    <header class="bi-page__head">
      <h2 class="bi-page__title">BI 统计报表</h2>
      <p class="bi-page__desc">分区统计、创作者排行与时序数据</p>
    </header>

    <el-tabs v-model="activeTab" @tab-change="onTabChange">
      <!-- 总览仪表盘 -->
      <el-tab-pane label="总览" name="summary">
        <div class="bi-summary-cards" v-if="summaryCards.length">
          <div v-for="c in summaryCards" :key="c.key" class="bi-summary-card">
            <span class="bi-summary-card__value">{{ fmtNum(c.value) }}</span>
            <span class="bi-summary-card__label">{{ c.label }}</span>
          </div>
        </div>
        <div class="bi-summary-update" v-if="summaryUpdated">数据更新于 {{ summaryUpdated }}</div>
      </el-tab-pane>

      <!-- 分区统计 -->
      <el-tab-pane label="分区统计" name="zone">
        <div class="bi-toolbar">
          <el-button type="primary" size="small" @click="exportCSV('zone')">导出 CSV</el-button>
        </div>
        <div class="bi-zone-chart" v-if="zoneData.length > 0">
          <svg :viewBox="`0 0 ${chartW} ${zoneChartH}`" class="bi-zone-svg">
            <line v-for="i in 5" :key="'gl'+i" :x1="60" :y1="10 + (i-1)*zoneBarStep" :x2="chartW" :y2="10 + (i-1)*zoneBarStep" stroke="#e8e9eb" stroke-width="1" />
            <g v-for="(z, idx) in zoneData" :key="idx" :transform="`translate(0, ${10 + idx * zoneBarStep})`">
              <text :x="56" :y="zoneBarH/2 + 2" text-anchor="end" font-size="11" fill="#61666d">{{ z.zone_name }}</text>
              <rect :x="60" :y="4" :width="barWidth(z.video_count)" :height="zoneBarH - 4" :fill="zoneBarColor(idx)" rx="3" />
              <text :x="60 + barWidth(z.video_count) + 6" :y="zoneBarH/2 + 2" font-size="11" fill="#9499a0">{{ z.video_count }}</text>
            </g>
          </svg>
        </div>
        <el-table :data="zoneData" stripe size="default" empty-text="暂无数据" style="margin-top: 14px">
          <el-table-column prop="zone_name" label="分区" min-width="120" />
          <el-table-column prop="video_count" label="视频数" width="100" sortable />
          <el-table-column label="平均播放" width="120">
            <template #default="{ row }">{{ fmtNum(row.avg_play_count) }}</template>
          </el-table-column>
          <el-table-column label="总播放" width="120">
            <template #default="{ row }">{{ fmtNum(row.total_play_count) }}</template>
          </el-table-column>
          <el-table-column label="平均互动率" width="110">
            <template #default="{ row }">{{ row.avg_interact_rate ? (row.avg_interact_rate * 100).toFixed(2) + '%' : '—' }}</template>
          </el-table-column>
        </el-table>
      </el-tab-pane>

      <!-- 创作者统计 -->
      <el-tab-pane label="创作者排行" name="creator">
        <div class="bi-toolbar">
          <el-select v-model="creatorMetric" size="small" style="width: 140px">
            <el-option label="按总播放" value="total_plays" />
            <el-option label="按投币数" value="total_coins" />
            <el-option label="按粉丝数" value="fans_count" />
          </el-select>
          <el-button type="primary" size="small" @click="exportCSV('creator')">导出 CSV</el-button>
        </div>
        <el-table :data="creatorData" stripe size="default" empty-text="暂无数据">
          <el-table-column label="排名" width="60">
            <template #default="{ $index }">{{ $index + 1 }}</template>
          </el-table-column>
          <el-table-column label="用户" min-width="140">
            <template #default="{ row }">
              <span>{{ row.nickname || row.username }}</span>
              <span class="bi-muted" style="margin-left: 4px">@{{ row.cake_id || row.username }}</span>
            </template>
          </el-table-column>
          <el-table-column label="总播放" width="110" sortable prop="total_plays">
            <template #default="{ row }">{{ fmtNum(row.total_plays) }}</template>
          </el-table-column>
          <el-table-column label="总投币" width="100" sortable prop="total_coins">
            <template #default="{ row }">{{ fmtNum(row.total_coins) }}</template>
          </el-table-column>
          <el-table-column label="粉丝数" width="100" sortable prop="fans_count">
            <template #default="{ row }">{{ fmtNum(row.fans_count) }}</template>
          </el-table-column>
          <el-table-column label="视频数" width="90" sortable prop="video_count">
            <template #default="{ row }">{{ row.video_count }}</template>
          </el-table-column>
          <el-table-column label="文章数" width="90" sortable prop="article_count">
            <template #default="{ row }">{{ row.article_count }}</template>
          </el-table-column>
        </el-table>
      </el-tab-pane>

      <!-- 时序数据 -->
      <el-tab-pane label="时序数据" name="timeseries">
        <div class="bi-toolbar">
          <el-date-picker
            v-model="tsRange"
            type="daterange"
            range-separator="至"
            start-placeholder="开始日期"
            end-placeholder="结束日期"
            size="small"
            style="width: 280px"
            @change="fetchTimeSeries"
          />
          <el-select v-model="tsMetric" size="small" style="width: 140px">
            <el-option label="每日播放量" value="daily_plays" />
            <el-option label="每日新用户" value="daily_users" />
            <el-option label="每日新视频" value="daily_videos" />
          </el-select>
          <el-button type="primary" size="small" @click="exportCSV('timeseries')">导出 CSV</el-button>
          <el-button size="small" @click="serverExport('plays')">服务端导出</el-button>
        </div>
        <div class="bi-ts-chart" v-if="tsData.length > 0">
          <svg :viewBox="`0 0 ${chartW} ${tsChartH}`" class="bi-ts-svg">
            <line v-for="i in 5" :key="'gl'+i" :x1="40" :y1="tsPad + (i-1)*tsStepH" :x2="chartW" :y2="tsPad + (i-1)*tsStepH" stroke="#e8e9eb" stroke-width="1" />
            <polyline
              :points="tsLinePoints"
              fill="none"
              stroke="#00a1d6"
              stroke-width="2"
            />
            <g v-for="(pt, idx) in tsData" :key="'pt'+idx">
              <circle
                v-if="idx % Math.ceil(tsData.length / 15) === 0 || idx === tsData.length - 1"
                :cx="tsX(idx)" :cy="tsY(pt[tsMetric])" r="3" fill="#00a1d6"
              />
              <text
                v-if="idx % Math.ceil(tsData.length / 7) === 0 || idx === tsData.length - 1"
                :x="tsX(idx)" :y="tsChartH - 4" text-anchor="middle" font-size="10" fill="#9499a0"
              >{{ pt.date }}</text>
            </g>
          </svg>
          <div class="bi-ts-legend">
            <span class="bi-ts-leg"><i style="background:#00a1d6"></i> {{ tsMetricLabel }}</span>
          </div>
        </div>
        <el-table :data="tsData" stripe size="default" empty-text="暂无数据" style="margin-top: 14px" max-height="300">
          <el-table-column prop="date" label="日期" width="120" />
          <el-table-column label="播放量" width="110">
            <template #default="{ row }">{{ fmtNum(row.daily_plays) }}</template>
          </el-table-column>
          <el-table-column label="新用户" width="100">
            <template #default="{ row }">{{ fmtNum(row.daily_users) }}</template>
          </el-table-column>
          <el-table-column label="新视频" width="100">
            <template #default="{ row }">{{ fmtNum(row.daily_videos) }}</template>
          </el-table-column>
        </el-table>
      </el-tab-pane>

      <!-- 稿件统计 -->
      <el-tab-pane label="稿件统计" name="manuscript">
        <div class="bi-toolbar">
          <el-button type="primary" size="small" @click="exportCSV('manuscript')">导出 CSV</el-button>
        </div>

        <!-- 视频稿件卡片 -->
        <h4 class="bi-subtitle">视频稿件</h4>
        <div class="bi-engage-cards">
          <div class="bi-engage-card">📹 视频总量 <b>{{ fmtNum(msVideoSummary.total || 0) }}</b></div>
          <div class="bi-engage-card">✅ 已发布 <b>{{ fmtNum(msVideoSummary.published || 0) }}</b></div>
          <div class="bi-engage-card">▶ 总播放 <b>{{ fmtNum(msVideoSummary.total_plays || 0) }}</b></div>
          <div class="bi-engage-card">🪙 总投币 <b>{{ fmtNum(msVideoSummary.total_coins || 0) }}</b></div>
          <div class="bi-engage-card">⭐ 总收藏 <b>{{ fmtNum(msVideoSummary.total_favs || 0) }}</b></div>
        </div>

        <!-- 图文稿件卡片 -->
        <h4 class="bi-subtitle">图文稿件</h4>
        <div class="bi-engage-cards">
          <div class="bi-engage-card">📝 图文总量 <b>{{ fmtNum(msArticleSummary.total || 0) }}</b></div>
          <div class="bi-engage-card">✅ 已发布 <b>{{ fmtNum(msArticleSummary.published || 0) }}</b></div>
          <div class="bi-engage-card">👁 总阅读 <b>{{ fmtNum(msArticleSummary.total_views || 0) }}</b></div>
          <div class="bi-engage-card">🪙 总投币 <b>{{ fmtNum(msArticleSummary.total_coins || 0) }}</b></div>
          <div class="bi-engage-card">⭐ 总收藏 <b>{{ fmtNum(msArticleSummary.total_favs || 0) }}</b></div>
        </div>

        <!-- 双栏 TOP 榜 -->
        <div style="display: flex; gap: 16px; flex-wrap: wrap; margin-top: 14px">
          <div style="flex: 1; min-width: 340px">
            <h4 class="bi-subtitle">热门视频 TOP10</h4>
            <el-table :data="msTopVideos" stripe size="small" max-height="360">
              <el-table-column label="排名" width="55">
                <template #default="{ $index }">{{ $index + 1 }}</template>
              </el-table-column>
              <el-table-column prop="title" label="标题" show-overflow-tooltip min-width="140" />
              <el-table-column label="播放" width="80">
                <template #default="{ row }">{{ fmtNum(row.play_count) }}</template>
              </el-table-column>
              <el-table-column prop="zone" label="分区" width="70" />
            </el-table>
          </div>
          <div style="flex: 1; min-width: 340px">
            <h4 class="bi-subtitle">热门图文 TOP10</h4>
            <el-table :data="msTopArticles" stripe size="small" max-height="360">
              <el-table-column label="排名" width="55">
                <template #default="{ $index }">{{ $index + 1 }}</template>
              </el-table-column>
              <el-table-column prop="title" label="标题" show-overflow-tooltip min-width="140" />
              <el-table-column label="阅读" width="80">
                <template #default="{ row }">{{ fmtNum(row.view_count) }}</template>
              </el-table-column>
              <el-table-column label="时间" width="90">
                <template #default="{ row }">{{ fmtDate(row.created_at) }}</template>
              </el-table-column>
            </el-table>
          </div>
        </div>
      </el-tab-pane>

      <!-- 互动统计 -->
      <el-tab-pane label="互动统计" name="engagement">
        <div class="bi-toolbar">
          <el-button type="primary" size="small" @click="exportCSV('engagement')">导出 CSV</el-button>
        </div>
        <div class="bi-engage-cards">
          <div class="bi-engage-card">👍 累计点赞 <b>{{ fmtNum(engageTotals.total_likes) }}</b></div>
          <div class="bi-engage-card">⭐ 累计收藏 <b>{{ fmtNum(engageTotals.total_favs) }}</b></div>
          <div class="bi-engage-card">🪙 视频投币 <b>{{ fmtNum(engageTotals.total_video_coins) }}</b></div>
          <div class="bi-engage-card">📝 文章投币 <b>{{ fmtNum(engageTotals.total_article_coins) }}</b></div>
        </div>
        <div class="bi-ts-chart" v-if="engagementTS.length > 0">
          <svg :viewBox="`0 0 ${chartW} ${tsChartH}`" class="bi-ts-svg">
            <line v-for="i in 5" :key="'egl'+i" :x1="40" :y1="tsPad + (i-1)*tsStepH" :x2="chartW" :y2="tsPad + (i-1)*tsStepH" stroke="#e8e9eb" stroke-width="1" />
            <g v-for="(series, si) in engagementTS" :key="'es'+si">
              <polyline :points="series.points.map((p, i) => `${engX(i)},${engY(p.value, series.maxVal)}`).join(' ')" fill="none" :stroke="series.color" stroke-width="2" />
            </g>
          </svg>
          <div class="bi-ts-legend">
            <span v-for="s in engagementTS" :key="s.key" class="bi-ts-leg"><i :style="{background: s.color}"></i> {{ s.label }}</span>
          </div>
        </div>
      </el-tab-pane>
    </el-tabs>

    <!-- 已保存报表 -->
    <div class="bi-saved">
      <div class="bi-saved__head">
        <h3 class="bi-saved__title">已保存报表</h3>
        <div class="bi-saved__actions">
          <el-input v-model="newReportName" placeholder="报表名称" size="small" style="width: 180px" />
          <el-button type="primary" size="small" @click="saveReport">保存当前报表</el-button>
        </div>
      </div>
      <el-table :data="savedReports" stripe size="small" empty-text="暂无已保存报表">
        <el-table-column prop="id" label="ID" width="60" />
        <el-table-column prop="name" label="名称" min-width="160" />
        <el-table-column prop="chart_type" label="图表类型" width="100" />
        <el-table-column label="创建时间" width="160">
          <template #default="{ row }">{{ fmtTime(row.created_at) }}</template>
        </el-table-column>
        <el-table-column label="操作" width="170">
          <template #default="{ row }">
            <el-button size="small" text type="primary" @click="loadReport(row)">加载</el-button>
            <el-button size="small" text type="success" @click="exportReport(row)">导出</el-button>
            <el-popconfirm title="确认删除？" @confirm="deleteReport(row)">
              <template #reference>
                <el-button size="small" text type="danger">删除</el-button>
              </template>
            </el-popconfirm>
          </template>
        </el-table-column>
      </el-table>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
import http from '@/utils/adminHttp'
import { ElMessage } from 'element-plus'

const API_BASE = import.meta.env.VITE_API_BASE || '/api/v1'
const ADMIN_API = API_BASE.replace('/api/v1', '/api/v1/admin')

async function api(path, opts = {}) {
  const m = (opts.method || 'GET').toLowerCase()
  let r
  if (m === 'get') r = await http.get(ADMIN_API + path)
  else if (m === 'post') r = await http.post(ADMIN_API + path, opts.body || {})
  else if (m === 'put') r = await http.put(ADMIN_API + path, opts.body || {})
  else if (m === 'delete') r = await http.delete(ADMIN_API + path)
  else r = await http.get(ADMIN_API + path)
  return r.data
}

const loading = ref(false)
const activeTab = ref('zone')

// Zone data
const zoneData = ref([])

// Creator data
const creatorData = ref([])
const creatorMetric = ref('total_plays')

// Time series data
const tsData = ref([])
const tsRange = ref([])
const tsMetric = ref('daily_plays')
const chartW = 680
const tsChartH = 240
const tsPad = 20

// Saved reports
const savedReports = ref([])
const newReportName = ref('')

// Summary dashboard
const summaryCards = ref([])
const summaryUpdated = ref('')

// Manuscript stats
const msVideoSummary = ref({})
const msArticleSummary = ref({})
const msTopVideos = ref([])
const msTopArticles = ref([])

// Engagement stats
const engagementTS = ref([])
const engageTotals = ref({})

const zoneChartH = computed(() => Math.max(120, zoneData.value.length * 36 + 20))
const zoneBarStep = 36
const zoneBarH = 28

const tsStepH = computed(() => (tsChartH - tsPad - 20) / 4)

const tsMax = computed(() => {
  let m = 1
  for (const p of tsData.value) {
    if (p[tsMetric.value] > m) m = p[tsMetric.value]
  }
  return m
})

const tsLinePoints = computed(() => {
  return tsData.value.map((p, i) => `${tsX(i)},${tsY(p[tsMetric.value])}`).join(' ')
})

function tsX(idx) {
  const n = tsData.value.length || 1
  const w = chartW - 40
  return 40 + (idx / (n - 1 || 1)) * w
}

function tsY(val) {
  const ratio = val / (tsMax.value || 1)
  return tsPad + (tsChartH - tsPad - 20) * (1 - ratio)
}

function barWidth(count) {
  const max = Math.max(1, ...zoneData.value.map(z => z.video_count || 0))
  return (count / max) * (chartW - 120) + 2
}

function zoneBarColor(idx) {
  const colors = ['#00a1d6', '#fb7299', '#02b340', '#e6a23c', '#909399']
  return colors[idx % colors.length]
}

const tsMetricLabel = computed(() => ({
  daily_plays: '每日播放量', daily_users: '每日新用户', daily_videos: '每日新视频',
}[tsMetric.value] || tsMetric.value))

async function fetchZones() {
  loading.value = true
  try {
    const d = await api('/bi/zone-stats')
    // Backend returns { zones: [{ zone, video_count, play_count, avg_plays_per_video }] }
    const raw = d.zones || d.items || d || []
    zoneData.value = raw.map(z => ({
      zone_name: z.zone || z.zone_name,
      video_count: z.video_count || 0,
      avg_play_count: z.avg_plays_per_video || z.avg_play_count || 0,
      total_play_count: z.play_count || z.total_play_count || 0,
    }))
  } catch (e) {
    ElMessage.error(e.message || '加载失败')
  } finally {
    loading.value = false
  }
}

async function fetchCreators() {
  loading.value = true
  try {
    const d = await api(`/bi/creator-stats?limit=20`)
    // Backend returns all dimensions at once: { creators: [{ user_id, username, total_plays, total_coins, fans_count, video_count, article_count }] }
    const raw = d.creators || d.items || d || []
    creatorData.value = raw.map(c => ({
      user_id: c.user_id,
      nickname: c.username || c.nickname,
      username: c.username || c.nickname,
      cake_id: c.username,
      total_plays: c.total_plays || 0,
      total_coins: c.total_coins || 0,
      fans_count: c.fans_count || 0,
      video_count: c.video_count || 0,
      article_count: c.article_count || 0,
    }))
  } catch (e) {
    ElMessage.error(e.message || '加载失败')
  } finally {
    loading.value = false
  }
}

async function fetchTimeSeries() {
  loading.value = true
  try {
    // Map frontend metric to backend metric name
    const metricMap = { daily_plays: 'plays', daily_users: 'new_users', daily_videos: 'new_videos' }
    // Fetch all three metrics in parallel and merge by date
    const fetches = ['plays', 'new_users', 'new_videos'].map(m =>
      api(`/bi/time-series?metric=${m}&days=30`)
    )
    const results = await Promise.all(fetches)
    // results: [{ metric, granularity, points: [{date, value}] }, ...]
    const dateMap = {}
    results.forEach((res, idx) => {
      const fieldName = ['daily_plays', 'daily_users', 'daily_videos'][idx]
      const pts = res.points || res.items || res || []
      pts.forEach(p => {
        if (!dateMap[p.date]) {
          dateMap[p.date] = { date: p.date, daily_plays: 0, daily_users: 0, daily_videos: 0 }
        }
        dateMap[p.date][fieldName] = p.value || 0
      })
    })
    tsData.value = Object.values(dateMap).sort((a, b) => a.date.localeCompare(b.date))
    if (tsData.value.length > 0) {
      tsMetric.value = 'daily_plays' // set active metric for chart
    }
  } catch (e) {
    ElMessage.error(e.message || '加载失败')
  } finally {
    loading.value = false
  }
}

async function fetchSavedReports() {
  try {
    const d = await api('/bi/reports')
    savedReports.value = d.reports || d.items || d || []
  } catch { /* ignore */ }
}

async function fetchSummary() {
  try {
    const d = await api('/bi/summary')
    summaryCards.value = d.cards || []
    summaryUpdated.value = d.updated || ''
  } catch (e) {
    ElMessage.error(e.message || '加载总览失败')
  }
}

async function fetchManuscriptStats() {
  loading.value = true
  try {
    const d = await api('/bi/manuscript-stats?days=30')
    msVideoSummary.value = d.video_summary || {}
    msArticleSummary.value = d.article_summary || {}
    msTopVideos.value = d.top_videos || []
    msTopArticles.value = d.top_articles || []
  } catch (e) {
    ElMessage.error(e.message || '加载稿件统计失败')
  } finally {
    loading.value = false
  }
}

async function fetchEngagementStats() {
  loading.value = true
  try {
    const d = await api('/bi/engagement-stats?days=30')
    engageTotals.value = {
      total_likes: d.total_likes || 0,
      total_favs: d.total_favs || 0,
      total_video_coins: d.total_video_coins || 0,
      total_article_coins: d.total_article_coins || 0,
    }
    const series = [
      { key: 'comments', label: '评论', color: '#00a1d6', data: d.comments_ts || [] },
      { key: 'danmaku', label: '弹幕', color: '#fb7299', data: d.danmaku_ts || [] },
      { key: 'likes', label: '点赞', color: '#02b340', data: d.likes_ts || [] },
      { key: 'favs', label: '收藏', color: '#e6a23c', data: d.favs_ts || [] },
      { key: 'coins', label: '投币', color: '#909399', data: d.coins_ts || [] },
    ]
    engagementTS.value = series.map(s => ({
      ...s,
      points: s.data,
      maxVal: Math.max(1, ...s.data.map(p => p.value || 0)),
    }))
  } catch (e) {
    ElMessage.error(e.message || '加载互动统计失败')
  } finally {
    loading.value = false
  }
}

function engX(idx) {
  const s = engagementTS.value[0]
  const n = (s?.points?.length) || 1
  const w = chartW - 40
  return 40 + (idx / (n - 1 || 1)) * w
}

function engY(val, max) {
  const ratio = val / (max || 1)
  return tsPad + (tsChartH - tsPad - 20) * (1 - ratio)
}

function onTabChange(tab) {
  if (tab === 'summary' && summaryCards.value.length === 0) fetchSummary()
  if (tab === 'zone' && zoneData.value.length === 0) fetchZones()
  if (tab === 'creator' && creatorData.value.length === 0) fetchCreators()
  if (tab === 'timeseries' && tsData.value.length === 0) fetchTimeSeries()
  if (tab === 'manuscript' && msTopVideos.value.length === 0) fetchManuscriptStats()
  if (tab === 'engagement' && engagementTS.value.length === 0) fetchEngagementStats()
}

async function saveReport() {
  if (!newReportName.value.trim()) {
    ElMessage.warning('请输入报表名称')
    return
  }
  try {
    await api('/bi/reports', {
      method: 'POST',
      body: {
        name: newReportName.value.trim(),
        description: '',
        query_config: JSON.stringify({ tab: activeTab.value, metric: activeTab.value === 'creator' ? creatorMetric.value : tsMetric.value }),
        chart_type: (activeTab.value === 'zone' || activeTab.value === 'creator') ? 'table' : 'line',
      },
    })
    newReportName.value = ''
    ElMessage.success('已保存')
    fetchSavedReports()
  } catch (e) {
    ElMessage.error(e.message || '保存失败')
  }
}

async function deleteReport(row) {
  try {
    await api(`/bi/reports/${row.id}`, { method: 'DELETE' })
    ElMessage.success('已删除')
    fetchSavedReports()
  } catch (e) {
    ElMessage.error(e.message || '删除失败')
  }
}

function loadReport(row) {
  let cfg = {}
  try { cfg = typeof row.query_config === 'string' ? JSON.parse(row.query_config) : (row.query_config || {}) } catch { /* ignore */ }
  if (cfg.tab) {
    activeTab.value = cfg.tab
    if (cfg.metric) {
      if (cfg.tab === 'creator') creatorMetric.value = cfg.metric
      if (cfg.tab === 'timeseries') tsMetric.value = cfg.metric
    }
    onTabChange(cfg.tab)
  }
}

function exportReport(row) {
  let cfg = {}
  try { cfg = typeof row.query_config === 'string' ? JSON.parse(row.query_config) : (row.query_config || {}) } catch { /* ignore */ }
  let metric = 'plays'
  if (cfg.tab === 'creator') {
    const dimMap = { total_plays: 'play_count', total_coins: 'coin_count', fans_count: 'fan_count' }
    metric = dimMap[cfg.metric] || 'play_count'
  } else if (cfg.tab === 'timeseries' && cfg.metric) {
    metric = cfg.metric
  }
  serverExport(metric)
}

function exportCSV(type) {
  let rows = []
  let filename = ''
  if (type === 'zone') {
    rows = zoneData.value.map(z => ({
      分区: z.zone_name, 视频数: z.video_count, 平均播放: z.avg_play_count, 总播放: z.total_play_count,
    }))
    filename = 'zone_stats.csv'
  } else if (type === 'creator') {
    rows = creatorData.value.map(c => ({
      用户: c.nickname || c.username, 总播放: c.total_plays, 总投币: c.total_coins, 粉丝数: c.fans_count, 视频数: c.video_count,
    }))
    filename = 'creator_stats.csv'
  } else if (type === 'timeseries') {
    rows = tsData.value.map(t => ({
      日期: t.date, 播放量: t.daily_plays, 新用户: t.daily_users, 新视频: t.daily_videos,
    }))
    filename = 'timeseries.csv'
  } else if (type === 'manuscript') {
    rows = msTopVideos.value.map(v => ({
      排名: msTopVideos.value.indexOf(v) + 1, 标题: v.title, 播放量: v.play_count, 分区: v.zone,
    }))
    filename = 'top_videos.csv'
  } else if (type === 'engagement') {
    if (engagementTS.value.length === 0) { ElMessage.warning('暂无互动数据'); return }
    const first = engagementTS.value[0]
    rows = first.points.map((p, i) => {
      const r = { 日期: p.date || '' }
      engagementTS.value.forEach(s => { r[s.label] = s.points[i]?.value ?? 0 })
      return r
    })
    filename = 'engagement_stats.csv'
  }
  if (rows.length === 0) {
    ElMessage.warning('暂无数据可导出')
    return
  }
  const headers = Object.keys(rows[0])
  const csv = [
    headers.join(','),
    ...rows.map(r => headers.map(h => `"${r[h] ?? ''}"`).join(',')),
  ].join('\n')
  const blob = new Blob(['\ufeff' + csv], { type: 'text/csv;charset=utf-8' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = filename
  a.click()
  URL.revokeObjectURL(url)
}

// Server-side export (with TaskLog tracking)
async function serverExport(metric) {
  try {
    const res = await http.post(ADMIN_API + '/bi/export', { metric, days: 30 }, { responseType: 'blob' })
    // responseType: 'blob' 时 res 是完整的 axios response（经过拦截器返回 body，即 Blob）
    const blob = res instanceof Blob ? res : new Blob([res], { type: 'text/csv;charset=utf-8' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `report_${metric}_${new Date().toISOString().slice(0, 10)}.csv`
    document.body.appendChild(a)
    a.click()
    document.body.removeChild(a)
    URL.revokeObjectURL(url)
    ElMessage.success(`已导出报表 (${metric})`)
  } catch (e) {
    ElMessage.error((e && e.message) || '导出失败')
  }
}

function fmtNum(n) {
  if (n == null) return '0'
  if (n >= 10000) return (n / 10000).toFixed(1) + '万'
  // Floats: round to 1 decimal; integers: keep as-is
  if (typeof n === 'number' && !Number.isInteger(n)) return n.toFixed(1)
  return String(n)
}

function fmtTime(t) {
  if (!t) return ''
  const d = new Date(t)
  const pad = (n) => String(n).padStart(2, '0')
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}`
}

function fmtDate(t) {
  if (!t) return ''
  const d = new Date(t)
  const pad = (n) => String(n).padStart(2, '0')
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`
}

onMounted(() => {
  fetchZones()
  fetchSummary()
  fetchCreators()
  fetchTimeSeries()
  fetchManuscriptStats()
  fetchEngagementStats()
  fetchSavedReports()
})
</script>

<style scoped>
.bi-page { padding: 20px 24px; }
.bi-page__head { margin-bottom: 14px; }
.bi-page__title { margin: 0 0 4px; font-size: 18px; font-weight: 600; color: #18191c; }
.bi-page__desc { margin: 0; font-size: 13px; color: #9499a0; }
.bi-toolbar { margin-bottom: 14px; display: flex; gap: 10px; align-items: center; }
.bi-muted { color: #9499a0; font-size: 12px; }
.bi-zone-svg, .bi-ts-svg { width: 100%; height: auto; }
.bi-ts-chart { background: #fff; border: 1px solid #e3e5e7; border-radius: 8px; padding: 16px 20px; }
.bi-ts-legend { display: flex; gap: 20px; margin-top: 8px; font-size: 12px; color: #61666d; }
.bi-ts-leg i { display: inline-block; width: 10px; height: 10px; border-radius: 2px; margin-right: 4px; vertical-align: -1px; }
.bi-saved { margin-top: 20px; background: #fff; border: 1px solid #e3e5e7; border-radius: 8px; padding: 16px 20px; }
.bi-saved__head { display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px; }
.bi-saved__title { margin: 0; font-size: 14px; font-weight: 600; color: #18191c; }
.bi-saved__actions { display: flex; gap: 8px; align-items: center; }

/* Summary cards */
.bi-summary-cards { display: grid; grid-template-columns: repeat(auto-fill, minmax(180px, 1fr)); gap: 14px; margin-bottom: 12px; }
.bi-summary-card { background: #fff; border: 1px solid #e3e5e7; border-radius: 8px; padding: 16px; text-align: center; }
.bi-summary-card__value { display: block; font-size: 24px; font-weight: 700; color: #18191c; }
.bi-summary-card__label { display: block; margin-top: 4px; font-size: 12px; color: #9499a0; }
.bi-summary-update { font-size: 12px; color: #c9ccd0; margin-bottom: 8px; }

/* Subtitle */
.bi-subtitle { margin: 0 0 10px; font-size: 13px; font-weight: 600; color: #18191c; }

/* Engagement cards */
.bi-engage-cards { display: grid; grid-template-columns: repeat(auto-fill, minmax(160px, 1fr)); gap: 10px; margin-bottom: 14px; }
.bi-engage-card { background: #fff; border: 1px solid #e3e5e7; border-radius: 8px; padding: 12px 16px; font-size: 13px; color: #61666d; }
.bi-engage-card b { font-size: 18px; color: #18191c; margin-left: 6px; }
</style>
