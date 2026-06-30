<template>
  <div class="pa-page" v-loading="loading">
    <header class="pa-page__head">
      <h2 class="pa-page__title">播放器高级</h2>
      <p class="pa-page__desc">管理视频章节标记与多码率版本</p>
    </header>

    <!-- Video selector -->
    <div class="pa-selector">
      <el-input v-model="videoQuery" placeholder="输入视频 ID 搜索..." size="default" style="width:200px" @keyup.enter="searchVideo" />
      <el-button type="primary" size="default" @click="searchVideo" :loading="searching">查询</el-button>
    </div>

    <template v-if="currentVideo">
      <!-- Video info -->
      <div class="pa-video-info">
        <span class="pa-video-info__label">当前视频：</span>
        <strong>{{ currentVideo.title || '(无标题)' }}</strong>
        <span class="pa-muted">（ID: {{ currentVideo.id }}）</span>
      </div>

      <el-tabs v-model="activeTab">
        <!-- Chapters tab -->
        <el-tab-pane label="章节管理" name="chapters">
          <div class="pa-toolbar">
            <el-button type="primary" size="default" @click="openChapterDialog(null)">添加章节</el-button>
          </div>
          <el-table :data="chapters" stripe size="default" empty-text="暂无章节">
            <el-table-column prop="id" label="ID" width="60" />
            <el-table-column prop="title" label="章节标题" min-width="180" show-overflow-tooltip />
            <el-table-column label="起始时间" width="120">
              <template #default="{ row }">{{ formatTime(row.time_sec) }}</template>
            </el-table-column>
            <el-table-column label="创建时间" width="160">
              <template #default="{ row }">{{ fmtTs(row.created_at) }}</template>
            </el-table-column>
            <el-table-column label="操作" width="120">
              <template #default="{ row }">
                <el-button size="small" text type="primary" @click="openChapterDialog(row)">编辑</el-button>
                <el-popconfirm title="确认删除？" @confirm="deleteChapter(row)">
                  <template #reference>
                    <el-button size="small" text type="danger">删除</el-button>
                  </template>
                </el-popconfirm>
              </template>
            </el-table-column>
          </el-table>
        </el-tab-pane>

        <!-- Bitrates tab -->
        <el-tab-pane label="码率管理" name="bitrates">
          <div class="pa-toolbar">
            <el-button type="primary" size="default" @click="openBitrateDialog(null)">添加码率</el-button>
          </div>
          <el-table :data="bitrates" stripe size="default" empty-text="暂无码率版本">
            <el-table-column prop="id" label="ID" width="60" />
            <el-table-column prop="label" label="清晰度" width="100" />
            <el-table-column label="分辨率" width="140">
              <template #default="{ row }">{{ row.width }}×{{ row.height }}</template>
            </el-table-column>
            <el-table-column label="码率" width="100">
              <template #default="{ row }">{{ row.kbps }} kbps</template>
            </el-table-column>
            <el-table-column prop="url" label="URL" min-width="220" show-overflow-tooltip />
            <el-table-column label="操作" width="120">
              <template #default="{ row }">
                <el-button size="small" text type="primary" @click="openBitrateDialog(row)">编辑</el-button>
                <el-popconfirm title="确认删除？" @confirm="deleteBitrate(row)">
                  <template #reference>
                    <el-button size="small" text type="danger">删除</el-button>
                  </template>
                </el-popconfirm>
              </template>
            </el-table-column>
          </el-table>
        </el-tab-pane>
      </el-tabs>
    </template>
    <div v-else-if="searched" class="pa-empty">未找到该视频</div>
    <div v-else class="pa-empty">请输入视频 ID 开始管理</div>

    <!-- Chapter dialog -->
    <el-dialog v-model="chapterDialogVisible" :title="chapterForm.id ? '编辑章节' : '添加章节'" width="480px" destroy-on-close>
      <el-form :model="chapterForm" label-width="90px" size="default">
        <el-form-item label="章节标题">
          <el-input v-model="chapterForm.title" placeholder="如：开场、高潮、结尾" />
        </el-form-item>
        <el-form-item label="起始时间(秒)">
          <el-input-number v-model="chapterForm.time_sec" :min="0" :precision="1" style="width:100%" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="chapterDialogVisible = false">取消</el-button>
        <el-button type="primary" :loading="saving" @click="saveChapter">保存</el-button>
      </template>
    </el-dialog>

    <!-- Bitrate dialog -->
    <el-dialog v-model="bitrateDialogVisible" :title="bitrateForm.id ? '编辑码率' : '添加码率'" width="480px" destroy-on-close>
      <el-form :model="bitrateForm" label-width="90px" size="default">
        <el-form-item label="清晰度标签">
          <el-select v-model="bitrateForm.label" style="width:100%">
            <el-option label="360P" value="360P" />
            <el-option label="480P" value="480P" />
            <el-option label="720P" value="720P" />
            <el-option label="1080P" value="1080P" />
            <el-option label="4K" value="4K" />
          </el-select>
        </el-form-item>
        <el-form-item label="宽度">
          <el-input-number v-model="bitrateForm.width" :min="1" :max="7680" style="width:100%" />
        </el-form-item>
        <el-form-item label="高度">
          <el-input-number v-model="bitrateForm.height" :min="1" :max="4320" style="width:100%" />
        </el-form-item>
        <el-form-item label="码率(kbps)">
          <el-input-number v-model="bitrateForm.kbps" :min="1" style="width:100%" />
        </el-form-item>
        <el-form-item label="播放URL">
          <el-input v-model="bitrateForm.url" placeholder="输入该版本的视频 URL" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="bitrateDialogVisible = false">取消</el-button>
        <el-button type="primary" :loading="saving" @click="saveBitrate">保存</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup>
