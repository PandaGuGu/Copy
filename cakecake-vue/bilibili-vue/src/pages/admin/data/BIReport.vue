<template>
  <div class="bi-page" v-loading="loading">
    <header class="bi-page__head">
      <h2 class="bi-page__title">BI 统计报表</h2>
      <p class="bi-page__desc">分区统计、创作者排行与时序数据</p>
    </header>

    <el-tabs v-model="activeTab" @tab-change="onTabChange">
      <!-- 总览仪表盘 -->
      <el-tab-pane label="总览" name="summary">
        <div class="bi-cards" v-if="summaryCards.length">
          <BiCard
            v-for="(c, idx) in summaryCards"
            :key="c.key"
            :label="c.label"
            :value="fmtNum(c.value)"
            :color="cardColors[idx % cardColors.length]"
          />
        </div>
        <div class="bi-summary-update" v-if="summaryUpdated">数据更新于 {{ summaryUpdated }}</div>
      </el-tab-pane>

      <!-- 分区统计 -->
      <el-tab-pane label="分区统计" name="zone">
        <div class="bi-toolbar">
          <el-button type="primary" size="small" @click="exportCSV('zone')">导出 CSV</el-button>
        </div>
        <div class="bi-charts-row" v-if="zoneData.length > 0">
          <div class="bi-charts-row__main">
            <BiChart :option="zoneBarOption" :height="zoneData.length * 36 + 60" />
          </div>
          <div class="bi-charts-row__side">
            <BiChart :option="zonePieOption" :height="240" />
          </div>
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
          <el-select v-model="creatorMetric" size="small" style="width: 140px" @change="updateCreatorOption">
            <el-option label="按总播放" value="total_plays" />
            <el-option label="按投币数" value="total_coins" />
            <el-option label="按粉丝数" value="fans_count" />
          </el-select>
          <el-button type="primary" size="small" @click="exportCSV('creator')">导出 CSV</el-button>
        </div>
        <BiChart v-if="creatorData.length > 0" :option="creatorBarOption" :height="Math.max(200, creatorData.length * 32 + 40)" />
        <el-table :data="creatorData" stripe size="default" empty-text="暂无数据" style="margin-top: 14px">
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
          <el-select v-model="tsMetric" size="small" style="width: 140px" @change="updateTimeSeriesOption">
            <el-option label="每日播放量" value="daily_plays" />
            <el-option label="每日新用户" value="daily_users" />
            <el-option label="每日新视频" value="daily_videos" />
          </el-select>
          <el-button type="primary" size="small" @click="exportCSV('timeseries')">导出 CSV</el-button>
          <el-button size="small" @click="serverExport('plays')">服务端导出</el-button>
        </div>
        <BiChart v-if="tsData.length > 0" :option="tsLineOption" :height="280" />
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

        <h4 class="bi-subtitle">视频稿件</h4>
        <div class="bi-cards">
          <BiCard label="视频总量" :value="fmtNum(msVideoSummary.total || 0)" color="blue" />
          <BiCard label="已发布" :value="fmtNum(msVideoSummary.published || 0)" color="teal" />
          <BiCard label="总播放" :value="fmtNum(msVideoSummary.total_plays || 0)" color="purple" />
          <BiCard label="总投币" :value="fmtNum(msVideoSummary.total_coins || 0)" color="amber" />
          <BiCard label="总收藏" :value="fmtNum(msVideoSummary.total_favs || 0)" color="pink" />
        </div>

        <h4 class="bi-subtitle">图文稿件</h4>
        <div class="bi-cards">
          <BiCard label="图文总量" :value="fmtNum(msArticleSummary.total || 0)" color="blue" />
          <BiCard label="已发布" :value="fmtNum(msArticleSummary.published || 0)" color="teal" />
          <BiCard label="总阅读" :value="fmtNum(msArticleSummary.total_views || 0)" color="purple" />
          <BiCard label="总投币" :value="fmtNum(msArticleSummary.total_coins || 0)" color="amber" />
          <BiCard label="总收藏" :value="fmtNum(msArticleSummary.total_favs || 0)" color="pink" />
        </div>

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
        <div class="bi-cards">
          <BiCard label="累计点赞" :value="fmtNum(engageTotals.total_likes)" color="pink" />
          <BiCard label="累计收藏" :value="fmtNum(engageTotals.total_favs)" color="amber" />
          <BiCard label="视频投币" :value="fmtNum(engageTotals.total_video_coins)" color="purple" />
          <BiCard label="文章投币" :value="fmtNum(engageTotals.total_article_coins)" color="teal" />
        </div>
        <BiChart v-if="engagementTS.length > 0" :option="engagementOption" :height="280" />
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
import { ref, computed, watch, onMounted } from 'vue'
import http from '@/utils/adminHttp'
import { ElMessage } from 'element-plus'
import BiChart from '@/components/admin/BiChart.vue'
import BiCard from '@/components/admin/BiCard.vue'

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
const cardColors = ['blue', 'teal', 'purple', 'amber', 'pink', 'coral']

