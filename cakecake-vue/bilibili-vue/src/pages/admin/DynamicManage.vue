<template>
  <div class="adm-panel">
    <div class="adm-panel__head">
      <h2>
        动态管理
        <el-tag type="info" size="small" class="adm-badge">无需审核</el-tag>
      </h2>
    </div>

    <p class="adm-hint">
      统一动态信息流：聚合视频投稿、专栏文章、图文动态。可按用户 ID 和内容类别筛选，支持查看详情与删除（仅图文动态）。
    </p>

    <div class="adm-table-wrap">
      <AdminDataTable
        :data="rows"
        :loading="loading"
        :page="page"
        :page-size="pageSize"
        :total="total"
        :show-pagination="true"
        class="adm-dyn-table"
        @update:page="page = $event; load()"
      >
        <template #search-bar>
          <el-input
            v-model="keyword"
            placeholder="搜索标题或正文"
            clearable
            style="width: 200px"
            @keyup.enter="onSearch"
          />
          <el-input
            v-model="filterUid"
            placeholder="用户 ID"
            clearable
            style="width: 120px"
            @keyup.enter="onSearch"
          />
          <el-select
            v-model="filterKind"
            placeholder="内容类别"
            clearable
            style="width: 120px"
            @change="onSearch"
          >
            <el-option label="全部" value="" />
            <el-option label="视频" value="video" />
            <el-option label="专栏" value="article" />
            <el-option label="图文" value="image" />
            <el-option label="纯文字" value="text" />
          </el-select>
          <el-button type="primary" @click="onSearch">搜索</el-button>
        </template>

        <el-table-column prop="id" label="ID" width="64" />
        <el-table-column label="类别" width="72" align="center">
          <template #default="{ row }">
            <el-tag :type="kindTagType(row.kind)" size="small" effect="plain">
              {{ kindLabel(row.kind) }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column label="图片" width="108">
          <template #default="{ row }">
            <img v-if="row.cover_url" :src="row.cover_url" class="adm-thumb" alt="" />
            <span v-else class="adm-no-cover">无图</span>
          </template>
        </el-table-column>
        <el-table-column prop="title" label="标题" min-width="120" show-overflow-tooltip />
        <el-table-column prop="content" label="正文" min-width="160" show-overflow-tooltip />
        <el-table-column prop="uploader_name" label="作者" width="100" show-overflow-tooltip />
        <el-table-column label="互动" width="100" align="center">
          <template #default="{ row }">
            <span class="adm-stat">赞 {{ row.like_count || 0 }}</span>
            <span class="adm-stat">评 {{ row.comment_count || 0 }}</span>
          </template>
        </el-table-column>
        <el-table-column label="发布时间" min-width="168" show-overflow-tooltip>
          <template #default="{ row }">{{ formatTime(row.created_at) }}</template>
        </el-table-column>
        <el-table-column label="操作" min-width="140" align="center">
          <template #default="{ row }">
            <el-button link type="primary" @click="openDetail(row)">详情</el-button>
            <el-button link type="danger" @click="onDelete(row)">删除</el-button>
          </template>
        </el-table-column>
      </AdminDataTable>
    </div>

    <el-dialog
      v-model="detailVisible"
      :title="detail ? `动态 #${detail.id}` : '动态详情'"
      width="720px"
      destroy-on-close
      @closed="detail = null"
    >
      <template v-if="detail">
        <div class="adm-review">
          <div class="adm-review__meta">
            <h3>{{ detail.title || "（无标题）" }}</h3>
            <p><strong>作者：</strong>{{ detail.uploader_name || detail.user_id }}</p>
            <p><strong>类别：</strong>{{ kindLabel(detail.kind) }}</p>
            <p><strong>点赞 / 评论：</strong>{{ detail.like_count || 0 }} / {{ detail.comment_count || 0 }}</p>
            <p><strong>发布时间：</strong>{{ formatTime(detail.created_at) }}</p>
            <p v-if="publicLink">
              <strong>前台链接：</strong>
              <a :href="publicLink" target="_blank" rel="noopener noreferrer">{{ publicLink }}</a>
            </p>
            <p class="adm-review__content"><strong>正文：</strong>{{ detail.content || "（无）" }}</p>
          </div>
          <div v-if="detail.cover_url" class="adm-dyn-cover">
            <img :src="detail.cover_url" class="adm-dyn-img" alt="" />
          </div>
        </div>
      </template>
      <template #footer>
        <el-button @click="detailVisible = false">关闭</el-button>
        <el-button
          v-if="detail && (detail.kind === 'image' || detail.kind === 'text')"
          type="danger"
          :loading="acting"
          @click="onDelete(detail)"
        >
          删除动态
        </el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script>
import {
  adminDeleteDynamic,
  adminListUnifiedDynamics
} from "@/api/admin";
import { ElMessage, ElMessageBox } from "element-plus";
import AdminDataTable from "@/components/admin/AdminDataTable.vue";

export default {
  components: { AdminDataTable },
  data() {
    return {
      loading: false,
      acting: false,
      rows: [],
      page: 1,
      pageSize: 20,
      total: 0,
      keyword: "",
      filterUid: "",
      filterKind: "",
      detailVisible: false,
      detail: null
    };
  },
  computed: {
    publicLink() {
      if (!this.detail || !this.detail.id) return "";
      const base = window.location.origin + window.location.pathname;
      const kind = this.detail.kind;
      if (kind === "video") return `${base}#/minibili/video/${this.detail.id}`;
      if (kind === "article") return `${base}#/minibili/article/${this.detail.id}`;
      return `${base}#/minibili/dynamic/${this.detail.id}`;
    }
  },
  created() {
    this.load();
  },
  methods: {
    async load() {
      this.loading = true;
      try {
        const body = await adminListUnifiedDynamics({
          page: this.page,
          page_size: this.pageSize,
          q: this.keyword.trim(),
          user_id: this.filterUid.trim(),
          kind: this.filterKind
        });
        const d = body.data || {};
        this.rows = d.items || [];
        this.total = d.total || 0;
        this.page = d.page || 1;
      } finally {
        this.loading = false;
      }
    },
    onSearch() {
      this.page = 1;
      void this.load();
    },
    formatTime(t) {
      if (!t) return "—";
      const d = new Date(t);
      if (Number.isNaN(d.getTime())) return String(t);
      const pad = (x) => String(x).padStart(2, "0");
      return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}`;
    },
    async openDetail(row) {
      this.detail = row;
      this.detailVisible = true;
    },
    kindLabel(k) {
      const map = { video: "视频", article: "专栏", image: "图文", text: "纯文字" };
      return map[k] || k || "—";
    },
    kindTagType(k) {
      const map = { video: "primary", article: "warning", image: "success", text: "info" };
      return map[k] || "info";
    },
    async onDelete(row) {
      const label = this.kindLabel(row.kind);
      await ElMessageBox.confirm(
        `确定删除${label}动态 #${row.id}？将同步删除数据库记录、评论、点赞及 OSS 上的全部图片，且不可恢复。`,
        "确认删除",
        { type: "warning" }
      );
      this.acting = true;
      try {
        await adminDeleteDynamic(row.id);
        ElMessage.success("已删除");
        this.detailVisible = false;
        await this.load();
      } finally {
        this.acting = false;
      }
    }
  }
};
</script>

