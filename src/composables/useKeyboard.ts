// src/composables/useKeyboard.ts
import { onMounted, onUnmounted } from 'vue'
import { currentVideoConfig } from './useConfig'

export interface KeyboardActions {
  isEnabled?: () => boolean // 作用域启用守卫 (如仅在播放态激活，避免主页键位拦截与滚动失效)
  seekRelative: (seconds: number) => void
  togglePause: () => void
  changeVolume: (delta: number) => void
  setTempSpeed: (speed: number | null) => void // 临时变频加速（长按倍速）
  exitFullscreen: () => void
  isFullscreen: () => boolean
  showOsd: (text: string, duration?: number) => void
  hideOsd: () => void
}

export function useKeyboard(actions: KeyboardActions) {
  let rightPressTimer: ReturnType<typeof setTimeout> | null = null
  let rightIsLongPress = false

  let leftPressTimer: ReturnType<typeof setTimeout> | null = null
  let leftPulseInterval: ReturnType<typeof setInterval> | null = null
  let leftIsLongPress = false

  let lastVolumeStepTime = 0

  const onKeyDown = (e: KeyboardEvent) => {
    // 避免在文本输入框中触发快捷键
    if ((e.target as HTMLElement)?.tagName === 'INPUT' || (e.target as HTMLElement)?.tagName === 'TEXTAREA') {
      return
    }

    // 作用域守卫：非激活状态直接放行，杜绝主页等视图下的键盘默认行为被误拦截
    if (actions.isEnabled && !actions.isEnabled()) {
      return
    }

    if (e.key === ' ' || e.code === 'Space') {
      e.preventDefault()
      actions.togglePause()
    } else if (e.key === 'ArrowRight') {
      e.preventDefault()
      if (e.repeat) return // 忽略系统自带重复事件，由状态机自主接管

      rightIsLongPress = false
      if (rightPressTimer) clearTimeout(rightPressTimer)

      // 超过 250ms 判定为长按加速
      rightPressTimer = setTimeout(() => {
        rightIsLongPress = true
        const speed = currentVideoConfig.longPressSpeed || 3.0
        actions.setTempSpeed(speed)
        actions.showOsd(`▶▶▶ ${speed.toFixed(1)}x 倍速播放中`, 0) // duration=0 保持常驻
      }, 250)
    } else if (e.key === 'ArrowLeft') {
      e.preventDefault()
      if (e.repeat) return

      leftIsLongPress = false
      if (leftPressTimer) clearTimeout(leftPressTimer)

      // 超过 250ms 判定为长按连续脉冲快退
      leftPressTimer = setTimeout(() => {
        leftIsLongPress = true
        const step = currentVideoConfig.seekStep || 5
        actions.seekRelative(-step)
        actions.showOsd(`-${step}秒`, 800)
        
        // 以 150ms 周期进行高频脉冲快退
        leftPulseInterval = setInterval(() => {
          actions.seekRelative(-step)
          actions.showOsd(`-${step}秒`, 800)
        }, 150)
      }, 250)
    } else if (e.key === 'ArrowUp' || e.key === 'ArrowDown') {
      e.preventDefault()
      const now = Date.now()
      // 系统按键重复 (Key Repeat) 节流至 100ms 一次，按住时平滑步进，杜绝高频冲击 IPC
      if (e.repeat && now - lastVolumeStepTime < 100) {
        return
      }
      lastVolumeStepTime = now
      const volStep = currentVideoConfig.wheelVolumeStep || 5
      actions.changeVolume(e.key === 'ArrowUp' ? volStep : -volStep)
    } else if (e.key === 'Escape') {
      if (actions.isFullscreen()) {
        e.preventDefault()
        actions.exitFullscreen()
      }
    }
  }

  const onKeyUp = (e: KeyboardEvent) => {
    if (actions.isEnabled && !actions.isEnabled()) {
      return
    }

    if (e.key === 'ArrowRight') {
      e.preventDefault()
      if (rightPressTimer) {
        clearTimeout(rightPressTimer)
        rightPressTimer = null
      }
      if (rightIsLongPress) {
        // 松开瞬时恢复原倍速并隐藏 OSD
        actions.setTempSpeed(null)
        actions.hideOsd()
      } else {
        // 250ms 内松开，判定为短按单次快进
        const step = currentVideoConfig.seekStep || 5
        actions.seekRelative(step)
        actions.showOsd(`+${step}秒`, 800)
      }
      rightIsLongPress = false
    } else if (e.key === 'ArrowLeft') {
      e.preventDefault()
      if (leftPressTimer) {
        clearTimeout(leftPressTimer)
        leftPressTimer = null
      }
      if (leftPulseInterval) {
        clearInterval(leftPulseInterval)
        leftPulseInterval = null
      }
      if (!leftIsLongPress) {
        // 250ms 内松开，判定为短按单次快退
        const step = currentVideoConfig.seekStep || 5
        actions.seekRelative(-step)
        actions.showOsd(`-${step}秒`, 800)
      }
      leftIsLongPress = false
    }
  }

  // 窗口失焦（如 Alt+Tab、切出应用）时安全复位所有长按与脉冲定时器，杜绝倍速/快退卡死
  const resetAllKeys = () => {
    if (rightPressTimer) {
      clearTimeout(rightPressTimer)
      rightPressTimer = null
    }
    if (rightIsLongPress) {
      actions.setTempSpeed(null)
      actions.hideOsd()
      rightIsLongPress = false
    }
    if (leftPressTimer) {
      clearTimeout(leftPressTimer)
      leftPressTimer = null
    }
    if (leftPulseInterval) {
      clearInterval(leftPulseInterval)
      leftPulseInterval = null
    }
    leftIsLongPress = false
  }

  onMounted(() => {
    window.addEventListener('keydown', onKeyDown)
    window.addEventListener('keyup', onKeyUp)
    window.addEventListener('blur', resetAllKeys)
  })

  onUnmounted(() => {
    window.removeEventListener('keydown', onKeyDown)
    window.removeEventListener('keyup', onKeyUp)
    window.removeEventListener('blur', resetAllKeys)
    resetAllKeys()
  })
}
