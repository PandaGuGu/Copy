<template>
  <div class="cse-root">
    <div class="cse-header">
      <h3 class="cse-title">字幕管理</h3>
      <span class="cse-tip" v-if="subs.length">{{ subs.length }} 条字幕</span>
      <el-button size="small" type="primary" @click="openAdd">新增字幕</el-button>
      <el-button size="small" @click="expandAll = !expandAll">
        {{ expandAll ? '收起' : '展开' }}全部
      </el-button>
    </div>

    <el-collapse v-model="activeIds" v-if="subs.length">
      <el-collapse-item
        v-for="sub in subs"
        :key="sub.id"
        :title="`${sub.title || '未命名'} (${langLabel(sub.lang)})`"
        :name="sub.id"
      >
        <template #title>
          <div class="cse-collapse-title">
            <span>{{ sub.title || '未命名' }}</span>
            <el-tag size="small" effect="plain" style="margin-left:8px">{{ langLabel(sub.lang) }}</el-tag>
            <el-tag
              :type="sub.auto_gen ? 'warning' : 'info'"
              size="small"
              effect="plain"
              style="margin-left:4px"
            >{{ sub.auto_gen ? 'AI生成' : '手动' }}</el-tag>
          </div>
        </template>
        <div class="cse-vtt-box">
          <div class="cse-vtt-toolbar">
            <span class="cse-vtt-format">{{ (sub.format || 'vtt').toUpperCase() }}</span>
            <el-button size="small" text type="primary" @click.stop="editSub(sub)">编辑</el-button>
            <el-popconfirm title="确认删除此字幕？" @confirm="delSub(sub)">
              <template #reference>
                <el-button size="small" text type="danger">删除</el-button>
              </template>
            </el-popconfirm>
          </div>
          <el-input
            v-model="sub._content"
            type="textarea"
            :rows="8"
            placeholder="WEBVTT&#10;&#10;00:00:01.000 --> 00:00:05.000&#10;字幕文本"
            v-if="sub._editing"
          />
          <pre class="cse-vtt-preview" v-else>{{ sub._content }}</pre>
        </div>
      </el-collapse-item>
    </el-collapse>
    <el-empty v-else description="暂无字幕" :image-size="60" />

    <el-dialog v-model="dialogVisible" title="新增字幕" width="560px" destroy-on-close>
      <el-form label-width="70px" size="default">
        <el-form-item label="标题">
          <el-input v-model="form.title" placeholder="如：简体中文" />
        </el-form-item>
        <el-form-item label="语言">
          <el-select v-model="form.lang" placeholder="选择语言">
            <el-option label="中文" value="zh" />
            <el-option label="英文" value="en" />
            <el-option label="日文" value="ja" />
            <el-option label="韩文" value="ko" />
          </el-select>
        </el-form-item>
        <el-form-item label="格式">
          <el-radio-group v-model="form.format">
            <el-radio label="vtt">VTT</el-radio>
          </el-radio-group>
        </el-form-item>
        <el-form-item label="内容">
          <el-input
            v-model="form.content"
            type="textarea"
            :rows="10"
            placeholder="WEBVTT&#10;&#10;00:00:01.000 --> 00:00:05.000&#10;示例字幕文本&#10;&#10;00:00:06.000 --> 00:00:10.000&#10;另一段字幕"
          />
          <div class="cse-form-hint">
            每段格式：<code>HH:MM:SS.mmm --> HH:MM:SS.mmm</code> 换行后接字幕文本，段间用空行分隔
          </div>
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="dialogVisible = false">取消</el-button>
        <el-button type="primary" :loading="saving" @click="doAdd">添加</el-button>
      </template>
    </el-dialog>

    <el-dialog v-model="editDialogVisible" title="编辑字幕" width="560px" destroy-on-close>
      <el-form label-width="70px" size="default">
        <el-form-item label="标题">
          <el-input v-model="editForm.title" />
        </el-form-item>
        <el-form-item label="语言">
          <el-input v-model="editForm.lang" disabled />
        </el-form-item>
        <el-form-item label="内容">
          <el-input
            v-model="editForm.content"
            type="textarea"
            :rows="10"
          />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="editDialogVisible = false">取消</el-button>
        <el-button type="primary" :loading="saving" @click="doEdit">保存</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script>