<style lang="scss" scoped>
@import "@/style/mixin";

.adm-panel {
  background: $white;
  border: 1px solid #e3e5e7;
  border-radius: 8px;
  padding: 20px;
}
.adm-panel__head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
  flex-wrap: wrap;
  gap: 12px;
  h2 {
    margin: 0;
    @include sc(18px, #18191c);
    display: flex;
    align-items: center;
    gap: 8px;
  }
}
.adm-toolbar {
  display: flex;
  gap: 8px;
}
.adm-hint {
  margin: 0 0 16px;
  @include sc(13px, #61666d);
  line-height: 1.5;
}
.adm-table-wrap {
  width: 100%;
  overflow-x: auto;
}
.adm-dyn-table {
  width: 100%;
  min-width: 1080px;
}
.adm-thumb {
  width: 96px;
  height: 54px;
  object-fit: cover;
  border-radius: 4px;
}
.adm-no-cover {
  @include sc(12px, #99a2aa);
}
.adm-stat {
  display: block;
  @include sc(12px, #61666d);
}
.adm-pager {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 16px;
  margin-top: 16px;
  @include sc(13px, #61666d);
}
.adm-review__meta {
  h3 {
    margin: 0 0 12px;
    @include sc(16px, #18191c);
  }
  p {
    margin: 0 0 8px;
    @include sc(13px, #61666d);
    line-height: 1.5;
  }
  a {
    color: $blue;
    word-break: break-all;
  }
}
.adm-review__content {
  white-space: pre-wrap;
  word-break: break-word;
}
.adm-dyn-images {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: 16px;
}
.adm-dyn-cover {
  margin-top: 16px;
}
.adm-dyn-img {
  width: 120px;
  height: 120px;
  object-fit: cover;
  border-radius: 6px;
  border: 1px solid #e3e5e7;
}
</style>