// Zone data
const zoneData = ref([])

// Creator data
const creatorData = ref([])
const creatorMetric = ref('total_plays')

// Time series data
const tsData = ref([])
const tsRange = ref([])
const tsMetric = ref('daily_plays')

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

// ── ECharts options ──

const zoneBarOption = computed(() => ({
  tooltip: { trigger: 'axis', axisPointer: { type: 'shadow' } },
  grid: { left: 90, right: 60, top: 10, bottom: 20 },
  xAxis: { type: 'value', axisLabel: { fontSize: 11, color: '#9499a0' } },
  yAxis: {
    type: 'category',
    data: zoneData.value.map(z => z.zone_name),
    axisLabel: { fontSize: 11, color: '#61666d' },
    inverse: true
  },
  series: [{
    type: 'bar',
    data: zoneData.value.map((z, i) => ({
      value: z.video_count,
      itemStyle: { color: ['#378add', '#1d9e75', '#7f77dd', '#ba7517', '#d4537e', '#d85a30'][i % 6], borderRadius: [0, 4, 4, 0] }
    })),
    barMaxWidth: 28,
    label: { show: true, position: 'right', fontSize: 11, color: '#61666d' }
  }]
}))

const zonePieOption = computed(() => ({
  tooltip: { trigger: 'item', formatter: '{b}: {c} ({d}%)' },
  legend: {
    orient: 'vertical',
    right: 0,
    top: 'center',
    itemGap: 12,
    itemWidth: 8,
    itemHeight: 8,
    textStyle: { fontSize: 12, color: '#61666d' }
  },
  series: [{
    type: 'pie',
    radius: ['42%', '68%'],
    center: ['38%', '50%'],
    data: zoneData.value.map((z, i) => ({
      name: z.zone_name,
      value: z.video_count,
      itemStyle: { color: ['#378add', '#1d9e75', '#7f77dd', '#ba7517', '#d4537e', '#d85a30'][i % 6] }
    })),
    label: { show: false },
    emphasis: { label: { show: true, fontSize: 13, fontWeight: 'bold' } }
  }]
}))

const creatorBarOption = computed(() => {
  const top10 = [...creatorData.value].sort((a, b) => b[creatorMetric.value] - a[creatorMetric.value]).slice(0, 10).reverse()
  return {
    tooltip: { trigger: 'axis', axisPointer: { type: 'shadow' } },
    grid: { left: 100, right: 60, top: 10, bottom: 20 },
    xAxis: { type: 'value', axisLabel: { fontSize: 11, color: '#9499a0' } },
    yAxis: {
      type: 'category',
      data: top10.map(c => (c.nickname || c.username || `ID:${c.user_id}`)),
      axisLabel: { fontSize: 11, color: '#61666d' },
      inverse: true
    },
    series: [{
      type: 'bar',
      data: top10.map(c => ({ value: c[creatorMetric.value], itemStyle: { color: '#378add', borderRadius: [0, 4, 4, 0] } })),
      barMaxWidth: 22,
      label: { show: true, position: 'right', fontSize: 11, color: '#61666d', formatter: p => fmtNum(p.value) }
    }]
  }
})

const tsLineOption = computed(() => ({
  tooltip: { trigger: 'axis' },
  grid: { left: 50, right: 20, top: 20, bottom: 30 },
  xAxis: {
    type: 'category',
    data: tsData.value.map(t => t.date),
    axisLabel: { fontSize: 10, color: '#9499a0', rotate: 30 }
  },
  yAxis: { type: 'value', axisLabel: { fontSize: 11, color: '#9499a0' } },
  series: [{
    type: 'line',
    data: tsData.value.map(t => t[tsMetric.value] || 0),
    smooth: true,
    symbol: 'circle',
    symbolSize: 4,
    lineStyle: { color: '#378add', width: 2 },
    itemStyle: { color: '#378add' },
    areaStyle: { color: { type: 'linear', x: 0, y: 0, x2: 0, y2: 1, colorStops: [{ offset: 0, color: 'rgba(55,138,221,0.2)' }, { offset: 1, color: 'rgba(55,138,221,0.02)' }] } }
  }]
}))