import { mbGetVideoSubtitles, mbUploadSubtitle, mbDeleteSubtitle } from '@/api/minibili';
import { mbAdminUpdateSubtitle } from '@/api/minibili';
import { ElMessage } from 'element-plus';
import { getAccessToken } from '@/utils/authTokens';

const LANG_LABELS = { zh: '中文', en: '英文', ja: '日文', ko: '韩文' };

export default {
  name: 'CreatorSubtitleEditor',
  props: {
    videoId: { type: Number, required: true }
  },
  data() {
    return {
      subs: [],
      loading: false,
      saving: false,
      dialogVisible: false,
      editDialogVisible: false,
      expandAll: false,
      activeIds: [],
      form: { title: '', lang: 'zh', format: 'vtt', content: '' },
      editForm: { id: 0, title: '', lang: '', content: '' }
    };
  },
  watch: {
    expandAll(v) {
      this.activeIds = v ? this.subs.map(s => s.id) : [];
    }
  },
  mounted() {
    this.fetch();
  },
  methods: {
    langLabel(l) { return LANG_LABELS[l] || l; },
    async fetch() {
      this.loading = true;
      try {
        const list = await mbGetVideoSubtitles(this.videoId);
        this.subs = (list || []).map(s => ({ ...s, _content: s.content || '', _editing: false }));
      } catch (e) {
        ElMessage.error(e.message || '加载字幕失败');
      } finally {
        this.loading = false;
      }
    },
    openAdd() {
      this.form = { title: '', lang: 'zh', format: 'vtt', content: '' };
      this.dialogVisible = true;
    },
    async doAdd() {
      if (!this.form.content.trim()) { ElMessage.warning('请输入字幕内容'); return; }
      if (!getAccessToken()) { ElMessage.warning('请先登录'); return; }
      this.saving = true;
      try {
        await mbUploadSubtitle(this.videoId, this.form.title, this.form.lang, this.form.content, this.form.format);
        ElMessage.success('字幕已添加');
        this.dialogVisible = false;
        this.fetch();
      } catch (e) {
        ElMessage.error(e.message || '添加失败');
      } finally {
        this.saving = false;
      }
    },
    editSub(sub) {
      this.editForm = { id: sub.id, title: sub.title, lang: sub.lang, content: sub._content || sub.content };
      this.editDialogVisible = true;
    },
    async doEdit() {
      if (!this.editForm.content.trim()) { ElMessage.warning('内容不能为空'); return; }
      this.saving = true;
      try {
        await mbAdminUpdateSubtitle(this.editForm.id, {
          title: this.editForm.title,
          content: this.editForm.content
        });
        ElMessage.success('已保存');
        this.editDialogVisible = false;
        this.fetch();
      } catch (e) {
        ElMessage.error(e.message || '保存失败');
      } finally {
        this.saving = false;
      }
    },
    async delSub(sub) {
      try {
        if (!getAccessToken()) { ElMessage.warning('请先登录'); return; }
        await mbDeleteSubtitle(this.videoId, sub.id);
        ElMessage.success('已删除');
        this.fetch();
      } catch (e) {
        ElMessage.error(e.message || '删除失败');
      }
    }
  }
};
</script>

<style scoped>
.cse-root { padding: 16px 0; border-top: 1px solid #e3e5e7; margin-top: 16px; }
.cse-header { display: flex; align-items: center; gap: 10px; margin-bottom: 12px; }
.cse-title { margin: 0; font-size: 15px; font-weight: 600; color: #18191c; }
.cse-tip { font-size: 12px; color: #9499a0; }
.cse-collapse-title { display: flex; align-items: center; }
.cse-vtt-box { padding: 4px 0; }
.cse-vtt-toolbar { display: flex; align-items: center; gap: 8px; margin-bottom: 8px; }
.cse-vtt-format { font-size: 11px; color: #9499a0; background: #f1f2f3; padding: 1px 6px; border-radius: 3px; }
.cse-vtt-preview {
  margin: 0;
  padding: 10px 12px;
  background: #f6f7f8;
  border-radius: 6px;
  font-size: 12px;
  line-height: 1.7;
  max-height: 200px;
  overflow-y: auto;
  white-space: pre-wrap;
  word-break: break-all;
}
.cse-form-hint { margin-top: 6px; font-size: 11px; color: #9499a0; }
.cse-form-hint code { background: #f1f2f3; padding: 1px 4px; border-radius: 2px; }
</style>
