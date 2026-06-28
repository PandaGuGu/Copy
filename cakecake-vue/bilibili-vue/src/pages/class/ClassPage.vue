<template>
  <div class="class-page">
    <!-- 顶部Banner -->
    <div class="top-banner">
      <div class="banner-inner skeleton"><div class="skeleton-shimmer"></div></div>
      <div class="banner-right-card">
        <div class="brc-title">热门课程</div>
        <div v-for="a in hotArticles.slice(0, 3)" :key="a.id" class="brc-item">
          <router-link :to="`/article/${a.id}`" class="brc-link">{{ a.title }}</router-link>
        </div>
      </div>
    </div>

    <!-- 筛选导航栏 -->
    <div class="nav-filter">
      <div v-for="(cat, idx) in categories" :key="idx" class="filter-item" :class="{ active: idx === 0 }">
        {{ cat }}
      </div>
    </div>

    <!-- 猜你想学 -->
    <div class="section-title">
      <span>猜你想学</span>
      <span class="section-more">换一换</span>
    </div>
    <div class="row-scroll">
      <div v-for="a in guessArticles" :key="a.id" class="course-card">
        <router-link :to="`/article/${a.id}`" class="card-cover-wrap">
          <img v-if="a.cover" :src="a.cover" class="card-cover" loading="lazy" />
          <div v-else class="card-cover skeleton"><div class="skeleton-shimmer"></div></div>
        </router-link>
        <span class="card-label">{{ a.title }}</span>
      </div>
      <div v-for="i in (6 - guessArticles.length)" :key="'g-sk-'+i" class="course-card">
        <div class="card-cover skeleton"><div class="skeleton-shimmer"></div></div>
        <div class="skeleton-text" style="width:70%;height:14px;"></div>
      </div>
    </div>

    <!-- 新课推荐 + 右侧排行 -->
    <div class="section-title">
      <span>新课推荐</span>
      <div>
        <span class="section-tab">最新</span>
        <span class="section-tab muted">最热</span>
      </div>
    </div>
    <div class="new-course-wrap">
      <div class="left-course-grid">
        <div v-for="a in newArticles" :key="a.id" class="course-card">
          <router-link :to="`/article/${a.id}`" class="card-cover-wrap">
            <img v-if="a.cover" :src="a.cover" class="card-cover" loading="lazy" />
            <div v-else class="card-cover skeleton"><div class="skeleton-shimmer"></div></div>
          </router-link>
          <span class="card-label">{{ a.title }}</span>
        </div>
        <div v-for="i in (8 - newArticles.length)" :key="'n-sk-'+i" class="course-card">
          <div class="card-cover skeleton"><div class="skeleton-shimmer"></div></div>
          <div class="skeleton-text" style="width:60%;height:14px;"></div>
        </div>
      </div>
      <div class="rank-list">
        <div v-for="(a, idx) in hotArticles.slice(0, 5)" :key="a.id" class="rank-item">
          <span :class="['rank-num', { top3: idx < 3 }]">{{ idx + 1 }}</span>
          <router-link :to="`/article/${a.id}`" class="rank-link">{{ a.title }}</router-link>
        </div>
        <div v-for="i in (5 - Math.min(hotArticles.length, 5))" :key="'r-sk-'+i" class="rank-item skeleton">
          <div class="skeleton-shimmer"></div>
        </div>
      </div>
    </div>

    <!-- 即将上线 -->
    <div class="section-title"><span>即将上线</span></div>
    <div class="coming-row">
      <div v-for="i in 5" :key="'c-'+i" class="course-card">
        <div class="card-cover tall skeleton"><div class="skeleton-shimmer"></div></div>
        <div class="skeleton-text" style="width:50%;height:14px;"></div>
      </div>
    </div>

    <!-- 底部推荐 -->
    <div class="section-title">
      <span>推荐课程</span>
      <span class="section-more">查看更多 &gt;</span>
    </div>
    <div class="bottom-row">
      <div v-for="a in allArticles" :key="a.id" class="course-card">
        <router-link :to="`/article/${a.id}`" class="card-cover-wrap">
          <img v-if="a.cover" :src="a.cover" class="card-cover" loading="lazy" />
          <div v-else class="card-cover skeleton"><div class="skeleton-shimmer"></div></div>
        </router-link>
        <span class="card-label">{{ a.title }}</span>
      </div>
      <div v-for="i in (6 - allArticles.length)" :key="'b-sk-'+i" class="course-card">
        <div class="card-cover skeleton"><div class="skeleton-shimmer"></div></div>
        <div class="skeleton-text" style="width:60%;height:14px;"></div>
      </div>
    </div>
  </div>
</template>

<script>
import http from "@/utils/http";

