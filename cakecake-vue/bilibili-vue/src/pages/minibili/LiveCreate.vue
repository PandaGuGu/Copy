<template>
  <div class="live-create-page">
    <h2 class="lc-title">开播设置</h2>

    <div v-loading="loading" class="lc-body">
      <el-form :model="form" label-width="80px" class="lc-form">
        <el-form-item label="直播标题">
          <el-input v-model="form.title" maxlength="60" placeholder="给你的直播取个名字" @blur="saveTitle" />
        </el-form-item>

        <el-form-item label="封面图">
          <div class="lc-cover-box">
            <div class="lc-cover-preview" v-if="form.cover_url">
              <img :src="form.cover_url" class="lc-cover-img" />
              <el-button class="lc-cover-remove" size="small" type="danger" circle @click="removeCover">&#x2715;</el-button>
            </div>
            <div class="lc-cover-empty" v-else @click="triggerUpload">
              <span class="lc-cover-icon">+</span>
              <span class="lc-cover-text">点击上传封面</span>
            </div>
            <input
              ref="fileInput"
              type="file"
              accept="image/jpeg,image/png,image/gif,image/webp"
              style="display:none"
              @change="onFileChange"
            />
            <div class="lc-cover-actions" v-if="!form.cover_url">
              <el-button size="small" type="primary" plain @click="triggerUpload" :loading="uploading">
                从电脑选择图片
              </el-button>
            </div>
          </div>
        </el-form-item>
      </el-form>

      <!-- 推流信息（系统分配，进入即就绪） -->
      <div class="lc-stream-card">
        <div class="lc-stream-head">
          <h3>推流信息</h3>
          <span class="lc-stream-badge">已就绪</span>
        </div>
        <p class="lc-stream-desc">以下信息已自动生成，复制到 OBS 即可开播</p>

        <div class="lc-stream-item">
          <span class="lc-stream-label">推流地址</span>
          <div class="lc-stream-value-row">
            <code class="lc-stream-value">{{ obsServer }}</code>
            <el-button size="small" text @click="copyText(obsServer)">复制</el-button>
          </div>
        </div>

        <div class="lc-stream-item">
          <span class="lc-stream-label">推流密钥</span>
          <div class="lc-stream-value-row">
            <code class="lc-stream-value">{{ maskedKey }}</code>
            <el-button size="small" text @click="copyText(form.stream_key)">复制</el-button>
            <el-button size="small" text type="warning" @click="regenerateKey" :loading="regenerating">重置</el-button>
          </div>
        </div>

        <details class="lc-stream-details">
          <summary>不知道怎么用？</summary>
          <ol class="lc-stream-steps">
            <li>下载并安装 <a href="https://obsproject.com" target="_blank">OBS Studio</a>（免费）</li>
            <li>打开 OBS → 右下角"设置" → 左侧"推流"</li>
            <li>服务选择"自定义"，粘贴上方<strong>推流地址</strong>和<strong>推流密钥</strong></li>
            <li>回到主界面点"开始推流"，你的直播就上线了</li>
          </ol>
        </details>
      </div>
    </div>
  </div>
</template>

<script>
import { ref, reactive, onMounted, computed } from "vue";
import { ElMessage } from "element-plus";
import { getMyLiveRoom, regenerateStreamKey, uploadLiveCover, updateLiveRoom } from "@/api/live";

