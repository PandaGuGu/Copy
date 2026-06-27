<template>
  <div class="mb-pc-history">
    <!-- ===== Row 1: Title + Controls ===== -->
    <header class="mb-pc-history__hd">
      <div class="mb-pc-history__row1">
        <div class="mb-pc-history__title-row">
          <svg class="mb-pc-history__title-ico" viewBox="0 0 24 24" width="28" height="28" aria-hidden="true">
            <circle cx="12" cy="12" r="10" fill="none" stroke="#18191c" stroke-width="2"/>
            <polyline points="12,6 12,12 16,14" fill="none" stroke="#18191c" stroke-width="2" stroke-linecap="round"/>
          </svg>
          <h1 class="mb-pc-history__title-text">历史记录</h1>
        </div>
        <div class="mb-pc-history__row1-actions">
          <label class="mb-pc-history__toggle-wrap">
            <span class="mb-pc-history__toggle-label">记录浏览历史</span>
            <input
              type="checkbox"
              class="mb-pc-history__toggle-input"
              :checked="!paused"
              :disabled="!isMinibiliMode || settingsLoading"
              @change="onTogglePause"
            />
            <span class="mb-pc-history__toggle-track" aria-hidden="true">
              <span class="mb-pc-history__toggle-thumb" />
            </span>
          </label>
          <button
            type="button"
            class="mb-pc-history__qr-btn"
            title="扫码分享"
            aria-label="扫码分享"
          >
            <svg viewBox="0 0 24 24" width="18" height="18" aria-hidden="true">
              <rect x="3" y="3" width="7" height="7" rx="1" fill="none" stroke="currentColor" stroke-width="1.5"/>
              <rect x="14" y="3" width="7" height="7" rx="1" fill="none" stroke="currentColor" stroke-width="1.5"/>
              <rect x="3" y="14" width="7" height="7" rx="1" fill="none" stroke="currentColor" stroke-width="1.5"/>
              <rect x="14" y="14" width="3" height="3" fill="currentColor"/>
              <rect x="18" y="14" width="3" height="7" rx="1" fill="currentColor"/>
              <rect x="14" y="18" width="7" height="3" rx="1" fill="currentColor"/>
            </svg>
          </button>
        </div>
      </div>

      <!-- ===== Row 2: Filters + Search + Actions ===== -->
      <div class="mb-pc-history__row2">
        <nav class="mb-pc-history__tabs">
          <button
            v-for="tab in filterTabs"
            :key="tab.key"
            type="button"
            class="mb-pc-history__tab"
            :class="{ 'mb-pc-history__tab--active': activeTab === tab.key }"
            @click="activeTab = tab.key"
          >
            {{ tab.label }}
          </button>
        </nav>
        <div class="mb-pc-history__filter-drop" ref="filterDrop">
          <button
            type="button"
            class="mb-pc-history__filter-btn"
            @click="filterMenuOpen = !filterMenuOpen"
          >
            <span>更多筛选</span>
            <svg
              class="mb-pc-history__filter-arrow"
              :class="{ 'mb-pc-history__filter-arrow--up': filterMenuOpen }"
              viewBox="0 0 24 24"
              width="14"
              height="14"
              aria-hidden="true"
            >
              <path fill="currentColor" d="M7 10l5 5 5-5z"/>
            </svg>
          </button>
          <div v-show="filterMenuOpen" class="mb-pc-history__filter-menu">
            <button
              v-for="opt in filterOptions"
              :key="opt.key"
              type="button"
              class="mb-pc-history__filter-item"
              :class="{ 'mb-pc-history__filter-item--sel': filterMenuVal === opt.key }"
              @click="filterMenuVal = opt.key; filterMenuOpen = false"
            >
              {{ opt.label }}
            </button>
          </div>
        </div>
        <label class="mb-pc-history__search">
          <input
            v-model.trim="keyword"
            type="search"
            class="mb-pc-history__search-input"
            placeholder="搜索标题 / up主昵称"
            autocomplete="off"
            @input="onSearchInput"
          />
          <svg class="mb-pc-history__search-ico" viewBox="0 0 24 24" width="16" height="16" aria-hidden="true">
            <path fill="currentColor" d="M15.5 14h-.79l-.28-.27A6.471 6.471 0 0016 9.5 6.5 6.5 0 109.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5 14 7.01 14 9.5 11.99 14 9.5 14z"/>
          </svg>
        </label>
        <button
          type="button"
          class="mb-pc-history__action-btn mb-pc-history__action-btn--danger"
          :disabled="!isMinibiliMode || !items.length || clearing"
          @click="onClearAll"
        >
          <svg viewBox="0 0 24 24" width="16" height="16" aria-hidden="true">
            <path fill="currentColor" d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z"/>
          </svg>
          <span>清空历史</span>
        </button>
        <button
          type="button"
          class="mb-pc-history__action-btn"
          :class="{ 'mb-pc-history__action-btn--active': batchMode }"
          @click="batchMode = !batchMode"
        >
          <svg viewBox="0 0 24 24" width="16" height="16" aria-hidden="true">
            <rect x="3" y="3" width="18" height="18" rx="1" fill="none" stroke="currentColor" stroke-width="1.5"/>
            <line x1="8" y1="8" x2="8" y2="8" stroke="currentColor" stroke-width="3" stroke-linecap="round"/>
            <line x1="8" y1="12" x2="8" y2="12" stroke="currentColor" stroke-width="3" stroke-linecap="round"/>
            <line x1="8" y1="16" x2="8" y2="16" stroke="currentColor" stroke-width="3" stroke-linecap="round"/>
            <line x1="12" y1="8" x2="16" y2="8" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
            <line x1="12" y1="12" x2="16" y2="12" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
            <line x1="12" y1="16" x2="16" y2="16" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
          </svg>
          <span>批量管理</span>
        </button>
      </div>
    </header>

    <div ref="scrollRoot" class="mb-pc-history__body">
      <p v-if="loading" class="mb-pc-history__loading">加载中…</p>
      <template v-else>
        <div
          v-if="paused && isMinibiliMode"
          class="mb-pc-history__paused-panel"
        >
          <p class="mb-pc-history__paused-msg">
            历史功能暂停中，就算看不可描述的视频也不会被记录下来了
          </p>
          <button
            type="button"
            class="mb-pc-history__paused-resume"
            :disabled="settingsLoading"
            @click="onTogglePause"
          >
            继续记录历史
          </button>
        </div>
        <p
          v-else-if="!displayGroups.length"
          class="mb-pc-history__empty"
        >
          {{ keyword ? "未找到相关历史记录" : "暂无浏览历史" }}
        </p>
        <div
          v-if="displayGroups.length"
          class="mb-pc-history__timeline"
          :class="{ 'mb-pc-history__timeline--paused': paused }"
        >
        <section
          v-for="group in displayGroups"
          :key="group.key"
          class="mb-pc-history__group"
          :class="'mb-pc-history__group--' + group.key"
        >
          <div class="mb-pc-history__group-head">
            <span class="mb-pc-history__pill">{{ group.pill }}</span>
          </div>
          <article
            v-for="row in group.rows"
            :key="entryKey(row)"
            class="mb-pc-history__entry"
            :class="{
              'mb-pc-history__entry--article': isArticleRow(row),
              'mb-pc-history__entry--live': isLiveRow(row)
            }"
          >
            <i
              v-if="row.showDate"
              class="mb-pc-history__stamp-caret mb-pc-history__stamp-caret--on-line"
              aria-hidden="true"
            />
            <i v-else class="mb-pc-history__axis-dot" aria-hidden="true" />
            <div
              v-if="row.showDate"
              class="mb-pc-history__stamp mb-pc-history__stamp--date"
            >
              <span>{{ row.dateLabel }}</span>
            </div>
            <div v-else class="mb-pc-history__stamp">
              <span class="mb-pc-history__stamp-time">{{ row.viewed_time }}</span>
            </div>
            <router-link
              class="mb-pc-history__thumb"
              :to="contentRoute(row)"
            >
                <img
                  class="mb-pc-history__thumb-img"
                  :src="row.cover_url || defaultCover"
                  :alt="row.title"
                  loading="lazy"
                  @error="onCoverError($event)"
                />
                <span
                  v-if="!isArticleRow(row) && !isLiveRow(row)"
                  class="mb-pc-history__thumb-track"
                  aria-hidden="true"
                />
                <span
                  v-if="!isArticleRow(row) && !isLiveRow(row)"
                  class="mb-pc-history__thumb-bar"
                  :style="{ width: progressBarPct(row) + '%' }"
                />
                <span
                  v-if="isLiveRow(row)"
                  class="mb-pc-history__thumb-badge"
                  aria-hidden="true"
                >直播</span>
                <span
                  v-else-if="isArticleRow(row)"
                  class="mb-pc-history__thumb-badge"
                  aria-hidden="true"
                >专栏</span>
            </router-link>
            <div class="mb-pc-history__meta">
              <router-link
                class="mb-pc-history__vtitle"
                :to="contentRoute(row)"
              >
                {{ row.title }}
              </router-link>
            </div>
            <p class="mb-pc-history__progress">
              <i
                class="mb-pc-history__dev-ico"
                :class="
                  row.device === 'mobile'
                    ? 'mb-pc-history__dev-ico--mobile'
                    : 'mb-pc-history__dev-ico--web'
                "
                aria-hidden="true"
              />
              <span>{{ progressLabel(row) }}</span>
            </p>
            <div class="mb-pc-history__up-slot">
              <router-link
                v-if="uploaderRoute(row)"
                class="mb-pc-history__up"
                :to="uploaderRoute(row)"
              >
                <img
                  class="mb-pc-history__up-face"
                  :src="row.uploader_avatar_url || defaultAvatar"
                  alt=""
                  loading="lazy"
                  @error="onAvatarError($event)"
                />
                <span class="mb-pc-history__up-name">{{
                  row.uploader_name
                }}</span>
                <span v-if="categoryLabel(row)" class="mb-pc-history__up-cat">{{
                  categoryLabel(row)
                }}</span>
              </router-link>
              <span v-else class="mb-pc-history__up">
                <img
                  class="mb-pc-history__up-face"
                  :src="row.uploader_avatar_url || defaultAvatar"
                  alt=""
                  loading="lazy"
                  @error="onAvatarError($event)"
                />
                <span class="mb-pc-history__up-name">{{
                  row.uploader_name
                }}</span>
                <span v-if="categoryLabel(row)" class="mb-pc-history__up-cat">{{
                  categoryLabel(row)
                }}</span>
              </span>
            </div>
            <span class="mb-pc-history__sep" aria-hidden="true" />
            <button
              type="button"
              class="mb-pc-history__del"
              title="删除"
              :disabled="deletingKey === entryKey(row)"
              @click="onDelete(row)"
            >
              <svg viewBox="0 0 24 24" width="18" height="18" aria-hidden="true">
                <path
                  fill="currentColor"
                  d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z"
                />
              </svg>
            </button>
          </article>
        </section>
        </div>
      </template>
    </div>

    <button
      v-show="showTop"
      type="button"
      class="mb-pc-history__top"
      title="回到顶部"
      aria-label="回到顶部"
      @click="scrollToTop"
    />
  </div>
