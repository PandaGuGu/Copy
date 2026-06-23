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
      <el-table-column label="操作" width="150" fixed="right">
        <template #default="{ row }">
          <template v-if="row.status === 'pending'">
            <el-popconfirm title="确认处理？" @confirm="doHandle(row, 'resolve')">
              <template #reference>
                <el-button size="small" text type="primary">处理</el-button>
              </template>
            </el-popconfirm>
            <el-popconfirm title="确认驳回？" @confirm="doHandle(row, 'dismiss')">
              <template #reference>
                <el-button size="small" text type="warning">驳回</el-button>
              </template>
            </el-popconfirm>
          </template>
          <span v-else class="rp-muted">已完成</span>
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
  </div>
</template>

<script>
import { ElMessage, ElMessageBox } from "element-plus";
import { adminListReports, adminHandleReport } from "@/api/admin";
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
    async doHandle(row, action) {
      try {
        await adminHandleReport(row.id, { action, handler_note: "" });
        ElMessage.success(action === "resolve" ? "已处理" : "已驳回");
        this.fetch();
      } catch (e) {
        ElMessage.error((e && e.message) || "操作失败");
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
</style>
