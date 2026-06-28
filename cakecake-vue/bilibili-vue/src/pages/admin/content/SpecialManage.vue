<template>
  <div class="sm-page" v-loading="loading">
    <header class="sm-head">
      <h2 class="sm-title">专题 &amp; 活动管理</h2>
    </header>

    <el-tabs v-model="tab" @tab-change="onTabChange">
      <el-tab-pane label="专题页" name="specials">
        <AdminDataTable :data="specials" :loading="loading" :show-pagination="false">
          <template #toolbar>
            <el-button type="primary" size="default" @click="openSpecialDialog(null)">新建专题</el-button>
          </template>
          <el-table-column prop="id" label="ID" width="60" />
          <el-table-column prop="title" label="标题" min-width="140" show-overflow-tooltip />
          <el-table-column prop="slug" label="标识" width="120" />
          <el-table-column label="状态" width="80">
            <template #default="{ row }">
              <el-tag :type="statusTag(row.status)" size="small">{{ statusLabel(row.status) }}</el-tag>
            </template>
          </el-table-column>
          <el-table-column prop="updated_at" label="更新时间" width="170">
            <template #default="{ row }">{{ fmtDate(row.updated_at || row.created_at) }}</template>
          </el-table-column>
          <el-table-column label="操作" width="180" fixed="right">
            <template #default="{ row }">
              <el-button size="small" type="primary" plain @click="openSpecialDialog(row)">编辑</el-button>
              <el-button size="small" type="danger" plain @click="confirmDeleteSpecial(row)">删除</el-button>
            </template>
          </el-table-column>
        </AdminDataTable>
      </el-tab-pane>

      <el-tab-pane label="活动" name="campaigns">
        <AdminDataTable :data="campaigns" :loading="loading" :show-pagination="false">
          <template #toolbar>
            <el-button type="primary" size="default" @click="openCampaignDialog(null)">新建活动</el-button>
          </template>
          <el-table-column prop="id" label="ID" width="60" />
          <el-table-column prop="title" label="标题" min-width="140" show-overflow-tooltip />
          <el-table-column prop="slug" label="标识" width="120" />
          <el-table-column label="状态" width="80">
            <template #default="{ row }">
              <el-tag :type="campaignStatusTag(row.status)" size="small">{{ statusLabel(row.status) }}</el-tag>
            </template>
          </el-table-column>
          <el-table-column label="开始" width="170">
            <template #default="{ row }">{{ row.start_time ? fmtDate(row.start_time) : '-' }}</template>
          </el-table-column>
          <el-table-column label="结束" width="170">
            <template #default="{ row }">{{ row.end_time ? fmtDate(row.end_time) : '-' }}</template>
          </el-table-column>
          <el-table-column label="操作" width="180" fixed="right">
            <template #default="{ row }">
              <el-button size="small" type="primary" plain @click="openCampaignDialog(row)">编辑</el-button>
              <el-button size="small" type="danger" plain @click="confirmDeleteCampaign(row)">删除</el-button>
            </template>
          </el-table-column>
        </AdminDataTable>
      </el-tab-pane>
    </el-tabs>

    <!-- Special Page Dialog -->
    <el-dialog v-model="specialFormOpen" :title="specialFormId ? '编辑专题' : '新建专题'" width="660px" destroy-on-close>
      <el-form label-width="80px">
        <el-form-item label="标题"><el-input v-model="specialForm.title" maxlength="100" /></el-form-item>
        <el-form-item label="标识"><el-input v-model="specialForm.slug" maxlength="60" placeholder="url-friendly" /></el-form-item>
        <el-form-item label="封面">
          <div style="display:flex;gap:12px;align-items:center;flex-wrap:wrap">
            <el-input v-model="specialForm.cover_url" maxlength="1024" placeholder="https://... 或上传本地图片" style="flex:1;min-width:240px" />
            <input ref="coverFileRef" type="file" accept="image/*" style="display:none" @change="onCoverFile" />
            <el-button size="small" @click="$refs.coverFileRef.click()" :loading="coverUploading">本地上传</el-button>
          </div>
          <img v-if="specialForm.cover_url" :src="specialForm.cover_url" style="margin-top:8px;max-height:120px;border-radius:4px" />
        </el-form-item>
        <el-form-item label="描述"><el-input v-model="specialForm.description" type="textarea" :rows="2" maxlength="500" /></el-form-item>

        <!-- 内容区块编辑器（可视化，无需写代码） -->
        <el-form-item label="内容">
          <div style="width:100%">
            <div v-for="(blk, i) in blockList" :key="i" class="sp-block-row">
              <el-select v-model="blk.type" size="small" style="width:90px" @change="onBlockTypeChange(i)">
                <el-option label="标题" value="title" />
                <el-option label="文字" value="text" />
                <el-option label="图片" value="banner" />
              </el-select>
              <el-input
                v-if="blk.type !== 'banner'"
                v-model="blk.content"
                size="small"
                :placeholder="blk.type === 'title' ? '区块标题，如：热门视频推荐' : '正文内容...'"
                :type="blk.type === 'text' ? 'textarea' : 'text'"
                :rows="blk.type === 'text' ? 3 : 1"
                style="flex:1"
              />
              <template v-else>
                <el-input v-model="blk.title" size="small" placeholder="图片说明（可选）" style="flex:1" />
                <el-input v-model="blk.content" size="small" placeholder="图片URL" style="flex:1" />
              </template>
              <el-button size="small" text type="danger" @click="blockList.splice(i, 1)">✕</el-button>
            </div>
            <el-button size="small" type="primary" plain @click="addBlock" style="margin-top:6px">+ 添加内容块</el-button>
          </div>
        </el-form-item>

        <el-form-item label="状态">
          <el-select v-model="specialForm.status" style="width:140px">
            <el-option label="草稿" value="draft" />
            <el-option label="已发布" value="published" />
            <el-option label="已归档" value="archived" />
          </el-select>
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="specialFormOpen = false">取消</el-button>
        <el-button type="primary" :loading="specialSaving" @click="saveSpecial">保存</el-button>
      </template>
    </el-dialog>

    <!-- Campaign Dialog -->
    <el-dialog v-model="campaignFormOpen" :title="campaignFormId ? '编辑活动' : '新建活动'" width="600px" destroy-on-close>
      <el-form label-width="80px">
        <el-form-item label="标题"><el-input v-model="campaignForm.title" maxlength="100" /></el-form-item>
        <el-form-item label="标识"><el-input v-model="campaignForm.slug" maxlength="60" /></el-form-item>
        <el-form-item label="封面URL"><el-input v-model="campaignForm.cover_url" maxlength="1024" /></el-form-item>
        <el-form-item label="描述"><el-input v-model="campaignForm.description" type="textarea" :rows="2" maxlength="500" /></el-form-item>
        <el-form-item label="规则"><el-input v-model="campaignForm.rules" type="textarea" :rows="3" /></el-form-item>
        <el-form-item label="奖励"><el-input v-model="campaignForm.rewards" type="textarea" :rows="2" /></el-form-item>
        <el-form-item label="开始时间">
          <el-date-picker v-model="campaignForm.start_time" type="datetime" placeholder="选择开始时间" format="YYYY-MM-DD HH:mm" value-format="YYYY-MM-DDTHH:mm:ssZ" />
        </el-form-item>
        <el-form-item label="结束时间">
          <el-date-picker v-model="campaignForm.end_time" type="datetime" placeholder="选择结束时间" format="YYYY-MM-DD HH:mm" value-format="YYYY-MM-DDTHH:mm:ssZ" />
        </el-form-item>
        <el-form-item label="状态">
          <el-select v-model="campaignForm.status" style="width:140px">
            <el-option label="草稿" value="draft" />
            <el-option label="进行中" value="active" />
            <el-option label="已结束" value="ended" />
          </el-select>
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="campaignFormOpen = false">取消</el-button>
        <el-button type="primary" :loading="campaignSaving" @click="saveCampaign">保存</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script>
