<template>
  <div class="se-page" v-loading="loading">
    <header class="se-head">
      <h2 class="se-title">字幕管理</h2>
      <span class="se-video">视频 #{{ videoId }}</span>
    </header>

    <!-- 已有字幕列表 -->
    <section class="se-section" v-if="subtitles.length > 0">
      <h3 class="se-section-title">已有字幕 ({{ subtitles.length }})</h3>
      <div class="se-list">
        <div v-for="sub in subtitles" :key="sub.id" class="se-item">
          <div class="se-item-info">
            <el-tag size="small" effect="plain">{{ langLabel(sub.lang) }}</el-tag>
            <span class="se-item-title">{{ sub.title }}</span>
            <span class="se-item-meta">{{ sub.format?.toUpperCase() }} · {{ sub.auto_gen ? '自动生成' : '手动上传' }}</span>
          </div>
          <div class="se-item-actions">
            <el-button size="small" text type="primary" @click="editSub(sub)">编辑</el-button>
            <el-popconfirm title="确认删除？" @confirm="deleteSub(sub)">
              <template #reference>
                <el-button size="small" text type="danger">删除</el-button>
              </template>
            </el-popconfirm>
          </div>
        </div>
      </div>
    </section>
    <section v-else class="se-empty">
      <p>暂无字幕，请添加第一个字幕轨道。</p>
    </section>

    <!-- 编辑面板 -->
    <section class="se-section">
      <h3 class="se-section-title">{{ editingId ? '编辑字幕' : '添加字幕' }}</h3>
      <div class="se-form">
        <div class="se-form-row">
          <label class="se-label">语言</label>
          <el-select v-model="form.lang" placeholder="选择语言" style="width: 200px">
            <el-option label="中文 (zh)" value="zh" />
            <el-option label="English (en)" value="en" />
            <el-option label="日本語 (ja)" value="ja" />
            <el-option label="한국어 (ko)" value="ko" />
          </el-select>
        </div>
        <div class="se-form-row">
          <label class="se-label">标题</label>
          <el-input v-model="form.title" placeholder="如：简体中文" style="width: 300px" maxlength="80" />
        </div>
        <div class="se-form-row">
          <label class="se-label">格式</label>
          <el-radio-group v-model="form.format">
            <el-radio value="vtt">VTT</el-radio>
            <el-radio value="srt">SRT</el-radio>
          </el-radio-group>
        </div>
        <div class="se-form-row se-form-row--col">
          <label class="se-label">
            字幕内容
            <span class="se-label-hint">— 可直接粘贴 VTT/SRT 文本</span>
          </label>
          <textarea
            v-model="form.content"
            class="se-editor"
            placeholder="WEBVTT&#10;&#10;00:00:01.000 --> 00:00:04.000&#10;这是第一句字幕&#10;&#10;00:00:05.000 --> 00:00:09.000&#10;这是第二句字幕"
            spellcheck="false"
            rows="14"
          />
        </div>

        <!-- 时间轴解析预览 -->
        <div class="se-form-row se-form-row--col" v-if="parsedCues.length > 0">
          <label class="se-label">时间轴预览 ({{ parsedCues.length }} 条)</label>
          <div class="se-cues">
            <div v-for="(cue, i) in parsedCues" :key="i" class="se-cue">
              <span class="se-cue-idx">{{ i + 1 }}</span>
              <span class="se-cue-time">{{ cue.start }} → {{ cue.end }}</span>
              <span class="se-cue-text">{{ cue.text }}</span>
            </div>
          </div>
        </div>

        <div class="se-form-row">
          <el-button type="primary" :loading="saving" @click="saveSub">
            {{ editingId ? '保存修改' : '上传字幕' }}
          </el-button>
          <el-button v-if="editingId" @click="cancelEdit">取消编辑</el-button>
        </div>
      </div>
    </section>
  </div>
</template>

<script>
import { ElMessage } from "element-plus";
import {
  mbGetVideoSubtitles,
  mbUploadSubtitle,
  mbDeleteSubtitle
} from "@/api/minibili";

const LANG_MAP = { zh: "中文", en: "English", ja: "日本語", ko: "한국어" };

function parseVttCues(raw) {
  const lines = String(raw || "").split(/\r?\n/);
  const cues = [];
  let i = 0;
  // skip WEBVTT header
  if (lines[0] && /^WEBVTT/i.test(lines[0].trim())) i = 1;
  while (i < lines.length) {
    // skip blank lines
    while (i < lines.length && !lines[i].trim()) i++;
    if (i >= lines.length) break;
    // timestamp line: 00:00:01.000 --> 00:00:04.000
    const tsMatch = lines[i].match(/^(\d{2}:\d{2}:\d{2}\.\d{3})\s*-->\s*(\d{2}:\d{2}:\d{2}\.\d{3})/);
    if (!tsMatch) { i++; continue; }
    const start = tsMatch[1];
    const end = tsMatch[2];
    i++;
    // text lines
    const textLines = [];
    while (i < lines.length && lines[i].trim()) {
      textLines.push(lines[i].trim());
      i++;
    }
    cues.push({ start, end, text: textLines.join(" ") });
  }
  return cues;
}