</template>

<script>
import { ElMessageBox } from "element-plus";
import {
  mbClearMeViewHistory,
  mbDeleteMeArticleViewHistoryEntry,
  mbDeleteMeLiveViewHistoryEntry,
  mbDeleteMeViewHistoryEntry,
  mbGetMeViewHistory,
  mbPutMeViewHistorySettings
} from "@/api/minibili";
import {
  minibiliArticleReadRoute,
  minibiliLiveRoomRoute,
  minibiliUserSpaceRoute,
  minibiliVideoPlayRoute
} from "@/utils/minibiliRoutes";

const PILL = {
  today: "今天",
  yesterday: "昨天",
  week: "近1周",
  older: "1周前"
};

function parseViewedAt(s) {
  if (!s) {
    return null;
  }
  const d = new Date(String(s).replace(/-/g, "/"));
  return Number.isNaN(d.getTime()) ? null : d;
}

function dayKey(d) {
  const y = d.getFullYear();
  const m = String(d.getMonth() + 1).padStart(2, "0");
  const day = String(d.getDate()).padStart(2, "0");
  return `${y}-${m}-${day}`;
}

function periodKey(d, now) {
  const today0 = new Date(now.getFullYear(), now.getMonth(), now.getDate());
  const viewed0 = new Date(d.getFullYear(), d.getMonth(), d.getDate());
  const diff = Math.floor((today0 - viewed0) / 86400000);
  if (diff <= 0) {
    return "today";
  }
  if (diff === 1) {
    return "yesterday";
  }
  if (diff <= 7) {
    return "week";
  }
  return "older";
}

