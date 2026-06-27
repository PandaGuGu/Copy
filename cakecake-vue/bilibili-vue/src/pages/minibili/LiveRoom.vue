<template>
  <div class="live-body">
    <div v-loading="loading" class="live-loading-wrapper">
      <div v-if="!loading && !room" class="live-empty">
        <p>直播间不存在或已结束</p>
        <router-link to="/minibili/live">返回直播列表</router-link>
      </div>

      <template v-else-if="room">
        <!-- 举报弹窗 -->
        <div class="report-overlay" v-if="showReportModal" @click.self="closeReportModal">
          <div class="report-dialog">
            <div class="report-header">
              <span class="report-title">举报</span>
              <span class="report-subtitle">举报本场直播</span>
              <span class="report-close" @click="closeReportModal">✕</span>
            </div>
            <div class="report-body">
              <div
                v-for="r in reportReasons"
                :key="r"
                class="report-card"
                :class="{ selected: selectedReason === r }"
                @click="selectedReason = r"
              >{{ r }}</div>
            </div>
            <div class="report-footer">
              <button class="report-submit-btn" :disabled="!selectedReason" @click="submitReport">发起举报</button>
            </div>
          </div>
        </div>

        <!-- 左侧主区域 -->
        <div class="left-wrap">
          <!-- 顶部标题栏 -->
          <div class="live-top-bar">
            <div class="live-title">
              <img v-if="room.avatar_url" :src="room.avatar_url" class="live-avatar-img" />
              <div v-else class="live-avatar-placeholder">{{ (room.host_name || "主")[0] }}</div>
              <span>{{ room.title }}</span>
              <span class="live-status">{{ room.status === 'live' ? '直播中' : '未开播' }}</span>
              <button v-if="!isBroadcaster" class="live-follow-btn" @click="sendFollowWs">{{ followed ? '已关注' : '+ 关注' }}</button>
            </div>
            <div class="tag-group">
              <div class="set-btn-wrap" ref="setBtnWrap">
                <span class="set-btn" @click.stop="toggleMore">更多设置</span>
                <div class="set-dropdown" v-show="showMore">
                  <div class="drop-row">
                    <div class="drop-item" @click="settingsVisible = !settingsVisible; reportVisible = false">
                      <span class="drop-icon">⚙</span>
                      <span class="drop-label">设置</span>
                    </div>
                    <div class="drop-item" @click="shareRoom">
                      <span class="drop-icon">🔗</span>
                      <span class="drop-label">分享</span>
                    </div>
                    <div class="drop-item" @click="copyRoomLink">
                      <span class="drop-icon">📋</span>
                      <span class="drop-label">复制</span>
                    </div>
                  </div>
                  <div class="drop-row">
                    <div class="drop-item" @click="openReportModal">
                      <span class="drop-icon">⚠</span>
                      <span class="drop-label">举报</span>
                    </div>
                  </div>
                  <!-- 设置子面板 -->
                  <div class="drop-sub" v-if="settingsVisible">
                    <div class="drop-sub-title">直播设置</div>
                    <label class="drop-check"><input type="checkbox" v-model="danmakuSetting" /> 弹幕飘屏</label>
                    <label class="drop-check"><input type="checkbox" v-model="audienceListSetting" /> 显示观众列表</label>
                    <label class="drop-check"><input type="checkbox" v-model="giftEffectSetting" /> 礼物特效</label>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <!-- 主画面 -->
          <div class="live-screen">
            <div v-if="warnMsg" class="admin-warn-banner">{{ warnMsg }}</div>
            <LivePlayer :room="room" :danmakus="danmakus" :gifts="giftQueue" />
          </div>

          <!-- 底部礼物栏 -->
          <div class="gift-bar">
            <div
              v-for="g in gifts"
              :key="g.key"
              class="gift-item"
              @click="sendGiftWs(g.key)"
              style="cursor:pointer"
            >
              <span class="gift-icon">{{ g.emoji }}</span>
              <span class="gift-name">{{ g.label }}</span>
            </div>
            <div class="gift-item" style="cursor:pointer" @click="sendGiftWs('rocket')">
              <span class="gift-icon">🎁</span>
              <span class="gift-name">天选福袋</span>
            </div>
            <div class="gift-item" style="cursor:pointer" @click="sendGiftWs('heart')">
              <span class="gift-icon">💝</span>
              <span class="gift-name">心动盲盒</span>
              <span class="gift-bls-tag">免费</span>
            </div>
            <div class="gift-more-wrap">
              <span class="recharge-label">大航海</span>
            </div>
          </div>
        </div>

        <!-- 右侧观众面板 -->
        <div class="right-sidebar">
          <div class="sidebar-title">
            房间观众
            <span class="sidebar-count" v-if="audienceCount > 0">({{ audienceCount }})</span>
          </div>

          <!-- 观众列表 -->
          <div class="audience-panel" v-if="audienceList.length > 0">
            <div class="audience-list">
              <span v-for="u in audienceList" :key="u" class="audience-user">{{ u }}</span>
            </div>
          </div>
          <div v-else class="audience-empty">暂无在线观众</div>

          <!-- 聊天消息 -->
          <div class="chat-msgs" ref="msgContainer">
            <div class="chat-msg chat-msg-sys notice">系统提示: 哔哩哔哩直播内容及互动评论须严格遵守直播规范，严禁传播违法违规、低俗血腥、吸烟酗酒、造谣诈骗等不良有害信息。</div>
            <template v-for="(msg, idx) in messages" :key="idx">
              <div v-if="msg.type === 'system'" class="chat-msg chat-msg-sys">{{ msg.content }}</div>
              <div v-else class="chat-msg">
                <span class="chat-msg-user">{{ msg.username }}</span>
                <span class="chat-msg-text">{{ msg.content }}</span>
              </div>
            </template>
          </div>

          <!-- 弹幕输入 -->
          <div class="danmaku-input-area">
            <div class="input-row">
              <span class="flag-tag">未佩戴</span>
              <input class="dan-input" v-model="inputText" placeholder="发个弹幕呗~" maxlength="100" @keyup.enter="sendChat" />
              <button class="send-btn" :disabled="!inputText.trim() || !connected" @click="sendChat">发送</button>
            </div>
          </div>
        </div>
      </template>
    </div>
  </div>