const engagementOption = computed(() => ({
  tooltip: { trigger: 'axis' },
  legend: { data: engagementTS.value.map(s => s.label), bottom: 0, textStyle: { fontSize: 11 } },
  grid: { left: 50, right: 20, top: 20, bottom: 40 },
  xAxis: {
    type: 'category',
    data: engagementTS.value[0]?.points?.map(p => p.date) || [],
    axisLabel: { fontSize: 10, color: '#9499a0', rotate: 30 }
  },
  yAxis: { type: 'value', axisLabel: { fontSize: 11, color: '#9499a0' } },
  series: engagementTS.value.map(s => ({
    name: s.label,
    type: 'line',
    data: s.points.map(p => p.value || 0),
    smooth: true,
    symbol: 'none',
    lineStyle: { color: s.color, width: 2 }
  }))
}))

function updateCreatorOption() {
  // triggers computed re-evaluation
}

function updateTimeSeriesOption() {
  // triggers computed re-evaluation
}

// ── Data fetching ──

async function fetchZones() {
  loading.value = true
  try {
    const d = await api('/bi/zone-stats')
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
    const d = await api('/bi/creator-stats?limit=20')
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
    const metricMap = { daily_plays: 'plays', daily_users: 'new_users', daily_videos: 'new_videos' }
    const fetches = ['plays', 'new_users', 'new_videos'].map(m =>
      api(`/bi/time-series?metric=${m}&days=30`)
    )
    const results = await Promise.all(fetches)
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
      tsMetric.value = 'daily_plays'
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
    const seriesDefs = [
      { key: 'comments', label: '评论', color: '#378add', raw: d.comments_ts || [] },
      { key: 'danmaku', label: '弹幕', color: '#d4537e', raw: d.danmaku_ts || [] },
      { key: 'likes', label: '点赞', color: '#1d9e75', raw: d.likes_ts || [] },
      { key: 'favs', label: '收藏', color: '#ba7517', raw: d.favs_ts || [] },
      { key: 'coins', label: '投币', color: '#7f77dd', raw: d.coins_ts || [] },
    ]

    // Build unified date axis from all series
    const dateSet = new Set()
    seriesDefs.forEach(s => s.raw.forEach(p => { if (p.date) dateSet.add(p.date) }))
    const allDates = Array.from(dateSet).sort()

    // Align each series to the unified date axis (fill 0 for missing dates)
    const pointsByDate = seriesDefs.map(() => {
      const valMap = {}
      return { valMap }
    })
    seriesDefs.forEach((s, si) => {
      s.raw.forEach(p => { if (p.date) pointsByDate[si].valMap[p.date] = p.value || 0 })
    })

    engagementTS.value = seriesDefs.map((s, si) => ({
      key: s.key,
      label: s.label,
      color: s.color,
      points: allDates.map(d => ({ date: d, value: pointsByDate[si].valMap[d] || 0 })),
      maxVal: Math.max(1, ...s.raw.map(p => p.value || 0)),
    }))
  } catch (e) {
    ElMessage.error(e.message || '加载互动统计失败')
  } finally {
    loading.value = false
  }
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

async function serverExport(metric) {
  try {
    const res = await http.post(ADMIN_API + '/bi/export', { metric, days: 30 }, { responseType: 'blob' })
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
.bi-toolbar { margin-bottom: 14px; display: flex; gap: 10px; align-items: center; flex-wrap: wrap; }
.bi-muted { color: #9499a0; font-size: 12px; }
.bi-subtitle { margin: 0 0 10px; font-size: 13px; font-weight: 600; color: #18191c; }

/* Cards grid */
.bi-cards {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
  gap: 12px;
  margin-bottom: 14px;
}

/* Charts row */
.bi-charts-row {
  display: flex;
  gap: 16px;
  flex-wrap: wrap;
}
.bi-charts-row__main { flex: 1 1 55%; min-width: 360px; }
.bi-charts-row__side { flex: 0 0 300px; min-width: 260px; }

/* Summary */
.bi-summary-update { font-size: 12px; color: #c9ccd0; margin-bottom: 8px; }

/* Saved */
.bi-saved { margin-top: 20px; background: #fff; border: 1px solid #e3e5e7; border-radius: 8px; padding: 16px 20px; }
.bi-saved__head { display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px; flex-wrap: wrap; gap: 8px; }
.bi-saved__title { margin: 0; font-size: 14px; font-weight: 600; color: #18191c; }
.bi-saved__actions { display: flex; gap: 8px; align-items: center; }
</style>