import { ElMessage, ElMessageBox } from "element-plus";
import AdminDataTable from "@/components/admin/AdminDataTable.vue";
import {
  adminListSpecials,
  adminCreateSpecial,
  adminUpdateSpecial,
  adminDeleteSpecial,
  adminListCampaigns,
  adminCreateCampaign,
  adminUpdateCampaign,
  adminDeleteCampaign,
  adminUploadSpecialCover,
} from "@/api/admin";

const STATUS = { draft: "草稿", published: "已发布", archived: "已归档", active: "进行中", ended: "已结束" };

export default {
  name: "SpecialManage",
  components: { AdminDataTable },
  data() {
    return {
      loading: false,
      tab: "specials",
      specials: [],
      campaigns: [],
      specialFormOpen: false,
      specialSaving: false,
      specialFormId: null,
      specialForm: { title: "", slug: "", cover_url: "", description: "", blocks: "", status: "draft" },
      blockList: [],
      coverUploading: false,
      campaignFormOpen: false,
      campaignSaving: false,
      campaignFormId: null,
      campaignForm: { title: "", slug: "", cover_url: "", description: "", rules: "", rewards: "", start_time: null, end_time: null, status: "draft" }
    };
  },
  mounted() { this.fetchAll(); },
  methods: {
    async fetchAll() {
      this.loading = true;
      try {
        const [sr, cr] = await Promise.all([
          adminListSpecials(),
          adminListCampaigns()
        ]);
        this.specials = Array.isArray(sr.data) ? sr.data : [];
        this.campaigns = Array.isArray(cr.data) ? cr.data : [];
      } catch { ElMessage.error("加载失败"); }
      finally { this.loading = false; }
    },
    onTabChange() { this.fetchAll(); },

    // ─── Special Pages ───
    openSpecialDialog(row) {
      this.specialFormId = row ? row.id : null;
      this.specialForm = row
        ? { title: row.title, slug: row.slug, cover_url: row.cover_url || "", description: row.description || "", blocks: row.blocks || "", status: row.status || "draft" }
        : { title: "", slug: "", cover_url: "", description: "", blocks: "", status: "draft" };
      this.parseBlocks();
      this.specialFormOpen = true;
    },
    parseBlocks() {
      try {
        const arr = typeof this.specialForm.blocks === "string"
          ? JSON.parse(this.specialForm.blocks || "[]")
          : (this.specialForm.blocks || []);
        this.blockList = arr.map(b => ({
          type: b.type || "text",
          title: b.title || "",
          content: b.type === "text" ? b.content : (b.type === "banner" ? (b.image_url || b.link_url || "") : (b.content || "")),
        }));
      } catch { this.blockList = []; }
    },
    addBlock() {
      this.blockList.push({ type: "title", title: "", content: "" });
    },
    onBlockTypeChange(idx) {
      // reset content field
      this.blockList[idx].content = "";
      this.blockList[idx].title = "";
    },
    blocksToJSON() {
      return JSON.stringify(this.blockList
        .filter(b => b.content || b.title)
        .map(b => {
          if (b.type === "banner") return { type: "banner", title: b.title, image_url: b.content };
          if (b.type === "title") return { type: "banner", title: b.content };
          return { type: "text", content: b.content };
        }));
    },
    async saveSpecial() {
      const f = this.specialForm;
      if (!f.title || !f.slug) { ElMessage.warning("标题和标识必填"); return; }
      this.specialSaving = true;
      try {
        const payload = { ...f, blocks: this.blocksToJSON() };
        if (this.specialFormId) {
          await adminUpdateSpecial(this.specialFormId, payload);
        } else {
          await adminCreateSpecial(payload);
        }
        ElMessage.success("保存成功");
        this.specialFormOpen = false;
        this.fetchAll();
      } catch { ElMessage.error("保存失败"); }
      finally { this.specialSaving = false; }
    },
    async onCoverFile(e) {
      const file = e.target.files[0];
      if (!file) return;
      this.coverUploading = true;
      try {
        const res = await adminUploadSpecialCover(file);
        const data = res?.data || res;
        if (data && data.cover_url) {
          this.specialForm.cover_url = data.cover_url;
          ElMessage.success("封面上传成功");
        }
      } catch { ElMessage.error("上传失败"); }
      finally { this.coverUploading = false; e.target.value = ""; }
    },
    async deleteSpecial(row) {
      try {
        await adminDeleteSpecial(row.id);
        ElMessage.success("已删除");
        this.specials = this.specials.filter(r => r.id !== row.id);
      } catch { ElMessage.error("删除失败"); }
    },
    async confirmDeleteSpecial(row) {
      try {
        await ElMessageBox.confirm(`确认删除专题「${row.title}」？`, "删除确认", { type: "warning" });
        await this.deleteSpecial(row);
      } catch {}
    },

    // ─── Campaigns ───
    openCampaignDialog(row) {
      this.campaignFormId = row ? row.id : null;
      this.campaignForm = row
        ? { title: row.title, slug: row.slug, cover_url: row.cover_url || "", description: row.description || "", rules: row.rules || "", rewards: row.rewards || "", start_time: row.start_time || null, end_time: row.end_time || null, status: row.status || "draft" }
        : { title: "", slug: "", cover_url: "", description: "", rules: "", rewards: "", start_time: null, end_time: null, status: "draft" };
      this.campaignFormOpen = true;
    },
    async saveCampaign() {
      const f = this.campaignForm;
      if (!f.title || !f.slug) { ElMessage.warning("标题和标识必填"); return; }
      this.campaignSaving = true;
      try {
        if (this.campaignFormId) {
          await adminUpdateCampaign(this.campaignFormId, f);
        } else {
          await adminCreateCampaign(f);
        }
        ElMessage.success("保存成功");
        this.campaignFormOpen = false;
        this.fetchAll();
      } catch { ElMessage.error("保存失败"); }
      finally { this.campaignSaving = false; }
    },
    async deleteCampaign(row) {
      try {
        await adminDeleteCampaign(row.id);
        ElMessage.success("已删除");
        this.campaigns = this.campaigns.filter(r => r.id !== row.id);
      } catch { ElMessage.error("删除失败"); }
    },
    async confirmDeleteCampaign(row) {
      try {
        await ElMessageBox.confirm(`确认删除活动「${row.title}」？`, "删除确认", { type: "warning" });
        await this.deleteCampaign(row);
      } catch {}
    },

    statusLabel(s) { return STATUS[s] || s; },
    statusTag(s) { return s === "published" || s === "active" ? "success" : s === "draft" ? "info" : "warning"; },
    campaignStatusTag(s) { return s === "active" ? "success" : s === "draft" ? "info" : "warning"; },
    fmtDate(iso) {
      if (!iso) return "";
      const d = new Date(iso);
      return `${d.getFullYear()}-${String(d.getMonth()+1).padStart(2,"0")}-${String(d.getDate()).padStart(2,"0")} ${String(d.getHours()).padStart(2,"0")}:${String(d.getMinutes()).padStart(2,"0")}`;
    }
  }
};
</script>

<style scoped>
.sm-page { padding: 0; }
.sm-head { margin-bottom: 18px; }
.sm-title { margin: 0; font-size: 20px; font-weight: 600; color: #1a1a1a; }
.sm-toolbar { margin-bottom: 14px; }
.sp-block-row { display: flex; gap: 8px; align-items: flex-start; margin-bottom: 8px; }
</style>
