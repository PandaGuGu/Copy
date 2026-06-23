<template>
  <div class="adm-page">
    <div class="adm-page__header">
      <h2 class="adm-page__title">用户管理</h2>
    </div>

    <!-- 搜索栏 -->
    <div class="adm-toolbar">
      <el-select v-model="filterStatus" placeholder="状态筛选" size="small" clearable @change="doSearch">
        <el-option label="全部" value="" />
        <el-option label="正常" value="active" />
        <el-option label="已封禁" value="banned" />
        <el-option label="已禁用" value="disabled" />
      </el-select>
      <el-input
        v-model="searchText"
        placeholder="搜索用户名 / CakeID / 昵称"
        size="small"
        style="width: 260px; margin-left: 12px"
        clearable
        @keyup.enter="doSearch"
      />
      <el-button type="primary" size="small" @click="doSearch">搜索</el-button>
    </div>

    <!-- 用户列表 -->
    <el-table :data="users" v-loading="loading" border stripe size="small" style="width: 100%">
      <el-table-column prop="id" label="ID" width="70" />
      <el-table-column prop="username" label="用户名" width="120" />
      <el-table-column prop="cake_id" label="CakeID" width="160" />
      <el-table-column prop="nickname" label="昵称" width="120" />
      <el-table-column label="头像" width="70">
        <template #default="{ row }">
          <img v-if="row.avatar_url" :src="row.avatar_url" class="user-avatar" />
        </template>
      </el-table-column>
      <el-table-column prop="status" label="状态" width="90">
        <template #default="{ row }">
          <el-tag v-if="row.status === 'active'" type="success" size="small">正常</el-tag>
          <el-tag v-else-if="row.status === 'banned'" type="danger" size="small">已封禁</el-tag>
          <el-tag v-else-if="row.status === 'disabled'" type="info" size="small">已禁用</el-tag>
          <el-tag v-else type="warning" size="small">{{ row.status }}</el-tag>
        </template>
      </el-table-column>
      <el-table-column prop="video_count" label="视频" width="60" sortable="custom" />
      <el-table-column prop="article_count" label="专栏" width="60" />
      <el-table-column prop="dynamic_count" label="动态" width="60" />
      <el-table-column prop="follower_count" label="粉丝" width="70" />
      <el-table-column prop="level" label="等级" width="60">
        <template #default="{ row }">Lv{{ row.level }}</template>
      </el-table-column>
      <el-table-column prop="coin_balance" label="硬币" width="70" />
      <el-table-column prop="created_at" label="注册时间" width="170">
        <template #default="{ row }">{{ formatTime(row.created_at) }}</template>
      </el-table-column>
      <el-table-column label="操作" width="260" fixed="right">
        <template #default="{ row }">
          <el-button type="primary" size="small" link @click="viewDetail(row)">详情</el-button>
          <template v-if="row.status === 'banned'">
            <el-button type="success" size="small" link @click="confirmUnban(row)">解封</el-button>
          </template>
          <template v-else>
            <el-button type="warning" size="small" link @click="confirmBan(row)">封禁</el-button>
          </template>
          <el-button type="danger" size="small" link @click="confirmDelete(row)">强制注销</el-button>
        </template>
      </el-table-column>
    </el-table>

    <!-- 分页 -->
    <div class="adm-pagination">
      <el-pagination
        v-model:current-page="page"
        :page-size="pageSize"
        :total="total"
        layout="total, prev, pager, next"
        @current-change="loadUsers"
      />
    </div>

    <!-- 用户详情对话框 -->
    <el-dialog v-model="detailVisible" title="用户详情" width="550px" destroy-on-close>
      <template v-if="detailUser">
        <el-descriptions :column="2" border size="small">
          <el-descriptions-item label="ID">{{ detailUser.id }}</el-descriptions-item>
          <el-descriptions-item label="用户名">{{ detailUser.username }}</el-descriptions-item>
          <el-descriptions-item label="CakeID">{{ detailUser.cake_id }}</el-descriptions-item>
          <el-descriptions-item label="昵称">{{ detailUser.nickname }}</el-descriptions-item>
          <el-descriptions-item label="状态">
            <el-tag v-if="detailUser.status === 'active'" type="success" size="small">正常</el-tag>
            <el-tag v-else-if="detailUser.status === 'banned'" type="danger" size="small">已封禁</el-tag>
            <el-tag v-else type="info" size="small">{{ detailUser.status }}</el-tag>
          </el-descriptions-item>
          <el-descriptions-item label="等级">Lv{{ detailUser.level }}</el-descriptions-item>
          <el-descriptions-item label="视频数">{{ detailUser.video_count }}</el-descriptions-item>
          <el-descriptions-item label="专栏数">{{ detailUser.article_count }}</el-descriptions-item>
          <el-descriptions-item label="动态数">{{ detailUser.dynamic_count }}</el-descriptions-item>
          <el-descriptions-item label="粉丝数">{{ detailUser.follower_count }}</el-descriptions-item>
          <el-descriptions-item label="硬币余额">{{ detailUser.coin_balance }}</el-descriptions-item>
          <el-descriptions-item label="经验值">{{ detailUser.experience }}</el-descriptions-item>
          <el-descriptions-item label="性别">{{ detailUser.gender || '-' }}</el-descriptions-item>
          <el-descriptions-item label="生日">{{ detailUser.birthday || '-' }}</el-descriptions-item>
          <el-descriptions-item label="个性签名" :span="2">{{ detailUser.sign || '-' }}</el-descriptions-item>
          <el-descriptions-item v-if="detailUser.banned_reason" label="封禁原因" :span="2">
            <span style="color: red">{{ detailUser.banned_reason }}</span>
          </el-descriptions-item>
          <el-descriptions-item label="注册时间">{{ formatTime(detailUser.created_at) }}</el-descriptions-item>
          <el-descriptions-item label="最后更新">{{ formatTime(detailUser.updated_at) }}</el-descriptions-item>
        </el-descriptions>
      </template>
    </el-dialog>

    <!-- 封禁确认 -->
    <el-dialog v-model="banVisible" title="封禁账号" width="420px" destroy-on-close>
      <p>确认封禁用户 <b>{{ banTarget?.username }}</b> (ID: {{ banTarget?.id }})？</p>
      <el-input
        v-model="banReason"
        type="textarea"
        :rows="2"
        placeholder="请输入封禁原因（必填）"
        style="margin-top: 10px"
      />
      <template #footer>
        <el-button @click="banVisible = false">取消</el-button>
        <el-button type="danger" :disabled="!banReason.trim()" :loading="banning" @click="doBan">确认封禁</el-button>
      </template>
    </el-dialog>

    <!-- 强制注销确认 -->
    <el-dialog v-model="deleteVisible" title="强制注销" width="420px" destroy-on-close>
      <el-alert
        type="error"
        :closable="false"
        show-icon
        title="⚠️ 此操作不可逆！"
        description="账号将被禁用，用户数据将被匿名化处理。"
        style="margin-bottom: 12px"
      />
      <p>确认强制注销用户 <b>{{ deleteTarget?.username }}</b> (ID: {{ deleteTarget?.id }})？</p>
      <template #footer>
        <el-button @click="deleteVisible = false">取消</el-button>
        <el-button type="danger" :loading="deleting" @click="doDelete">确认注销</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script>
