<template>
  <div class="lf-panel">
    <header class="lf-panel__head">
      <h2>直播运营 · 置顶</h2>
      <div class="lf-panel__actions">
        <el-button :loading="loading" @click="load">刷新</el-button>
        <span class="lf-panel__tip">置顶到直播广场顶部，第 1 个占大图，第 2～6 个占右侧小格。</span>
      </div>
    </header>

    <div v-loading="loading" class="lf-board">
      <!-- 左：主站展示榜（可拖拽排序） -->
      <section class="lf-col lf-col--main">
        <header class="lf-col__head">
          <h3>主站展示榜</h3>
          <span class="lf-col__hint">拖拽把手调整顺序 · 自动保存</span>
        </header>

        <ol v-if="featured.length" class="lf-list">
          <li
            v-for="(item, index) in featured"
            :key="'lf_' + item.id"
            class="lf-row"
            :class="{
              'lf-row--dragging': dragIndex === index,
              'lf-row--over': overIndex === index && dragIndex !== null
            }"
            @dragover.prevent="onDragOver(index)"
            @drop.prevent="onDrop(index)"
          >
            <span
              class="lf-row__handle"
              title="拖拽排序"
              draggable="true"
              @dragstart="onDragStart(index, $event)"
              @dragend="onDragEnd"
            >⋮⋮</span>
            <span class="lf-row__rank" :class="{ 'lf-row__rank--top': index < 3 }">{{ index + 1 }}</span>
            <span class="lf-row__title">{{ item.title || ('直播间 #' + item.id) }}</span>
            <span v-if="index === 0" class="lf-row__tag lf-row__tag--big">大图</span>
            <span v-else-if="index < 6" class="lf-row__tag">小格</span>
            <span class="lf-row__viewers">👁 {{ item.viewer_count || 0 }}</span>
            <button
              v-if="index >= 6"
              type="button"
              class="lf-row__revoke"
              title="第 7 个起不在页面展示"
            ></button>
            <button type="button" class="lf-row__remove" @click="removeRoom(item)">移除</button>
          </li>
        </ol>
        <p v-else class="lf-col__empty">尚未置顶直播间，可从右侧「可选直播间」加入</p>
        <p v-if="saving" class="lf-col__saving">正在保存排序…</p>
      </section>

      <!-- 右：可选直播间 -->
      <section class="lf-col lf-col--cand">
        <header class="lf-col__head">
          <h3>可选直播间</h3>
          <span class="lf-col__hint">{{ candidates.length }} 个直播中</span>
        </header>
        <div v-if="candidates.length" class="lf-cand-list">
          <div v-for="room in candidates" :key="'c_' + room.id" class="lf-cand-row">
            <span class="lf-cand-title">{{ room.title || ('直播间 #' + room.id) }}</span>
            <span class="lf-cand-viewers">👁 {{ room.viewer_count || 0 }}</span>
            <el-button size="small" link type="primary" @click="addRoom(room)">加入</el-button>
          </div>
        </div>
        <p v-else class="lf-col__empty">没有其它可加入的直播房间</p>
      </section>
    </div>
  </div>
</template>

<script>
import { ElMessage } from "element-plus";
import { adminListLiveRooms, adminListLiveFeatured, adminSetLiveFeatured } from "@/api/live";

const MAX_FEATURED = 12;

