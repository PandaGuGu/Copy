<template>
  <div class="ls-page">
    <h2 class="ls-title">直播间设置</h2>

    <el-form :model="form" label-width="100px" class="ls-form" v-loading="loading">
      <el-form-item label="直播标题">
        <el-input v-model="form.title" maxlength="60" />
      </el-form-item>
      <el-form-item label="封面图">
        <el-input v-model="form.cover_url" placeholder="https://..." />
      </el-form-item>
      <el-form-item>
        <el-button type="primary" :loading="saving" @click="handleSave">保存</el-button>
      </el-form-item>
    </el-form>

    <div class="ls-danger-zone">
      <h3>危险操作</h3>
      <el-button type="danger" text @click="regenerateKey" :loading="regenerating">
        重新生成串流密钥（旧密钥将立即失效）
      </el-button>
    </div>
  </div>
</template>

<script>
import { ref, reactive, onMounted } from "vue";
import { useRoute } from "vue-router";
import { ElMessage } from "element-plus";
import { getLiveRoom, updateLiveRoom, regenerateStreamKey } from "@/api/live";

export default {
  name: "LiveSettings",
  setup() {
    const route = useRoute();
    const loading = ref(true);
    const saving = ref(false);
    const regenerating = ref(false);

    const form = reactive({
      title: "",
      cover_url: ""
    });

    async function fetchRoom() {
      const roomId = Number(route.params.roomId) || 0;
      if (!roomId) return;
      try {
        const res = await getLiveRoom(roomId);
        const data = (res.data || res).data || res.data || res;
        if (data) {
          form.title = data.title || "";
          form.cover_url = data.cover_url || "";
        }
      } catch (e) {
        ElMessage.warning("加载直播间信息失败");
      } finally {
        loading.value = false;
      }
    }

    async function handleSave() {
      saving.value = true;
      const roomId = Number(route.params.roomId) || 0;
      try {
        await updateLiveRoom(roomId, { title: form.title, cover_url: form.cover_url });
        ElMessage.success("保存成功");
      } catch (e) {
        ElMessage.error("保存失败");
      } finally {
        saving.value = false;
      }
    }

    async function regenerateKey() {
      const roomId = Number(route.params.roomId) || 0;
      regenerating.value = true;
      try {
        await regenerateStreamKey(roomId);
        ElMessage.success("串流密钥已重新生成");
      } catch (e) {
        ElMessage.error("操作失败");
      } finally {
        regenerating.value = false;
      }
    }

    onMounted(() => fetchRoom());

    return { form, loading, saving, regenerating, handleSave, regenerateKey };
  }
};
</script>

<style scoped>
.ls-page {
  max-width: 560px;
  margin: 0 auto;
  padding: 30px 24px;
}
.ls-title {
  font-size: 20px;
  font-weight: 500;
  margin-bottom: 24px;
}
.ls-danger-zone {
  margin-top: 40px;
  padding: 20px;
  border: 1px solid var(--color-border-danger);
  border-radius: 8px;
}
.ls-danger-zone h3 {
  font-size: 14px;
  color: var(--color-text-danger);
  margin: 0 0 12px;
}
</style>