function pad2(n) {
  return String(Math.floor(n)).padStart(2, "0");
}

function formatProgressTime(sec) {
  const s = Math.max(0, Math.floor(sec));
  const m = Math.floor(s / 60);
  const r = s % 60;
  return `${pad2(m)}:${pad2(r)}`;
}

export default {
  name: "ViewHistoryPanel",
  props: {
    isMinibiliMode: {
      type: Boolean,
      default: false
    }
  },
  data() {
    return {
      loading: false,
      settingsLoading: false,
      clearing: false,
      paused: false,
      items: [],
      keyword: "",
      searchTimer: null,
      deletingKey: "",
      showTop: false,
      activeTab: "all",
      filterMenuOpen: false,
      filterMenuVal: "",
      batchMode: false,
      filterTabs: [
        { key: "all", label: "综合" },
        { key: "video", label: "视频" },
        { key: "live", label: "直播" },
        { key: "article", label: "专栏" }
      ],
      filterOptions: [
        { key: "all", label: "全部" },
        { key: "watched", label: "已看完" },
        { key: "watching", label: "未看完" }
      ],
      defaultCover:
        "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='160' height='90'%3E%3Crect fill='%23e3e5e7' width='100%25' height='100%25'/%3E%3C/svg%3E",
      defaultAvatar:
        "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='20' height='20'%3E%3Ccircle fill='%23e3e5e7' cx='10' cy='10' r='10'/%3E%3C/svg%3E"
    };
  },
  computed: {
    filteredItems() {
      let list = this.items;
      if (this.activeTab === "video") {
        list = list.filter(
          r => !this.isArticleRow(r) && !this.isLiveRow(r)
        );
      } else if (this.activeTab === "live") {
        list = list.filter(r => this.isLiveRow(r));
      } else if (this.activeTab === "article") {
        list = list.filter(r => this.isArticleRow(r));
      }
      if (this.filterMenuVal === "watched") {
        list = list.filter(r => {
          if (this.isLiveRow(r) || this.isArticleRow(r)) return false;
          const dur = Number(r.duration_sec);
          const prog = Number(r.progress_sec);
          return Number.isFinite(dur) && dur > 0 && Number.isFinite(prog) && prog >= dur * 0.95;
        });
      } else if (this.filterMenuVal === "watching") {
        list = list.filter(r => {
          if (this.isLiveRow(r) || this.isArticleRow(r)) return true;
          const dur = Number(r.duration_sec);
          const prog = Number(r.progress_sec);
          return !(Number.isFinite(dur) && dur > 0 && Number.isFinite(prog) && prog >= dur * 0.95);
        });
      }
      return list;
    },
    displayGroups() {
      const now = new Date();
      const groups = [];
      const map = new Map();
      for (const item of this.filteredItems) {
        const d = parseViewedAt(item.viewed_at);
        if (!d) {
          continue;
        }
        const pk = periodKey(d, now);
        if (!map.has(pk)) {
          const g = { key: pk, pill: PILL[pk], rows: [] };
          map.set(pk, g);
          groups.push(g);
        }
        map.get(pk).rows.push({ ...item, _date: d });
      }
      const order = ["today", "yesterday", "week", "older"];
      const sorted = order
        .map(k => groups.find(g => g.key === k))
        .filter(Boolean);
      for (const g of sorted) {
        let lastDay = "";
        for (let i = 0; i < g.rows.length; i++) {
          const row = g.rows[i];
          const dk = dayKey(row._date);
          const showDate =
            g.key === "week" || g.key === "older" ? dk !== lastDay : false;
          if (showDate) {
            lastDay = dk;
          }
          row.showDate = showDate;
          row.dateLabel = dk;
        }
      }
      return sorted;
    }
  },
  mounted() {
    void this.refresh();
    const onScroll = () => {
      const top =
        typeof window !== "undefined"
          ? window.scrollY || document.documentElement.scrollTop
          : 0;
      this.showTop = top > 320;
    };
    this._onScroll = onScroll;
    if (typeof window !== "undefined") {
      window.addEventListener("scroll", onScroll, { passive: true });
    }
    this._onClickDoc = (e) => {
      const drop = this.$refs.filterDrop;
      if (drop && !drop.contains(e.target)) {
        this.filterMenuOpen = false;
      }
    };
    document.addEventListener("click", this._onClickDoc);
  },
  beforeUnmount() {
    if (this.searchTimer) {
      clearTimeout(this.searchTimer);
    }
    if (typeof window !== "undefined" && this._onScroll) {
      window.removeEventListener("scroll", this._onScroll);
    }
    if (this._onClickDoc) {
      document.removeEventListener("click", this._onClickDoc);
    }
  },
  methods: {
    isArticleRow(row) {
      return (
        row &&
        (row.media_type === "article" ||
          (Number(row.article_id) > 0 && !Number(row.video_id) && !Number(row.live_room_id)))
      );
    },
    isLiveRow(row) {
      return (
        row &&
        (row.media_type === "live" || Number(row.live_room_id) > 0)
      );
    },
    entryKey(row) {
      if (this.isLiveRow(row)) {
        return `l-${Number(row.live_room_id)}-${row.viewed_at}`;
      }
      const kind = this.isArticleRow(row) ? "a" : "v";
      const id = this.isArticleRow(row)
        ? Number(row.article_id)
        : Number(row.video_id);
      return `${kind}-${id}-${row.viewed_at}`;
    },
    contentRoute(row) {
      if (this.isLiveRow(row)) {
        return (
          minibiliLiveRoomRoute(row.live_room_id) || {
            name: "minibiliLiveRoom",
            params: { roomId: "0" }
          }
        );
      }
      if (this.isArticleRow(row)) {
        return (
          minibiliArticleReadRoute(row.article_id) || {
            name: "minibiliArticleRead",
            params: { id: "0" }
          }
        );
      }
      return (
        minibiliVideoPlayRoute(row.video_id) || {
          name: "video",
          params: { aid: "0" }
        }
      );
    },
    uploaderRoute(row) {
      return minibiliUserSpaceRoute(row.uploader_id);
    },
    categoryLabel(row) {
      const raw = row && row.category;
      if (!raw) {
        return "";
      }
      const parts = String(raw)
        .split(/[>＞/／]/)
        .map((s) => s.trim())
        .filter(Boolean);
      return parts.length ? parts[parts.length - 1] : String(raw).trim();
    },
    progressBarPct(row) {
      if (this.isLiveRow(row)) {
        return 0;
      }
      const dur = Number(row.duration_sec);
      const prog = Number(row.progress_sec);
      if (!Number.isFinite(dur) || dur <= 0 || !Number.isFinite(prog)) {
        return 0;
      }
      return Math.min(100, Math.max(0, (prog / dur) * 100));
    },
    progressLabel(row) {
      if (this.isLiveRow(row)) {
        return "已观看";
      }
      if (this.isArticleRow(row)) {
        return "已阅读";
      }
      const dur = Number(row.duration_sec);
      const prog = Number(row.progress_sec);
      if (!Number.isFinite(dur) || dur <= 0) {
        return "刚开始看";
      }
      if (!Number.isFinite(prog) || prog <= 3) {
        return "刚开始看";
      }
      if (prog >= dur * 0.95) {
        return "已看完";
      }
      return `看到 ${formatProgressTime(prog)}`;
    },
    onCoverError(e) {
      const img = e && e.target;
      if (img) {
        img.src = this.defaultCover;
      }
    },
    onAvatarError(e) {
      const img = e && e.target;
      if (img) {
        img.src = this.defaultAvatar;
      }
    },
    onSearchInput() {
      if (this.searchTimer) {
        clearTimeout(this.searchTimer);
      }
      this.searchTimer = setTimeout(() => {
        void this.loadList();
      }, 300);
    },
    scrollToTop() {
      if (typeof window !== "undefined") {
        window.scrollTo({ top: 0, behavior: "smooth" });
      }
    },
    async refresh() {
      await this.loadList();
    },
    async loadList() {
      if (!this.isMinibiliMode) {
        this.items = [];
        return;
      }
      this.loading = true;
      try {
        const res = await mbGetMeViewHistory(this.keyword || undefined);
        this.items = res.items || [];
        this.paused = !!res.paused;
      } catch {
        this.items = [];
      } finally {
        this.loading = false;
      }
    },
    histMsgboxOptions(confirmText) {
      return {
        confirmButtonText: confirmText,
        cancelButtonText: "取消",
        center: true,
        showClose: false,
        customClass: "mb-hist-msgbox",
        confirmButtonClass: "mb-hist-msgbox__ok",
        cancelButtonClass: "mb-hist-msgbox__cancel",
        distinguishCancelAndClose: true
      };
    },
    async confirmHistAction(message, confirmText) {
      try {
        await ElMessageBox.confirm(
          message,
          "",
          this.histMsgboxOptions(confirmText)
        );
        return true;
      } catch {
        return false;
      }
    },
    async onTogglePause(e) {
      if (!this.isMinibiliMode || this.settingsLoading) {
        return;
      }
      const wantedChecked = e && e.target ? e.target.checked : !this.paused;
      const wantPaused = !wantedChecked;
      if (wantPaused) {
        const ok = await this.confirmHistAction(
          "啊叻？你要暂停历史记录功能吗？",
          "确定暂停"
        );
        if (!ok) {
          return;
        }
      }
      this.settingsLoading = true;
      try {
        const res = await mbPutMeViewHistorySettings(wantPaused);
        this.paused = !!res.paused;
      } catch {
        /* ignore */
      } finally {
        this.settingsLoading = false;
      }
    },
    async onClearAll() {
      if (!this.isMinibiliMode || this.clearing || !this.items.length) {
        return;
      }
      const ok = await this.confirmHistAction(
        "清空之后就什么都没有了哦~",
        "确定清空"
      );
      if (!ok) {
        return;
      }
      this.clearing = true;
      try {
        await mbClearMeViewHistory();
        this.items = [];
      } catch {
        /* ignore */
      } finally {
        this.clearing = false;
      }
    },
    async onDelete(row) {
      if (!this.isMinibiliMode || !row || this.deletingKey) {
        return;
      }
      const key = this.entryKey(row);
      this.deletingKey = key;
      try {
        if (this.isLiveRow(row)) {
          await mbDeleteMeLiveViewHistoryEntry(Number(row.live_room_id));
          this.items = this.items.filter(
            (r) =>
              !(
                this.isLiveRow(r) &&
                Number(r.live_room_id) === Number(row.live_room_id)
              )
          );
        } else if (this.isArticleRow(row)) {
          await mbDeleteMeArticleViewHistoryEntry(Number(row.article_id));
          this.items = this.items.filter(
            (r) =>
              !(
                this.isArticleRow(r) &&
                Number(r.article_id) === Number(row.article_id)
              )
          );
        } else {
          await mbDeleteMeViewHistoryEntry(Number(row.video_id));
          this.items = this.items.filter(
            (r) =>
              !(
                !this.isArticleRow(r) &&
                !this.isLiveRow(r) &&
                Number(r.video_id) === Number(row.video_id)
              )
          );
        }
      } catch {
        /* ignore */
      } finally {
        this.deletingKey = "";
      }
    }
  }
};
</script>

<style lang="scss" scoped>
@import "./view-history.scss";
</style>

<style lang="scss">
@import "@/styles/mb-hist-msgbox.scss";
</style>
