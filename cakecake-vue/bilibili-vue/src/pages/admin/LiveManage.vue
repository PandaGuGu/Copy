<template>
  <div class="lm-page" v-loading="loading">
    <header class="lm-head">
      <h2 class="lm-title">直播管理</h2>
    </header>

    <el-tabs v-model="tab" @tab-change="fetchRooms">
      <el-tab-pane label="全部" name="all" />
      <el-tab-pane label="直播中" name="live" />
      <el-tab-pane label="已结束" name="ended" />
    </el-tabs>

    <el-table :data="rooms" stripe size="default" empty-text="暂无直播间">
      <el-table-column prop="id" label="ID" width="60" />
      <el-table-column prop="title" label="标题" min-width="160" show-overflow-tooltip />
      <el-table-column label="主播" width="120">
        <template #default="{ row }">{{ row.host_name || "-" }}</template>
      </el-table-column>
      <el-table-column label="状态" width="90">
        <template #default="{ row }">
          <el-tag :type="statusTag(row.status)" size="small">{{ statusLabel(row.status) }}</el-tag>
        </template>
      </el-table-column>
      <el-table-column label="观看" width="70">
        <template #default="{ row }">{{ row.viewer_count || 0 }}</template>
      </el-table-column>
      <el-table-column label="创建时间" width="170">
        <template #default="{ row }">{{ fmt(row.created_at) }}</template>
      </el-table-column>
      <el-table-column label="操作" width="200" fixed="right">
        <template #default="{ row }">
          <el-button size="small" text type="primary" @click="openDetail(row)">详情</el-button>
          <el-button
            v-if="row.status === 'live'"
            size="small" text type="danger"
            @click="banRoom(row)">封禁</el-button>
          <el-button
            v-if="row.status === 'banned'"
            size="small" text type="success"
            @click="unbanRoom(row)">解封</el-button>
          <el-popconfirm title="确认删除？" @confirm="deleteRoom(row)">
            <template #reference>
              <el-button size="small" text type="danger">删除</el-button>
            </template>
          </el-popconfirm>
        </template>
      </el-table-column>
    </el-table>

    <div v-if="total > pageSize" class="lm-pagination">
      <el-pagination
        background
        layout="prev, pager, next"
        :total="total"
        :page-size="pageSize"
        v-model:current-page="page"
        @current-change="fetchRooms"
      />
    </div>

    <!-- 详情弹窗 -->
    <el-dialog
      v-model="detailVisible"
      :title="detail ? detail.title : '直播详情'"
      width="960px"
      destroy-on-close
      @closed="closeDetail"
    >
      <template v-if="detail">
        <div class="lm-detail">
          <!-- 直播画面 -->
          <div class="lm-detail__player">
            <video
              v-if="detail.status === 'live'"
              ref="liveVideoRef"
              class="lm-detail__video"
              controls
              autoplay
              muted
            ></video>
            <div v-else class="lm-detail__placeholder">
              <p class="lm-detail__ph-icon">📡</p>
              <p>{{ detail.status === 'ended' ? '直播已结束' : detail.status === 'banned' ? '已封禁' : '未开播' }}</p>
            </div>
          </div>
          <!-- 右侧面板：信息 + 聊天 -->
          <div class="lm-detail__side">
            <div class="lm-detail__meta">
              <p><strong>ID：</strong>{{ detail.id }}</p>
              <p><strong>主播：</strong>{{ detail.host_name || detail.username || "—" }}</p>
              <p><strong>状态：</strong><el-tag :type="statusTag(detail.status)" size="small">{{ statusLabel(detail.status) }}</el-tag></p>
              <p><strong>观看人数：</strong>{{ detail.viewer_count || 0 }}</p>
              <p v-if="detail.started_at"><strong>开播时间：</strong>{{ fmt(detail.started_at) }}</p>
              <p><strong>创建时间：</strong>{{ fmt(detail.created_at) }}</p>
            </div>
            <LiveChat
              v-if="detail.id"
              class="lm-detail__chat"
              :room-id="detail.id"
              :audience="[]"
              :audience-count="detail.viewer_count || 0"
            />
          </div>
        </div>
      </template>
      <template #footer>
        <el-button @click="detailVisible = false">关闭</el-button>
        <template v-if="detail && detail.status === 'live'">
          <el-button type="warning" :loading="acting" @click="openWarnDialog">⚠ 警告</el-button>
          <el-popconfirm title="确认封禁该直播间？" @confirm="banDetailRoom">
            <template #reference>
              <el-button type="danger">封禁</el-button>
            </template>
          </el-popconfirm>
        </template>
        <template v-if="detail && detail.status === 'banned'">
          <el-button type="success" :loading="acting" @click="unbanDetailRoom">解封</el-button>
        </template>
      </template>
    </el-dialog>

    <!-- 警告原因弹窗 -->
    <el-dialog
      v-model="warnVisible"
      title="发送警告"
      width="560px"
      destroy-on-close
      @closed="warnReason = ''"
      @open="loadWarnTemplates"
    >
      <div class="lm-warn-templates">
        <span class="lm-warn-tpl-label">快捷模板：</span>
        <el-tag
          v-for="tpl in warnTemplates"
          :key="tpl.id"
          class="lm-warn-tpl"
          size="small"
          @click="warnReason = tpl.content"
          style="cursor:pointer"
        >{{ tpl.name }}</el-tag>
        <el-button size="small" text type="primary" @click="openTplManager">管理模板</el-button>
      </div>
      <el-form label-width="72px" style="margin-top:12px">
        <el-form-item label="警告原因" required>
          <el-input
            v-model="warnReason"
            type="textarea"
            :rows="4"
            maxlength="200"
            show-word-limit
            placeholder="输入或点击上方模板选择，将显示在直播间画面中（5秒）"
          />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="warnVisible = false">取消</el-button>
        <el-button type="danger" :loading="acting" :disabled="!warnReason.trim()" @click="confirmWarn">发送警告</el-button>
      </template>
    </el-dialog>

    <!-- 模板管理弹窗 -->
    <el-dialog
      v-model="tplMgrVisible"
      title="警告模板管理"
      width="560px"
      destroy-on-close
      @open="loadWarnTemplates"
    >
      <div style="margin-bottom:12px">
        <el-button type="primary" size="small" @click="openTplForm(null)">+ 新建模板</el-button>
      </div>
      <el-table :data="warnTemplates" stripe size="small" max-height="300">
        <el-table-column prop="name" label="名称" width="120" show-overflow-tooltip />
        <el-table-column prop="content" label="内容" min-width="200" show-overflow-tooltip />
        <el-table-column label="操作" width="120" align="center">
          <template #default="{ row }">
            <el-button link type="primary" size="small" @click="openTplForm(row)">编辑</el-button>
            <el-popconfirm title="确认删除？" @confirm="delTpl(row.id)">
              <template #reference>
                <el-button link type="danger" size="small">删除</el-button>
              </template>
            </el-popconfirm>
          </template>
        </el-table-column>
      </el-table>
    </el-dialog>

    <!-- 模板编辑弹窗 -->
    <el-dialog
      v-model="tplFormVisible"
      :title="tplForm.id ? '编辑模板' : '新建模板'"
      width="460px"
      destroy-on-close
    >
      <el-form :model="tplForm" label-width="70px" size="default">
        <el-form-item label="名称" required>
          <el-input v-model="tplForm.name" placeholder="如：违规内容" maxlength="20" show-word-limit />
        </el-form-item>
        <el-form-item label="内容" required>
          <el-input v-model="tplForm.content" type="textarea" :rows="4" placeholder="警告文字内容" maxlength="200" show-word-limit />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="tplFormVisible = false">取消</el-button>
        <el-button type="primary" :loading="acting" :disabled="!tplForm.name.trim() || !tplForm.content.trim()" @click="saveTpl">保存</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script>