</template>

<script>
import { ref, reactive, nextTick, onMounted, onBeforeUnmount } from "vue";
import { useRoute } from "vue-router";
import { ElMessage } from "element-plus";
import { getLiveRoom } from "@/api/live";
import { mbToggleUserFollow } from "@/api/minibili";
import { getAccessToken } from "@/utils/authTokens";
import LivePlayer from "@/components/live/LivePlayer.vue";

const GIFT_ITEMS = [
  { key: "rose", emoji: "🌹", label: "玫瑰" },
  { key: "heart", emoji: "❤️", label: "小心心" },
  { key: "rocket", emoji: "🚀", label: "火箭" },
  { key: "star", emoji: "⭐", label: "星星" },
  { key: "cake", emoji: "🍰", label: "蛋糕" },
  { key: "flower", emoji: "🌸", label: "花束" }
];

export default {
  name: "LiveRoom",
  components: { LivePlayer },
  setup() {
    const route = useRoute();
    const loading = ref(true);
    const room = ref(null);
    const messages = reactive([]);
    const danmakus = ref([]);
    const giftQueue = reactive([]);
    const audienceList = ref([]);
    const audienceCount = ref(0);
    const showMore = ref(false);
    const inputText = ref("");
    const connected = ref(false);
    const followed = ref(false);
    const isBroadcaster = ref(false);
    const msgContainer = ref(null);
    const setBtnWrap = ref(null);
    const gifts = ref(GIFT_ITEMS);
    const settingsVisible = ref(false);
    const reportVisible = ref(false);
    const showReportModal = ref(false);
    const selectedReason = ref("");
    const warnMsg = ref("");
    const danmakuSetting = ref(true);
    const audienceListSetting = ref(true);
    const giftEffectSetting = ref(true);
    const reportReasons = [
      "违法违禁", "色情低俗", "赌博诈骗", "血腥暴力",
      "违规营销", "侵犯未成年", "人身攻击", "垃圾广告",
      "青少年不良", "引人不适", "传播谣言", "侵权投诉"
    ];
    let ws = null;

    function scrollChat() {
      nextTick(() => { if (msgContainer.value) msgContainer.value.scrollTop = msgContainer.value.scrollHeight; });
    }

    function connectWS() {
      const roomId = Number(route.params.roomId);
      if (!roomId) return;
      const token = getAccessToken();
      const url = `${location.protocol === "https:" ? "wss:" : "ws:"}//${location.host}/api/v1/ws/live?room_id=${roomId}${token ? "&token=" + encodeURIComponent(token) : ""}`;
      ws = new WebSocket(url);
      ws.onopen = () => { connected.value = true; };
      ws.onmessage = (e) => {
        try {
          const m = JSON.parse(e.data);
          if (m.type === "message") {
            messages.push({ username: m.username || "匿名", content: m.content });
            danmakus.value.push(m);
            while (messages.length > 300) messages.shift();
            scrollChat();
          } else if (m.type === "gift") {
            giftQueue.push(m);
            setTimeout(() => giftQueue.shift(), 4000);
          } else if (m.type === "audience" || m.type === "user_info") {
            audienceList.value = m.users || [];
            audienceCount.value = m.count || m.user_count || 0;
            if (m.broadcaster !== undefined) isBroadcaster.value = !!m.broadcaster;
          } else if (m.type === "system") {
            messages.push({ type: "system", content: m.msg || m.content });
            while (messages.length > 300) messages.shift();
            scrollChat();
          } else if (m.type === "admin_warning") {
            warnMsg.value = m.msg || `⚠ 管理员警告：${m.reason || "违规直播"}`;
            setTimeout(() => { warnMsg.value = ""; }, 5000);
            messages.push({ type: "system", content: warnMsg.value });
            while (messages.length > 300) messages.shift();
            scrollChat();
          } else if (m.type === "admin_ban") {
            warnMsg.value = m.msg || "直播间已被管理员封禁";
            setTimeout(() => { warnMsg.value = ""; }, 60000);
            messages.push({ type: "system", content: warnMsg.value });
            scrollChat();
          }
        } catch (_) {}
      };
      ws.onclose = () => { connected.value = false; setTimeout(connectWS, 3000); };
    }

    function sendChat() {
      const text = inputText.value.trim();
      if (!text || !ws || ws.readyState !== WebSocket.OPEN) return;
      ws.send(JSON.stringify({ content: text }));
      inputText.value = "";
    }

    function sendGiftWs(gift) {
      if (!ws || ws.readyState !== WebSocket.OPEN) return;
      ws.send(JSON.stringify({ gift }));
    }

    async function sendFollowWs() {
      if (!room.value) return;
      if (isBroadcaster.value) {
        ElMessage.warning("不能关注自己");
        return;
      }
      try {
        const data = await mbToggleUserFollow(room.value.user_id);
        followed.value = data.followed;
        ElMessage.success(data.followed ? "已关注" : "已取消关注");
      } catch (e) {
        ElMessage.warning("操作失败，请先登录");
      }
    }

    function toggleMore() {
      showMore.value = !showMore.value;
      if (!showMore.value) {
        settingsVisible.value = false;
        reportVisible.value = false;
      }
    }

    function openReportModal() {
      showMore.value = false;
      settingsVisible.value = false;
      reportVisible.value = false;
      selectedReason.value = "";
      showReportModal.value = true;
    }

    function closeReportModal() {
      showReportModal.value = false;
      selectedReason.value = "";
    }

    function submitReport() {
      if (!selectedReason.value) return;
      ElMessage.success(`已提交举报: ${selectedReason.value}`);
      showReportModal.value = false;
      selectedReason.value = "";
    }

    function shareRoom() {
      ElMessage.info("分享功能 - 复制链接分享给好友");
      showMore.value = false;
    }

    function copyRoomLink() {
      const url = window.location.href;
      navigator.clipboard.writeText(url).then(() => {
        ElMessage.success("链接已复制到剪贴板");
      }).catch(() => {
        ElMessage.success("复制成功");
      });
      showMore.value = false;
    }

    function onDocClick(e) {
      if (setBtnWrap.value && !setBtnWrap.value.contains(e.target)) {
        showMore.value = false;
        settingsVisible.value = false;
        reportVisible.value = false;
      }
    }

    onMounted(async () => {
      document.addEventListener("click", onDocClick);
      const roomId = Number(route.params.roomId) || 0;
      if (!roomId) return;
      try {
        const res = await getLiveRoom(roomId);
        room.value = (res.data || res).data || res.data || res;
      } catch (e) { ElMessage.warning("加载直播间信息失败"); }
      finally { loading.value = false; }
      connectWS();
    });

    onBeforeUnmount(() => {
      document.removeEventListener("click", onDocClick);
      if (ws) { ws.onclose = null; ws.close(); }
    });

    return { loading, room, messages, danmakus, giftQueue, audienceList, audienceCount, showMore, inputText, connected, msgContainer, setBtnWrap, gifts, settingsVisible, reportVisible, showReportModal, selectedReason, danmakuSetting, audienceListSetting, giftEffectSetting, warnMsg, reportReasons, sendChat, sendGiftWs, sendFollowWs, toggleMore, openReportModal, closeReportModal, submitReport, shareRoom, copyRoomLink };
  }
};
</script>