export default {
  name: "ClassPage",
  data() {
    return {
      items: [],
      categories: ["全部", "编程开发", "AI人工智能", "设计创意", "职场通用", "语言学习", "考试考证", "兴趣爱好", "音乐舞蹈", "生活技能", "运动健身", "更多"],
    };
  },
  computed: {
    allArticles() {
      return this.items.map((a) => this.mapArt(a)).slice(0, 6);
    },
    guessArticles() {
      return this.shuffle(this.items.map((a) => this.mapArt(a))).slice(0, 6);
    },
    newArticles() {
      return this.items.map((a) => this.mapArt(a)).slice(0, 8);
    },
    hotArticles() {
      return [...this.items].sort((a, b) => (b.view_count || 0) - (a.view_count || 0))
        .map((a) => this.mapArt(a)).slice(0, 5);
    },
  },
  created() {
    this.fetch();
  },
  methods: {
    mapArt(a) {
      return {
        id: a.id,
        title: a.title || "课程",
        cover: a.cover_url || "",
        desc: (a.summary || "").slice(0, 60),
      };
    },
    async fetch() {
      try {
        const res = await http.get("/api/v1/articles", { params: { limit: 20 } });
        if (res && res.code === 0 && res.data) {
          this.items = res.data.items || [];
        }
      } catch (e) {
        console.warn("ClassPage fetch:", e);
      }
    },
    shuffle(arr) {
      const a = [...arr];
      for (let i = a.length - 1; i > 0; i--) {
        const j = Math.floor(Math.random() * (i + 1));
        [a[i], a[j]] = [a[j], a[i]];
      }
      return a;
    },
  },
};
</script>

<style scoped>
.class-page {
  background: #fff;
  min-height: 100vh;
  max-width: 1400px;
  margin: 0 auto;
}

/* Top banner */
.top-banner {
  width: 100%; height: 220px;
  position: relative; display: flex;
}
.banner-inner {
  flex: 1; background: #ddd;
}
.banner-right-card {
  position: absolute; top: 16px; right: 16px;
  width: 220px; background: #f3f3f3; border-radius: 6px;
  padding: 12px;
}
.brc-title { font-size: 14px; font-weight: 600; margin-bottom: 8px; }
.brc-item { margin: 6px 0; }
.brc-link {
  font-size: 13px; color: #333; text-decoration: none;
  overflow: hidden; text-overflow: ellipsis; white-space: nowrap; display: block;
}
.brc-link:hover { color: #00a1d6; }

/* Nav filter */
.nav-filter {
  padding: 16px 24px; border-bottom: 1px solid #eee;
  display: flex; gap: 10px; align-items: center; flex-wrap: wrap;
}
.filter-item {
  padding: 6px 14px; background: #f5f5f6; border-radius: 4px;
  font-size: 14px; color: #333; cursor: pointer;
}
.filter-item.active { background: #00a1d6; color: #fff; }
.filter-item:hover { background: #e0e0e0; }
.filter-item.active:hover { background: #00a1d6; }

/* Section title */
.section-title {
  padding: 20px 24px 12px;
  display: flex; justify-content: space-between; align-items: center;
  font-size: 18px; font-weight: 600;
}
.section-more { font-size: 13px; color: #999; cursor: pointer; font-weight: 400; }
.section-more:hover { color: #00a1d6; }
.section-tab { font-size: 14px; cursor: pointer; color: #00a1d6; font-weight: 500; }
.section-tab.muted { color: #999; margin-left: 20px; font-weight: 400; }

/* Row scroll */
.row-scroll {
  display: flex; gap: 14px; padding: 0 24px 16px; overflow-x: auto;
}
.course-card {
  width: 150px; flex-shrink: 0;
  display: flex; flex-direction: column; gap: 6px;
}
.card-cover-wrap { display: block; text-decoration: none; }
.card-cover {
  width: 100%; height: 90px; background: #ddd; border-radius: 4px;
  object-fit: cover; display: block;
}
.card-cover.tall { height: 160px; }
.card-label {
  font-size: 13px; line-height: 1.3; color: #222;
  display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical; overflow: hidden;
}

/* New course wrap */
.new-course-wrap {
  display: grid; grid-template-columns: 1fr 220px;
  gap: 20px; padding: 0 24px 20px;
}
.left-course-grid {
  display: grid; grid-template-columns: repeat(4, 1fr); gap: 14px;
}

/* Rank list */
.rank-list { display: flex; flex-direction: column; gap: 10px; }
.rank-item {
  display: flex; align-items: center; gap: 8px;
  height: 50px; padding: 0 8px; background: #f8f8f8; border-radius: 4px;
}
.rank-num { font-size: 16px; color: #999; width: 22px; text-align: center; flex-shrink: 0; }
.rank-num.top3 { color: #00a1d6; font-weight: 600; }
.rank-link {
  flex: 1; font-size: 13px; color: #333; text-decoration: none;
  overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
}
.rank-link:hover { color: #00a1d6; }

/* Coming row */
.coming-row { display: flex; gap: 14px; padding: 0 24px 20px; }

/* Bottom */
.bottom-row { display: flex; gap: 14px; padding: 0 24px 30px; }

/* skeleton */
.skeleton { position: relative; background: #e8e8e8 !important; overflow: hidden; }
.skeleton-shimmer {
  position: absolute; inset: 0;
  background: linear-gradient(90deg, transparent 0%, rgba(255,255,255,0.5) 50%, transparent 100%);
  animation: shimmer 1.5s infinite;
}
@keyframes shimmer { 0%{transform:translateX(-100%)} 100%{transform:translateX(100%)} }
.skeleton-text { background: #e8e8e8; border-radius: 4px; }

@media (max-width: 1000px) {
  .new-course-wrap { grid-template-columns: 1fr; }
  .left-course-grid { grid-template-columns: repeat(3, 1fr); }
}
</style>
