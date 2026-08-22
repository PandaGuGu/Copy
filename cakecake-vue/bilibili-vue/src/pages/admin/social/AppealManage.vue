<template>
  <div class="ap-page" v-loading="loading">
    <header class="ap-page__head">
      <h2 class="ap-page__title">治理申诉</h2>
      <p class="ap-page__desc">处理用户对账号封禁 / 内容下架 / 警告的申诉</p>
    </header>

    <!-- 统计卡片 -->
    <div class="ap-stats" v-if="stats">
      <div class="ap-stat ap-stat--warn" @click="filterStatus='pending';search()">
        <span class="ap-stat__val">{{ stats.pending_count || 0 }}</span>
        <span class="ap-stat__label">待处理</span>
      </div>
      <div class="ap-stat ap-stat--ok" @click="filterStatus='approved';search()">
        <span class="ap-stat__val">{{ stats.approved_count || 0 }}</span>
        <span class="ap-stat__label">已通过</span>
      </div>
      <div class="ap-stat ap-stat--dim" @click="filterStatus='rejected';search()">
        <span class="ap-stat__val">{{ stats.rejected_count || 0 }}</span>
        <span class="ap-stat__label">已驳回</span>
      </div>
    </div>

    <!-- 筛选栏 -->
    <div class="ap-toolbar">
      <el-select v-model="filterStatus" placeholder="状态" clearable size="default" style="width:110px" @change="search">
        <el-option label="全部" value="" />
        <el-option label="待处理" value="pending" />
        <el-option label="已通过" value="approved" />
        <el-option label="已驳回" value="rejected" />
      </el-select>
      <el-select v-model="filterTarget" placeholder="目标类型" clearable size="default" style="width:110px" @change="search">
        <el-option label="全部" value="" />
        <el-option label="用户" value="user" />
        <el-option label="视频" value="video" />
        <el-option label="文章" value="article" />
        <el-option label="动态" value="dynamic" />
        <el-option label="评论" value="comment" />
      </el-select>
      <el-select v-model="filterReason" placeholder="申诉类型" clearable size="default" style="width:120px" @change="search">
        <el-option label="全部" value="" />
        <el-option label="账号封禁" value="ban" />
        <el-option label="内容下架" value="takedown" />
        <el-option label="警告" value="warn" />
      </el-select>
    </div>

    <el-table :data="items" stripe size="default" empty-text="暂无申诉">
      <el-table-column prop="id" label="ID" width="65" />
      <el-table-column label="申诉人" width="120">
        <template #default="{ row }">
          <span v-if="row.applicant">{{ row.applicant.nickname || row.applicant.username }}</span>
          <span v-else class="ap-muted">#{{ row.user_id }}</span>
        </template>
      </el-table-column>
      <el-table-column label="目标" width="90">
        <template #default="{ row }">
          <el-tag size="small" effect="plain">{{ typeLabel(row.target_type) }}</el-tag>
          <span class="ap-muted">#{{ row.target_id }}</span>
        </template>
      </el-table-column>
      <el-table-column label="申诉类型" width="100">
        <template #default="{ row }">
          <el-tag size="small" effect="dark" :color="reasonColor(row.reason_type)">
            {{ reasonLabel(row.reason_type) }}
          </el-tag>
        </template>
      </el-table-column>
      <el-table-column label="申诉内容" min-width="180" show-overflow-tooltip>
        <template #default="{ row }">{{ row.content || '—' }}</template>
      </el-table-column>
      <el-table-column label="处理备注" min-width="140" show-overflow-tooltip>
        <template #default="{ row }">
          <span v-if="row.admin_note">{{ row.admin_note }}</span>
          <span v-else class="ap-muted">—</span>
        </template>
      </el-table-column>
      <el-table-column label="状态" width="86">
        <template #default="{ row }">
          <el-tag :type="statusTag(row.status)" size="small" effect="plain">{{ statusLabel(row.status) }}</el-tag>
        </template>
      </el-table-column>
      <el-table-column label="时间" width="150">
        <template #default="{ row }">{{ fmtTime(row.created_at) }}</template>
      </el-table-column>
      <el-table-column label="操作" width="80" fixed="right">
        <template #default="{ row }">
          <el-button v-if="row.status === 'pending'" size="small" plain type="primary" @click="openHandle(row)">处理</el-button>
        </template>
      </el-table-column>
    </el-table>

    <div class="ap-pager" v-if="total > pageSize">
      <el-pagination
        v-model:current-page="page"
        :page-size="pageSize"
        :total="total"
        layout="prev, pager, next, total"
        @current-change="fetch"
      />
    </div>

    <!-- 处理弹窗 -->
    <el-dialog
      :model-value="handleVisible"
      title="处理申诉"
      width="480px"
      @update:model-value="handleVisible = $event"
    >
      <div v-if="handleTarget">
        <div class="aph-info">
          <p><b>目标：</b>{{ typeLabel(handleTarget.target_type) }} #{{ handleTarget.target_id }}</p>
          <p><b>申诉类型：</b>{{ reasonLabel(handleTarget.reason_type) }}</p>
          <p><b>申诉内容：</b>{{ handleTarget.content }}</p>
          <p v-if="handleTarget.evidence_urls"><b>证据：</b>{{ handleTarget.evidence_urls }}</p>
        </div>

        <el-divider />

        <div class="aph-actions">
          <el-radio-group v-model="handleAction">
            <el-radio value="approve">通过并恢复</el-radio>
            <el-radio value="reject">驳回，维持处罚</el-radio>
          </el-radio-group>
          <p v-if="handleAction === 'approve'" class="aph-hint">
            通过后将自动解封账号 / 恢复被下架的内容。
          </p>
        </div>

        <div class="aph-note" style="margin-top:14px">
          <label class="aph-label">
            处理备注{{ handleAction === 'reject' ? '（驳回必填）' : '（选填）' }}：
          </label>
          <el-input v-model="handleNote" placeholder="填写处理说明..." size="default" />
        </div>
      </div>

      <template #footer>
        <el-button @click="handleVisible = false">取消</el-button>
        <el-button
          type="primary"
          :loading="handling"
          :disabled="!handleAction || (handleAction === 'reject' && !handleNote.trim())"
          @click="confirmHandle"
        >
          {{ handleAction === 'approve' ? '通过并恢复' : '确认驳回' }}
        </el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script>