<style scoped>
.live-body {
  background: #e9edf2;
  padding: 10px 120px;
  display: flex;
  height: calc(100vh - 56px);
}
.live-loading-wrapper {
  display: flex;
  width: 100%;
  gap: 10px;
}
.live-empty {
  margin: auto; text-align: center; color: #999;
}

/* ======== 举报弹窗 ======== */
.report-overlay {
  position: fixed; inset: 0; background: rgba(0,0,0,0.4);
  display: flex; align-items: center; justify-content: center;
  z-index: 999;
}
.report-dialog {
  width: 440px; background: #fff; border-radius: 12px;
  overflow: hidden;
}
.report-header {
  padding: 20px 24px 12px; text-align: center;
  position: relative;
}
.report-title {
  font-size: 18px; font-weight: bold; display: block;
}
.report-subtitle {
  font-size: 13px; color: #999; margin-top: 4px; display: block;
}
.report-close {
  position: absolute; top: 12px; right: 16px;
  font-size: 18px; color: #999; cursor: pointer;
}
.report-body {
  display: flex; flex-wrap: wrap; gap: 10px;
  padding: 16px 24px;
}
.report-card {
  width: calc(33.33% - 7px); padding: 12px 0;
  text-align: center; font-size: 13px; color: #555;
  background: #f7f7f7; border-radius: 8px;
  cursor: pointer; border: 2px solid transparent;
  transition: all 0.15s;
}
.report-card:hover { background: #eee; }
.report-card.selected {
  border-color: #ff5599; background: #fff0f5; color: #ff5599;
}
.report-footer {
  padding: 12px 24px 20px;
}
.report-submit-btn {
  width: 100%; padding: 12px; border: none; border-radius: 8px;
  background: #ff5599; color: #fff; font-size: 15px;
  font-weight: bold; cursor: pointer;
}
.report-submit-btn:disabled {
  background: #ddd; color: #aaa; cursor: default;
}

/* ======== 左侧主区域 ======== */
.left-wrap {
  flex: 1;
  display: flex; flex-direction: column;
  min-width: 0;
}

/* 顶部标题栏 */
.live-top-bar {
  background: #fff; padding: 10px 14px;
  display: flex; align-items: center; gap: 12px;
  border-radius: 6px 6px 0 0;
  flex-shrink: 0;
}
.live-title {
  font-size: 16px; font-weight: bold;
  display: flex; align-items: center; gap: 6px;
}
.live-avatar-placeholder {
  width: 26px; height: 26px; border-radius: 50%;
  background: #666; color: #fff; flex-shrink: 0;
  display: flex; align-items: center; justify-content: center;
  font-size: 12px; font-weight: bold;
}
.live-avatar-img {
  width: 26px; height: 26px; border-radius: 50%;
  object-fit: cover; flex-shrink: 0;
}
.live-status { color: #999; font-size: 13px; font-weight: normal; }
.live-follow-btn {
  background: #ff5599; color: #fff; border: none;
  padding: 3px 12px; border-radius: 4px; font-size: 13px; cursor: pointer;
}
.tag-group { display: flex; gap: 8px; margin-left: auto; align-items: center; }

/* 更多设置 - 按钮 + 下拉 */
.set-btn-wrap { position: relative; }
.set-btn {
  border: 1px solid #ddd; padding: 4px 10px; border-radius: 4px;
  font-size: 13px; color: #333; cursor: pointer; background: #fff;
  display: inline-block; user-select: none;
}
.set-btn:hover { background: #f5f5f5; }
.set-dropdown {
  position: absolute; top: 100%; right: 0; margin-top: 6px;
  width: 240px; background: #fff; border-radius: 8px;
  box-shadow: 0 4px 20px rgba(0,0,0,0.12);
  padding: 4px 0; z-index: 100;
}
.drop-row {
  display: flex; border-bottom: 1px solid #eee; padding: 4px 0;
}
.drop-row:last-child { border-bottom: none; }
.drop-item {
  flex: 1; display: flex; flex-direction: column;
  align-items: center; gap: 2px; padding: 10px 6px;
  font-size: 12px; color: #333; cursor: pointer;
}
.drop-item:hover { background: #f0f0f0; }
.drop-icon { font-size: 18px; }
.drop-label { font-size: 12px; }

.drop-sub { border-top: 1px solid #eee; padding: 8px 12px; }
.drop-sub-title { font-size: 12px; color: #999; margin-bottom: 6px; }
.drop-check {
  display: flex; align-items: center; gap: 6px;
  padding: 6px 0; font-size: 13px; cursor: pointer;
}
.drop-check input[type="checkbox"] { margin: 0; }

/* 主画面 */
.live-screen {
  flex: 1; background: #222; min-height: 0; position: relative;
}
.live-screen :deep(.lp-container) { width: 100%; height: 100%; }
.live-screen :deep(.lp-placeholder) { background: #222; color: #aaa; }

.admin-warn-banner {
  position: absolute; top: 50%; left: 50%; transform: translate(-50%, -50%);
  z-index: 20; pointer-events: none;
  background: rgba(200, 0, 0, 0.88); color: #fff; font-size: 14px;
  text-align: center; padding: 10px 24px; line-height: 1.5;
  border-radius: 12px; max-width: 85%;
  animation: warn-flash 0.5s ease-in-out infinite alternate;
}
@keyframes warn-flash {
  from { background: rgba(200, 0, 0, 0.88); }
  to { background: rgba(230, 20, 20, 0.92); }
}

/* 底部礼物栏 */
.gift-bar {
  background: #fff; padding: 8px 10px;
  display: flex; align-items: center; gap: 16px;
  border-radius: 0 0 6px 6px;
  flex-shrink: 0; overflow-x: auto;
}
.gift-item {
  display: flex; flex-direction: column; align-items: center;
  font-size: 12px; color: #333; min-width: 50px; flex-shrink: 0;
}
.gift-item:hover { opacity: 0.7; }
.gift-icon { font-size: 28px; margin-bottom: 2px; }
.gift-name { font-size: 11px; text-align: center; }
.gift-bls-tag {
  background: #ff77aa; color: #fff; font-size: 10px;
  padding: 1px 4px; border-radius: 3px;
  position: relative; top: -22px; left: 16px;
}
.gift-more-wrap {
  display: flex; flex-direction: column; align-items: center;
  font-size: 12px; margin-left: auto;
}

/* ======== 右侧观众面板 ======== */
.right-sidebar {
  width: 310px; flex-shrink: 0;
  background: #fff; border-radius: 6px;
  display: flex; flex-direction: column;
}
.sidebar-title {
  text-align: center; padding: 10px;
  font-size: 15px; border-bottom: 1px solid #eee;
  flex-shrink: 0;
}
.sidebar-count { color: #999; font-size: 13px; }

/* 观众列表 */
.audience-panel { padding: 8px 10px; border-bottom: 1px solid #eee; flex-shrink: 0; }
.audience-list { display: flex; flex-wrap: wrap; gap: 4px; }
.audience-user {
  font-size: 11px; padding: 2px 8px; border-radius: 4px;
  background: #f0f0f0; color: #666;
}
.audience-empty {
  text-align: center; padding: 30px;
  color: #ccc; font-size: 14px; flex-shrink: 0;
}

/* 聊天消息 */
.chat-msgs { flex: 1; overflow-y: auto; padding: 8px 10px; }
.chat-msg { margin-bottom: 6px; display: flex; gap: 4px; align-items: baseline; font-size: 12px; }
.chat-msg-sys { justify-content: center; color: #bbb; }
.chat-msg-sys.notice { color: #ff4488; margin-bottom: 10px; padding-bottom: 8px; border-bottom: 1px solid #eee; }
.chat-msg-user { color: #4488ee; white-space: nowrap; }
.chat-msg-text { word-break: break-all; }

/* 弹幕输入 */
.danmaku-input-area {
  padding: 10px; border-top: 1px solid #eee;
  flex-shrink: 0;
}
.input-row { display: flex; gap: 6px; align-items: center; }
.flag-tag {
  border: 1px solid #ccc; padding: 3px 6px;
  border-radius: 4px; font-size: 12px; color: #666; flex-shrink: 0;
}
.dan-input {
  flex: 1; padding: 6px 8px; border: 1px solid #ddd;
  border-radius: 4px; outline: none; font-size: 13px;
}
.send-btn {
  background: #ff5599; color: #fff; border: none;
  padding: 6px 16px; border-radius: 4px; cursor: pointer; flex-shrink: 0;
}
.send-btn:disabled { opacity: 0.5; cursor: default; }
</style>
