<template>
  <CreatorShell>
    <div class="ap-wrap">
      <div class="ap-panel">
        <div class="ap-tabs">
          <button
            type="button"
            class="ap-tab"
            :class="{ on: tab === 'all' }"
            @click="tab = 'all'"
          >
            全部({{ totalCount }})
          </button>
          <button
            type="button"
            class="ap-tab"
            :class="{ on: tab === 'processing' }"
            @click="tab = 'processing'"
          >
            进行中({{ processingCount }})
          </button>
          <button
            type="button"
            class="ap-tab"
            :class="{ on: tab === 'completed' }"
            @click="tab = 'completed'"
          >
            已完成({{ completedCount }})
          </button>
          <span class="ap-tabs__spacer" />
          <button type="button" class="ap-new" @click="openNew">发起申诉</button>
        </div>

        <div v-if="filteredAppeals.length" class="ap-list">
          <div v-for="row in filteredAppeals" :key="row.id" class="ap-row">
            <div class="ap-row-body">
              <span class="ap-row-title">{{ appealSubject(row) }}</span>
              <span class="ap-row-content">{{ row.content }}</span>
              <span v-if="row.admin_note" class="ap-row-note">管理员备注：{{ row.admin_note }}</span>
            </div>
            <span class="ap-status" :class="'ap-status--' + row.status">
              {{ row.status_label || statusText(row.status) }}
            </span>
          </div>
        </div>

        <div v-else class="ap-empty">
          <img class="ap-empty-img" :src="emptyIllus" alt="" />
          <p class="ap-empty-txt">{{ emptyHint }}</p>
        </div>
      </div>
    </div>

    <!-- 发起申诉弹窗 -->
    <el-dialog
      :model-value="newVisible"
      title="发起申诉"
      width="520px"
      @update:model-value="newVisible = $event"
    >
      <div v-if="newVisible">
        <div class="apf-row">
          <label class="apf-label">申诉对象</label>
          <el-select v-model="form.target_type" style="width:100%">
            <el-option label="我的账号（封禁/警告）" value="user" />
            <el-option label="视频（下架）" value="video" />
            <el-option label="文章（下架）" value="article" />
            <el-option label="动态（被删）" value="dynamic" />
            <el-option label="评论（被删）" value="comment" />
          </el-select>
        </div>
        <div class="apf-row">
          <label class="apf-label">申诉类型</label>
          <el-select v-model="form.reason_type" style="width:100%">
            <el-option label="账号封禁" value="ban" />
            <el-option label="内容下架" value="takedown" />
            <el-option label="警告" value="warn" />
          </el-select>
        </div>
        <div class="apf-row">
          <label class="apf-label">
            目标 ID
            <span class="apf-hint">{{ form.target_type === 'user' ? '（我的账号，无需填写）' : '（内容数字 ID）' }}</span>
          </label>
          <el-input v-model.number="form.target_id" :disabled="form.target_type === 'user'" placeholder="请输入目标 ID" />
        </div>
        <div class="apf-row">
          <label class="apf-label">申诉说明</label>
          <el-input v-model="form.content" type="textarea" :rows="4" maxlength="2000" show-word-limit placeholder="请描述申诉理由..." />
        </div>
      </div>

      <template #footer>
        <el-button @click="newVisible = false">取消</el-button>
        <el-button type="primary" :loading="submitting" :disabled="!canSubmit" @click="submit">提交申诉</el-button>
      </template>
    </el-dialog>
  </CreatorShell>
</template>

<script>
import CreatorShell from "@/components/creator/CreatorShell.vue";
import { ElMessage } from "element-plus";
import { mbGetMe, mbListMyAppeals, mbPostAppeal } from "@/api/minibili";
import emptyIllus from "@/assets/err-no-list.716e40d2.png";

const targetTypeLabel = (t) => ({ user: "账号", video: "视频", article: "文章", dynamic: "动态", comment: "评论" }[t] || t);
const reasonLabel = (t) => ({ ban: "封禁", takedown: "下架", warn: "警告" }[t] || t);

