<template>
  <div class="sm-page" v-loading="loading">
    <header class="sm-head">
      <h2 class="sm-title">专题 &amp; 活动管理</h2>
    </header>

    <el-tabs v-model="tab" @tab-change="onTabChange">
      <el-tab-pane label="专题页" name="specials">
        <div class="sm-toolbar">
          <el-button type="primary" size="default" @click="openSpecialDialog(null)">新建专题</el-button>
        </div>
        <el-table :data="specials" stripe size="default" empty-text="暂无专题">
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
          <el-table-column label="操作" width="160" fixed="right">
            <template #default="{ row }">
              <el-button size="small" text type="primary" @click="openSpecialDialog(row)">编辑</el-button>
              <el-popconfirm title="确认删除？" @confirm="deleteSpecial(row)">
                <template #reference>
                  <el-button size="small" text type="danger">删除</el-button>
                </template>
              </el-popconfirm>
            </template>
          </el-table-column>
        </el-table>
      </el-tab-pane>

      <el-tab-pane label="活动" name="campaigns">
        <div class="sm-toolbar">
          <el-button type="primary" size="default" @click="openCampaignDialog(null)">新建活动</el-button>
        </div>
        <el-table :data="campaigns" stripe size="default" empty-text="暂无活动">
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
          <el-table-column label="操作" width="160" fixed="right">
            <template #default="{ row }">
              <el-button size="small" text type="primary" @click="openCampaignDialog(row)">编辑</el-button>
              <el-popconfirm title="确认删除？" @confirm="deleteCampaign(row)">
                <template #reference>
                  <el-button size="small" text type="danger">删除</el-button>
                </template>
              </el-popconfirm>
            </template>
          </el-table-column>
        </el-table>
      </el-tab-pane>
    </el-tabs>

    <!-- Special Page Dialog -->
    <el-dialog v-model="specialFormOpen" :title="specialFormId ? '编辑专题' : '新建专题'" width="600px" destroy-on-close>
      <el-form label-width="80px">
        <el-form-item label="标题"><el-input v-model="specialForm.title" maxlength="100" /></el-form-item>
        <el-form-item label="标识"><el-input v-model="specialForm.slug" maxlength="60" placeholder="url-friendly" /></el-form-item>
        <el-form-item label="封面URL"><el-input v-model="specialForm.cover_url" maxlength="1024" placeholder="https://..." /></el-form-item>
        <el-form-item label="描述"><el-input v-model="specialForm.description" type="textarea" :rows="2" maxlength="500" /></el-form-item>
        <el-form-item label="内容块"><el-input v-model="specialForm.blocks" type="textarea" :rows="4" placeholder='JSON: [{"type":"video","id":21},{"type":"text","content":"..."}]' /></el-form-item>
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
import http from "@/utils/adminHttp";
import { ElMessage } from "element-plus";

const STATUS = { draft: "草稿", published: "已发布", archived: "已归档", active: "进行中", ended: "已结束" };

export default {
  name: "SpecialManage",
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
          http.get("/api/v1/admin/specials"),
          http.get("/api/v1/admin/campaigns")
        ]);
        this.specials = Array.isArray(sr.data?.data) ? sr.data.data : [];
        this.campaigns = Array.isArray(cr.data?.data) ? cr.data.data : [];
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
      this.specialFormOpen = true;
    },
    async saveSpecial() {
      const f = this.specialForm;
      if (!f.title || !f.slug) { ElMessage.warning("标题和标识必填"); return; }
      this.specialSaving = true;
      try {
        if (this.specialFormId) {
          await http.put(`/api/v1/admin/specials/${this.specialFormId}`, f);
        } else {
          await http.post("/api/v1/admin/specials", f);
        }
        ElMessage.success("保存成功");
        this.specialFormOpen = false;
        this.fetchAll();
      } catch { ElMessage.error("保存失败"); }
      finally { this.specialSaving = false; }
    },
    async deleteSpecial(row) {
      try {
        await http.delete(`/api/v1/admin/specials/${row.id}`);
        ElMessage.success("已删除");
        this.specials = this.specials.filter(r => r.id !== row.id);
      } catch { ElMessage.error("删除失败"); }
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
          await http.put(`/api/v1/admin/campaigns/${this.campaignFormId}`, f);
        } else {
          await http.post("/api/v1/admin/campaigns", f);
        }
        ElMessage.success("保存成功");
        this.campaignFormOpen = false;
        this.fetchAll();
      } catch { ElMessage.error("保存失败"); }
      finally { this.campaignSaving = false; }
    },
    async deleteCampaign(row) {
      try {
        await http.delete(`/api/v1/admin/campaigns/${row.id}`);
        ElMessage.success("已删除");
        this.campaigns = this.campaigns.filter(r => r.id !== row.id);
      } catch { ElMessage.error("删除失败"); }
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
</style>
