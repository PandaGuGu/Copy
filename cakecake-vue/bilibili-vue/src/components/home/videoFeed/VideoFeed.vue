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
      loading: false,
      loopMode: false,  // 是否进入循环模式（后端数据已耗尽）
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
  },
  methods: {
    async loadVideos() {
      if (this.loading) return;

      // 循环模式：直接前端追加，不调 API
      if (this.loopMode) {
        this.appendLoopVideos();
        return;
      }

      this.loading = true;
      try {
        const params = { limit: 15 };
        if (this.cursor) {
          params.cursor = this.cursor;
        } else {
          params.sort = "hot";  // 首次加载：算法推荐优先
        }

        const res = await http.get("/api/v1/videos", { params });
        const data = res.data || res || {};
        const items = (data.items || []).map(this.mapVideoItem);

        if (this.cursor === "") {
          // 首次加载：保存原始列表，循环复制填满 15 条
          this.baseVideos = items;
          this.videos = [...items];
          while (this.videos.length < 15 && this.baseVideos.length > 0) {
            const copy = this.shuffle([...this.baseVideos]);
            this.videos = this.videos.concat(copy);
          }
          this.videos = this.videos.slice(0, 15);
        } else {
          // 后续加载：去重追加
          const existingIds = new Set(this.videos.map(v => v.aid));
          const newItems = items.filter(v => !existingIds.has(v.aid));
          if (newItems.length > 0) {
            this.videos = this.videos.concat(newItems);
          }
        }

        this.cursor = (data.next_cursor || "").toString();
        // 后端数据已耗尽 → 进入循环模式
        if (!this.cursor && this.baseVideos.length > 0) {
          this.loopMode = true;
        }
      } catch (e) {
        console.error("加载推荐视频失败", e);
      }
      this.loading = false;
    },

    appendLoopVideos() {
      // 循环模式：从 baseVideos 随机取 15 条追加
      const shuffled = this.shuffle([...this.baseVideos]);
      const append = shuffled.slice(0, 15);
      this.videos = this.videos.concat(append);

      // 防止列表无限增长：超过 200 条时保留最后 100 条
      if (this.videos.length > 200) {
        this.videos = this.videos.slice(-100);
      }
      this.loading = false;
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
      this.observer = new IntersectionObserver(
        (entries) => {
          if (entries[0].isIntersecting) {
            this.loadVideos();
          }
        },
        { rootMargin: "400px" }
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
  }
}
</style>
