<template>
  <div class="lp-container">
    <div v-if="!isLive" class="lp-placeholder">
      <p class="lp-placeholder-icon">📡</p>
      <p>主播正在赶来的路上...</p>
      <p class="lp-placeholder-sub">等待推流中</p>
    </div>

    <video
      v-show="isLive"
      ref="videoRef"
      class="lp-video"
      controls
      autoplay
      muted
    ></video>

    <!-- 弹幕飘屏 -->
    <LiveDanmaku v-if="isLive" :items="danmakus" />

    <!-- 礼物特效 -->
    <div v-if="currentGift" class="lp-gift-effect" :key="currentGift.id">
      <span class="lp-gift-emoji">{{ giftEmoji(currentGift.gift) }}</span>
      <span class="lp-gift-text">{{ currentGift.content }}</span>
    </div>
  </div>
</template>

<script>
import { ref, computed, watch, onBeforeUnmount } from "vue";
import flvjs from "flv.js";
import LiveDanmaku from "./LiveDanmaku.vue";

const GIFT_EMOJIS = {
  rose: "🌹", heart: "❤️", rocket: "🚀", star: "⭐", cake: "🍰", flower: "🌸"
};

export default {
  name: "LivePlayer",
  components: { LiveDanmaku },
  props: {
    room: { type: Object, required: true },
    danmakus: { type: Array, default: () => [] },
    gifts: { type: Array, default: () => [] }
  },
  setup(props) {
    const videoRef = ref(null);
    const isLive = ref(false);
    const currentGift = ref(null);
    let flvPlayer = null;

    const playUrl = () => {
      if (!props.room || !props.room.stream_key) return "";
      return `http://localhost:8000/live/${props.room.stream_key}.flv`;
    };

    function startPlay() {
      if (!videoRef.value) return;
      const url = playUrl();
      if (!url) return;
      try {
        if (flvjs.isSupported()) {
          flvPlayer = flvjs.createPlayer({ type: "flv", url, isLive: true });
          flvPlayer.attachMediaElement(videoRef.value);
          flvPlayer.load();
          flvPlayer.play();
        }
      } catch (e) { console.error(e); }
    }

    function stopPlay() {
      if (flvPlayer) {
        try { flvPlayer.pause(); flvPlayer.unload(); flvPlayer.detachMediaElement(); flvPlayer.destroy(); } catch (e) {}
        flvPlayer = null;
      }
    }

    function giftEmoji(gift) { return GIFT_EMOJIS[gift] || "🎁"; }

    watch(() => props.gifts.length, () => {
      if (props.gifts.length > 0) {
        const g = props.gifts[props.gifts.length - 1];
        currentGift.value = { ...g, id: Date.now() };
        setTimeout(() => { currentGift.value = null; }, 3500);
      }
    });

    watch(() => props.room && props.room.status, (status) => {
      isLive.value = status === "live";
      if (status === "live") setTimeout(startPlay, 500);
      else stopPlay();
    }, { immediate: true });

    onBeforeUnmount(() => stopPlay());

    return { videoRef, isLive, currentGift, giftEmoji };
  }
};
</script>

<style scoped>
.lp-container {
  width: 100%; height: 100%; display: flex; align-items: center; justify-content: center;
  background: #1a1a2e; position: relative;
}
.lp-video { width: 100%; height: 100%; object-fit: contain; }
.lp-placeholder { text-align: center; color: #8888aa; }
.lp-placeholder-icon { font-size: 64px; margin-bottom: 16px; }
.lp-placeholder-sub { font-size: 13px; opacity: 0.6; margin-top: 8px; }

.lp-gift-effect {
  position: absolute; top: 30%; left: 50%; transform: translate(-50%, -50%);
  z-index: 10; pointer-events: none;
  animation: gift-pop 3.5s ease-out forwards;
  display: flex; flex-direction: column; align-items: center; gap: 4px;
}
.lp-gift-emoji { font-size: 48px; filter: drop-shadow(0 0 12px gold); }
.lp-gift-text {
  font-size: 14px; color: #fff; font-weight: bold;
  text-shadow: 0 0 8px rgba(255,255,255,0.6);
  white-space: nowrap;
}
@keyframes gift-pop {
  0% { opacity: 0; transform: translate(-50%, -50%) scale(0.3); }
  20% { opacity: 1; transform: translate(-50%, -50%) scale(1.2); }
  40% { transform: translate(-50%, -50%) scale(1); }
  80% { opacity: 1; }
  100% { opacity: 0; transform: translate(-50%, -80%) scale(0.8); }
}
</style>
