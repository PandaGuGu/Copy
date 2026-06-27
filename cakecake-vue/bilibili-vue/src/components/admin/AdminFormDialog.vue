<template>
  <el-dialog
    v-model="visible"
    :title="isEdit ? `编辑${entityLabel}` : `新增${entityLabel}`"
    :width="width"
    destroy-on-close
    :close-on-click-modal="false"
  >
    <slot :saving="saving" :ok="doSave" :cancel="close" />
    <template #footer>
      <el-button @click="close">取消</el-button>
      <el-button type="primary" :loading="saving" @click="doSave">保存</el-button>
    </template>
  </el-dialog>
</template>

<script>
export default {
  name: 'AdminFormDialog',
  props: {
    modelValue: { type: Boolean, default: false },
    entityLabel: { type: String, default: '' },
    isEdit: { type: Boolean, default: false },
    width: { type: String, default: '480px' },
    onSave: { type: Function, required: true }
  },
  emits: ['update:modelValue', 'saved'],
  data() {
    return { saving: false };
  },
  computed: {
    visible: {
      get() { return this.modelValue; },
      set(v) { if (!this.saving) this.$emit('update:modelValue', v); }
    }
  },
  methods: {
    close() {
      this.$emit('update:modelValue', false);
    },
    async doSave() {
      this.saving = true;
      try {
        await this.onSave();
        this.$emit('saved');
        this.close();
      } catch (e) {
        // error handled by parent or adminHttp interceptor
      } finally {
        this.saving = false;
      }
    }
  }
};
</script>
