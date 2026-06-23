<template>
  <div class="cm-page" v-loading="loading">
    <header class="cm-page__head">
      <h2 class="cm-page__title">评论管理</h2>
      <p class="cm-page__desc">全局查看和管理所有评论内容</p>
    </header>

    <!-- 筛选栏 -->
    <div class="cm-toolbar">
      <div class="cm-toolbar__filters">
        <el-select v-model="filterType" placeholder="评论类型" clearable size="default" style="width:130px" @change="search">
          <el-option label="全部类型" value="" />
          <el-option label="视频评论" value="video" />
          <el-option label="文章评论" value="article" />
          <el-option label="动态评论" value="dynamic" />
        </el-select>
        <el-input
          v-model="filterQ"
          placeholder="搜索评论内容..."
          clearable
          size="default"
          style="width:240px"
          @keyup.enter="search"
          @clear="search"
        >
          <template #prefix>
            <el-icon><Search /></el-icon>
          </template>
        </el-input>
        <el-button type="primary" size="default" @click="search">搜索</el-button>
      </div>
    </div>

    <!-- 表格 -->
    <el-table :data="items" stripe size="default" style="width:100%" empty-text="暂无评论数据">
      <el-table-column prop="id" label="ID" width="70" />
      <el-table-column label="类型" width="80">
        <template #default="{ row }">
          <el-tag :type="typeTag(row.type)" size="small" effect="plain">{{ typeLabel(row.type) }}</el-tag>
        </template>
      </el-table-column>
      <el-table-column label="内容" min-width="200" show-overflow-tooltip>
        <template #default="{ row }">{{ row.content }}</template>
      </el-table-column>
      <el-table-column label="作者" width="130">
        <template #default="{ row }">
          <span v-if="row.author">{{ row.author.nickname || row.author.username }}</span>
          <span v-else class="cm-muted">—</span>
        </template>
      </el-table-column>
      <el-table-column label="所属作品" width="150">
        <template #default="{ row }">
          <span v-if="row.target && row.target.title" class="cm-target">{{ row.target.title }}</span>
          <span v-else class="cm-muted">{{ targetFallback(row) }}</span>
        </template>
      </el-table-column>
      <el-table-column label="状态" width="90">
        <template #default="{ row }">
          <el-tag :type="statusTag(row.status)" size="small" effect="plain">{{ statusLabel(row.status) }}</el-tag>
        </template>
      </el-table-column>
      <el-table-column label="时间" width="160">
        <template #default="{ row }">{{ fmtTime(row.created_at) }}</template>
      </el-table-column>
      <el-table-column label="操作" width="120" fixed="right">
        <template #default="{ row }">
          <el-button size="small" text type="primary" @click="openDetail(row)">详情</el-button>
          <el-popconfirm title="确定删除该评论？不可恢复" @confirm="doDelete(row)">
            <template #reference>
              <el-button size="small" text type="danger">删除</el-button>
            </template>
          </el-popconfirm>
        </template>
      </el-table-column>
    </el-table>

    <!-- 分页 -->
    <div class="cm-pager" v-if="total > pageSize">
      <el-pagination
        v-model:current-page="page"
        :page-size="pageSize"
        :total="total"
        layout="prev, pager, next, total"
        @current-change="fetch"
      />
    </div>

    <!-- 详情弹窗 -->
    <el-dialog v-model="detailVisible" title="评论详情" width="520px" destroy-on-close>
      <template v-if="detail">
        <el-descriptions :column="1" border size="small">
          <el-descriptions-item label="ID">{{ detail.id }}</el-descriptions-item>
          <el-descriptions-item label="类型">{{ typeLabel(detail.type) }}</el-descriptions-item>
          <el-descriptions-item label="内容">
            <div class="cm-detail-content">{{ detail.content }}</div>
          </el-descriptions-item>
          <el-descriptions-item label="作者">
            <template v-if="detail.author">
              <span>{{ detail.author.nickname || detail.author.username }}</span>
              <span class="cm-muted" style="margin-left:6px">@{{ detail.author.cake_id || detail.author.username }}</span>
              <el-tag size="small" style="margin-left:8px" :type="detail.author.status === 'banned' ? 'danger' : 'info'">
                {{ detail.author.status === 'banned' ? '已封禁' : '正常' }}
              </el-tag>
            </template>
          </el-descriptions-item>
          <el-descriptions-item label="状态">{{ statusLabel(detail.status) }}</el-descriptions-item>
          <el-descriptions-item label="点赞">{{ detail.like_count }}</el-descriptions-item>
          <el-descriptions-item label="时间">{{ fmtTime(detail.created_at) }}</el-descriptions-item>
        </el-descriptions>
      </template>
    </el-dialog>
  </div>
</template>

<script>
import { ElMessage } from "element-plus";
import { Search } from "@element-plus/icons-vue";
import { adminListComments, adminGetComment, adminDeleteComment } from "@/api/admin";

export default {
  name: "CommentManage",
  components: { Search },
  data() {
    return {
      loading: false,
      items: [],
      total: 0,
      page: 1,
      pageSize: 20,
      filterType: "",
      filterQ: "",
      detail: null,
      detailVisible: false,
    };
  },
  created() {
    this.fetch();
  },
  methods: {
    async fetch() {
      this.loading = true;
      try {
        const body = await adminListComments({
          page: this.page,
          page_size: this.pageSize,
          type: this.filterType || undefined,
          q: this.filterQ || undefined,
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
    async openDetail(row) {
      try {
        const body = await adminGetComment(row.id, row.type);
        this.detail = (body && body.data) || null;
        this.detailVisible = true;
      } catch (e) {
        ElMessage.error((e && e.message) || "获取详情失败");
      }
    },
    async doDelete(row) {
      try {
        await adminDeleteComment(row.id, row.type);
        ElMessage.success("已删除");
        this.fetch();
      } catch (e) {
        ElMessage.error((e && e.message) || "删除失败");
      }
    },
    typeLabel(t) {
      return { video: "视频评论", article: "文章评论", dynamic: "动态评论" }[t] || t;
    },
    typeTag(t) {
      return { video: "", article: "success", dynamic: "warning" }[t] || "";
    },
    statusLabel(s) {
      return { approved: "正常", pending: "待审", ignored: "已忽略" }[s] || s;
    },
    statusTag(s) {
      return { approved: "success", pending: "warning", ignored: "info" }[s] || "";
    },
    targetFallback(row) {
      if (row.target && row.target.id) return `#${row.target.id}`;
      return "—";
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
.cm-page { padding: 20px 24px; }
.cm-page__head { margin-bottom: 16px; }
.cm-page__title { margin: 0 0 4px; font-size: 18px; font-weight: 600; color: #18191c; }
.cm-page__desc { margin: 0; font-size: 13px; color: #9499a0; }
.cm-toolbar { margin-bottom: 14px; }
.cm-toolbar__filters { display: flex; gap: 10px; align-items: center; flex-wrap: wrap; }
.cm-pager { margin-top: 16px; display: flex; justify-content: flex-end; }
.cm-muted { color: #9499a0; }
.cm-target { color: #18191c; max-width: 140px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; display: inline-block; }
.cm-detail-content { white-space: pre-wrap; word-break: break-word; line-height: 1.6; }
</style>
