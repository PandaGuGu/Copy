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
      <el-table-column label="操作" width="140" fixed="right">
        <template #default="{ row }">
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
  </div>
</template>

<script>
import { ref } from "vue";
import { ElMessage } from "element-plus";
import { adminListLiveRooms, adminBanLiveRoom, adminUnbanLiveRoom, adminDeleteLiveRoom } from "@/api/live";

export default {
  name: "LiveManage",
  setup() {
    const loading = ref(false);
    const rooms = ref([]);
    const tab = ref("all");
    const page = ref(1);
    const pageSize = ref(20);
    const total = ref(0);

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

    async function banRoom(row) {
      try {
        await adminBanLiveRoom(row.id);
        ElMessage.success("已封禁");
        fetchRooms();
      } catch (e) {
        ElMessage.error("操作失败");
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

    return { loading, rooms, tab, page, pageSize, total, fetchRooms, banRoom, unbanRoom, deleteRoom, statusTag, statusLabel, fmt };
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
</style>
