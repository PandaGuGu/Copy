<template>
  <div class="secondary-nav">
    <div class="sn-inner">
      <!-- 左侧快捷入口 -->
      <div class="sn-left">
        <a href="https://t.bilibili.com" class="sn-quick-entry" title="动态" target="_blank">
          <span class="sn-icon-circle sn-icon-dynamic">
            <svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
              <path d="M12 2L15.09 8.26L22 9.27L17 14.14L18.18 21.02L12 17.77L5.82 21.02L7 14.14L2 9.27L8.91 8.26L12 2Z" fill="white" stroke="white" stroke-width="1"/>
            </svg>
          </span>
          <span class="sn-entry-text">动态</span>
        </a>
        <a href="https://www.bilibili.com/v/popular" class="sn-quick-entry" title="热门" target="_blank">
          <span class="sn-icon-circle sn-icon-hot">
            <svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
              <path d="M12 2C12 2 10 7 10 10C10 13 12 16 12 16C12 16 14 13 14 10C14 7 12 2 12 2Z" fill="white"/>
              <path d="M8 8C8 8 5 11 5 15C5 18.86 8.14 22 12 22C15.86 22 19 18.86 19 15C19 11 16 8 16 8C16 8 14 11 12 11C10 11 8 8 8 8Z" fill="white"/>
            </svg>
          </span>
          <span class="sn-entry-text">热门</span>
        </a>
      </div>

      <!-- 中部分类标签：2行11列，均匀分布 -->
      <div class="sn-center">
        <template v-for="item in categoryAll" :key="item.name">
          <router-link
            v-if="item.innerZone"
            :to="'/zone/' + item.innerZone"
            class="sn-cat-btn"
          >
            {{ item.name }}
          </router-link>
          <a
            v-else
            :href="item.url"
            target="_blank"
            class="sn-cat-btn"
          >
            {{ item.name }}
            <span v-if="item.hasArrow" class="sn-arrow-down">
              <svg viewBox="0 0 12 12" fill="none" xmlns="http://www.w3.org/2000/svg">
                <path d="M3 5L6 8L9 5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
              </svg>
            </span>
          </a>
        </template>
      </div>

      <!-- 右侧附加栏目：二行三列 -->
      <div class="sn-right">
        <div class="sn-right-grid">
          <a href="https://www.bilibili.com/v/column" target="_blank" class="sn-right-link">专栏</a>
          <a href="https://www.bilibili.com/blackboard/activity" target="_blank" class="sn-right-link">活动</a>
          <a href="https://www.bilibili.com/blackboard/community" target="_blank" class="sn-right-link">社区中心</a>
          <a href="/#/minibili/live" class="sn-right-link">直播</a>
          <a href="https://www.bilibili.com/v/classroom" target="_blank" class="sn-right-link">课堂</a>
          <a href="https://www.bilibili.com/v/music" target="_blank" class="sn-right-link sn-right-highlight">新歌热榜</a>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
export default {
  name: "SecondaryNav",
  data() {
    return {
      categoryAll: [
        { name: "番剧", url: "https://www.bilibili.com/v/anime", innerZone: "番剧" },
        { name: "国创", url: "https://www.bilibili.com/v/guochuang", innerZone: "国创" },
        { name: "综艺", url: "https://www.bilibili.com/v/variety", innerZone: "综艺" },
        { name: "动画", url: "https://www.bilibili.com/v/douga", innerZone: "动画" },
        { name: "鬼畜", url: "https://www.bilibili.com/v/kichiku", innerZone: "鬼畜" },
        { name: "舞蹈", url: "https://www.bilibili.com/v/dance", innerZone: "舞蹈" },
        { name: "娱乐", url: "https://www.bilibili.com/v/ent", innerZone: "娱乐" },
        { name: "科技数码", url: "https://www.bilibili.com/v/tech", innerZone: "科技" },
        { name: "美食", url: "https://www.bilibili.com/v/life", innerZone: "生活" },
        { name: "汽车", url: "https://www.bilibili.com/v/car", innerZone: "汽车" },
        { name: "体育运动", url: "https://www.bilibili.com/v/sports", innerZone: "体育运动" },
        { name: "电影", url: "https://www.bilibili.com/v/movie", innerZone: "放映厅" },
        { name: "电视剧", url: "https://www.bilibili.com/v/tv", innerZone: "放映厅" },
        { name: "纪录片", url: "https://www.bilibili.com/v/documentary", innerZone: "放映厅" },
        { name: "游戏", url: "https://www.bilibili.com/v/game", innerZone: "游戏" },
        { name: "音乐", url: "https://www.bilibili.com/v/music", innerZone: "音乐" },
        { name: "影视", url: "https://www.bilibili.com/v/cinephile", innerZone: "影视" },
        { name: "知识", url: "https://www.bilibili.com/v/knowledge", innerZone: "知识" },
        { name: "资讯", url: "https://www.bilibili.com/v/information", innerZone: "资讯" },
        { name: "小剧场", url: "https://www.bilibili.com/v/theatre", innerZone: "小剧场" },
        { name: "时尚美妆", url: "https://www.bilibili.com/v/fashion", innerZone: "时尚" },
        { name: "更多", url: "https://www.bilibili.com/v/category", hasArrow: true }
      ]
    };
  }
};
</script>