import { ref, reactive } from 'vue'
import http from '@/utils/adminHttp'
import { ElMessage } from 'element-plus'

const ADMIN_API = '/api/v1/admin'

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
const searching = ref(false)
const saving = ref(false)
const searched = ref(false)
const activeTab = ref('chapters')
const videoQuery = ref('')

const currentVideo = ref(null)
const chapters = ref([])
const bitrates = ref([])

// Chapter form
const chapterDialogVisible = ref(false)
const chapterForm = reactive({ id: null, title: '', time_sec: 0 })

// Bitrate form
const bitrateDialogVisible = ref(false)
const bitrateForm = reactive({ id: null, label: '720P', width: 1280, height: 720, kbps: 2000, url: '' })

async function searchVideo() {
  const q = videoQuery.value.trim()
  if (!q) return
  searching.value = true
  searched.value = true
  try {
    // Use public API to get video info
    const resp = await http.get(`/api/v1/videos/${q}`)
    const d = resp.data || resp
    if (d && d.id) {
      currentVideo.value = d
      loadChapters()
      loadBitrates()
    } else {
      currentVideo.value = null
    }
  } catch (e) {
    currentVideo.value = null
    ElMessage.error('未找到该视频')
  } finally {
    searching.value = false
  }
}

async function loadChapters() {
  if (!currentVideo.value) return
  loading.value = true
  try {
    const d = await api(`/videos/${currentVideo.value.id}/chapters`)
    chapters.value = d.items || d || []
  } catch (e) {
    ElMessage.error(e.message || '加载章节失败')
  } finally {
    loading.value = false
  }
}

async function loadBitrates() {
  if (!currentVideo.value) return
  loading.value = true
  try {
    const d = await api(`/videos/${currentVideo.value.id}/bitrates`)
    bitrates.value = d.items || d || []
  } catch (e) {
    ElMessage.error(e.message || '加载码率失败')
  } finally {
    loading.value = false
  }
}