function parseSrtCues(raw) {
  const lines = String(raw || "").split(/\r?\n/);
  const cues = [];
  let i = 0;
  while (i < lines.length) {
    while (i < lines.length && !lines[i].trim()) i++;
    if (i >= lines.length) break;
    // sequence number
    if (!/^\d+$/.test(lines[i].trim())) { i++; continue; }
    i++;
    if (i >= lines.length) break;
    // timestamp: 00:00:01,000 --> 00:00:04,000
    const tsMatch = lines[i].match(/(\d{2}:\d{2}:\d{2}[.,]\d{3})\s*-->\s*(\d{2}:\d{2}:\d{2}[.,]\d{3})/);
    if (!tsMatch) { i++; continue; }
    const start = tsMatch[1].replace(",", ".");
    const end = tsMatch[2].replace(",", ".");
    i++;
    const textLines = [];
    while (i < lines.length && lines[i].trim()) {
      textLines.push(lines[i].trim());
      i++;
    }
    cues.push({ start, end, text: textLines.join(" ") });
  }
  return cues;
}

export default {
  name: "SubtitleEdit",
  props: {
    videoId: { type: Number, required: true }
  },
  data() {
    return {
      loading: false,
      saving: false,
      subtitles: [],
      editingId: null,
      form: { lang: "zh", title: "", format: "vtt", content: "" }
    };
  },
  computed: {
    parsedCues() {
      const raw = (this.form.content || "").trim();
      if (!raw) return [];
      if (this.form.format === "srt") return parseSrtCues(raw);
      return parseVttCues(raw);
    }
  },
  mounted() {
    this.fetchSubtitles();
  },
  methods: {
    langLabel(l) { return LANG_MAP[l] || l || "未知"; },
    async fetchSubtitles() {
      this.loading = true;
      try {
        this.subtitles = await mbGetVideoSubtitles(this.videoId);
      } catch {
        ElMessage.error("加载字幕失败");
      } finally {
        this.loading = false;
      }
    },
    editSub(sub) {
      this.editingId = sub.id;
      this.form = {
        lang: sub.lang || "zh",
        title: sub.title || "",
        format: sub.format || "vtt",
        content: sub.content || ""
      };
    },
    cancelEdit() {
      this.editingId = null;
      this.form = { lang: "zh", title: "", format: "vtt", content: "" };
    },
    async saveSub() {
      const content = (this.form.content || "").trim();
      if (!content) { ElMessage.warning("请输入字幕内容"); return; }
      const title = (this.form.title || "").trim() || "未命名";
      this.saving = true;
      try {
        const sub = await mbUploadSubtitle(
          this.videoId,
          content,
          this.form.lang,
          title,
          this.form.format
        );
        ElMessage.success(this.editingId ? "修改成功" : "上传成功");
        this.cancelEdit();
        // Refresh list
        await this.fetchSubtitles();
      } catch (e) {
        ElMessage.error("保存失败");
      } finally {
        this.saving = false;
      }
    },
    async deleteSub(sub) {
      try {
        await mbDeleteSubtitle(this.videoId, sub.id);
        ElMessage.success("已删除");
        this.subtitles = this.subtitles.filter((s) => s.id !== sub.id);
      } catch {
        ElMessage.error("删除失败");
      }
    }
  }
};
</script>

<style scoped>
.se-page { max-width: 860px; margin: 0 auto; padding: 24px 20px 80px; }
.se-head { margin-bottom: 24px; display: flex; align-items: center; gap: 12px; }
.se-title { margin: 0; font-size: 20px; font-weight: 600; color: #1a1a1a; }
.se-video { font-size: 13px; color: #888; }

.se-section { margin-bottom: 28px; }
.se-section-title { margin: 0 0 12px; font-size: 15px; font-weight: 600; color: #333; }

.se-list { border: 1px solid #ebeef5; border-radius: 6px; overflow: hidden; }
.se-item {
  display: flex; align-items: center; justify-content: space-between;
  padding: 10px 16px; border-bottom: 1px solid #f0f0f0;
}
.se-item:last-child { border-bottom: none; }
.se-item-info { display: flex; align-items: center; gap: 10px; min-width: 0; }
.se-item-title { font-size: 14px; font-weight: 500; color: #333; }
.se-item-meta { font-size: 12px; color: #999; }
.se-item-actions { flex-shrink: 0; display: flex; gap: 4px; }

.se-empty { padding: 32px 0; text-align: center; color: #999; font-size: 14px; }

.se-form { display: flex; flex-direction: column; gap: 16px; }
.se-form-row { display: flex; align-items: center; gap: 12px; }
.se-form-row--col { flex-direction: column; align-items: flex-start; }
.se-label { font-size: 13px; color: #555; min-width: 64px; flex-shrink: 0; }
.se-label-hint { color: #bbb; font-weight: 400; }

.se-editor {
  width: 100%;
  padding: 12px 14px;
  border: 1px solid #dcdfe6;
  border-radius: 6px;
  font-family: "Consolas", "Monaco", monospace;
  font-size: 12px;
  line-height: 1.5;
  resize: vertical;
  box-sizing: border-box;
  background: #fafbfc;
  outline: none;
}
.se-editor:focus { border-color: #409eff; background: #fff; }

.se-cues {
  width: 100%;
  max-height: 260px;
  overflow-y: auto;
  border: 1px solid #ebeef5;
  border-radius: 6px;
}
.se-cue {
  display: flex; align-items: center; gap: 10px;
  padding: 6px 12px; border-bottom: 1px solid #f5f5f5; font-size: 12px;
}
.se-cue:last-child { border-bottom: none; }
.se-cue-idx { width: 24px; height: 20px; border-radius: 10px; background: #ecf5ff; color: #409eff; display: flex; align-items: center; justify-content: center; font-weight: 600; flex-shrink: 0; }
.se-cue-time { color: #999; white-space: nowrap; font-variant-numeric: tabular-nums; flex-shrink: 0; }
.se-cue-text { color: #333; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
</style>
