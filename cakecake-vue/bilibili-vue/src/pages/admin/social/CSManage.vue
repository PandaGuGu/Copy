<template>
  <div class="cs-page" v-loading="loading">
    <header class="cs-page__head">
      <h2 class="cs-page__title">客服管理</h2>
      <p class="cs-page__desc">管理客服会话与回复模板</p>
    </header>

    <el-tabs v-model="activeTab" @tab-change="onTabChange">
      <!-- 会话列表 -->
      <el-tab-pane label="会话" name="conversations">
        <AdminDataTable :data="conversations" :loading="loading" :show-pagination="false">
          <template #search-bar>
            <el-select v-model="filterStatus" placeholder="状态" clearable size="default" style="width: 120px" @change="fetchConversations">
              <el-option label="全部" value="" />
              <el-option label="进行中" value="active" />
              <el-option label="已关闭" value="closed" />
            </el-select>
          </template>
          <el-table-column prop="id" label="ID" width="60" />
          <el-table-column label="用户" width="140">
            <template #default="{ row }">
              <span v-if="row.user">{{ row.user.nickname || row.user.username }}</span>
              <span v-else class="cs-muted">#{{ row.user_id }}</span>
            </template>
          </el-table-column>
          <el-table-column label="客服" width="130">
            <template #default="{ row }">
              <span v-if="row.admin">{{ row.admin.nickname || row.admin.username }}</span>
              <span v-else class="cs-muted">未指派</span>
            </template>
          </el-table-column>
          <el-table-column label="最后消息" min-width="200" show-overflow-tooltip>
            <template #default="{ row }">{{ row.last_message || '—' }}</template>
          </el-table-column>
          <el-table-column label="状态" width="80">
            <template #default="{ row }">
              <el-tag :type="row.status === 'active' ? 'success' : 'info'" size="small" effect="plain">
                {{ row.status === 'active' ? '进行中' : '已关闭' }}
              </el-tag>
            </template>
          </el-table-column>
          <el-table-column label="更新时间" width="155">
            <template #default="{ row }">{{ fmtTime(row.updated_at) }}</template>
          </el-table-column>
          <el-table-column label="操作" width="150" fixed="right">
            <template #default="{ row }">
              <el-button size="small" text type="primary" @click="openConversation(row)">查看</el-button>
              <el-popconfirm
                v-if="row.status === 'active'"
                title="确认关闭会话？"
                @confirm="closeConversation(row)"
              >
                <template #reference>
                  <el-button size="small" type="danger" plain>关闭</el-button>
                </template>
              </el-popconfirm>
            </template>
          </el-table-column>
        </AdminDataTable>
      </el-tab-pane>

      <!-- 回复模板 -->
      <el-tab-pane label="回复模板" name="templates">
        <div class="cs-toolbar">
          <el-button type="primary" size="default" @click="openTemplateDialog(null)">新建模板</el-button>
        </div>
        <el-table :data="templates" stripe size="default" empty-text="暂无模板">
          <el-table-column prop="id" label="ID" width="60" />
          <el-table-column prop="name" label="名称" min-width="140" />
          <el-table-column label="分类" width="100">
            <template #default="{ row }">
              <el-tag size="small" effect="plain">{{ templateCategoryLabel(row.category) }}</el-tag>
            </template>
          </el-table-column>
          <el-table-column label="内容" min-width="240" show-overflow-tooltip>
            <template #default="{ row }">{{ row.content }}</template>
          </el-table-column>
          <el-table-column label="操作" width="120" fixed="right">
            <template #default="{ row }">
              <el-button size="small" text type="primary" @click="openTemplateDialog(row)">编辑</el-button>
              <el-popconfirm title="确认删除？" @confirm="deleteTemplate(row)">
                <template #reference>
                  <el-button size="small" text type="danger">删除</el-button>
                </template>
              </el-popconfirm>
            </template>
          </el-table-column>
        </el-table>
      </el-tab-pane>
    </el-tabs>

    <!-- 会话详情弹窗 -->
    <el-dialog v-model="convDialogVisible" title="客服会话" width="640px" destroy-on-close>
      <template v-if="convDetail">
        <el-descriptions :column="2" border size="small">
          <el-descriptions-item label="会话ID">{{ convDetail.id }}</el-descriptions-item>
          <el-descriptions-item label="状态">
            <el-tag :type="convDetail.status === 'active' ? 'success' : 'info'" size="small">
              {{ convDetail.status === 'active' ? '进行中' : '已关闭' }}
            </el-tag>
          </el-descriptions-item>
          <el-descriptions-item label="用户" v-if="convDetail.user">
            {{ convDetail.user.nickname || convDetail.user.username }}
          </el-descriptions-item>
          <el-descriptions-item label="客服" v-if="convDetail.admin">
            {{ convDetail.admin.nickname || convDetail.admin.username }}
          </el-descriptions-item>
          <el-descriptions-item label="客服" v-else>
            <span class="cs-muted">未指派</span>
          </el-descriptions-item>
        </el-descriptions>

        <!-- 聊天消息 -->
        <div class="cs-chat">
          <div class="cs-chat__list" ref="chatBox">
            <div
              v-for="msg in convDetail.messages"
              :key="msg.id"
              class="cs-chat__msg"
              :class="{ 'cs-chat__msg--admin': msg.is_admin }"
            >
              <div class="cs-chat__bubble">
                <div class="cs-chat__meta">
                  <span>{{ msg.is_admin ? '客服' : '用户' }}</span>
                  <span class="cs-chat__time">{{ fmtTime(msg.created_at) }}</span>
                </div>
                <div class="cs-chat__text">{{ msg.content }}</div>
              </div>
            </div>
            <div v-if="!convDetail.messages || convDetail.messages.length === 0" class="cs-muted cs-chat__empty">
              暂无消息
            </div>
          </div>
        </div>

        <!-- 回复区 -->
        <div class="cs-reply" v-if="convDetail.status !== 'closed'">
          <div class="cs-reply__templates" v-if="templates.length > 0">
            <el-select
              v-model="selectedTemplate"
              placeholder="选择模板快速回复"
              size="small"
              style="width: 100%; margin-bottom: 8px"
              @change="applyTemplate"
            >
              <el-option
                v-for="t in templates"
                :key="t.id"
                :label="t.name"
                :value="t.id"
              />
            </el-select>
          </div>
          <div class="cs-reply__row">
            <el-input
              v-model="replyContent"
              type="textarea"
              :rows="3"
              placeholder="输入回复内容..."
              size="small"
            />
          </div>
          <div class="cs-reply__actions">
            <el-button
              size="small"
              type="primary"
              @click="sendMessage"
              :disabled="!replyContent.trim()"
            >发送</el-button>
            <el-button
              v-if="!convDetail.admin_id"
              size="small"
              @click="assignSelf"
            >指派给我</el-button>
            <el-popconfirm
              title="确认关闭此会话？用户将无法继续发送消息。"
              @confirm="closeConversationFromDetail"
            >
              <template #reference>
                <el-button size="small" type="danger">关闭会话</el-button>
              </template>
            </el-popconfirm>
          </div>
        </div>
      </template>
    </el-dialog>

    <!-- 模板编辑弹窗 -->
    <el-dialog v-model="templateDialogVisible" :title="templateForm.id ? '编辑模板' : '新建模板'" width="480px" destroy-on-close>
      <el-form :model="templateForm" label-width="70px" size="default">
        <el-form-item label="名称">
          <el-input v-model="templateForm.name" placeholder="模板名称" />
        </el-form-item>
        <el-form-item label="分类">
          <el-select v-model="templateForm.category" style="width: 100%">
            <el-option label="通用" value="general" />
            <el-option label="账号" value="account" />
            <el-option label="内容" value="content" />
            <el-option label="支付" value="payment" />
            <el-option label="技术" value="technical" />
          </el-select>
        </el-form-item>
        <el-form-item label="内容">
          <el-input v-model="templateForm.content" type="textarea" :rows="5" placeholder="模板回复内容" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="templateDialogVisible = false">取消</el-button>
        <el-button type="primary" :loading="saving" @click="saveTemplate">保存</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup>
