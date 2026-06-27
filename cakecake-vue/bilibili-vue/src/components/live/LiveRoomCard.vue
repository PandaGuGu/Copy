<template>
  <div class="lrc-card" @click="$emit('click')">
    <div class="lrc-cover">
      <img
        v-if="room.cover_url"
        :src="room.cover_url"
        :alt="room.title"
        class="lrc-cover-img"
      />
      <div v-else class="lrc-cover-fallback">
        <span class="lrc-cover-icon">📡</span>
      </div>

      <!-- 状态角标 -->
      <span class="lrc-badge" :class="'lrc-badge-' + (room.status || 'idle')">
        {{ statusText }}
      </span>

      <!-- 观看人数 -->
      <span class="lrc-viewers" v-if="room.status === 'live'">
        {{ room.viewer_count || 0 }} 人观看
      </span>
    </div>

    <div class="lrc-info">
      <h3 class="lrc-title" :title="room.title">{{ room.title || "未命名直播间" }}</h3>
      <p class="lrc-host">
        <span class="lrc-host-avatar">{{ (room.host_name || "主")[0] }}</span>
        {{ room.host_name || "未知主播" }}
      </p>
    </div>
  </div>
</template>

<script>
export default {
  name: "LiveRoomCard",
  props: {
    room: { type: Object, required: true }
  },
  emits: ["click"],
  computed: {
    statusText() {
      const map = { idle: "未开播", live: "直播中", ended: "已结束", banned: "已封禁" };
      return map[this.room.status] || this.room.status || "未知";
    }
  }
};
</script>

<style scoped>
.lrc-card {
  border-radius: 8px;
  overflow: hidden;
  cursor: pointer;
  background: var(--color-background-secondary);
  transition: transform 0.15s;
}
.lrc-card:hover {
  transform: scale(1.02);
}
.lrc-cover {
  position: relative;
  width: 100%;
  padding-top: 56.25%;
  overflow: hidden;
  background: #1a1a2e;
}
.lrc-cover-img {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  object-fit: cover;
}
.lrc-cover-fallback {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
}
.lrc-cover-icon { font-size: 48px; }
.lrc-badge {
  position: absolute;
  top: 8px;
  left: 8px;
  padding: 2px 8px;
  border-radius: 4px;
  font-size: 11px;
  color: #fff;
}
.lrc-badge-live { background: #e24b4a; }
.lrc-badge-idle { background: #888; }
.lrc-badge-ended { background: #555; }
.lrc-badge-banned { background: #a32d2d; }
.lrc-viewers {
  position: absolute;
  bottom: 8px;
  right: 8px;
  padding: 2px 8px;
  border-radius: 4px;
  font-size: 11px;
  background: rgba(0,0,0,0.6);
  color: #fff;
}
.lrc-info {
  padding: 10px 12px;
}
.lrc-title {
  font-size: 14px;
  font-weight: 500;
  margin: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.lrc-host {
  font-size: 12px;
  color: var(--color-text-secondary);
  margin: 6px 0 0;
  display: flex;
  align-items: center;
  gap: 6px;
}
.lrc-host-avatar {
  width: 20px;
  height: 20px;
  border-radius: 50%;
  background: var(--color-background-info);
  color: #fff;
  font-size: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
}
</style>