import { adminListUsers, adminGetUser, adminBanUser, adminUnbanUser, adminDeleteUser } from "@/api/admin";

export default {
  name: "AdminUserManage",
  data() {
    return {
      loading: false,
      users: [],
      page: 1,
      pageSize: 20,
      total: 0,
      filterStatus: "",
      searchText: "",
      // detail
      detailVisible: false,
      detailUser: null,
      // ban
      banVisible: false,
      banTarget: null,
      banReason: "",
      banning: false,
      // delete
      deleteVisible: false,
      deleteTarget: null,
      deleting: false
    };
  },
  created() {
    this.loadUsers();
  },
  methods: {
    async loadUsers() {
      this.loading = true;
      try {
        const res = await adminListUsers({
          page: this.page,
          page_size: this.pageSize,
          status: this.filterStatus || undefined,
          q: this.searchText || undefined
        });
        const d = res.data || res || {};
        this.users = d.items || [];
        this.total = d.total || 0;
      } finally {
        this.loading = false;
      }
    },
    doSearch() {
      this.page = 1;
      this.loadUsers();
    },
    async viewDetail(row) {
      try {
        const res = await adminGetUser(row.id);
        this.detailUser = res.data || res || {};
        this.detailVisible = true;
      } catch {
        this.$message.error("获取用户详情失败");
      }
    },
    confirmBan(row) {
      this.banTarget = row;
      this.banReason = "";
      this.banVisible = true;
    },
    async doBan() {
      if (!this.banReason.trim()) return;
      this.banning = true;
      try {
        await adminBanUser(this.banTarget.id, this.banReason.trim());
        this.$message.success("封禁成功");
        this.banVisible = false;
        this.loadUsers();
      } catch {
        this.$message.error("封禁失败");
      } finally {
        this.banning = false;
      }
    },
    confirmUnban(row) {
      this.$confirm(`确认解封用户 ${row.username}？`, "解封确认", {
        confirmButtonText: "确认解封",
        type: "warning"
      }).then(async () => {
        try {
          await adminUnbanUser(row.id);
          this.$message.success("解封成功");
          this.loadUsers();
        } catch {
          this.$message.error("解封失败");
        }
      }).catch(() => {});
    },
    confirmDelete(row) {
      this.deleteTarget = row;
      this.deleteVisible = true;
    },
    async doDelete() {
      this.deleting = true;
      try {
        await adminDeleteUser(this.deleteTarget.id);
        this.$message.success("已强制注销");
        this.deleteVisible = false;
        this.loadUsers();
      } catch {
        this.$message.error("操作失败");
      } finally {
        this.deleting = false;
      }
    },
    formatTime(ts) {
      if (!ts) return "-";
      return new Date(ts).toLocaleString("zh-CN");
    }
  }
};
</script>

<style scoped>
.adm-toolbar {
  display: flex;
  align-items: center;
  margin-bottom: 16px;
}
.user-avatar {
  width: 36px;
  height: 36px;
  border-radius: 50%;
  object-fit: cover;
}
.adm-pagination {
  margin-top: 16px;
  display: flex;
  justify-content: flex-end;
}
</style>
