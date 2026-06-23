<template>
  <div class="st-page" v-loading="loading">
    <header class="st-page__head">
      <h2 class="st-page__title">系统设置</h2>
      <p class="st-page__desc">管理 .env 中的运行时开关和参数，保存后即时生效</p>
    </header>

    <el-card shadow="never" class="st-section">
      <template #header>
        <div class="st-section__header">
          <span class="st-section__title">内容审核开关</span>
        </div>
      </template>
      <el-form label-width="180px">
        <el-form-item label="视频上传禁用">
          <el-switch v-model="form.video_upload_disabled" active-text="禁用" inactive-text="正常" />
          <p class="st-hint">开启后用户无法上传新视频，已上传的不受影响</p>
        </el-form-item>
        <el-form-item label="视频审核模式">
          <el-switch v-model="form.video_review_required" active-text="需审核" inactive-text="直接发布" />
          <p class="st-hint">关闭后视频上传成功即发布，无需管理员审核</p>
        </el-form-item>
        <el-form-item label="文章审核模式">
          <el-switch v-model="form.article_review_required" active-text="需审核" inactive-text="直接发布" />
          <p class="st-hint">关闭后文章发布即上线，无需管理员审核</p>
        </el-form-item>
      </el-form>
    </el-card>

    <el-card shadow="never" class="st-section">
      <template #header>
        <div class="st-section__header">
          <span class="st-section__title">AI 助手配置</span>
        </div>
      </template>
      <el-form label-width="180px">
        <el-form-item label="AI 助手开关">
          <el-switch v-model="form.agent_enabled" active-text="启用" inactive-text="停用" />
          <p class="st-hint">关闭后用户无法与 AI 助手对话</p>
        </el-form-item>
        <el-form-item label="每日配额">
          <el-input-number v-model="form.agent_daily_quota" :min="1" :max="9999" />
          <p class="st-hint">每个用户每天可发送的消息数上限</p>
        </el-form-item>
        <el-form-item label="对话历史条数">
          <el-input-number v-model="form.agent_max_history" :min="1" :max="200" />
          <p class="st-hint">每次对话保留的历史消息轮数</p>
        </el-form-item>
        <el-form-item label="历史有效期">
          <el-input v-model="form.agent_history_ttl" style="width:180px" placeholder="720h" />
          <p class="st-hint">格式如 720h / 48h / 168h（小时）</p>
        </el-form-item>
        <el-form-item label="请求超时">
          <el-input v-model="form.agent_request_timeout" style="width:180px" placeholder="90s" />
          <p class="st-hint">格式如 90s / 120s / 3m</p>
        </el-form-item>
      </el-form>
    </el-card>

    <div class="st-actions">
      <el-button type="primary" :loading="saving" @click="save">保存设置</el-button>
    </div>
  </div>
</template>

<script>
import { ElMessage } from "element-plus";
import { adminGetSettings, adminPutSettings } from "@/api/admin";

export default {
  name: "Settings",
  data() {
    return {
      loading: false,
      saving: false,
      form: {
        video_upload_disabled: false,
        video_review_required: true,
        article_review_required: true,
        agent_enabled: false,
        agent_daily_quota: 80,
        agent_max_history: 20,
        agent_history_ttl: "720h",
        agent_request_timeout: "90s",
      },
    };
  },
  created() {
    this.load();
  },
  methods: {
    async load() {
      this.loading = true;
      try {
        const body = await adminGetSettings();
        const d = (body && body.data) || {};
        Object.keys(this.form).forEach((k) => {
          if (d[k] !== undefined) this.form[k] = d[k];
        });
      } catch (e) {
        ElMessage.error((e && e.message) || "加载失败");
      } finally {
        this.loading = false;
      }
    },
    async save() {
      this.saving = true;
      try {
        const payload = {};
        Object.keys(this.form).forEach((k) => {
          if (typeof this.form[k] === "boolean") {
            payload[k] = this.form[k];
          } else if (typeof this.form[k] === "number") {
            payload[k] = this.form[k];
          } else if (typeof this.form[k] === "string" && this.form[k]) {
            payload[k] = this.form[k];
          }
        });
        await adminPutSettings(payload);
        ElMessage.success("设置已保存");
      } catch (e) {
        ElMessage.error((e && e.message) || "保存失败");
      } finally {
        this.saving = false;
      }
    },
  },
};
</script>

<style scoped>
.st-page { padding: 20px 24px; max-width: 720px; }
.st-page__head { margin-bottom: 20px; }
.st-page__title { margin: 0 0 4px; font-size: 18px; font-weight: 600; color: #18191c; }
.st-page__desc { margin: 0; font-size: 13px; color: #9499a0; }
.st-section { margin-bottom: 16px; }
.st-section__header { display: flex; align-items: center; gap: 8px; }
.st-section__title { font-size: 14px; font-weight: 600; color: #18191c; }
.st-hint { margin: 4px 0 0; font-size: 12px; color: #9499a0; line-height: 1.5; }
.st-actions { padding-top: 8px; }
</style>
