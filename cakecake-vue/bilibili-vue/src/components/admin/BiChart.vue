<template>
  <div class="bi-chart">
    <div ref="chartRef" class="bi-chart__canvas" :style="{ height: height + 'px' }" />
    <div v-if="loading" class="bi-chart__loading">加载中...</div>
    <div v-if="!loading && empty" class="bi-chart__empty">暂无数据</div>
  </div>
</template>

<script setup>
import { ref, watch, nextTick, onMounted, onBeforeUnmount } from 'vue'
import * as echarts from 'echarts'

const props = defineProps({
  option: { type: Object, default: () => ({}) },
  height: { type: [Number, String], default: 300 },
  loading: { type: Boolean, default: false },
  empty: { type: Boolean, default: false }
})

const chartRef = ref(null)
let instance = null
let resizeHandler = null

function initChart() {
  if (!chartRef.value) return
  if (instance) instance.dispose()
  instance = echarts.init(chartRef.value)
  if (props.option && Object.keys(props.option).length > 0) {
    instance.setOption(props.option, true)
  }
}

watch(() => props.option, (val) => {
  if (!instance || !val || Object.keys(val).length === 0) return
  nextTick(() => instance.setOption(val, true))
}, { deep: true })

watch(() => props.loading, (v) => {
  if (!instance) return
  if (v) instance.showLoading()
  else instance.hideLoading()
})

onMounted(() => {
  nextTick(() => initChart())
  resizeHandler = () => { if (instance) instance.resize() }
  window.addEventListener('resize', resizeHandler)
})

onBeforeUnmount(() => {
  if (instance) instance.dispose()
  if (resizeHandler) window.removeEventListener('resize', resizeHandler)
})
</script>

<style scoped>
.bi-chart {
  position: relative;
}
.bi-chart__canvas {
  width: 100%;
}
.bi-chart__loading,
.bi-chart__empty {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  color: #9499a0;
  font-size: 13px;
}
</style>
