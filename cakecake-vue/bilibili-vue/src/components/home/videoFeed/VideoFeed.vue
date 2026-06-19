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
  </div>
</template>

<script>
import http from "../../../utils/http";

export default {
  name: "VideoFeed",
  data() {
    return {
      videos: [],
      baseVideos: [],
      cursor: "",
      isFirstLoad: true,
      loading: false,
      loopMode: false,
      loadLock: false,
    };
  },
  mounted() {
    this.loadVideos();
    window.addEventListener("scroll", this.handleScroll);
    // 初始检查一次（内容不足一屏时）
    this.$nextTick(() => this.checkAndLoad());
  },
  beforeDestroy() {
    window.removeEventListener("scroll", this.handleScroll);
  },
  methods: {
    // scroll 事件处理：距底部 600px 内即触发
    handleScroll() {
      this.checkAndLoad();
    },

    // 检查是否需要加载更多
    checkAndLoad() {
      const scrollY = window.scrollY || document.documentElement.scrollTop;
      const windowH = window.innerHeight;
      const docH = document.documentElement.offsetHeight;
      // 距底部 600px 内即触发加载
      if (scrollY + windowH >= docH - 600) {
        this.loadVideos();
      }
    },

    async loadVideos() {
      if (this.loadLock) return;
      this.loadLock = true;

      // 循环模式：直接追加
      if (this.loopMode) {
        this.appendLoopVideos();
        setTimeout(() => { this.loadLock = false; }, 800);
        // 追加完后再次检查（可能还不足一屏）
        this.$nextTick(() => this.checkAndLoad());
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
          this.baseVideos = items;
          if (items.length > 0) {
            this.videos = [...items];
            while (this.videos.length < 15 && this.baseVideos.length > 0) {
              const copy = this.shuffle([...this.baseVideos]);
              this.videos = this.videos.concat(copy);
            }
            this.videos = this.videos.slice(0, 15);
          }
          this.isFirstLoad = false;
          if (items.length < 15) this.loopMode = true;
        } else {
          const existingIds = new Set(this.videos.map(v => v.aid));
          const newItems = items.filter(v => !existingIds.has(v.aid));
          if (newItems.length > 0) {
            this.videos = this.videos.concat(newItems);
          }
          if (items.length === 0 || items.length < 15) {
            this.loopMode = true;
          }
        }

        this.cursor = (data.next_cursor || "").toString();
      } catch (e) {
        console.error("加载推荐视频失败", e);
      }
      this.loading = false;
      setTimeout(() => { this.loadLock = false; }, 500);
      // 加载完后再次检查（内容可能还不足一屏）
      this.$nextTick(() => this.checkAndLoad());
    },

    appendLoopVideos() {
      if (this.baseVideos.length === 0) return;
      const shuffled = this.shuffle([...this.baseVideos]);
      const append = shuffled.slice(0, 5);
      this.videos = this.videos.concat(append);
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
}
</style>
