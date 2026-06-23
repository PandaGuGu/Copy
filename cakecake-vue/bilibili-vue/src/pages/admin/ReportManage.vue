<template>
  <div class="rp-page" v-loading="loading">
    <header class="rp-page__head">
      <h2 class="rp-page__title">举报处理</h2>
      <p class="rp-page__desc">处理用户提交的内容举报</p>
    </header>

    <!-- 筛选栏 -->
    <div class="rp-toolbar">
      <el-select v-model="filterStatus" placeholder="状态筛选" clearable size="default" style="width:130px" @change="search">
        <el-option label="全部" value="" />
        <el-option label="待处理" value="pending" />
        <el-option label="已处理" value="resolved" />
        <el-option label="已驳回" value="dismissed" />
      </el-select>
      <el-button type="primary" size="default" @click="search">刷新</el-button>
    </div>

    <el-table :data="items" stripe size="default" empty-text="暂无举报">
      <el-table-column prop="id" label="ID" width="70" />
      <el-table-column label="举报人" width="120">
        <template #default="{ row }">
          <span v-if="row.reporter">{{ row.reporter.nickname || row.reporter.username }}</span>
          <span v-else class="rp-muted">#{{ row.reporter_id }}</span>
        </template>
      </el-table-column>
      <el-table-column label="类型" width="80">
        <template #default="{ row }">
          <el-tag size="small" effect="plain">{{ typeLabel(row.target_type) }}</el-tag>
        </template>
      </el-table-column>
      <el-table-column label="目标 ID" width="80">
        <template #default="{ row }">{{ row.target_id }}</template>
      </el-table-column>
      <el-table-column label="举报原因" min-width="200" show-overflow-tooltip>
        <template #default="{ row }">{{ row.reason }}</template>
      </el-table-column>
      <el-table-column label="状态" width="90">
        <template #default="{ row }">
          <el-tag :type="statusTag(row.status)" size="small" effect="plain">{{ statusLabel(row.status) }}</el-tag>
        </template>
      </el-table-column>
      <el-table-column label="处理备注" min-width="140" show-overflow-tooltip>
        <template #default="{ row }">{{ row.handler_note || "—" }}</template>
      </el-table-column>
      <el-table-column label="时间" width="160">
        <template #default="{ row }">{{ fmtTime(row.created_at) }}</template>
      </el-table-column>
      <el-table-column label="操作" width="180" fixed="right">
        <template #default="{ row }">
          <template v-if="row.status === 'pending'">
            <el-popconfirm title="确认处理该举报？" @confirm="doHandle(row, 'resolve')">
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
import { ElMessage } from "element-plus";
import { adminListReports, adminHandleReport } from "@/api/admin";

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
    };
  },
  created() {
    this.fetch();
  },
  methods: {
    async fetch() {
      this.loading = true;
      try {
        const body = await adminListReports({
          page: this.page,
          page_size: this.pageSize,
          status: this.filterStatus || undefined,
        });
        const d = (body && body.data) || {};
        this.items = d.items || [];
        this.total = d.total || 0;
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
.rp-page__head { margin-bottom: 16px; }
.rp-page__title { margin: 0 0 4px; font-size: 18px; font-weight: 600; color: #18191c; }
.rp-page__desc { margin: 0; font-size: 13px; color: #9499a0; }
.rp-toolbar { margin-bottom: 14px; display: flex; gap: 10px; align-items: center; }
.rp-pager { margin-top: 16px; display: flex; justify-content: flex-end; }
.rp-muted { color: #9499a0; }
</style>