import { ref, reactive, onMounted, nextTick } from 'vue'
import { ElMessage } from 'element-plus'
import AdminDataTable from '@/components/admin/AdminDataTable.vue'
import {
  adminListConversations,
  adminGetConversation,
  adminSendConversationMessage,
  adminCloseConversation,
  adminAssignConversation,
  adminListCsTemplates,
  adminCreateCsTemplate,
  adminUpdateCsTemplate,
  adminDeleteCsTemplate,
} from '@/api/admin'



const loading = ref(false)
const saving = ref(false)
const activeTab = ref('conversations')

// Conversations
const conversations = ref([])
const filterStatus = ref('')
const convDialogVisible = ref(false)
const convDetail = ref(null)
const replyContent = ref('')
const selectedTemplate = ref(null)
const chatBox = ref(null)

// Templates
const templates = ref([])
const templateDialogVisible = ref(false)
const templateForm = reactive({
  id: null, name: '', category: 'general', content: '',
})

async function fetchConversations() {
  loading.value = true
  try {
    const params = {}
    if (filterStatus.value) params.status = filterStatus.value
    const d = await adminListConversations(params)
    conversations.value = d.data.items || []
  } catch (e) {
    ElMessage.error(e.message || '加载失败')
  } finally {
    loading.value = false
  }
}

async function fetchTemplates() {
  loading.value = true
  try {
    const d = await adminListCsTemplates()
    templates.value = d.data.templates || d.data || []
  } catch (e) {
    ElMessage.error(e.message || '加载失败')
  } finally {
    loading.value = false
  }
}

function onTabChange(tab) {
  if (tab === 'conversations' && conversations.value.length === 0) fetchConversations()
  if (tab === 'templates' && templates.value.length === 0) fetchTemplates()
}