export default {
  name: "AppealPage",
  components: { CreatorShell },
  data() {
    return {
      tab: "all",
      appeals: [],
      loading: false,
      emptyIllus,
      // New appeal dialog
      newVisible: false,
      submitting: false,
      meId: 0,
      defaultTargetType: "user",
      defaultReasonType: "ban",
      form: {
        target_type: "user",
        target_id: 0,
        reason_type: "ban",
        content: ""
      }
    };
  },
  computed: {
    totalCount() {
      return this.appeals.length;
    },
    processingCount() {
      return this.appeals.filter((a) => a.status === "pending").length;
    },
    completedCount() {
      return this.appeals.filter((a) => a.status === "approved" || a.status === "rejected").length;
    },
    filteredAppeals() {
      if (this.tab === "all") return this.appeals;
      if (this.tab === "processing") {
        return this.appeals.filter((a) => a.status === "pending");
      }
      return this.appeals.filter((a) => a.status === "approved" || a.status === "rejected");
    },
    emptyHint() {
      if (this.tab === "all" && this.totalCount === 0) {
        return '你还没有发起过申诉("▔□▔)/';
      }
      return '没有该类型的申诉("▔□▔)';
    },
    canSubmit() {
      const t = String(this.form.content || "").trim();
      if (!t) return false;
      if (this.form.target_type === "user") return true;
      return Number(this.form.target_id) > 0;
    }
  },
  async created() {
    await this.fetch();
    try {
      const me = await mbGetMe();
      this.meId = Number((me && me.id) || (me && me.user_id) || 0);
    } catch {
      /* ignore */
    }
  },
  methods: {
    async fetch() {
      this.loading = true;
      try {
        const d = await mbListMyAppeals();
        this.appeals = (d && d.items) || [];
      } catch (e) {
        ElMessage.error((e && e.message) || "加载申诉失败");
      } finally {
        this.loading = false;
      }
    },
    openNew() {
      this.form = {
        target_type: this.defaultTargetType,
        target_id: 0,
        reason_type: this.defaultReasonType,
        content: ""
      };
      this.newVisible = true;
    },
    async submit() {
      this.submitting = true;
      try {
        const body = {
          target_type: this.form.target_type,
          target_id: this.form.target_type === "user" ? this.meId : Number(this.form.target_id),
          reason_type: this.form.reason_type,
          content: String(this.form.content || "").trim()
        };
        if (!body.target_id) {
          ElMessage.warning("缺少目标 ID");
          return;
        }
        await mbPostAppeal(body);
        ElMessage.success("申诉已提交，请等待处理");
        this.newVisible = false;
        await this.fetch();
      } catch (e) {
        ElMessage.error((e && e.message) || "提交失败");
      } finally {
        this.submitting = false;
      }
    },
    appealSubject(row) {
      return `${targetTypeLabel(row.target_type)} · ${reasonLabel(row.reason_type)} 申诉`;
    },
    statusText(s) {
      return { pending: "进行中", approved: "已完成", rejected: "已完成" }[s] || s;
    }
  }
};
</script>

<style lang="scss" scoped>
$c-blue: #00a1d6;
$c-text: #18191c;
$c-sub: #99a2aa;
$c-line: #e3e5e7;

.ap-wrap {
  max-width: 1120px;
  margin: 0 auto;
}

.ap-panel {
  background: #fff;
  border: 1px solid $c-line;
  border-radius: 8px;
  min-height: 420px;
  box-sizing: border-box;
}

.ap-tabs {
  display: flex;
  align-items: center;
  gap: 28px;
  padding: 0 24px;
  border-bottom: 1px solid $c-line;
}
.ap-tabs__spacer {
  flex: 1;
}
.ap-new {
  padding: 7px 18px;
  border: none;
  border-radius: 4px;
  background: $c-blue;
  color: #fff;
  font-size: 13px;
  cursor: pointer;
}
.ap-new:hover {
  filter: brightness(1.05);
}

.ap-tab {
  position: relative;
  padding: 16px 2px 14px;
  margin-bottom: -1px;
  border: none;
  background: none;
  font-size: 15px;
  color: #505050;
  cursor: pointer;
  border-bottom: 3px solid transparent;
}
.ap-tab:hover {
  color: $c-blue;
}
.ap-tab.on {
  color: $c-blue;
  font-weight: 600;
  border-bottom-color: $c-blue;
}

.ap-list {
  padding: 16px 24px 24px;
}

.ap-row {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
  padding: 14px 0;
  border-bottom: 1px solid #f0f1f2;
  font-size: 14px;
}
.ap-row:last-child {
  border-bottom: none;
}

.ap-row-body {
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-width: 0;
}
.ap-row-title {
  color: $c-text;
  font-weight: 600;
}
.ap-row-content {
  color: #61666d;
  font-size: 13px;
}
.ap-row-note {
  color: $c-sub;
  font-size: 12px;
}

.ap-status {
  flex-shrink: 0;
  font-size: 13px;
  color: $c-sub;
}
.ap-status--pending {
  color: #e6a23c;
}
.ap-status--approved {
  color: #67c23a;
}
.ap-status--rejected {
  color: #f56c6c;
}

.ap-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 48px 24px 64px;
}
.ap-empty-img {
  width: 280px;
  max-width: 86%;
  height: auto;
  object-fit: contain;
  display: block;
}
.ap-empty-txt {
  margin: 20px 0 0;
  font-size: 14px;
  color: $c-sub;
  text-align: center;
  line-height: 1.6;
}

.apf-row {
  margin-bottom: 14px;
}
.apf-label {
  display: block;
  margin-bottom: 6px;
  font-size: 13px;
  font-weight: 600;
  color: $c-text;
}
.apf-hint {
  font-weight: 400;
  color: $c-sub;
}
</style>