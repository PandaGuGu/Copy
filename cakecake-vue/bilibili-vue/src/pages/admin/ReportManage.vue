<template>
  <div class="rp-page" v-loading="loading">
    <header class="rp-page__head">
      <h2 class="rp-page__title">举报处理</h2>
      <p class="rp-page__desc">处理用户提交的内容举报</p>
    </header>

    <!-- 统计卡片 -->
    <div class="rp-stats" v-if="stats">
      <div class="rp-stat rp-stat--warn" @click="filterStatus='pending';search()">
        <span class="rp-stat__val">{{ stats.pending_count || 0 }}</span>
        <span class="rp-stat__label">待处理</span>
      </div>
      <div class="rp-stat rp-stat--ok" @click="filterStatus='resolved';search()">
        <span class="rp-stat__val">{{ stats.resolved_count || 0 }}</span>
        <span class="rp-stat__label">已处理</span>
      </div>
      <div class="rp-stat rp-stat--dim" @click="filterStatus='dismissed';search()">
        <span class="rp-stat__val">{{ stats.dismissed_count || 0 }}</span>
        <span class="rp-stat__label">已驳回</span>
      </div>
    </div>

    <!-- 分类标签 -->
    <div class="rp-reason-tags" v-if="reasonTypes.length > 0">
      <span class="rp-reason-tag" v-for="(rt, i) in reasonTypes" :key="i">
        {{ rt.icon }} {{ rt.label }}
        <strong>{{ rt.count }}</strong>
      </span>
    </div>

    <!-- 筛选栏 -->
    <div class="rp-toolbar">
      <el-select v-model="filterStatus" placeholder="状态" clearable size="default" style="width:110px" @change="search">
        <el-option label="全部" value="" />
        <el-option label="待处理" value="pending" />
        <el-option label="已处理" value="resolved" />
        <el-option label="已驳回" value="dismissed" />
      </el-select>
      <el-select v-model="filterTarget" placeholder="目标类型" clearable size="default" style="width:110px" @change="search">
        <el-option label="全部" value="" />
        <el-option label="视频" value="video" />
        <el-option label="文章" value="article" />
        <el-option label="动态" value="dynamic" />
        <el-option label="评论" value="comment" />
        <el-option label="用户" value="user" />
      </el-select>
      <el-select v-model="filterReason" placeholder="举报类型" clearable size="default" style="width:120px" @change="search">
        <el-option label="全部" value="" />
        <el-option v-for="rt in reasonTypes" :key="rt.type" :label="rt.label" :value="rt.type" />
      </el-select>
      <el-button v-if="selectedIds.length > 0" type="primary" size="default" @click="batchResolve">
        批量处理 ({{ selectedIds.length }})
      </el-button>
      <el-button v-if="selectedIds.length > 0" type="warning" size="default" @click="batchDismiss">
        批量驳回
      </el-button>
    </div>

    <el-table
      :data="items"
      stripe
      size="default"
      empty-text="暂无举报"
      @selection-change="onSelectionChange"
    >
      <el-table-column type="selection" width="45" />
      <el-table-column prop="id" label="ID" width="65" />
      <el-table-column label="举报人" width="110">
        <template #default="{ row }">
          <span v-if="row.reporter">{{ row.reporter.nickname || row.reporter.username }}</span>
          <span v-else class="rp-muted">#{{ row.reporter_id }}</span>
        </template>
      </el-table-column>
      <el-table-column label="目标" width="70">
        <template #default="{ row }">
          <el-tag size="small" effect="plain">{{ typeLabel(row.target_type) }}</el-tag>
        </template>
      </el-table-column>
      <el-table-column label="分类" width="90">
        <template #default="{ row }">
          <el-tag size="small" effect="dark" :color="reasonColor(row.reason_type)">
            {{ row.reason_label || row.reason_type }}
          </el-tag>
        </template>
      </el-table-column>
      <el-table-column label="原因详情" min-width="180" show-overflow-tooltip>
        <template #default="{ row }">
          <span v-if="row.reason_detail">{{ row.reason_detail }}</span>
          <span v-else class="rp-muted">—</span>
        </template>
      </el-table-column>
      <el-table-column label="状态" width="80">
        <template #default="{ row }">
          <el-tag :type="statusTag(row.status)" size="small" effect="plain">{{ statusLabel(row.status) }}</el-tag>
        </template>
      </el-table-column>
      <el-table-column label="时间" width="150">
        <template #default="{ row }">{{ fmtTime(row.created_at) }}</template>
      </el-table-column>
      <el-table-column label="操作" width="210" fixed="right">
        <template #default="{ row }">
          <template v-if="row.status === 'pending'">
            <el-button size="small" text type="primary" @click="openHandle(row)">处理</el-button>
            <el-popconfirm title="确认驳回？" @confirm="doHandle(row, 'dismiss', 'none')">
              <template #reference>
                <el-button size="small" text type="warning">驳回</el-button>
              </template>
            </el-popconfirm>
            <el-popconfirm title="确认删除此记录？不可恢复" @confirm="doDelete(row.id)">
              <template #reference>
                <el-button size="small" text type="danger">删除</el-button>
              </template>
            </el-popconfirm>
          </template>
          <template v-else>
            <span class="rp-muted" style="margin-right:4px">{{ row.handler_note || '已完成' }}</span>
            <el-button size="small" text type="warning" @click="doHandle(row, 'revert', 'none')">撤回</el-button>
            <el-popconfirm title="确认删除此记录？不可恢复" @confirm="doDelete(row.id)">
              <template #reference>
                <el-button size="small" text type="danger">删除</el-button>
              </template>
            </el-popconfirm>
          </template>
        </template>
      </el-table-column>
    </el-table>

    <div class="rp-pager" v-if="total > pageSize">
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
      title="处理举报"
      width="480px"
      @update:model-value="handleVisible = $event"
    >
      <div v-if="handleTarget">
        <div class="rph-info">
          <p><b>目标：</b>{{ typeLabel(handleTarget.target_type) }} #{{ handleTarget.target_id }}</p>
          <p><b>分类：</b>{{ handleTarget.reason_label || handleTarget.reason_type }}</p>
          <p v-if="handleTarget.reason_detail"><b>详情：</b>{{ handleTarget.reason_detail }}</p>
        </div>

        <el-divider />

        <div class="rph-actions">
          <label class="rph-label">对目标内容的处理：</label>
          <div class="rph-radio-list">
            <div
              v-for="act in contentActions"
              :key="act.value"
              class="rph-radio-item"
              :class="{ 'is-active': handleContentAction === act.value }"
              @click="handleContentAction = act.value"
            >
              <span class="rph-radio-dot" :class="{ 'is-on': handleContentAction === act.value }" />
              <div>
                <div class="rph-radio-title">{{ act.label }}</div>
                <div class="rph-radio-desc">{{ act.desc }}</div>
              </div>
            </div>
          </div>
        </div>

        <div class="rph-note" style="margin-top:14px">
          <label class="rph-label">处理备注（选填）：</label>
          <el-input v-model="handleNote" placeholder="可填写处理说明..." size="default" />
        </div>
      </div>

      <template #footer>
        <el-button @click="handleVisible = false">取消</el-button>
        <el-button
          type="primary"
          :loading="handling"
          :disabled="!handleContentAction"
          @click="confirmHandle"
        >
          {{ handleContentAction === 'ban' ? '确认封禁' : handleContentAction === 'takedown' ? '确认下架' : handleContentAction === 'warn' ? '确认警告' : '确认处理' }}
        </el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script>
