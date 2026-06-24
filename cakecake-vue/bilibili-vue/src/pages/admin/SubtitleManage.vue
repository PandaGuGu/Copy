<template>
  <div class="sub-page" v-loading="loading">
    <header class="sub-page__head">
      <h2 class="sub-page__title">字幕管理</h2>
      <p class="sub-page__desc">管理全站视频字幕</p>
    </header>

    <div class="sub-toolbar">
      <el-input
        v-model="filterVideoId"
        placeholder="视频 ID"
        clearable
        style="width: 180px"
        @clear="fetchList"
        @keydown.enter="fetchList"
      />
      <el-select v-model="filterLang" placeholder="语言" clearable style="width: 130px" @change="fetchList">
        <el-option label="中文" value="zh" />
        <el-option label="英文" value="en" />
        <el-option label="日文" value="ja" />
        <el-option label="韩文" value="ko" />
      </el-select>
      <el-button type="primary" @click="fetchList">查询</el-button>
    </div>

    <el-table :data="list" stripe size="default" empty-text="暂无字幕">
      <el-table-column prop="id" label="ID" width="70" />
      <el-table-column prop="video_id" label="视频 ID" width="90" />
      <el-table-column label="语言" width="80">
        <template #default="{ row }">
          <el-tag size="small" effect="plain">{{ langLabel(row.lang) }}</el-tag>
        </template>
      </el-table-column>
      <el-table-column prop="title" label="标题" min-width="140" show-overflow-tooltip />
      <el-table-column label="格式" width="70">
        <template #default="{ row }">{{ row.format?.toUpperCase() }}</template>
      </el-table-column>
      <el-table-column label="自动生成" width="90">
        <template #default="{ row }">
          <el-tag :type="row.auto_gen ? 'warning' : 'info'" size="small">
            {{ row.auto_gen ? '是' : '否' }}
          </el-tag>
        </template>
      </el-table-column>
      <el-table-column prop="created_at" label="上传时间" width="170">
        <template #default="{ row }">{{ fmtDate(row.created_at) }}</template>
      </el-table-column>
      <el-table-column label="操作" width="180" fixed="right">
        <template #default="{ row }">
          <el-button size="small" text type="primary" @click="viewContent(row)">查看</el-button>
          <el-popconfirm title="确认删除此字幕？" @confirm="doDelete(row)">
            <template #reference>
              <el-button size="small" text type="danger">删除</el-button>
            </template>
          </el-popconfirm>
        </template>
      </el-table-column>
    </el-table>

    <!-- 内容预览弹框 -->
    <el-dialog v-model="previewOpen" title="字幕内容预览" width="700px" destroy-on-close>
      <div class="sub-preview-meta">
        <el-tag size="small" effect="plain">{{ langLabel(previewRow?.lang) }}</el-tag>
        <span class="sub-preview-title">{{ previewRow?.title }}</span>
        <span class="sub-preview-format">{{ previewRow?.format?.toUpperCase() }}</span>
      </div>
      <pre class="sub-preview-content">{{ previewRow?.content }}</pre>
    </el-dialog>
  </div>
</template>

<script>
import { mbAdminListSubtitles, mbAdminDeleteSubtitle } from "@/api/minibili";
import { ElMessage } from "element-plus";

const LANG_MAP = { zh: "中文", en: "English", ja: "日本語", ko: "한국어" };

export default {
  name: "SubtitleManage",
  data() {
    return {
      loading: false,
      list: [],
      filterVideoId: "",
      filterLang: "",
      previewOpen: false,
      previewRow: null
    };
  },
  mounted() {
    this.fetchList();
  },
  methods: {
    async fetchList() {
      this.loading = true;
      try {
        const params = {};
        const vid = (this.filterVideoId || "").trim();
        const lang = (this.filterLang || "").trim();
        if (vid) params.video_id = vid;
        if (lang) params.lang = lang;
        this.list = await mbAdminListSubtitles(params);
      } catch (e) {
        ElMessage.error("加载字幕列表失败");
      } finally {
        this.loading = false;
      }
    },
    async doDelete(row) {
      try {
        await mbAdminDeleteSubtitle(row.id);
        ElMessage.success("已删除");
        this.list = this.list.filter((r) => r.id !== row.id);
      } catch (e) {
        ElMessage.error("删除失败");
      }
    },
    viewContent(row) {
      this.previewRow = row;
      this.previewOpen = true;
    },
    langLabel(lang) {
      return LANG_MAP[lang] || lang || "未知";
    },
    fmtDate(iso) {
      if (!iso) return "";
      const d = new Date(iso);
      const y = d.getFullYear();
      const mo = String(d.getMonth() + 1).padStart(2, "0");
      const da = String(d.getDate()).padStart(2, "0");
      const h = String(d.getHours()).padStart(2, "0");
      const mi = String(d.getMinutes()).padStart(2, "0");
      return `${y}-${mo}-${da} ${h}:${mi}`;
    }
  }
};
</script>

<style scoped>
.sub-page { padding: 0; }
.sub-page__head { margin-bottom: 20px; }
.sub-page__title { margin: 0 0 4px; font-size: 20px; font-weight: 600; color: #1a1a1a; }
.sub-page__desc { margin: 0; font-size: 13px; color: #888; }

.sub-toolbar {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 16px;
}

.sub-preview-meta {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 12px;
}
.sub-preview-title { font-size: 14px; font-weight: 500; color: #333; }
.sub-preview-format { font-size: 12px; color: #999; }

.sub-preview-content {
  max-height: 420px;
  overflow-y: auto;
  background: #f5f6f7;
  border-radius: 6px;
  padding: 14px 16px;
  margin: 0;
  font-family: "Consolas", "Monaco", monospace;
  font-size: 12px;
  line-height: 1.6;
  color: #333;
  white-space: pre-wrap;
  word-break: break-all;
}
</style>