function openChapterDialog(row) {
  if (row) {
    Object.assign(chapterForm, { id: row.id, title: row.title, time_sec: row.time_sec })
  } else {
    Object.assign(chapterForm, { id: null, title: '', time_sec: 0 })
  }
  chapterDialogVisible.value = true
}

async function saveChapter() {
  if (!chapterForm.title.trim()) { ElMessage.warning('请输入章节标题'); return }
  saving.value = true
  try {
    if (chapterForm.id) {
      await api(`/videos/${currentVideo.value.id}/chapters/${chapterForm.id}`, { method: 'DELETE' })
    }
    await api(`/videos/${currentVideo.value.id}/chapters`, { method: 'POST', body: { title: chapterForm.title, time_sec: chapterForm.time_sec } })
    ElMessage.success('已保存')
    chapterDialogVisible.value = false
    loadChapters()
  } catch (e) {
    ElMessage.error(e.message || '保存失败')
  } finally {
    saving.value = false
  }
}

async function deleteChapter(row) {
  try {
    await api(`/videos/${currentVideo.value.id}/chapters/${row.id}`, { method: 'DELETE' })
    ElMessage.success('已删除')
    loadChapters()
  } catch (e) {
    ElMessage.error(e.message || '删除失败')
  }
}

function openBitrateDialog(row) {
  if (row) {
    Object.assign(bitrateForm, { id: row.id, label: row.label, width: row.width, height: row.height, kbps: row.kbps, url: row.url })
  } else {
    Object.assign(bitrateForm, { id: null, label: '720P', width: 1280, height: 720, kbps: 2000, url: '' })
  }
  bitrateDialogVisible.value = true
}

async function saveBitrate() {
  if (!bitrateForm.url.trim()) { ElMessage.warning('请输入播放URL'); return }
  saving.value = true
  try {
    const body = { label: bitrateForm.label, width: bitrateForm.width, height: bitrateForm.height, kbps: bitrateForm.kbps, url: bitrateForm.url }
    if (bitrateForm.id) {
      await api(`/videos/${currentVideo.value.id}/bitrates/${bitrateForm.id}`, { method: 'DELETE' })
    }
    await api(`/videos/${currentVideo.value.id}/bitrates`, { method: 'POST', body })
    ElMessage.success('已保存')
    bitrateDialogVisible.value = false
    loadBitrates()
  } catch (e) {
    ElMessage.error(e.message || '保存失败')
  } finally {
    saving.value = false
  }
}

async function deleteBitrate(row) {
  try {
    await api(`/videos/${currentVideo.value.id}/bitrates/${row.id}`, { method: 'DELETE' })
    ElMessage.success('已删除')
    loadBitrates()
  } catch (e) {
    ElMessage.error(e.message || '删除失败')
  }
}

function formatTime(sec) {
  if (sec == null) return '—'
  const m = Math.floor(sec / 60)
  const s = Math.floor(sec % 60)
  return `${m}:${String(s).padStart(2, '0')}`
}

function fmtTs(t) {
  if (!t) return ''
  const d = new Date(t)
  const pad = (n) => String(n).padStart(2, '0')
  return `${d.getFullYear()}-${pad(d.getMonth()+1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}`
}
</script>

<style scoped>
.pa-page { padding: 20px 24px; }
.pa-page__head { margin-bottom: 14px; }
.pa-page__title { margin: 0 0 4px; font-size: 18px; font-weight: 600; color: #18191c; }
.pa-page__desc { margin: 0; font-size: 13px; color: #9499a0; }
.pa-selector { display: flex; gap: 10px; align-items: center; margin-bottom: 16px; }
.pa-video-info { margin-bottom: 14px; padding: 10px 14px; background: #f0f6ff; border-radius: 8px; font-size: 14px; }
.pa-video-info__label { color: #9499a0; }
.pa-muted { color: #9499a0; font-size: 12px; }
.pa-toolbar { margin-bottom: 14px; }
.pa-empty { text-align: center; padding: 60px 0; color: #9499a0; font-size: 14px; }
</style>