export default {
  name: "LiveCreate",
  setup() {
    const form = reactive({
      id: 0,
      title: "",
      cover_url: "",
      stream_key: ""
    });
    const fileInput = ref(null);
    const loading = ref(true);
    const uploading = ref(false);
    const regenerating = ref(false);
    const obsServer = ref("rtmp://localhost:1935/live");

    const maskedKey = computed(() => {
      const k = form.stream_key;
      if (!k || k.length <= 8) return k || "加载中...";
      return k.slice(0, 4) + "****" + k.slice(-4);
    });

    async function initRoom() {
      loading.value = true;
      try {
        const res = await getMyLiveRoom();
        const data = (res.data || res).data || res.data || res;
        if (data) {
          form.id = data.id;
          form.title = data.title || "未命名直播间";
          form.cover_url = data.cover_url || "";
          form.stream_key = data.stream_key || "";
        }
      } catch (e) {
        ElMessage.error("加载失败，请刷新重试");
      } finally {
        loading.value = false;
      }
    }

    async function saveTitle() {
      if (!form.id || !form.title.trim()) return;
      try {
        await updateLiveRoom(form.id, { title: form.title.trim() });
      } catch (e) { /* 静默 */ }
    }

    function triggerUpload() {
      fileInput.value?.click();
    }

    async function onFileChange(e) {
      const file = e.target.files?.[0];
      if (!file || !form.id) return;
      uploading.value = true;
      try {
        const res = await uploadLiveCover(form.id, file);
        const data = (res.data || res).data || res.data || res;
        if (data && data.cover_url) {
          form.cover_url = data.cover_url;
          ElMessage.success("封面上传成功");
        }
      } catch (e) {
        ElMessage.error("上传失败");
      } finally {
        uploading.value = false;
        if (fileInput.value) fileInput.value.value = "";
      }
    }

    function removeCover() {
      form.cover_url = "";
    }

    async function regenerateKey() {
      if (!form.id) return;
      regenerating.value = true;
      try {
        const res = await regenerateStreamKey(form.id);
        const data = (res.data || res).data || res.data || res;
        if (data && data.stream_key) form.stream_key = data.stream_key;
        ElMessage.success("密钥已重置");
      } catch (e) {
        ElMessage.error("操作失败");
      } finally {
        regenerating.value = false;
      }
    }

    async function copyText(text) {
      try {
        await navigator.clipboard.writeText(text);
        ElMessage.success("已复制");
      } catch {
        ElMessage.warning("复制失败，请手动复制");
      }
    }

    onMounted(() => initRoom());

    return { form, fileInput, loading, uploading, regenerating, obsServer, maskedKey, saveTitle, triggerUpload, onFileChange, removeCover, regenerateKey, copyText };
  }
};
</script>

<style scoped>
.live-create-page {
  max-width: 560px;
  margin: 0 auto;
  padding: 30px 24px;
}
.lc-title {
  font-size: 22px;
  font-weight: 500;
  margin-bottom: 24px;
}

/* 封面 */
.lc-cover-box { display: flex; flex-direction: column; gap: 10px; }
.lc-cover-preview {
  position: relative; width: 200px; height: 112px;
  border-radius: 6px; overflow: hidden; border: 1px solid var(--color-border-tertiary);
}
.lc-cover-img { width: 100%; height: 100%; object-fit: cover; }
.lc-cover-remove { position: absolute; top: 4px; right: 4px; }
.lc-cover-empty {
  width: 200px; height: 112px;
  border-radius: 6px; border: 2px dashed var(--color-border-tertiary);
  display: flex; flex-direction: column; align-items: center; justify-content: center;
  cursor: pointer; transition: border-color .2s;
}
.lc-cover-empty:hover { border-color: var(--color-text-info); }
.lc-cover-icon { font-size: 28px; color: var(--color-text-tertiary); line-height: 1; }
.lc-cover-text { font-size: 12px; color: var(--color-text-tertiary); margin-top: 4px; }
.lc-cover-actions { display: flex; gap: 8px; }

/* 推流卡片 */
.lc-stream-card {
  margin-top: 32px; padding: 20px 24px;
  background: var(--color-background-secondary); border-radius: 10px;
}
.lc-stream-head { display: flex; align-items: center; gap: 10px; margin-bottom: 6px; }
.lc-stream-head h3 { font-size: 15px; font-weight: 500; margin: 0; }
.lc-stream-badge {
  font-size: 11px; padding: 1px 8px; border-radius: 4px;
  background: #639922; color: #fff;
}
.lc-stream-desc {
  font-size: 12px; color: var(--color-text-tertiary); margin: 0 0 16px;
}
.lc-stream-item {
  display: flex; flex-direction: column; gap: 4px; margin-bottom: 12px;
}
.lc-stream-label { font-size: 12px; color: var(--color-text-secondary); }
.lc-stream-value-row { display: flex; align-items: center; gap: 6px; }
.lc-stream-value {
  font-size: 12px; padding: 4px 10px; border-radius: 4px;
  background: var(--color-background-tertiary); color: var(--color-text-primary);
  font-family: monospace; max-width: 320px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
}

/* 使用教程 */
.lc-stream-details { margin-top: 12px; }
.lc-stream-details summary {
  font-size: 12px; color: var(--color-text-info); cursor: pointer;
}
.lc-stream-steps {
  margin: 8px 0 0 18px; padding: 0;
  font-size: 12px; color: var(--color-text-secondary); line-height: 1.8;
}
.lc-stream-steps a { color: var(--color-text-info); }
</style>
