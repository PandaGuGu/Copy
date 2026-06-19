<template>
  <div class="video-feed-module">
    <div class="video-feed-grid">
      <div
        class="feed-video-card"
        v-for="(item, idx) in videos"
        :key="'feed-' + item.aid + '-' + idx"
      >
        <div class="cover-link">
          <router-link :to="{ name: 'video', params: { aid: 'BV' + item.aid } }">
            <div class="cover-img">
              <img v-lazy="item.pic" :alt="item.title" />
            </div>
          </router-link>
        </div>
        <div class="cover-info-bar">
          <span class="cover-title">{{ item.title }}</span>
          <span class="cover-meta">{{ formatPlay(item.play) }}播放 · {{ item.author }}</span>
        </div>
      </div>
    </div>
    <div class="feed-loading" v-if="loading">加载中...</div>
    <div ref="feedTrigger" class="feed-trigger"></div>
  </div>
</template>

<script>
import http from "../../../utils/http";

export default {
  name: "VideoFeed",
  data() {
    return {
      videos: [],
      baseVideos: [],   // 原始视频列表（用于循环追加）
      cursor: "",
      isFirstLoad: true, // 是否首次加载（替代 cursor === "" 判断）
      loading: false,
      loopMode: false,
      loadLock: false,     // 防止 Observer 短时间多次触发
      _lockTimer: null,
    };
  },
  mounted() {
    this.loadVideos();
    this.$nextTick(() => this.setupObserver());
  },
  beforeDestroy() {
    if (this.observer) {
      this.observer.disconnect();
    }
    if (this._lockTimer) {
      clearTimeout(this._lockTimer);
    }
  },
  methods: {
    async loadVideos() {
      // 锁机制：防止 Observer 短时间多次触发
      if (this.loadLock) return;
      this.loadLock = true;

      // 循环模式：直接追加，不做 API 请求
      if (this.loopMode) {
        this.appendLoopVideos();
        // 800ms 后解锁（循环模式用较长锁，避免列表膨胀太快）
        this._lockTimer = setTimeout(() => { this.loadLock = false; }, 800);
        return;
      }

      this.loading = true;
      try {
        const params = { limit: 15 };
        if (!this.isFirstLoad && this.cursor) {
          params.cursor = this.cursor;
        } else {
          params.sort = "hot";
        }

        const res = await http.get("/api/v1/videos", { params });
        const data = res.data || res || {};
        const items = (data.items || []).map(this.mapVideoItem);

        if (this.isFirstLoad) {
          // 首次加载：初始化 baseVideos 和 videos
          this.baseVideos = items;
          if (items.length > 0) {
            this.videos = [...items];
            // 如果不足 15 条，循环复制填满
            while (this.videos.length < 15 && this.baseVideos.length > 0) {
              const copy = this.shuffle([...this.baseVideos]);
              this.videos = this.videos.concat(copy);
            }
            this.videos = this.videos.slice(0, 15);
          }
          this.isFirstLoad = false;
          // 首次返回不足 15 条 → 后端数据少，直接进入循环模式
          if (items.length < 15) {
            this.loopMode = true;
          }
        } else {
          // 非首次加载：追加新视频（去重）
          const existingIds = new Set(this.videos.map(v => v.aid));
          const newItems = items.filter(v => !existingIds.has(v.aid));
          if (newItems.length > 0) {
            this.videos = this.videos.concat(newItems);
          }
          // 返回空或不足 15 条 → 后端没更多数据了
          if (items.length === 0 || items.length < 15) {
            this.loopMode = true;
          }
        }

        this.cursor = (data.next_cursor || "").toString();
      } catch (e) {
        console.error("加载推荐视频失败", e);
      }
      this.loading = false;
      // 非循环模式：500ms 后解锁
      this._lockTimer = setTimeout(() => { this.loadLock = false; }, 500);
    },

    appendLoopVideos() {
      if (this.baseVideos.length === 0) return;
      const shuffled = this.shuffle([...this.baseVideos]);
      // 每次追加 5 条（不要一次加 15 条，避免列表跳跃）
      const append = shuffled.slice(0, 5);
      this.videos = this.videos.concat(append);
      // 防止列表无限增长，保留最近 100 条
      if (this.videos.length > 100) {
        this.videos = this.videos.slice(-60);
      }
    },

    shuffle(arr) {
      for (let i = arr.length - 1; i > 0; i--) {
        const j = Math.floor(Math.random() * (i + 1));
        [arr[i], arr[j]] = [arr[j], arr[i]];
      }
      return arr;
    },

    mapVideoItem(v) {
      return {
        aid: v.id,
        title: v.title,
        pic: v.cover_url || "",
        author: v.uploader || "未知UP主",
        play: v.play_count || 0,
      };
    },

    formatPlay(n) {
      if (n >= 10000) return (n / 10000).toFixed(1) + "万";
      if (n >= 1000) return (n / 1000).toFixed(1) + "千";
      return n || 0;
    },

    setupObserver() {
      if (!this.$refs.feedTrigger) return;
      if (this.observer) {
        this.observer.disconnect();
      }
      this.observer = new IntersectionObserver(
        (entries) => {
          if (entries[0].isIntersecting) {
            this.loadVideos();
          }
        },
        { rootMargin: "600px" }
      );
      this.observer.observe(this.$refs.feedTrigger);
    },
  },
};
</script>

<style lang="scss" scoped>
.video-feed-module {
  margin-top: 28px;
  .video-feed-grid {
    display: grid;
    grid-template-columns: repeat(5, 1fr);
    gap: 16px 14px;
  }
  .feed-video-card {
    .cover-link {
      display: block;
      border-radius: 4px;
      overflow: hidden;
      .cover-img {
        width: 100%;
        aspect-ratio: 16 / 9;
        overflow: hidden;
        background: #f0f0f0;
        img {
          width: 100%;
          height: 100%;
          object-fit: cover;
          display: block;
          border-radius: 4px;
        }
      }
    }
    .cover-info-bar {
      padding: 8px 0 4px;
      .cover-title {
        display: block;
        font-size: 13px;
        line-height: 18px;
        color: #222;
        overflow: hidden;
        white-space: nowrap;
        text-overflow: ellipsis;
        margin-bottom: 4px;
      }
      .cover-meta {
        display: block;
        font-size: 12px;
        color: #999;
      }
    }
  }
  .feed-loading {
    text-align: center;
    padding: 28px 0;
    color: #999;
    font-size: 14px;
  }
  .feed-trigger {
    height: 1px;
    margin-top: 12px;
  }
}
</style>
