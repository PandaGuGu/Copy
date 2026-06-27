<template>
  <div class="dm-canvas" ref="canvas">
    <span
      v-for="d in visibleDanmakus"
      :key="d.id"
      class="dm-item"
      :style="d.style"
    >{{ d.username }}: {{ d.content }}</span>
  </div>
</template>

<script>
import { ref, watch, nextTick } from "vue";

let idCounter = 0;

export default {
  name: "LiveDanmaku",
  props: {
    items: { type: Array, default: () => [] }
  },
  setup(props) {
    const visibleDanmakus = ref([]);
    const canvas = ref(null);

    const colors = ["#fff", "#0ff", "#ff0", "#f80", "#f0f", "#0f0", "#48f"];
    const maxItems = 40;

    watch(
      () => props.items.length,
      () => {
        const newItems = props.items.slice(-3);
        newItems.forEach((msg) => {
          const id = ++idCounter;
          const color = colors[Math.floor(Math.random() * colors.length)];
          const top = 8 + Math.floor(Math.random() * 80) + "%"; // random vertical

          const item = {
            id,
            username: msg.username || "",
            content: msg.content || "",
            style: {
              color,
              top,
              animationDuration: 5 + Math.random() * 4 + "s",
              textShadow: `0 0 4px ${color}, 0 0 6px rgba(0,0,0,0.8)`
            }
          };

          visibleDanmakus.value.push(item);
          if (visibleDanmakus.value.length > maxItems) {
            visibleDanmakus.value.shift();
          }

          // Auto-remove after animation
          const dur = parseFloat(item.style.animationDuration) * 1000;
          setTimeout(() => {
            const idx = visibleDanmakus.value.findIndex((d) => d.id === id);
            if (idx >= 0) visibleDanmakus.value.splice(idx, 1);
          }, dur + 200);
        });
      }
    );

    return { visibleDanmakus, canvas };
  }
};
</script>

<style scoped>
.dm-canvas {
  position: absolute; inset: 0; pointer-events: none; overflow: hidden; z-index: 5;
}
.dm-item {
  position: absolute; right: -100%; white-space: nowrap;
  font-size: 18px; font-weight: bold; font-family: system-ui, "Microsoft YaHei", sans-serif;
  animation: dm-fly linear forwards;
  user-select: none;
  line-height: 1.2;
  max-width: 80%;
  overflow: hidden; text-overflow: ellipsis;
}
@keyframes dm-fly {
  from { transform: translateX(0); }
  to { transform: translateX(-100vw); }
}
</style>