export default {
  name: "LiveFeaturedManage",
  data() {
    return {
      loading: false,
      saving: false,
      dragIndex: null,
      overIndex: null,
      featured: [],     // ordered featured rooms
      candidates: []    // live rooms not featured
    };
  },
  created() {
    this.load();
  },
  methods: {
    async load() {
      this.loading = true;
      try {
        const roomsRes = await adminListLiveRooms({ status: "live", page: 1, page_size: 100 });
        const rd = (roomsRes && roomsRes.data) || roomsRes || {};
        const allLive = rd.rooms || rd.list || rd.items || [];

        const featRes = await adminListLiveFeatured();
        const fd = ((featRes && featRes.data) || featRes || {});
        const featItems = fd.items || [];

        const byId = {};
        allLive.forEach((r) => { byId[r.id] = r; });

        // Build featured in stored order.
        const featured = featItems
          .filter((it) => byId[it.room_id])
          .sort((a, b) => (a.sort_order || 0) - (b.sort_order || 0))
          .map((it) => byId[it.room_id]);
        this.featured = featured;

        // Candidates = live rooms not featured, sorted by viewer desc.
        const featSet = new Set(featItems.map((it) => it.room_id));
        this.candidates = allLive.filter((r) => !featSet.has(r.id));
      } catch {
        ElMessage.error("加载失败");
      } finally {
        this.loading = false;
      }
    },
    onDragStart(index, ev) {
      this.dragIndex = index;
      this.overIndex = index;
      if (ev && ev.dataTransfer) {
        ev.dataTransfer.effectAllowed = "move";
        ev.dataTransfer.setData("text/plain", String(index));
      }
    },
    onDragOver(index) {
      if (this.dragIndex === null) return;
      this.overIndex = index;
    },
    onDragEnd() {
      this.dragIndex = null;
      this.overIndex = null;
    },
    async onDrop(index) {
      if (this.dragIndex === null || this.dragIndex === index) {
        this.onDragEnd();
        return;
      }
      const list = this.featured.slice();
      const [moved] = list.splice(this.dragIndex, 1);
      list.splice(index, 0, moved);
      this.featured = list;
      this.onDragEnd();
      await this.saveOrder();
    },
    async saveOrder() {
      if (this.featured.length > MAX_FEATURED) {
        ElMessage.warning(`最多置顶 ${MAX_FEATURED} 个`);
        return;
      }
      this.saving = true;
      try {
        await adminSetLiveFeatured(this.featured.map((r) => r.id));
        ElMessage.success("置顶顺序已保存");
        await this.load();
      } catch {
        ElMessage.error("保存失败");
      } finally {
        this.saving = false;
      }
    },
    async addRoom(room) {
      if ((this.featured || []).length >= MAX_FEATURED) {
        ElMessage.warning(`最多置顶 ${MAX_FEATURED} 个`);
        return;
      }
      this.featured = (this.featured || []).concat(room);
      await this.saveOrder();
    },
    async removeRoom(room) {
      this.featured = this.featured.filter((r) => r.id !== room.id);
      await this.saveOrder();
    }
  }
};
</script>

<style lang="scss" scoped>
$blue: #00a1d6;
$white: #fff;

.lf-panel {
  background: $white;
  border: 1px solid #e3e5e7;
  border-radius: 8px;
  padding: 20px;
}
.lf-panel__head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;
  h2 { margin: 0; font-size: 18px; color: #18191c; }
}
.lf-panel__actions { display: flex; align-items: center; gap: 12px; }
.lf-panel__tip { font-size: 12px; color: #9499a0; }

.lf-board {
  display: grid;
  grid-template-columns: 1.4fr 1fr;
  gap: 16px;
  margin-top: 14px;
  min-height: 300px;
}
.lf-col {
  border: 1px solid #e3e5e7;
  border-radius: 8px;
  padding: 14px 16px;
  background: #fafbfc;
}
.lf-col__head { display: flex; align-items: baseline; justify-content: space-between; margin-bottom: 12px; }
.lf-col__head h3 { margin: 0; font-size: 15px; color: #18191c; font-weight: 600; }
.lf-col__hint { font-size: 11px; color: #9499a0; }
.lf-col__empty { margin: 24px 0; text-align: center; font-size: 13px; color: #9499a0; }
.lf-col__saving { margin: 8px 0 0; font-size: 12px; color: $blue; }

.lf-list { list-style: none; margin: 0; padding: 0; }
.lf-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 9px 0;
  border-bottom: 1px solid #eef0f2;
  &--dragging { opacity: 0.45; }
  &--over { background: #e3f3ff; }
}
.lf-row__handle { flex: 0 0 18px; cursor: grab; user-select: none; text-align: center; color: #9499a0; letter-spacing: -2px; }
.lf-row__rank { flex: 0 0 22px; font-weight: 600; color: #9499a0; font-variant-numeric: tabular-nums; }
.lf-row__rank--top { color: #ff6699; }
.lf-row__title { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-size: 14px; color: #18191c; }
.lf-row__tag { flex-shrink: 0; padding: 0 6px; border-radius: 3px; font-size: 11px; line-height: 18px; color: #00a1d6; background: #e3f3ff; }
.lf-row__tag--big { color: #f25d8e; background: #ffeef4; }
.lf-row__viewers { flex-shrink: 0; font-size: 12px; color: #9499a0; }
.lf-row__revoke { flex-shrink: 0; width: 8px; border: none; background: transparent; }
.lf-row__remove { flex-shrink: 0; border: none; background: transparent; cursor: pointer; font-size: 12px; color: #f25d8e; }

.lf-cand-list { display: flex; flex-direction: column; }
.lf-cand-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 0;
  border-bottom: 1px solid #eef0f2;
}
.lf-cand-title { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-size: 13px; color: #18191c; }
.lf-cand-viewers { flex-shrink: 0; font-size: 12px; color: #9499a0; }

@media (max-width: 960px) {
  .lf-board { grid-template-columns: 1fr; }
}
</style>