import { ref, nextTick, reactive } from "vue";
import { ElMessage } from "element-plus";
import flvjs from "flv.js";
import LiveChat from "@/components/live/LiveChat.vue";
import {
  adminListLiveRooms, adminBanLiveRoom, adminUnbanLiveRoom, adminDeleteLiveRoom,
  adminWarnLiveRoom, adminGetLiveRoom,
  listWarnTemplates, createWarnTemplate, updateWarnTemplate, deleteWarnTemplate
} from "@/api/live";

export default {
  name: "LiveManage",
  components: { LiveChat },
  setup() {
    const loading = ref(false);
    const acting = ref(false);
    const rooms = ref([]);
    const tab = ref("all");
    const page = ref(1);
    const pageSize = ref(20);
    const total = ref(0);

    const detailVisible = ref(false);
    const detail = ref(null);
    const liveVideoRef = ref(null);
    let flvPlayer = null;

    const warnVisible = ref(false);
    const warnReason = ref("");
    const warnTemplates = ref([]);

    const tplMgrVisible = ref(false);
    const tplFormVisible = ref(false);
    const tplForm = reactive({ id: 0, name: "", content: "" });

    // 模板CRUD
    async function loadWarnTemplates() {
      try {
        const res = await listWarnTemplates();
        const d = res.data || res;
        warnTemplates.value = d.data ? (d.data.templates || []) : (d.templates || []);
      } catch (e) { /* ignore */ }
    }
    function openTplManager() { tplMgrVisible.value = true; }
    function openTplForm(row) {
      if (row) {
        Object.assign(tplForm, { id: row.id, name: row.name, content: row.content });
      } else {
        Object.assign(tplForm, { id: 0, name: "", content: "" });
      }
      tplFormVisible.value = true;
    }
    async function saveTpl() {
      if (!tplForm.name.trim() || !tplForm.content.trim()) return;
      acting.value = true;
      try {
        if (tplForm.id) {
          await updateWarnTemplate(tplForm.id, { name: tplForm.name, content: tplForm.content });
          ElMessage.success("已更新");
        } else {
          await createWarnTemplate({ name: tplForm.name, content: tplForm.content });
          ElMessage.success("已创建");
        }
        tplFormVisible.value = false;
        await loadWarnTemplates();
      } catch (e) {
        ElMessage.error("操作失败");
      } finally {
        acting.value = false;
      }
    }
    async function delTpl(id) {
      try {
        await deleteWarnTemplate(id);
        ElMessage.success("已删除");
        await loadWarnTemplates();
      } catch (e) {
        ElMessage.error("删除失败");
      }
    }

    async function fetchRooms() {
      loading.value = true;
      try {
        const status = tab.value === "all" ? "" : tab.value;
        const res = await adminListLiveRooms({ status, page: page.value, page_size: pageSize.value });
        const data = res.data || res;
        rooms.value = data.data ? (data.data.rooms || data.data.list || []) : (data.list || data.rooms || []);
        total.value = data.total || 0;
      } catch (e) {
        ElMessage.warning("加载直播列表失败");
      } finally {
        loading.value = false;
      }
    }

    async function openDetail(row) {
      try {
        const res = await adminGetLiveRoom(row.id);
        const d = res.data || res;
        detail.value = d.data || d;
        detailVisible.value = true;
        await nextTick();
        startPlayer();
      } catch (e) {
        ElMessage.error("获取直播间详情失败");
      }
    }

    function startPlayer() {
      stopPlayer();
      if (!detail.value || detail.value.status !== "live") return;
      if (!detail.value.stream_key) return;
      const url = `http://localhost:8000/live/${detail.value.stream_key}.flv`;
      nextTick(() => {
        if (!liveVideoRef.value) return;
        try {
          if (flvjs.isSupported()) {
            flvPlayer = flvjs.createPlayer({ type: "flv", url, isLive: true });
            flvPlayer.attachMediaElement(liveVideoRef.value);
            flvPlayer.load();
            flvPlayer.play();
          }
        } catch (e) { console.error(e); }
      });
    }

    function stopPlayer() {
      if (flvPlayer) {
        try { flvPlayer.pause(); flvPlayer.unload(); flvPlayer.detachMediaElement(); flvPlayer.destroy(); } catch (e) {}
        flvPlayer = null;
      }
    }

    function closeDetail() {
      stopPlayer();
      detail.value = null;
      fetchRooms();
    }

    function openWarnDialog() {
      warnReason.value = "";
      warnVisible.value = true;
    }

    async function confirmWarn() {
      const reason = warnReason.value.trim();
      if (!reason) { ElMessage.warning("请输入警告原因"); return; }
      if (!detail.value) return;
      acting.value = true;
      try {
        await adminWarnLiveRoom(detail.value.id, reason);
        ElMessage.success("警告已发送");
        warnVisible.value = false;
      } catch (e) {
        ElMessage.error("发送警告失败");
      } finally {
        acting.value = false;
      }
    }

    async function banRoom(row) {
      try {
        await adminBanLiveRoom(row.id);
        ElMessage.success("已封禁");
        fetchRooms();
      } catch (e) {
        ElMessage.error("操作失败");
      }
    }

    async function banDetailRoom() {
      if (!detail.value) return;
      acting.value = true;
      try {
        await adminBanLiveRoom(detail.value.id);
        ElMessage.success("已封禁");
        detail.value.status = "banned";
        stopPlayer();
      } catch (e) {
        ElMessage.error("操作失败");
      } finally {
        acting.value = false;
      }
    }

    async function unbanRoom(row) {
      try {
        await adminUnbanLiveRoom(row.id);
        ElMessage.success("已解封");
        fetchRooms();
      } catch (e) {
        ElMessage.error("操作失败");
      }
    }

    async function unbanDetailRoom() {
      if (!detail.value) return;
      acting.value = true;
      try {
        await adminUnbanLiveRoom(detail.value.id);
        ElMessage.success("已解封");
        detail.value.status = "idle";
      } catch (e) {
        ElMessage.error("操作失败");
      } finally {
        acting.value = false;
      }
    }

    async function deleteRoom(row) {
      try {
        await adminDeleteLiveRoom(row.id);
        ElMessage.success("已删除");
        fetchRooms();
      } catch (e) {
        ElMessage.error("删除失败");
      }
    }

    function statusTag(status) {
      const map = { idle: "info", live: "success", ended: "", banned: "danger" };
      return map[status] || "info";
    }

    function statusLabel(status) {
      const map = { idle: "未开播", live: "直播中", ended: "已结束", banned: "已封禁" };
      return map[status] || status;
    }

    function fmt(ts) {
      if (!ts) return "-";
      return new Date(ts).toLocaleString("zh-CN");
    }

    return {
      loading, acting, rooms, tab, page, pageSize, total,
      detailVisible, detail, liveVideoRef,
      warnVisible, warnReason, warnTemplates,
      tplMgrVisible, tplFormVisible, tplForm,
      fetchRooms, openDetail, closeDetail,
      openWarnDialog, confirmWarn,
      loadWarnTemplates, openTplManager, openTplForm, saveTpl, delTpl,
      banRoom, banDetailRoom, unbanRoom, unbanDetailRoom, deleteRoom,
      statusTag, statusLabel, fmt
    };
  },
  mounted() {
    this.fetchRooms();
  }
};
</script>

