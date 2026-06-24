<template>
  <div class="bi-page" v-loading="loading">
    <header class="bi-page__head">
      <h2 class="bi-page__title">BI 统计报表</h2>
      <p class="bi-page__desc">分区统计、创作者排行与时序数据</p>
    </header>

    <el-tabs v-model="activeTab" @tab-change="onTabChange">
      <!-- 分区统计 -->
      <el-tab-pane label="分区统计" name="zone">
        <div class="bi-toolbar">
          <el-button type="primary" size="small" @click="exportCSV('zone')">导出 CSV</el-button>
        </div>
        <div class="bi-zone-chart" v-if="zoneData.length > 0">
          <svg :viewBox="`0 0 ${chartW} ${zoneChartH}`" class="bi-zone-svg">
            <line v-for="i in 5" :key="'gl'+i" :x1="60" :y1="10 + (i-1)*zoneBarStep" :x2="chartW" :y2="10 + (i-1)*zoneBarStep" stroke="#e8e9eb" stroke-width="1" />
            <g v-for="(z, idx) in zoneData" :key="z.zone_id" :transform="`translate(0, ${10 + idx * zoneBarStep})`">
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
          <el-select v-model="creatorMetric" size="small" style="width: 140px" @change="fetchCreators">
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
          <el-table-column label="总播放" width="110" sortable :prop="creatorMetric === 'total_plays' ? 'total_plays' : null">
            <template #default="{ row }">{{ fmtNum(row.total_plays) }}</template>
          </el-table-column>
          <el-table-column label="总投币" width="100" sortable>
            <template #default="{ row }">{{ fmtNum(row.total_coins) }}</template>
          </el-table-column>
          <el-table-column label="粉丝数" width="100" sortable>
            <template #default="{ row }">{{ fmtNum(row.fans_count) }}</template>
          </el-table-column>
          <el-table-column label="视频数" width="90">
            <template #default="{ row }">{{ row.video_count }}</template>
          </el-table-column>
          <el-table-column label="文章数" width="90">
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
          <el-select v-model="tsMetric" size="small" style="width: 140px" @change="fetchTimeSeries">
            <el-option label="每日播放量" value="daily_plays" />
            <el-option label="每日新用户" value="daily_users" />
            <el-option label="每日新视频" value="daily_videos" />
          </el-select>
          <el-button type="primary" size="small" @click="exportCSV('timeseries')">导出 CSV</el-button>
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
        <el-table-column prop="type" label="类型" width="100" />
        <el-table-column label="创建时间" width="160">
          <template #default="{ row }">{{ fmtTime(row.created_at) }}</template>
        </el-table-column>
        <el-table-column label="操作" width="120">
          <template #default="{ row }">
            <el-button size="small" text type="primary" @click="loadReport(row)">加载</el-button>
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

function getToken() { return localStorage.getItem("minibili_admin_access_token") || "" }
async function api(path, opts = {}) {
  const res = await fetch(ADMIN_API + path, {
    method: opts.method || "GET",
    headers: { "Content-Type": "application/json", Authorization: "Bearer " + getToken(), ...(opts.headers || {}) },
    body: opts.body ? JSON.stringify(opts.body) : undefined,
  })
  const body = await res.json()
  if (!res.ok || (body.code != null && body.code !== 0)) { throw new Error(body.msg || body.message || "请求失败") }
  return body.data || body
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
    const d = await api('/bi/zones')
    zoneData.value = d.items || d || []
  } catch (e) {
    ElMessage.error(e.message || '加载失败')
  } finally {
    loading.value = false
  }
}

async function fetchCreators() {
  loading.value = true
  try {
    const d = await api(`/bi/creators?metric=${creatorMetric.value}`)
    creatorData.value = d.items || d || []
  } catch (e) {
    ElMessage.error(e.message || '加载失败')
  } finally {
    loading.value = false
  }
}

async function fetchTimeSeries() {
  loading.value = true
  try {
    const params = new URLSearchParams({ metric: tsMetric.value })
    if (tsRange.value && tsRange.value.length === 2) {
      params.set('start_date', new Date(tsRange.value[0]).toISOString().slice(0, 10))
      params.set('end_date', new Date(tsRange.value[1]).toISOString().slice(0, 10))
    }
    const d = await api(`/bi/timeseries?${params}`)
    tsData.value = d.items || d || []
  } catch (e) {
    ElMessage.error(e.message || '加载失败')
  } finally {
    loading.value = false
  }
}

async function fetchSavedReports() {
  try {
    const d = await api('/bi/reports')
    savedReports.value = d.items || d || []
  } catch { /* ignore */ }
}

function onTabChange(tab) {
  if (tab === 'zone' && zoneData.value.length === 0) fetchZones()
  if (tab === 'creator' && creatorData.value.length === 0) fetchCreators()
  if (tab === 'timeseries' && tsData.value.length === 0) fetchTimeSeries()
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
        type: activeTab.value,
        config: { tab: activeTab.value, metric: activeTab.value === 'creator' ? creatorMetric.value : tsMetric.value },
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
  if (row.config?.tab) {
    activeTab.value = row.config.tab
    if (row.config.metric) {
      if (row.config.tab === 'creator') creatorMetric.value = row.config.metric
      if (row.config.tab === 'timeseries') tsMetric.value = row.config.metric
    }
    onTabChange(row.config.tab)
  }
}

function exportCSV(type) {
  let r; ows = []
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

function fmtNum(n) {
  if (n == null) return '0'
  if (n >= 10000) return (n / 10000).toFixed(1) + '万'
  return String(n)
}

function fmtTime(t) {
  if (!t) return ''
  const d = new Date(t)
  const pad = (n) => String(n).padStart(2, '0')
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}`
}

onMounted(() => {
  fetchZones()
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
</style>
