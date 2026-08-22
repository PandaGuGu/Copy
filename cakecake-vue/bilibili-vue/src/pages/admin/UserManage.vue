<template>
  <div class="adm-page">
    <div class="adm-page__header">
      <h2 class="adm-page__title">用户管理</h2>
    </div>

    <AdminDataTable
      :data="users"
      :loading="loading"
      :page="page"
      :page-size="pageSize"
      :total="total"
      @update:page="page = $event; loadUsers()"
    >
      <template #search-bar>
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
          style="width: 260px"
          clearable
          @keyup.enter="doSearch"
        />
        <el-button type="primary" size="small" @click="doSearch">搜索</el-button>
      </template>

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
      <el-table-column label="操作" width="320" fixed="right">
        <template #default="{ row }">
          <el-button type="primary" size="small" link @click="viewDetail(row)">详情</el-button>
          <el-button type="primary" size="small" link @click="openCapabilities(row)">能力</el-button>
          <template v-if="row.status === 'banned'">
            <el-button type="success" size="small" link @click="confirmUnban(row)">解封</el-button>
          </template>
          <template v-else>
            <el-button type="warning" size="small" link @click="confirmBan(row)">封禁</el-button>
          </template>
          <el-button type="danger" size="small" link @click="confirmDelete(row)">强制注销</el-button>
        </template>
      </el-table-column>
    </AdminDataTable>

    <!-- 用户详情弹窗 -->
    <el-dialog v-model="detailVisible" title="用户详情" width="620px" destroy-on-close top="5vh">
      <template v-if="detailUser">
        <!-- 头像 + 身份区 -->
        <div class="user-detail-hero">
          <img
            v-if="detailUser.avatar_url"
            :src="detailUser.avatar_url"
            class="user-detail-avatar"
          />
          <div class="user-detail-identity">
            <div class="user-detail-name">
              {{ detailUser.nickname || detailUser.username }}
              <el-tag
                v-if="detailUser.status === 'banned'"
                type="danger"
                size="small"
                effect="dark"
                >已封禁</el-tag
              >
              <el-tag
                v-else-if="detailUser.status === 'disabled'"
                type="info"
                size="small"
                effect="dark"
                >已禁用</el-tag
              >
            </div>
            <div class="user-detail-sub">@{{ detailUser.username }} · CakeID: {{ detailUser.cake_id }}</div>
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
            <span class="user-detail-stat__val">{{ detailUser.follower_count || 0 }}</span>
            <span class="user-detail-stat__label">粉丝</span>
          </div>
          <div class="user-detail-stat">
            <span class="user-detail-stat__val" :class="{ 'user-detail-stat__warn': (detailUser.report_count || 0) > 0 }">
              {{ detailUser.report_count || 0 }}
            </span>
            <span class="user-detail-stat__label">被举报</span>
          </div>
        </div>

        <!-- 违规记录 -->
        <div class="user-detail-section">
          <h4 class="user-detail-section__title">
            违规记录
            <el-tag v-if="vioLoading" size="small" type="info">加载中...</el-tag>
          </h4>
          <div v-if="!violations || violations.length === 0" class="user-detail-empty">暂无违规记录</div>
          <div v-else class="user-detail-violation-list">
            <div
              v-for="v in violations"
              :key="v.id"
              class="user-detail-violation-item"
            >
              <div class="user-detail-violation__head">
                <span class="user-detail-violation__from">
                  被 {{ v.reporter ? (v.reporter.nickname || v.reporter.username) : '#' + v.reporter_id }} 举报
                </span>
                <el-tag
                  :type="v.status === 'pending' ? 'warning' : v.status === 'resolved' ? 'success' : 'info'"
                  size="small"
                  effect="plain"
                >
                  {{ v.status === 'pending' ? '待处理' : v.status === 'resolved' ? '已处理' : '已驳回' }}
                </el-tag>
              </div>
              <p class="user-detail-violation__reason">
              <el-tag size="small" effect="dark" :color="viceColor(v.reason_type)" style="margin-right:5px">
                {{ v.reason_label || v.reason_type }}
              </el-tag>
              {{ v.reason_detail }}
            </p>
              <span class="user-detail-violation__time">{{ fmtTime(v.created_at) }}</span>
            </div>
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
              <span class="user-detail-cell__label">硬币</span>
              <span class="user-detail-cell__value">{{ detailUser.coin_balance || 0 }}</span>
            </div>
            <div class="user-detail-cell">
              <span class="user-detail-cell__label">经验</span>
              <span class="user-detail-cell__value">{{ detailUser.experience || 0 }}</span>
            </div>
            <div class="user-detail-cell">
              <span class="user-detail-cell__label">性别</span>
              <span class="user-detail-cell__value">{{ detailUser.gender || "未设置" }}</span>
            </div>
            <div class="user-detail-cell">
              <span class="user-detail-cell__label">生日</span>
              <span class="user-detail-cell__value">{{ detailUser.birthday || "未设置" }}</span>
            </div>
            <div class="user-detail-cell">
              <span class="user-detail-cell__label">注册时间</span>
              <span class="user-detail-cell__value">{{ formatTime(detailUser.created_at) }}</span>
            </div>
            <div class="user-detail-cell">
              <span class="user-detail-cell__label">最后更新</span>
              <span class="user-detail-cell__value">{{ formatTime(detailUser.updated_at) }}</span>
            </div>
          </div>
        </div>

        <!-- 个性签名 -->
        <div class="user-detail-section" v-if="detailUser.sign">
          <h4 class="user-detail-section__title">个性签名</h4>
          <p class="user-detail-sign">{{ detailUser.sign }}</p>
        </div>
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

    <!-- 能力限制管理 -->
    <el-dialog v-model="capVisible" :title="`能力限制：${capUser ? (capUser.nickname || capUser.username) : ''}`" width="560px" destroy-on-close>
      <div v-if="capUser">
        <p class="cap-head-desc">可单独限制用户某项功能；被限制的能力用户将无法使用，但账号仍可正常登录其它功能。</p>
        <div v-loading="capLoading" class="cap-grid">
          <div v-for="c in capItems" :key="c.name" class="cap-item">
            <div class="cap-item__main">
              <div class="cap-item__label">{{ c.label }}</div>
              <div v-if="c.activeReason" class="cap-item__reason">{{ c.activeReason }}</div>
            </div>
            <el-switch
              :model-value="c.active"
              :loading="c.saving"
              @change="(v) => toggleCap(c, v)"
            />
          </div>
        </div>
        <div class="cap-reason">
          <div class="cap-reason__head">
            <div class="cap-reason__label">限制原因</div>
            <el-button size="small" link type="primary" @click="capTplManageVisible = !capTplManageVisible">
              {{ capTplManageVisible ? '收起模板管理' : '管理模板' }}
            </el-button>
          </div>
          <div v-if="!capTplManageVisible" class="cap-reason__tpls">
            <span
              v-for="t in capReasonTemplates"
              :key="t"
              class="cap-reason__tpl"
              :class="{ 'is-on': capReason === t }"
              @click="capReason = t"
            >{{ t }}</span>
          </div>
          <div v-else class="cap-tpl-manage">
            <div class="cap-tpl-row">
              <el-input v-model="capTplInput" placeholder="输入新模板，回车添加" size="small" @keyup.enter="addCapTpl" />
              <el-button type="primary" size="small" @click="addCapTpl">添加</el-button>
            </div>
            <div v-for="tpl in capTplItems" :key="tpl.id" class="cap-tpl-row">
              <template v-if="tpl.editing">
                <el-input v-model="tpl.draft" size="small" @keyup.enter="saveCapTpl(tpl)" />
                <el-button type="primary" size="small" @click="saveCapTpl(tpl)">保存</el-button>
                <el-button size="small" @click="tpl.editing = false">取消</el-button>
              </template>
              <template v-else>
                <span class="cap-tpl-text">{{ tpl.content }}</span>
                <el-button size="small" link type="primary" @click="startEditCapTpl(tpl)">改</el-button>
                <el-button size="small" link type="danger" @click="delCapTpl(tpl)">删</el-button>
              </template>
            </div>
          </div>
          <el-input
            v-model="capReason"
            type="textarea"
            :rows="2"
            placeholder="开启任何一项能力限制前，请先在此填写原因（开启时必填；可点上方模板快速填入）"
            style="margin-top: 8px"
          />
          <div class="cap-reason__hint">此原因会展示给该用户，并记录在后台可追溯。</div>
        </div>
      </div>
      <template #footer>
        <el-button @click="capVisible = false">关闭</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script>