import { ElMessage } from "element-plus";
import { adminListAppeals, adminHandleAppeal } from "@/api/admin";

export default {
  name: "AppealManage",
  data() {
    return {
      loading: false,
      items: [],
      total: 0,
      page: 1,
      pageSize: 20,
      filterStatus: "",
      filterTarget: "",
      filterReason: "",
      stats: null,
      // Handle dialog
      handleVisible: false,
      handleTarget: null,
      handleAction: "approve",
      handleNote: "",
      handling: false,
    };
  },
  created() {
    this.fetch();
  },
  methods: {
    async fetch() {
      this.loading = true;
      try {
        const body = await adminListAppeals({
          page: this.page,
          page_size: this.pageSize,
          status: this.filterStatus || undefined,
          target: this.filterTarget || undefined,
          reason_type: this.filterReason || undefined,
        });
        const d = (body && body.data) || {};
        this.items = d.items || [];
        this.total = d.total || 0;
        this.stats = {
          pending_count: d.pending_count || 0,
          approved_count: d.approved_count || 0,
          rejected_count: d.rejected_count || 0,
        };
      } catch (e) {
        ElMessage.error((e && e.message) || "加载失败");
      } finally {
        this.loading = false;
      }
    },
    search() {
      this.page = 1;
      this.fetch();
    },
    openHandle(row) {
      this.handleTarget = row;
      this.handleAction = "approve";
      this.handleNote = "";
      this.handleVisible = true;
    },
    async confirmHandle() {
      if (!this.handleTarget) return;
      this.handling = true;
      try {
        await adminHandleAppeal(this.handleTarget.id, {
          action: this.handleAction,
          admin_note: this.handleNote.trim(),
        });
        ElMessage.success(this.handleAction === "approve" ? "已通过并恢复" : "已驳回");
        this.handleVisible = false;
        this.fetch();
      } catch (e) {
        ElMessage.error((e && e.message) || "操作失败");
      } finally {
        this.handling = false;
      }
    },
    typeLabel(t) {
      return { user: "用户", video: "视频", article: "文章", dynamic: "动态", comment: "评论" }[t] || t;
    },
    reasonLabel(t) {
      return { ban: "账号封禁", takedown: "内容下架", warn: "警告" }[t] || t;
    },
    reasonColor(t) {
      return { ban: "#f56c6c", takedown: "#e6a23c", warn: "#909399" }[t] || "#909399";
    },
    statusLabel(s) {
      return { pending: "待处理", approved: "已通过", rejected: "已驳回" }[s] || s;
    },
    statusTag(s) {
      return { pending: "warning", approved: "success", rejected: "info" }[s] || "";
    },
    fmtTime(t) {
      if (!t) return "";
      const d = new Date(t);
      const pad = (n) => String(n).padStart(2, "0");
      return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}`;
    },
  },
};
</script>

<style scoped>
.ap-page { padding: 20px 24px; }
.ap-page__head { margin-bottom: 14px; }
.ap-page__title { margin: 0 0 4px; font-size: 18px; font-weight: 600; color: #18191c; }
.ap-page__desc { margin: 0; font-size: 13px; color: #9499a0; }

.ap-stats { display: flex; gap: 10px; margin-bottom: 14px; }
.ap-stat { flex: 1; max-width: 140px; padding: 12px 16px; border-radius: 8px; background: #fff; border: 1px solid #e3e5e7; cursor: pointer; transition: all .15s; }
.ap-stat:hover { transform: translateY(-1px); box-shadow: 0 2px 8px rgba(0,0,0,.06); }
.ap-stat--warn { border-color: #ffe0b0; background: #fffaf3; }
.ap-stat--ok { border-color: #d4edda; background: #f0faf3; }
.ap-stat--dim { border-color: #e3e5e7; background: #fafafa; }
.ap-stat__val { font-size: 24px; font-weight: 700; color: #18191c; }
.ap-stat--warn .ap-stat__val { color: #e6a23c; }
.ap-stat--ok .ap-stat__val { color: #67c23a; }
.ap-stat__label { font-size: 12px; color: #9499a0; }

.ap-toolbar { margin-bottom: 14px; display: flex; gap: 10px; align-items: center; flex-wrap: wrap; }
.ap-muted { color: #9499a0; }
.ap-pager { margin-top: 16px; display: flex; justify-content: flex-end; }

.aph-info { font-size: 13px; color: #61666d; line-height: 1.8; }
.aph-info b { color: #18191c; }
.aph-label { font-size: 13px; font-weight: 600; color: #18191c; display: block; margin-bottom: 8px; }
.aph-actions .el-radio-group { display: flex; flex-direction: column; align-items: flex-start; gap: 8px; }
.aph-hint { margin: 8px 0 0; font-size: 12px; color: #909399; }
</style>