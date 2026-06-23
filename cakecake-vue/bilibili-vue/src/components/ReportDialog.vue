<template>
  <el-dialog
    :model-value="visible"
    title="举报内容"
    width="440px"
    destroy-on-close
    top="10vh"
    @update:model-value="$emit('update:visible', $event)"
  >
    <div class="rp-dlg">
      <div class="rp-dlg__target" v-if="targetLabel">
        举报对象：<strong>{{ targetLabel }}</strong>
      </div>

      <div class="rp-dlg__types">
        <label class="rp-dlg__label">举报类型</label>
        <div class="rp-dlg__type-grid">
          <button
            v-for="rt in reasonTypes"
            :key="rt.type"
            type="button"
            class="rp-dlg__type-btn"
            :class="{ 'is-active': selectedType === rt.type }"
            @click="selectedType = rt.type"
          >
            <span class="rp-dlg__type-icon">{{ rt.icon }}</span>
            <span class="rp-dlg__type-text">{{ rt.label }}</span>
          </button>
        </div>
      </div>

      <div class="rp-dlg__detail">
        <label class="rp-dlg__label">补充说明（选填）</label>
        <el-input
          v-model="detail"
          type="textarea"
          :rows="3"
          maxlength="500"
          show-word-count
          placeholder="可在此补充详细描述..."
        />
      </div>

      <div class="rp-dlg__actions">
        <el-button @click="$emit('update:visible', false)">取消</el-button>
        <el-button type="danger" :disabled="!selectedType" :loading="submitting" @click="submit">
          提交举报
        </el-button>
      </div>
    </div>
  </el-dialog>
</template>

<script>
import { ElMessage } from "element-plus";
import http from "@/utils/http";

const reasonTypes = [
  { type: "nsfw", label: "色情低俗", icon: "🔞" },
  { type: "violence", label: "暴力血腥", icon: "🩸" },
  { type: "spam", label: "垃圾广告", icon: "📢" },
  { type: "harassment", label: "引战谩骂", icon: "🔥" },
  { type: "illegal", label: "违法信息", icon: "⚖️" },
  { type: "copyright", label: "侵权投诉", icon: "©️" },
  { type: "other", label: "其他", icon: "📌" },
];

export default {
  name: "ReportDialog",
  props: {
    visible: { type: Boolean, default: false },
    targetType: { type: String, default: "video" },
    targetId: { type: [Number, String], default: 0 },
    targetLabel: { type: String, default: "" },
  },
  emits: ["update:visible", "submitted"],
  data() {
    return { reasonTypes, selectedType: "", detail: "", submitting: false };
  },
  watch: {
    visible(val) {
      if (val) {
        this.selectedType = "";
        this.detail = "";
      }
    },
  },
  methods: {
    async submit() {
      if (!this.selectedType) return;
      this.submitting = true;
      try {
        await http.post("/api/v1/reports", {
          target_type: this.targetType,
          target_id: Number(this.targetId),
          reason_type: this.selectedType,
          reason_detail: this.detail.trim(),
        });
        ElMessage.success("举报已提交，感谢您的反馈");
        this.$emit("update:visible", false);
        this.$emit("submitted");
      } catch (e) {
        ElMessage.error((e && e.message) || "提交失败，请稍后重试");
      } finally {
        this.submitting = false;
      }
    },
  },
};
</script>

<style scoped>
.rp-dlg { display: flex; flex-direction: column; gap: 18px; }
.rp-dlg__target { font-size: 13px; color: #61666d; }
.rp-dlg__label { font-size: 13px; font-weight: 600; color: #18191c; display: block; margin-bottom: 8px; }
.rp-dlg__type-grid { display: grid; grid-template-columns: repeat(3, 1fr); gap: 8px; }
.rp-dlg__type-btn {
  display: flex; flex-direction: column; align-items: center; gap: 4px;
  padding: 12px 8px;
  border: 2px solid #e3e5e7; border-radius: 8px;
  background: #fff; cursor: pointer; transition: all .15s;
  font-family: inherit;
}
.rp-dlg__type-btn:hover { border-color: #00a1d6; background: #f0fafe; }
.rp-dlg__type-btn.is-active { border-color: #00a1d6; background: #e6f7ff; }
.rp-dlg__type-icon { font-size: 22px; }
.rp-dlg__type-text { font-size: 12px; color: #61666d; font-weight: 500; }
.rp-dlg__type-btn.is-active .rp-dlg__type-text { color: #00a1d6; }
.rp-dlg__actions { display: flex; gap: 8px; justify-content: flex-end; padding-top: 4px; }
</style>
