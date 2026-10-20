// src/composables/useAutoHide.ts
// 视频播放器控制栏自动隐藏状态机
import { ref, computed } from 'vue'
import { globalConfig } from './useConfig'

export function useAutoHide() {
  const visible = ref(true)
  const isDraggingProgress = ref(false)
  const hasActivePopup = ref(false) // 倍速、设置、选集等卡片展开状态

  let timer: ReturnType<typeof setTimeout> | null = null

  // 豁免判定：当处于拖拽进度条或弹窗浮层处于激活交互时，不自动倒计时淡出
  const isExempt = computed(() => {
    return isDraggingProgress.value || hasActivePopup.value
  })

  const resetTimer = () => {
    visible.value = true
    if (timer) {
      clearTimeout(timer)
      timer = null
    }

    // 若当前正在拖拽进度条或打开了卡片浮层，则不启动隐藏倒计时
    if (isExempt.value) {
      return
    }

    // 若配置为从不自动隐藏 (autoHideDelay = 0)
    if (globalConfig.autoHideDelay <= 0) {
      return
    }

    // 光标在画面中静止超过指定秒数后平滑渐变消失 (默认 2.5s，受全局配置调控)
    const delayMs = Math.round(globalConfig.autoHideDelay * 1000)
    timer = setTimeout(() => {
      if (!isExempt.value) {
        visible.value = false
      }
    }, delayMs)
  }

  // 光标从画面中移开到播放器之外：立刻渐变消失（无延迟），同时重置浮层状态避免重入后锁死
  const onMouseLeaveWindow = () => {
    // 若正在拖拽进度条则暂不隐藏，否则立即渐变隐藏
    if (isDraggingProgress.value) {
      return
    }
    hasActivePopup.value = false
    if (timer) {
      clearTimeout(timer)
      timer = null
    }
    visible.value = false
  }

  const onMouseMove = () => {
    resetTimer()
  }

  return {
    visible,
    isDraggingProgress,
    hasActivePopup,
    resetTimer,
    onMouseMove,
    onMouseLeaveWindow,
  }
}