<style scoped>
.lm-page { padding: 20px; }
.lm-head { margin-bottom: 16px; }
.lm-title { font-size: 20px; font-weight: 500; margin: 0; }
.lm-pagination { margin-top: 16px; display: flex; justify-content: center; }

.lm-detail { display: flex; gap: 16px; }
.lm-detail__player {
  flex: 1; min-width: 400px; min-height: 300px;
  background: #1a1a2e; border-radius: 8px; overflow: hidden;
  display: flex; align-items: center; justify-content: center;
}
.lm-detail__video { width: 100%; height: 100%; object-fit: contain; }
.lm-detail__placeholder { text-align: center; color: #8888aa; }
.lm-detail__ph-icon { font-size: 48px; margin-bottom: 8px; }

.lm-detail__side {
  width: 300px; flex-shrink: 0; display: flex; flex-direction: column; gap: 12px;
}
.lm-detail__meta {
  background: #f6f7f8; border-radius: 8px; padding: 14px;
}
.lm-detail__meta p {
  margin: 0 0 8px; font-size: 13px; color: #61666d; line-height: 1.5;
}
.lm-detail__meta strong { color: #18191c; }

.lm-detail__chat {
  flex: 1; min-height: 200px; max-height: 300px; overflow: hidden;
  border: 1px solid #e3e5e7; border-radius: 8px;
}
.lm-detail__chat :deep(.lc-gift-bar) { display: none; }
.lm-detail__chat :deep(.lc-input-box) { display: none; }
.lm-detail__chat :deep(.lc-follow-btn) { display: none; }
.lm-detail__chat :deep(.lc-header-right) { display: none; }

.lm-warn-templates { display: flex; flex-wrap: wrap; align-items: center; gap: 6px; }
.lm-warn-tpl-label { font-size: 12px; color: #888; white-space: nowrap; }
.lm-warn-tpl { transition: transform 0.15s; }
.lm-warn-tpl:hover { transform: scale(1.05); }
</style>