import { ElMessage, ElMessageBox } from "element-plus";
import { adminListReports, adminHandleReport, adminDeleteReport } from "@/api/admin";
import adminHttp from "@/utils/adminHttp";

const reasonIcons = {
  nsfw: "🔞", violence: "🩸", spam: "📢", harassment: "🔥", illegal: "⚖️", copyright: "©️", other: "📌"
};
const reasonColors = {
  nsfw: "#e6a23c", violence: "#f56c6c", spam: "#909399", harassment: "#e6a23c", illegal: "#f56c6c", copyright: "#409eff", other: "#909399"
};

export default {
  name: "ReportManage",
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
      selectedIds: [],
      reasonTypes: [],
      // Handle dialog
      handleVisible: false,
      handleTarget: null,
      handleContentAction: "none",
      handleNote: "",
      handling: false,
      contentActions: [
        { value: "none", label: "仅标记已处理", desc: "不操作内容，仅标记举报为已处理" },
        { value: "takedown", label: "下架/删除内容", desc: "下架视频/文章，删除动态/评论" },
        { value: "warn", label: "警告发布者", desc: "标记作者账号为警告状态" },
        { value: "ban", label: "封禁发布者", desc: "封禁作者账号，拒绝登录" },
      ],
    };
  },
  created() {
    this.fetch();
  },
  methods: {
    async fetch() {
      this.loading = true;
      this.selectedIds = [];
      try {
        const body = await adminListReports({
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
          resolved_count: d.resolved_count || 0,
          dismissed_count: d.dismissed_count || 0,
        };
        this.reasonTypes = (d.reason_stats || []).map(r => ({
          ...r,
          icon: reasonIcons[r.type] || "",
        }));
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
    onSelectionChange(rows) {
      this.selectedIds = rows.filter(r => r.status === "pending").map(r => r.id);
    },
    async batchResolve() {
      try {
        await ElMessageBox.confirm(`确认批量处理 ${this.selectedIds.length} 条举报？`);
      } catch { return; }
      try {
        await adminHttp.post("/api/v1/admin/reports/batch", { ids: this.selectedIds, action: "resolve", handler_note: "" });
        ElMessage.success("已批量处理");
        this.fetch();
      } catch (e) {
        ElMessage.error((e && e.message) || "操作失败");
      }
    },
    async batchDismiss() {
      try {
        await ElMessageBox.confirm(`确认批量驳回 ${this.selectedIds.length} 条举报？`);
      } catch { return; }
      try {
        await adminHttp.post("/api/v1/admin/reports/batch", { ids: this.selectedIds, action: "dismiss", handler_note: "" });
        ElMessage.success("已批量驳回");
        this.fetch();
      } catch (e) {
        ElMessage.error((e && e.message) || "操作失败");
      }
    },
    async doHandle(row, action, contentAction = "none") {
      try {
        await adminHandleReport(row.id, {
          action,
          content_action: contentAction,
          handler_note: "",
        });
        ElMessage.success(action === "resolve" ? "已处理" : "已驳回");
        this.fetch();
      } catch (e) {
        ElMessage.error((e && e.message) || "操作失败");
      }
    },
    openHandle(row) {
      this.handleTarget = row;
      this.handleContentAction = "none";
      this.handleNote = "";
      this.handleVisible = true;
    },
    async confirmHandle() {
      if (!this.handleTarget) return;
      this.handling = true;
      try {
        await adminHandleReport(this.handleTarget.id, {
          action: "resolve",
          content_action: this.handleContentAction,
          handler_note: this.handleNote.trim(),
        });
        const actionLabel = { none: "已标记处理", takedown: "已处理并下架", warn: "已处理并警告", ban: "已处理并封禁" };
        ElMessage.success(actionLabel[this.handleContentAction] || "已处理");
        this.handleVisible = false;
        this.fetch();
      } catch (e) {
        ElMessage.error((e && e.message) || "操作失败");
      } finally {
        this.handling = false;
      }
    },
    async doDelete(id) {
      try {
        await adminDeleteReport(id);
        ElMessage.success("已删除");
        this.fetch();
      } catch (e) {
        ElMessage.error((e && e.message) || "删除失败");
      }
    },
    typeLabel(t) {
      return { video: "视频", article: "文章", dynamic: "动态", comment: "评论", user: "用户" }[t] || t;
    },
    statusLabel(s) {
      return { pending: "待处理", resolved: "已处理", dismissed: "已驳回" }[s] || s;
    },
    statusTag(s) {
      return { pending: "warning", resolved: "success", dismissed: "info" }[s] || "";
    },
    reasonColor(t) {
      return reasonColors[t] || "#909399";
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
.rp-page { padding: 20px 24px; }
.rp-page__head { margin-bottom: 14px; }
.rp-page__title { margin: 0 0 4px; font-size: 18px; font-weight: 600; color: #18191c; }
.rp-page__desc { margin: 0; font-size: 13px; color: #9499a0; }

.rp-stats { display: flex; gap: 10px; margin-bottom: 14px; }
.rp-stat { flex: 1; max-width: 140px; padding: 12px 16px; border-radius: 8px; background: #fff; border: 1px solid #e3e5e7; cursor: pointer; transition: all .15s; }
.rp-stat:hover { transform: translateY(-1px); box-shadow: 0 2px 8px rgba(0,0,0,.06); }
.rp-stat--warn { border-color: #ffe0b0; background: #fffaf3; }
.rp-stat--ok { border-color: #d4edda; background: #f0faf3; }
.rp-stat--dim { border-color: #e3e5e7; background: #fafafa; }
.rp-stat__val { font-size: 24px; font-weight: 700; color: #18191c; }
.rp-stat--warn .rp-stat__val { color: #e6a23c; }
.rp-stat--ok .rp-stat__val { color: #67c23a; }
.rp-stat__label { font-size: 12px; color: #9499a0; }

.rp-reason-tags { display: flex; flex-wrap: wrap; gap: 6px; margin-bottom: 12px; }
.rp-reason-tag { font-size: 12px; padding: 3px 10px; border-radius: 12px; background: #f6f7f8; color: #61666d; }
.rp-reason-tag strong { margin-left: 2px; color: #18191c; }

.rp-toolbar { margin-bottom: 14px; display: flex; gap: 10px; align-items: center; flex-wrap: wrap; }
.rp-pager { margin-top: 16px; display: flex; justify-content: flex-end; }
.rp-muted { color: #9499a0; }

/* Handle dialog */
.rph-info { font-size: 13px; color: #61666d; line-height: 1.8; }
.rph-info b { color: #18191c; }
.rph-label { font-size: 13px; font-weight: 600; color: #18191c; display: block; margin-bottom: 8px; }

.rph-radio-list { display: flex; flex-direction: column; gap: 6px; }
.rph-radio-item {
  display: flex; align-items: flex-start; gap: 12px;
  padding: 12px 14px;
  border: 2px solid #e3e5e7; border-radius: 8px;
  cursor: pointer; transition: all .15s;
}
.rph-radio-item:hover { border-color: #00a1d6; background: #f0fafe; }
.rph-radio-item.is-active { border-color: #00a1d6; background: #e6f7ff; }
.rph-radio-dot {
  width: 18px; height: 18px; border-radius: 50%;
  border: 2px solid #c0c4cc; flex-shrink: 0; margin-top: 2px;
  transition: all .15s;
}
.rph-radio-dot.is-on { border-color: #00a1d6; background: #00a1d6; box-shadow: inset 0 0 0 3px #fff; }
.rph-radio-title { font-size: 14px; font-weight: 600; color: #18191c; }
.rph-radio-desc { font-size: 12px; color: #9499a0; margin-top: 2px; }
.rph-radio-item.is-active .rph-radio-title { color: #00a1d6; }
</style>
