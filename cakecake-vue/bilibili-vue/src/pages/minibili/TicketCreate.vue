<template>
  <div class="tk-create-page">
    <div class="tk-create-card">
      <h2 class="tk-create-title">提交工单</h2>
      <p class="tk-create-hint">遇到问题？提交工单，客服将尽快处理</p>

      <el-form ref="formRef" :model="form" :rules="rules" label-width="80px" size="default">
        <el-form-item label="类型" prop="category">
          <el-select v-model="form.category" placeholder="选择工单类型" style="width: 100%">
            <el-option label="举报投诉" value="report">
              <span>📋 举报投诉</span>
              <span class="tk-opt-hint">举报违规内容或用户</span>
            </el-option>
            <el-option label="版权申诉" value="copyright">
              <span>©️ 版权申诉</span>
              <span class="tk-opt-hint">版权相关问题</span>
            </el-option>
            <el-option label="账号申诉" value="appeal">
              <span>🔐 账号申诉</span>
              <span class="tk-opt-hint">封禁申诉、账号问题</span>
            </el-option>
            <el-option label="其他问题" value="general">
              <span>💬 其他问题</span>
              <span class="tk-opt-hint">技术故障、功能建议等</span>
            </el-option>
          </el-select>
        </el-form-item>

        <el-form-item label="标题" prop="subject">
          <el-input v-model="form.subject" placeholder="简要描述您的问题（最多200字）" maxlength="200" show-word-limit />
        </el-form-item>

        <el-form-item label="详情" prop="description">
          <el-input
            v-model="form.description"
            type="textarea"
            :rows="6"
            placeholder="请详细描述您遇到的问题，包括时间、操作步骤、截图链接等"
            maxlength="5000"
            show-word-limit
          />
        </el-form-item>

        <el-form-item>
          <el-button type="primary" size="large" :loading="submitting" @click="submit" style="width: 100%">
            {{ submitting ? '提交中...' : '提交工单' }}
          </el-button>
        </el-form-item>
      </el-form>

      <!-- 已有工单 -->
      <el-divider />
      <div class="tk-my-list">
        <h3 class="tk-my-list__title">
          我的工单
          <el-button size="small" text type="primary" @click="fetchMine" :loading="listLoading">刷新</el-button>
        </h3>
        <el-table :data="mineList" stripe size="small" empty-text="暂无工单">
          <el-table-column prop="id" label="ID" width="60" />
          <el-table-column prop="subject" label="标题" min-width="180" show-overflow-tooltip />
          <el-table-column label="状态" width="90">
            <template #default="{ row }">
              <el-tag :type="statusTag(row.status)" size="small">{{ row.status }}</el-tag>
            </template>
          </el-table-column>
          <el-table-column label="时间" width="160">
            <template #default="{ row }">{{ row.created_at?.slice(0, 16).replace('T', ' ') }}</template>
          </el-table-column>
        </el-table>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, reactive } from 'vue'
import { ElMessage } from 'element-plus'
import http from '@/utils/http'

const formRef = ref(null)
const submitting = ref(false)
const listLoading = ref(false)
const mineList = ref([])

const form = reactive({
  category: '',
  subject: '',
  description: ''
})

const rules = {
  category: [{ required: true, message: '请选择工单类型', trigger: 'change' }],
  subject: [{ required: true, message: '请输入标题', trigger: 'blur' }],
  description: [{ required: true, message: '请输入详情', trigger: 'blur' }]
}

function statusTag(s) {
  const m = { open: 'danger', assigned: 'warning', processing: '', resolved: 'success', closed: 'info' }
  return m[s] || ''
}

async function submit() {
  const valid = await formRef.value.validate().catch(() => false)
  if (!valid) return

  submitting.value = true
  try {
    const res = await http.post('/api/v1/tickets', {
      category: form.category,
      subject: form.subject.trim(),
      description: form.description.trim()
    })
    if (res.code !== 0) throw new Error(res.msg || '提交失败')
    ElMessage.success('工单已提交，客服将尽快处理（工单ID: ' + res.data.id + '）')
    form.subject = ''
    form.description = ''
    form.category = ''
    fetchMine()
  } catch (e) {
    ElMessage.error(e.message || '提交失败')
  } finally {
    submitting.value = false
  }
}

async function fetchMine() {
  listLoading.value = true
  try {
    const res = await http.get('/api/v1/users/me/tickets')
    if (res.code !== 0) throw new Error(res.msg)
    mineList.value = res.data.items || []
  } catch {
    mineList.value = []
  } finally {
    listLoading.value = false
  }
}

fetchMine()
</script>

<style scoped>
.tk-create-page {
  max-width: 640px;
  margin: 0 auto;
  padding: 40px 16px 80px;
}

.tk-create-card {
  background: #fff;
  border-radius: 12px;
  padding: 32px;
  box-shadow: 0 1px 3px rgba(0,0,0,0.06);
}

.tk-create-title {
  font-size: 22px;
  font-weight: 600;
  color: #222;
  margin: 0 0 8px;
}

.tk-create-hint {
  font-size: 13px;
  color: #999;
  margin: 0 0 28px;
}

.tk-opt-hint {
  display: block;
  font-size: 12px;
  color: #999;
}

.tk-my-list {
  margin-top: 8px;
}

.tk-my-list__title {
  font-size: 15px;
  font-weight: 600;
  color: #333;
  margin: 0 0 12px;
  display: flex;
  align-items: center;
  justify-content: space-between;
}
</style>
