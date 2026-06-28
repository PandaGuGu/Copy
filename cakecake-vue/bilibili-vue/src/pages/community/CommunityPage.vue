<template>
  <div class="community-page">
    <!-- 平台规则 -->
    <h2 class="section-title">平台规则</h2>
    <div class="rule-card">
      <div class="card-left">
        <div class="banner-block">
          <div class="banner-placeholder">📋 社区公约</div>
        </div>
        <div class="link-list">
          <div class="link-item"><span>📜</span><span>社区行为规范</span></div>
          <div class="link-item"><span>⚖️</span><span>创作规范与审核标准</span></div>
          <div class="link-item"><span>🛡️</span><span>版权与投诉指引</span></div>
        </div>
      </div>
      <div class="card-right">
        <div class="notice-title">📢 公告与通知</div>
        <div v-for="n in notices" :key="n.id" class="notice-item">
          <span :class="['tag', n.type === 'blue' ? 'tag-blue' : 'tag-pink']">{{ n.badge }}</span>
          <span>{{ n.text }}</span>
        </div>
        <div class="more-notice">查看更多 &gt;</div>
      </div>
    </div>

    <!-- 规则解读 -->
    <h2 class="section-title">规则解读</h2>
    <div class="slide-row">
      <div v-for="(s, i) in slides" :key="i" class="slide-item">
        <div class="slide-icon">{{ s.icon }}</div>
        <span>{{ s.label }}</span>
      </div>
    </div>

    <!-- 社区参与 -->
    <h2 class="section-title">社区参与</h2>
    <div class="participate-card">
      <div class="func-row">
        <div class="func-item" v-for="f in funcs" :key="f.label">
          <div class="func-icon">{{ f.icon }}</div>
          <span>{{ f.label }}</span>
        </div>
      </div>

      <div v-for="(block, bi) in topicBlocks" :key="bi" class="topic-block">
        <h4>{{ block.title }}</h4>
        <div v-for="(t, ti) in block.items" :key="ti" class="topic-item">
          <div class="topic-left">
            <span>{{ t.text }}</span>
            <span v-if="t.hot" class="hot-tag">热</span>
            <span v-if="t.new" class="new-tag">新</span>
          </div>
          <span class="topic-arrow">&gt;</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
export default {
  name: "CommunityPage",
  data() {
    return {
      notices: [
        { id: 1, type: "blue", badge: "公告", text: "关于近期内容审核标准更新的说明" },
        { id: 2, type: "pink", badge: "活动", text: "2026创作者激励计划第二期火热进行中" },
        { id: 3, type: "pink", badge: "活动", text: "社区投稿大赛 - 赢取限定徽章" },
        { id: 4, type: "blue", badge: "公告", text: "平台功能更新：新增视频章节与字幕支持" },
      ],
      slides: [
        { icon: "📹", label: "视频创作规范" },
        { icon: "💬", label: "评论互动准则" },
        { icon: "🎵", label: "音乐版权须知" },
        { icon: "🏷️", label: "标签使用规范" },
        { icon: "📡", label: "直播行为守则" },
      ],
      funcs: [
        { icon: "📝", label: "投稿反馈" },
        { icon: "🔄", label: "申诉通道" },
      ],
      topicBlocks: [
        {
          title: "社区话题",
          items: [
            { text: "你最期待的下一个平台功能是？", hot: true },
            { text: "分享你的创作故事" },
          ],
        },
        {
          title: "产品话题",
          items: [
            { text: "新播放器体验反馈收集", new: true },
            { text: "字幕功能使用调研", new: true },
            { text: "移动端适配建议征集" },
            { text: "对于搜索体验的改进意见", hot: true },
            { text: "首页推荐算法的优化方向" },
          ],
        },
      ],
    };
  },
};
</script>

<style scoped>
.community-page {
  max-width: 1160px;
  margin: 0 auto;
  padding: 20px 60px 60px;
  background: #f7f8fa;
  min-height: 100vh;
}
.section-title {
  text-align: center;
  font-size: 22px;
  color: #333;
  margin: 40px 0 24px;
}

/* Rule card */
.rule-card {
  background: #fff;
  border-radius: 8px;
  padding: 20px;
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 20px;
}
.card-left {
  display: flex;
  flex-direction: column;
  gap: 16px;
}
.banner-block {
  width: 100%;
  height: 140px;
  background: linear-gradient(135deg, #e8f4fd, #d0eafc);
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
}
.banner-placeholder {
  font-size: 20px;
  color: #5a9fd4;
  font-weight: 600;
}
.link-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.link-item {
  display: flex;
  align-items: center;
  gap: 8px;
  color: #555;
  font-size: 14px;
}
.card-right {
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.notice-title {
  font-size: 18px;
  font-weight: 600;
  margin-bottom: 4px;
}
.notice-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 4px 0;
  font-size: 14px;
}
.tag {
  padding: 2px 6px;
  font-size: 12px;
  border-radius: 3px;
  color: #fff;
  flex-shrink: 0;
}
.tag-blue { background: #82c8f0; }
.tag-pink { background: #f8a0c0; }
.more-notice { text-align: right; font-size: 14px; color: #999; cursor: pointer; }

/* Slide row */
.slide-row {
  display: flex;
  gap: 12px;
  justify-content: center;
  flex-wrap: wrap;
}
.slide-item {
  width: 160px;
  height: 100px;
  background: #fff;
  border-radius: 8px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  box-shadow: 0 2px 8px rgba(0,0,0,0.06);
}
.slide-icon { font-size: 32px; }
.slide-item span { font-size: 13px; color: #555; }

/* Participate card */
.participate-card {
  background: #fff;
  border-radius: 8px;
  padding: 30px 24px;
}
.func-row {
  display: flex;
  justify-content: space-around;
  margin-bottom: 30px;
}
.func-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  font-size: 14px;
  color: #555;
}
.func-icon {
  width: 60px;
  height: 60px;
  background: #f0f4f8;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 24px;
}
.topic-block {
  margin-bottom: 20px;
}
.topic-block h4 {
  margin-bottom: 12px;
  font-size: 16px;
  font-weight: 600;
}
.topic-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 0;
  border-bottom: 1px solid #eee;
  cursor: pointer;
}
.topic-item:hover { background: #f9f9f9; }
.topic-left {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
}
.topic-arrow { color: #ccc; }
.hot-tag {
  font-size: 12px;
  padding: 1px 4px;
  background: #ffdf90;
  border-radius: 2px;
}
.new-tag {
  font-size: 12px;
  padding: 1px 4px;
  background: #ffbaba;
  border-radius: 2px;
}

@media (max-width: 768px) {
  .community-page { padding: 20px 16px 40px; }
  .rule-card { grid-template-columns: 1fr; }
  .slide-row { justify-content: flex-start; overflow-x: auto; }
}
</style>
