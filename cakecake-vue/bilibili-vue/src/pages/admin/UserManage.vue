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

    <!-- 用户详情 Drawer -->
    <el-drawer
      v-model="detailVisible"
      title="用户详情"
      size="480px"
      destroy-on-close
      direction="rtl"
    >
      <template v-if="detailUser">
        <!-- 头像 + 身份区 -->
        <div class="user-detail-hero">
          <img v-if="detailUser.avatar_url" :src="detailUser.avatar_url" class="user-detail-avatar" />
          <div class="user-detail-identity">
            <div class="user-detail-name">
              {{ detailUser.nickname || detailUser.username }}
              <el-tag
                v-if="detailUser.status === 'banned'"
                type="danger"
                size="small"
                effect="dark"
              >已封禁</el-tag>
              <el-tag
                v-else-if="detailUser.status === 'disabled'"
                type="info"
                size="small"
                effect="dark"
              >已禁用</el-tag>
            </div>
            <div class="user-detail-sub">
              @{{ detailUser.username }} · CakeID: {{ detailUser.cake_id }}
            </div>
          </div>
        </div>

        <!-- 封禁提示 -->
        <el-alert
          v-if="detailUser.banned_reason"
          type="error"
          :closable="false"
          show-icon
          class="user-detail-alert"
        >
          <template #title>封禁原因：{{ detailUser.banned_reason }}</template>
        </el-alert>

        <!-- 数据看板 -->
        <div class="user-detail-stats">
          <div class="user-detail-stat">
            <span class="user-detail-stat__val">{{ detailUser.video_count || 0 }}</span>
            <span class="user-detail-stat__label">视频</span>
          </div>
          <div class="user-detail-stat">
            <span class="user-detail-stat__val">{{ detailUser.article_count || 0 }}</span>
            <span class="user-detail-stat__label">专栏</span>
          </div>
          <div class="user-detail-stat">
            <span class="user-detail-stat__val">{{ detailUser.dynamic_count || 0 }}</span>
            <span class="user-detail-stat__label">动态</span>
          </div>
          <div class="user-detail-stat">
            <span class="user-detail-stat__val">{{ detailUser.follower_count || 0 }}</span>
            <span class="user-detail-stat__label">粉丝</span>
          </div>
        </div>

        <!-- 基础信息 -->
        <div class="user-detail-section">
          <h4 class="user-detail-section__title">基础信息</h4>
          <div class="user-detail-grid">
            <div class="user-detail-cell">
              <span class="user-detail-cell__label">用户 ID</span>
              <span class="user-detail-cell__value">#{{ detailUser.id }}</span>
            </div>
            <div class="user-detail-cell">
              <span class="user-detail-cell__label">等级</span>
              <span class="user-detail-cell__value">Lv{{ detailUser.level }}</span>
            </div>
            <div class="user-detail-cell">
              <span class="user-detail-cell__label">经验值</span>
              <span class="user-detail-cell__value">{{ detailUser.experience || 0 }}</span>
            </div>
            <div class="user-detail-cell">
              <span class="user-detail-cell__label">硬币余额</span>
              <span class="user-detail-cell__value">{{ detailUser.coin_balance || 0 }}</span>
            </div>
            <div class="user-detail-cell">
              <span class="user-detail-cell__label">性别</span>
              <span class="user-detail-cell__value">{{ detailUser.gender || "未设置" }}</span>
            </div>
            <div class="user-detail-cell">
              <span class="user-detail-cell__label">生日</span>
              <span class="user-detail-cell__value">{{ detailUser.birthday || "未设置" }}</span>
            </div>
          </div>
        </div>

        <!-- 个性签名 -->
        <div class="user-detail-section" v-if="detailUser.sign">
          <h4 class="user-detail-section__title">个性签名</h4>
          <p class="user-detail-sign">{{ detailUser.sign }}</p>
        </div>

        <!-- 时间信息 -->
        <div class="user-detail-section">
          <h4 class="user-detail-section__title">时间记录</h4>
          <div class="user-detail-timeline">
            <div class="user-detail-time-item">
              <span class="user-detail-time__dot"></span>
              <div>
                <span class="user-detail-time__label">注册时间</span>
                <span class="user-detail-time__value">{{ formatTime(detailUser.created_at) }}</span>
              </div>
            </div>
            <div class="user-detail-time-item">
              <span class="user-detail-time__dot user-detail-time__dot--minor"></span>
              <div>
                <span class="user-detail-time__label">最后更新</span>
                <span class="user-detail-time__value">{{ formatTime(detailUser.updated_at) }}</span>
              </div>
            </div>
          </div>
        </div>
      </template>
    </el-drawer>

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

/* ——— 用户详情 Drawer ——— */

.user-detail-hero {
  display: flex;
  align-items: center;
  gap: 16px;
  padding-bottom: 20px;
  border-bottom: 1px solid #ebeef5;
  margin-bottom: 16px;
}
.user-detail-avatar {
  width: 64px;
  height: 64px;
  border-radius: 50%;
  object-fit: cover;
  border: 2px solid #e3e5e7;
  flex-shrink: 0;
}
.user-detail-identity {
  min-width: 0;
}
.user-detail-name {
  font-size: 18px;
  font-weight: 600;
  color: #18191c;
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}
.user-detail-sub {
  font-size: 13px;
  color: #9499a0;
  margin-top: 4px;
  word-break: break-all;
}
.user-detail-alert {
  margin-bottom: 16px;
}

/* 数据看板 */
.user-detail-stats {
  display: flex;
  gap: 12px;
  margin-bottom: 20px;
}
.user-detail-stat {
  flex: 1;
  text-align: center;
  background: #f6f7f8;
  border-radius: 8px;
  padding: 12px 8px;
}
.user-detail-stat__val {
  display: block;
  font-size: 20px;
  font-weight: 700;
  color: #18191c;
  line-height: 1.2;
}
.user-detail-stat__label {
  display: block;
  font-size: 12px;
  color: #9499a0;
  margin-top: 4px;
}

/* 分组区块 */
.user-detail-section {
  margin-bottom: 20px;
}
.user-detail-section__title {
  font-size: 13px;
  font-weight: 600;
  color: #9499a0;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin: 0 0 12px;
  padding-bottom: 8px;
  border-bottom: 1px solid #ebeef5;
}

/* 网格信息 */
.user-detail-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 6px 16px;
}
.user-detail-cell {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 0;
}
.user-detail-cell__label {
  font-size: 13px;
  color: #9499a0;
}
.user-detail-cell__value {
  font-size: 13px;
  color: #18191c;
  font-weight: 500;
}

/* 签名 */
.user-detail-sign {
  font-size: 13px;
  color: #61666d;
  line-height: 1.6;
  margin: 0;
  padding: 8px 12px;
  background: #f6f7f8;
  border-radius: 6px;
  white-space: pre-wrap;
  word-break: break-word;
}

/* 时间线 */
.user-detail-timeline {
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.user-detail-time-item {
  display: flex;
  align-items: flex-start;
  gap: 10px;
}
.user-detail-time__dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #00a1d6;
  margin-top: 5px;
  flex-shrink: 0;
}
.user-detail-time__dot--minor {
  background: #c0c4cc;
}
.user-detail-time__label {
  display: block;
  font-size: 12px;
  color: #9499a0;
}
.user-detail-time__value {
  display: block;
  font-size: 13px;
  color: #18191c;
  margin-top: 2px;
}
</style>
