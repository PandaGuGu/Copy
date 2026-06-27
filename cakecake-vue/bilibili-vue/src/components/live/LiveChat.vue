<template>
  <div class="lc-chat">
    <!-- Header with settings toggle -->
    <div class="lc-chat-header">
      <span class="lc-chat-title">直播聊天</span>
      <div class="lc-header-right">
        <span class="lc-audience-count" v-if="audienceCount > 0">{{ audienceCount }} 人</span>
        <el-popover trigger="click" width="180">
          <template #reference>
            <el-button size="small" text>⚙</el-button>
          </template>
          <div class="lc-settings">
            <label><input type="checkbox" :checked="danmakuOn" @change="$emit('toggle-danmaku', !danmakuOn)"> 弹幕飘屏</label>
          </div>
        </el-popover>
      </div>
    </div>

    <!-- Audience list -->
    <div class="lc-audience" v-if="audience.length > 0">
      <button class="lc-audience-toggle" @click="showAudience = !showAudience">
        观众 ({{ audience.length }})
        <span class="lc-arrow" :class="{ open: showAudience }">▾</span>
      </button>
      <div v-if="showAudience" class="lc-audience-list">
        <span v-for="u in audience" :key="u" class="lc-audience-user">{{ u }}</span>
      </div>
    </div>

    <!-- Chat messages -->
    <div class="lc-messages" ref="msgContainer">
      <template v-for="(msg, idx) in messages" :key="idx">
        <div v-if="msg.type === 'system'" class="lc-msg lc-msg-system">{{ msg.content }}</div>
        <div v-else class="lc-msg">
          <span class="lc-msg-user">{{ msg.username }}</span>
          <span class="lc-msg-text">{{ msg.content }}</span>
        </div>
      </template>
    </div>

    <!-- Gift bar -->
    <div class="lc-gift-bar">
      <button v-for="g in gifts" :key="g.key" class="lc-gift-btn" @click="$emit('gift', g.key)" :title="g.label">
        {{ g.emoji }}
      </button>
      <el-button size="small" text type="primary" @click="$emit('follow')" class="lc-follow-btn">
        + 关注
      </el-button>
    </div>

    <!-- Input -->
    <div class="lc-input-box">
      <el-input
        v-model="inputText"
        placeholder="说点什么..."
        maxlength="100"
        show-word-limit
        @keyup.enter="sendMessage"
      >
        <template #append>
          <el-button :disabled="!inputText.trim() || !connected" @click="sendMessage">发送</el-button>
        </template>
      </el-input>
    </div>
  </div>
</template>

<script>
import { ref, reactive, nextTick, onMounted, onBeforeUnmount } from "vue";
import { getAccessToken } from "@/utils/authTokens";

const GIFT_ITEMS = [
  { key: "rose", emoji: "🌹", label: "玫瑰" },
  { key: "heart", emoji: "❤️", label: "小心心" },
  { key: "rocket", emoji: "🚀", label: "火箭" },
  { key: "star", emoji: "⭐", label: "星星" },
  { key: "cake", emoji: "🍰", label: "蛋糕" },
  { key: "flower", emoji: "🌸", label: "花束" }
];

export default {
  name: "LiveChat",
  props: {
    roomId: { type: Number, required: true },
    broadcasterId: { type: Number, default: 0 },
    audience: { type: Array, default: () => [] },
    audienceCount: { type: Number, default: 0 },
    danmakuOn: { type: Boolean, default: true }
  },
  emits: ["toggle-danmaku", "gift", "follow"],
  setup(props, { emit }) {
    const messages = reactive([]);
    const inputText = ref("");
    const connected = ref(false);
    const showAudience = ref(false);
    const msgContainer = ref(null);
    const gifts = ref(GIFT_ITEMS);
    let ws = null;

    function scrollToBottom() {
      nextTick(() => {
        if (msgContainer.value) msgContainer.value.scrollTop = msgContainer.value.scrollHeight;
      });
    }

    function addMessage(msg) {
      messages.push(msg);
      while (messages.length > 300) messages.shift();
      scrollToBottom();
    }

    function connectWS() {
      const token = getAccessToken();
      const url = `${location.protocol === "https:" ? "wss:" : "ws:"}//${
        location.host
      }/api/v1/ws/live?room_id=${props.roomId}${token ? "&token=" + encodeURIComponent(token) : ""}`;

      ws = new WebSocket(url);
      ws.onopen = () => { connected.value = true; };
      ws.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data);
          if (data.type === "message") {
            addMessage({ username: data.username || "匿名", content: data.content });
          } else if (data.type === "system") {
            addMessage({ type: "system", content: data.msg || data.content });
          }
        } catch (e) {}
      };
      ws.onclose = () => { connected.value = false; setTimeout(connectWS, 3000); };
      ws.onerror = () => { connected.value = false; };
    }

    function sendMessage() {
      const text = inputText.value.trim();
      if (!text || !ws || ws.readyState !== WebSocket.OPEN) return;
      ws.send(JSON.stringify({ content: text }));
      inputText.value = "";
    }

    onMounted(connectWS);
    onBeforeUnmount(() => { if (ws) { ws.onclose = null; ws.close(); } });

    return { messages, inputText, connected, showAudience, msgContainer, gifts, sendMessage };
  }
};
</script>

<style scoped>
.lc-chat { display: flex; flex-direction: column; height: 100%; }
.lc-chat-header {
  padding: 10px 16px; border-bottom: 1px solid var(--color-border-tertiary);
  display: flex; align-items: center; justify-content: space-between;
}
.lc-chat-title { font-size: 14px; font-weight: 500; }
.lc-header-right { display: flex; align-items: center; gap: 8px; }
.lc-audience-count { font-size: 12px; color: var(--color-text-tertiary); }

.lc-audience { border-bottom: 1px solid var(--color-border-tertiary); }
.lc-audience-toggle {
  width: 100%; padding: 8px 16px; background: none; border: none;
  font-size: 12px; color: var(--color-text-secondary); cursor: pointer;
  display: flex; align-items: center; justify-content: space-between;
}
.lc-arrow { transition: transform .2s; }
.lc-arrow.open { transform: rotate(180deg); }
.lc-audience-list {
  padding: 0 16px 8px; display: flex; flex-wrap: wrap; gap: 4px;
}
.lc-audience-user {
  font-size: 12px; padding: 2px 8px; border-radius: 4px;
  background: var(--color-background-tertiary); color: var(--color-text-secondary);
}

.lc-messages { flex: 1; overflow-y: auto; padding: 12px 16px; }
.lc-msg { margin-bottom: 10px; display: flex; flex-wrap: wrap; gap: 4px; align-items: baseline; }
.lc-msg-system { justify-content: center; font-size: 12px; color: var(--color-text-tertiary); }
.lc-msg-user { color: var(--color-text-info); font-size: 12px; white-space: nowrap; }
.lc-msg-text { font-size: 13px; word-break: break-all; }

.lc-gift-bar {
  padding: 8px 12px; border-top: 1px solid var(--color-border-tertiary);
  display: flex; align-items: center; gap: 6px; flex-wrap: wrap;
}
.lc-gift-btn {
  font-size: 20px; padding: 2px 6px; border: 1px solid var(--color-border-tertiary);
  border-radius: 6px; background: var(--color-background-tertiary); cursor: pointer;
  transition: transform .1s;
}
.lc-gift-btn:hover { transform: scale(1.2); }
.lc-follow-btn { margin-left: auto; }

.lc-input-box { padding: 10px 12px; border-top: 1px solid var(--color-border-tertiary); }
</style>