<style lang="scss" scoped>
@import "../../../style/mixin";

.secondary-nav {
  width: 100%;
  background: #fff;
  border-bottom: 1px solid #e5e9ef;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.06);
}

.sn-inner {
  display: flex;
  align-items: flex-start;
  width: 100%;
  margin: 0 auto;
  padding: 23px 0;
}

/* 左侧快捷入口 */
.sn-left {
  display: flex;
  align-items: center;
  gap: 16px;
  padding-left: 8px;
  padding-right: 16px;
  margin-right: 12px;
  border-right: 1px solid #e5e9ef;
  flex: 0 0 7%;
}

.sn-quick-entry {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  text-decoration: none;
  cursor: pointer;
  padding: 2px 4px;
  transition: opacity 0.2s;

  &:hover {
    opacity: 0.8;
    .sn-entry-text {
      color: #00a1d6;
    }
  }
}

.sn-icon-circle {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 34px;
  height: 34px;
  border-radius: 50%;
  flex-shrink: 0;

  svg {
    width: 18px;
    height: 18px;
  }
}

.sn-icon-dynamic {
  background: #ff8c00;
}

.sn-icon-hot {
  background: #ff4444;
}

.sn-entry-text {
  font-size: 11px;
  color: #666;
  line-height: 1;
  transition: color 0.2s;
}

/* 中部分类标签：2行×11列，占满剩余空间 */
.sn-center {
  flex: 1;
  min-width: 0;
  display: grid;
  grid-template-columns: repeat(11, 1fr);
  grid-template-rows: repeat(2, auto);
  gap: 4px 6px;
  align-items: center;
  margin-right: 12px;
}

.sn-cat-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 5px 8px;
  font-size: 12px;
  color: #555;
  text-decoration: none;
  border-radius: 3px;
  background: #f5f5f5;
  transition: all 0.2s;
  white-space: nowrap;
  cursor: pointer;

  &:hover {
    color: #00a1d6;
    background: #e5f4fb;
  }
}

.sn-arrow-down {
  display: inline-flex;
  align-items: center;
  margin-left: 2px;

  svg {
    width: 10px;
    height: 10px;
  }
}

/* 右侧附加栏目：固定 160px 宽度 */
.sn-right {
  flex: 0 0 240px;
  min-width: 0;
  padding-left: 12px;
  border-left: 1px solid #e5e9ef;
}

.sn-right-grid {
  display: grid;
  grid-template-columns: repeat(3, auto);
  grid-template-rows: repeat(2, auto);
  gap: 4px 8px;
}

.sn-right-link {
  font-size: 12px;
  color: #555;
  text-decoration: none;
  padding: 5px 10px;
  border-radius: 3px;
  background: #f5f5f5;
  transition: all 0.2s;
  white-space: nowrap;
  cursor: pointer;
  text-align: center;

  &:hover {
    color: #00a1d6;
    background: #e5f4fb;
  }
}

.sn-right-highlight {
  color: #333;
  font-weight: 500;

  &:hover {
    color: #333;
    background: #e5e5e5;
  }
}
</style>