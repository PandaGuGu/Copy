import { ref, onMounted, onBeforeUnmount } from 'vue'
import * as echarts from 'echarts'

export function useECharts() {
  const chartRef = ref(null)
  let instance = null

  function init() {
    if (!chartRef.value) return
    instance = echarts.init(chartRef.value)
  }

  function setOption(option, notMerge = true) {
    if (!instance) return
    instance.setOption(option, notMerge)
  }

  function resize() {
    if (!instance) return
    instance.resize()
  }

  function dispose() {
    if (instance) {
      instance.dispose()
      instance = null
    }
  }

  onMounted(() => {
    init()
  })

  onBeforeUnmount(() => {
    dispose()
  })

  return { chartRef, init, setOption, resize, dispose, echarts }
}