import { adminListUsers, adminGetUser, adminBanUser, adminUnbanUser, adminDeleteUser, adminGetUserViolations, adminListUserCapabilities, adminRestrictUserCapability, adminRestoreUserCapability, adminListCapReasonTemplates, adminAddCapReasonTemplate, adminUpdateCapReasonTemplate, adminDeleteCapReasonTemplate } from "@/api/admin";
import AdminDataTable from "@/components/admin/AdminDataTable.vue";

export default {
  name: "AdminUserManage",
  components: { AdminDataTable },
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
      violations: [],
      vioLoading: false,
      // ban
      banVisible: false,
      banTarget: null,
      banReason: "",
      banning: false,
      // delete
      deleteVisible: false,
      deleteTarget: null,
      deleting: false,
      // capabilities
      capVisible: false,
      capUser: null,
      capLoading: false,
      capItems: [],
      capReason: "",
      capReasonTemplates: [],
      capTplManageVisible: false,
      capTplItems: [],
      capTplInput: ""
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
        this.violations = [];
        this.loadViolations(row.id);
      } catch {
        this.$message.error("获取用户详情失败");
      }
    },
    async loadViolations(uid) {
      this.vioLoading = true;
      try {
        const d = await adminGetUserViolations(uid);
        this.violations = (d.data && d.data.reports) || [];
      } catch {
        this.violations = [];
      } finally {
        this.vioLoading = false;
      }
    },
    fmtTime(ts) {
      if (!ts) return "";
      return new Date(ts).toLocaleString("zh-CN");
    },
    viceColor(t) {
      const m = { nsfw: "#e6a23c", violence: "#f56c6c", spam: "#909399", harassment: "#e6a23c", illegal: "#f56c6c", copyright: "#409eff" };
      return m[t] || "#909399";
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
    async openCapabilities(row) {
      this.capUser = row;
      this.capVisible = true;
      this.capReason = "";
      this.capLoading = true;
      try {
        const d = await adminListUserCapabilities(row.id);
        const data = (d && d.data) || d || {};
        const activeMap = {};
        (data.active || []).forEach((c) => { activeMap[c] = true; });
        const reasons = {};
        (data.items || []).forEach((it) => {
          if (it.status === "active" && it.reason) reasons[it.capability] = it.reason;
        });
        const caps = data.capabilities || [];
        this.capItems = caps.map((c) => ({
          name: c.name,
          label: c.label,
          active: !!activeMap[c.name],
          activeReason: reasons[c.name] || "",
          saving: false
        }));
        this.capReasonTemplates = data.reason_templates || [];
      } catch {
        this.$message.error("加载能力信息失败");
        this.capItems = [];
      } finally {
        this.capLoading = false;
      }
      this.loadCapTpls();
    },
    async loadCapTpls() {
      try {
        const d = await adminListCapReasonTemplates();
        const data = (d && d.data) || d || {};
        this.capTplItems = (data.templates || []).map((t) => ({
          id: t.id, content: t.content, editing: false, draft: ""
        }));
        this.capReasonTemplates = this.capTplItems.map((t) => t.content);
      } catch {
        /* ignore */
      }
    },
    async addCapTpl() {
      const c = String(this.capTplInput || "").trim();
      if (!c) { this.$message.warning("请输入模板内容"); return; }
      try {
        await adminAddCapReasonTemplate(c);
        this.capTplInput = "";
        this.$message.success("已添加");
        await this.loadCapTpls();
      } catch {
        this.$message.error("添加失败");
      }
    },
    startEditCapTpl(tpl) {
      tpl.draft = tpl.content;
      tpl.editing = true;
    },
    async saveCapTpl(tpl) {
      const c = String(tpl.draft || "").trim();
      if (!c) { this.$message.warning("内容不能为空"); return; }
      try {
        await adminUpdateCapReasonTemplate(tpl.id, c);
        this.$message.success("已保存");
        await this.loadCapTpls();
      } catch {
        this.$message.error("保存失败");
      }
    },
    async delCapTpl(tpl) {
      try {
        await adminDeleteCapReasonTemplate(tpl.id);
        this.$message.success("已删除");
        await this.loadCapTpls();
      } catch {
        this.$message.error("删除失败");
      }
    },
    async toggleCap(item, on) {
      if (!this.capUser) return;
      if (on && !String(this.capReason || "").trim()) {
        this.$message.warning(`请先在下方填写限制原因，再开启「${item.label}」`);
        this.capItems = this.capItems.map((c) => ({ ...c, active: c.active }));
        return;
      }
      item.saving = true;
      try {
        if (on) {
          await adminRestrictUserCapability(this.capUser.id, { capability: item.name, reason: String(this.capReason).trim() });
          item.activeReason = String(this.capReason).trim();
          this.capReason = "";
        } else {
          await adminRestoreUserCapability(this.capUser.id, item.name);
          item.activeReason = "";
        }
        item.active = on;
        this.$message.success(on ? "已限制" : "已恢复");
      } catch {
        this.$message.error("操作失败");
      } finally {
        item.saving = false;
      }
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

/* ——— 用户详情弹窗 ——— */

.user-detail-hero {
  display: flex;
  align-items: center;
  gap: 16px;
  padding-bottom: 16px;
  border-bottom: 1px solid #ebeef5;
  margin-bottom: 14px;
}
.user-detail-avatar {
  width: 56px;
  height: 56px;
  border-radius: 50%;
  object-fit: cover;
  border: 2px solid #e3e5e7;
  flex-shrink: 0;
}
.user-detail-identity { min-width: 0; }
.user-detail-name {
  font-size: 17px;
  font-weight: 600;
  color: #18191c;
  display: flex;
  align-items: center;
  gap: 8px;
}
.user-detail-sub { font-size: 13px; color: #9499a0; margin-top: 4px; }
.user-detail-alert { margin-bottom: 14px; }

.user-detail-stats {
  display: flex;
  gap: 10px;
  margin-bottom: 16px;
}
.user-detail-stat {
  flex: 1;
  text-align: center;
  background: #f6f7f8;
  border-radius: 8px;
  padding: 10px 6px;
}
.user-detail-stat__val { font-size: 18px; font-weight: 700; color: #18191c; }
.user-detail-stat__warn { color: #e6a23c !important; }
.user-detail-stat__label { font-size: 12px; color: #9499a0; }

.user-detail-section { margin-bottom: 14px; }
.user-detail-section__title {
  font-size: 13px; font-weight: 600; color: #9499a0;
  margin: 0 0 10px; padding-bottom: 6px;
  border-bottom: 1px solid #ebeef5;
  display: flex; align-items: center; gap: 8px;
}
.user-detail-empty { font-size: 13px; color: #c0c4cc; padding: 12px 0; }

.user-detail-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 4px 16px; }
.user-detail-cell {
  display: flex; justify-content: space-between; align-items: center;
  padding: 6px 0;
}
.user-detail-cell__label { font-size: 13px; color: #9499a0; }
.user-detail-cell__value { font-size: 13px; color: #18191c; font-weight: 500; }

.user-detail-sign {
  font-size: 13px; color: #61666d; line-height: 1.6;
  padding: 8px 12px; background: #f6f7f8; border-radius: 6px;
  white-space: pre-wrap; word-break: break-word;
}

/* 违规记录列表 */
.user-detail-violation-list {
  max-height: 240px;
  overflow-y: auto;
  display: flex; flex-direction: column; gap: 8px;
}
.user-detail-violation-item {
  padding: 10px 12px;
  background: #fafafa;
  border-radius: 6px;
  border: 1px solid #eee;
}
.user-detail-violation__head {
  display: flex; justify-content: space-between; align-items: center;
  margin-bottom: 4px;
}
.user-detail-violation__from { font-size: 13px; font-weight: 500; color: #18191c; }
.user-detail-violation__reason {
  font-size: 12px; color: #61666d; line-height: 1.5;
  margin: 0; word-break: break-word;
}
.user-detail-violation__time { font-size: 11px; color: #c0c4cc; }

/* ——— 能力限制管理 ——— */
.cap-head-desc { font-size: 13px; color: #61666d; line-height: 1.6; margin: 0 0 14px; }
.cap-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 10px;
  min-height: 120px;
}
.cap-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 12px 14px;
  background: #f6f7f8;
  border: 1px solid #e9ecef;
  border-radius: 8px;
}
.cap-item__main { min-width: 0; }
.cap-item__label { font-size: 14px; font-weight: 600; color: #18191c; }
.cap-item__reason {
  font-size: 12px; color: #e6a23c; margin-top: 2px;
  white-space: nowrap; overflow: hidden; text-overflow: ellipsis; max-width: 160px;
}

.cap-reason { margin-top: 14px; }
.cap-reason__head { display: flex; align-items: center; justify-content: space-between; margin-bottom: 6px; }
.cap-reason__label { font-size: 13px; font-weight: 600; color: #18191c; }
.cap-reason__tpls { display: flex; flex-wrap: wrap; gap: 6px; margin-bottom: 8px; }
.cap-reason__tpl {
  font-size: 12px;
  padding: 4px 10px;
  border-radius: 12px;
  border: 1px solid #e3e5e7;
  background: #fafafa;
  color: #61666d;
  cursor: pointer;
  transition: all .15s;
  user-select: none;
}
.cap-reason__tpl:hover { border-color: #00a1d6; color: #00a1d6; background: #f0faff; }
.cap-reason__tpl.is-on { border-color: #00a1d6; color: #00a1d6; background: #e6f7ff; font-weight: 600; }
.cap-reason__hint { font-size: 12px; color: #9499a0; margin-top: 6px; }

.cap-tpl-manage {
  border: 1px solid #ebeef5;
  border-radius: 8px;
  padding: 10px;
  background: #fafafa;
  margin-bottom: 8px;
}
.cap-tpl-row {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-bottom: 6px;
}
.cap-tpl-row:last-child { margin-bottom: 0; }
.cap-tpl-row .el-input { flex: 1; }
.cap-tpl-text {
  flex: 1;
  font-size: 13px;
  color: #18191c;
  word-break: break-all;
}
</style>