async function openConversation(row) {
  try {
    const d = await adminGetConversation(row.id)
    convDetail.value = d.data
    convDialogVisible.value = true
    replyContent.value = ''
    selectedTemplate.value = null
    if (templates.value.length === 0) {
      try { await fetchTemplates() } catch { /* ignore */ }
    }
    nextTick(() => {
      if (chatBox.value) chatBox.value.scrollTop = chatBox.value.scrollHeight
    })
  } catch (e) {
    ElMessage.error(e.message || '获取详情失败')
  }
}

async function sendMessage() {
  if (!convDetail.value || !replyContent.value.trim()) return
  try {
    await adminSendConversationMessage(convDetail.value.id, replyContent.value.trim())
    replyContent.value = ''
    selectedTemplate.value = null
    ElMessage.success('已发送')
    await openConversation(convDetail.value)
    fetchConversations()
  } catch (e) {
    ElMessage.error(e.message || '发送失败')
  }
}

async function closeConversation(row) {
  try {
    await adminCloseConversation(row.id)
    ElMessage.success('会话已关闭')
    fetchConversations()
  } catch (e) {
    ElMessage.error(e.message || '操作失败')
  }
}

async function closeConversationFromDetail() {
  if (!convDetail.value) return
  try {
    await adminCloseConversation(convDetail.value.id)
    ElMessage.success('会话已关闭')
    convDialogVisible.value = false
    fetchConversations()
  } catch (e) {
    ElMessage.error(e.message || '操作失败')
  }
}

async function assignSelf() {
  if (!convDetail.value) return
  try {
    await adminAssignConversation(convDetail.value.id)
    ElMessage.success('已指派')
    await openConversation(convDetail.value)
  } catch (e) {
    ElMessage.error(e.message || '操作失败')
  }
}

function applyTemplate(tid) {
  const t = templates.value.find(t => t.id === tid)
  if (t) replyContent.value = t.content
}

function openTemplateDialog(row) {
  if (row) {
    Object.assign(templateForm, {
      id: row.id, name: row.name, category: row.category, content: row.content || '',
    })
  } else {
    Object.assign(templateForm, {
      id: null, name: '', category: 'general', content: '',
    })
  }
  templateDialogVisible.value = true
}

async function saveTemplate() {
  if (!templateForm.name.trim() || !templateForm.content.trim()) {
    ElMessage.warning('请填写名称和内容')
    return
  }
  saving.value = true
  try {
    if (templateForm.id) {
      await adminUpdateCsTemplate(templateForm.id, { ...templateForm })
    } else {
      await adminCreateCsTemplate({ ...templateForm })
    }
    ElMessage.success('已保存')
    templateDialogVisible.value = false
    fetchTemplates()
  } catch (e) {
    ElMessage.error(e.message || '保存失败')
  } finally {
    saving.value = false
  }
}

async function deleteTemplate(row) {
  try {
    await adminDeleteCsTemplate(row.id)
    ElMessage.success('已删除')
    fetchTemplates()
  } catch (e) {
    ElMessage.error(e.message || '删除失败')
  }
}

function templateCategoryLabel(c) {
  return { general: '通用', account: '账号', content: '内容', payment: '支付', technical: '技术' }[c] || c
}

function fmtTime(t) {
  if (!t) return ''
  const d = new Date(t)
  const pad = (n) => String(n).padStart(2, '0')
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}`
}

onMounted(() => fetchConversations())
</script>

<style scoped>
.cs-page { padding: 20px 24px; }
.cs-page__head { margin-bottom: 14px; }
.cs-page__title { margin: 0 0 4px; font-size: 18px; font-weight: 600; color: #18191c; }
.cs-page__desc { margin: 0; font-size: 13px; color: #9499a0; }
.cs-toolbar { margin-bottom: 14px; display: flex; gap: 10px; align-items: center; }
.cs-muted { color: #9499a0; }

.cs-chat { margin-top: 16px; }
.cs-chat__list { max-height: 320px; overflow-y: auto; display: flex; flex-direction: column; gap: 8px; padding: 8px 0; }
.cs-chat__empty { text-align: center; padding: 20px; }
.cs-chat__msg { display: flex; }
.cs-chat__msg--admin { justify-content: flex-end; }
.cs-chat__bubble { max-width: 75%; padding: 10px 14px; border-radius: 12px; background: #f6f7f8; }
.cs-chat__msg--admin .cs-chat__bubble { background: #e6f7ff; }
.cs-chat__meta { display: flex; gap: 8px; font-size: 11px; color: #9499a0; margin-bottom: 4px; }
.cs-chat__msg--admin .cs-chat__meta { justify-content: flex-end; }
.cs-chat__text { font-size: 14px; color: #18191c; line-height: 1.6; white-space: pre-wrap; word-break: break-word; }

.cs-reply { margin-top: 14px; }
.cs-reply__row { margin-bottom: 8px; }
.cs-reply__actions { display: flex; gap: 8px; }
</style>
