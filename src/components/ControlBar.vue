<template>
  <div class="controlbar">
    <!-- 悬浮展开粉色进度条 -->
    <div
      ref="progressContainerRef"
      class="progress-bar-container"
      :class="{ 'is-dragging': isDraggingProgress }"
      @mousedown="startSeek"
      @mousemove="onProgressHover"
      @mouseleave="hoveringProgress = false"
    >
      <div class="progress-track">
        <div class="progress-played" :style="{ width: `${progressPercent}%` }"></div>
        <div class="progress-thumb" :style="{ left: `${progressPercent}%` }"></div>
      </div>

      <!-- 时间预览气泡 (跟随鼠标) -->
      <div
        v-if="hoveringProgress"
        class="time-tooltip"
        :style="{ left: `${hoverTooltipLeft}px` }"
      >
        {{ formatTime(hoverTime) }}
      </div>
    </div>

    <!-- 控制台主体按钮排布 -->
    <div class="control-row">
      <!-- 左侧控制项 -->
      <div class="control-left">
        <!-- 播放/暂停 -->
        <button class="icon-btn" @click="emit('toggle-pause')" :title="isPaused ? '播放 (空格)' : '暂停 (空格)'">
          <SvgIcon :name="isPaused ? 'play' : 'pause'" size="20" />
        </button>

        <!-- 上一集 (人性化判定: >3s重播开头，<=3s切上一个) -->
        <button class="icon-btn" @click="emit('prev-video')" title="上一集">
          <SvgIcon name="prev" size="18" />
        </button>

        <!-- 下一集 -->
        <button class="icon-btn" @click="emit('next-video')" title="下一集">
          <SvgIcon name="next" size="18" />
        </button>

        <!-- 时间标签 -->
        <div class="time-label">
          <span class="curr-time">{{ formatTime(currentTime) }}</span>
          <span class="sep">/</span>
          <span class="total-time">{{ formatTime(duration) }}</span>
        </div>
      </div>

      <!-- 右侧控制项 -->
      <div class="control-right">
        <!-- 【选集】 -->
        <div class="popup-anchor">
          <button
            class="text-btn"
            :class="{ active: activePopup === 'playlist' }"
            @mouseenter="triggerPopup('playlist')"
            @click="togglePopup('playlist')"
          >
            选集
          </button>
          <transition name="popup-fade">
            <PlaylistDrawer
              v-if="activePopup === 'playlist'"
              class="popup-pos"
              :playlist="playlist"
              :current-path="currentPath"
              @select-video="(p) => emit('select-video', p)"
              @update-playlist="(list) => emit('update-playlist', list)"
              @mouse-enter="cancelHidePopup"
              @mouse-leave="scheduleHidePopup"
            />
          </transition>
        </div>

        <!-- 【倍速】 -->
        <div class="popup-anchor">
          <button
            class="text-btn"
            :class="{ active: activePopup === 'speed' || speed !== 1 }"
            @mouseenter="triggerPopup('speed')"
            @click="togglePopup('speed')"
          >
            {{ speed === 1 ? '倍速' : (Number.isInteger(speed * 10) ? `${speed.toFixed(1)}x` : `${speed.toFixed(2)}x`) }}
          </button>
          <transition name="popup-fade">
            <SpeedPopup
              v-if="activePopup === 'speed'"
              class="popup-pos"
              :current-speed="speed"
              @change-speed="(s) => emit('update-speed', s)"
              @mouse-enter="cancelHidePopup"
              @mouse-leave="scheduleHidePopup"
            />
          </transition>
        </div>

        <!-- 【设置】 -->
        <div class="popup-anchor">
          <button
            class="icon-btn"
            :class="{ active: activePopup === 'settings' }"
            @mouseenter="triggerPopup('settings')"
            @click="togglePopup('settings')"
            title="播放设置"
          >
            <SvgIcon name="settings" size="19" />
          </button>
          <transition name="popup-fade">
            <SettingsPopup
              v-if="activePopup === 'settings'"
              class="popup-pos-settings"
              :play-mode="playMode"
              :aspect-ratio="aspectRatio"
              :is-mirrored="isMirrored"
              :rotation="rotation"
              :hwdec-mode="hwdecMode"
              @update-play-mode="(m) => emit('update-play-mode', m)"
              @update-aspect-ratio="(r) => emit('update-aspect-ratio', r)"
              @toggle-mirror="emit('toggle-mirror')"
              @rotate-clockwise="emit('rotate-clockwise')"
              @update-hwdec="(h) => emit('update-hwdec', h)"
              @mouse-enter="cancelHidePopup"
              @mouse-leave="scheduleHidePopup"
            />
          </transition>
        </div>

        <!-- 独立软件音量调节：B 站风格垂直向上弹出 -->
        <div
          ref="volumeGroupRef"
          class="volume-group"
          @mouseenter="onVolumeMouseEnter"
          @mouseleave="onVolumeMouseLeave"
          @wheel.prevent="handleVolumeWheel"
        >
          <button
            class="icon-btn"
            @click="emit('toggle-mute')"
            :title="isMuted ? '取消静音' : '静音'"
          >
            <SvgIcon :name="isMuted || volume === 0 ? 'volume-mute' : 'volume'" size="19" />
          </button>
          <transition name="popup-fade">
            <div
              v-if="showVolumePopup || isDraggingVolume"
              ref="volumePopupWrapperRef"
              class="volume-popup-wrapper"
              @mouseenter="onVolumeMouseEnter"
              @mouseleave="onVolumeMouseLeave"
            >
              <div class="volume-popup">
                <div class="volume-num">{{ displayVolume }}</div>
                <div
                  ref="volumeTrackRef"
                  class="volume-track-container"
                  @mousedown.stop.prevent="startVolumeDrag"
                >
                  <div class="volume-track-groove"></div>
                  <div class="volume-track-fill" :style="{ height: `${displayVolume}%` }"></div>
                  <div class="volume-thumb" :style="{ bottom: `calc(${displayVolume}% - 6px)` }"></div>
                </div>
              </div>
            </div>
          </transition>
        </div>

        <!-- 全屏切换 -->
        <button class="icon-btn" @click="emit('toggle-fullscreen')" :title="isFullscreen ? '退出全屏 (Esc)' : '全屏'">
          <SvgIcon :name="isFullscreen ? 'fullscreen-exit' : 'fullscreen'" size="19" />
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import SvgIcon from './SvgIcon.vue'
import PlaylistDrawer, { PlaylistItem } from './PlaylistDrawer.vue'
import SpeedPopup from './SpeedPopup.vue'
import SettingsPopup, { PlayMode, AspectRatio, HwdecMode } from './SettingsPopup.vue'
import { currentVideoConfig } from '../composables/useConfig'
import { throttle } from '../utils/throttle'

const props = defineProps<{
  currentTime: number
  duration: number
  isPaused: boolean
  speed: number
  volume: number
  isMuted: boolean
  isFullscreen: boolean
  playlist: PlaylistItem[]
  currentPath: string
  playMode: PlayMode
  aspectRatio: AspectRatio
  isMirrored: boolean
  rotation: number
  hwdecMode: HwdecMode
  visible?: boolean
}>()

const emit = defineEmits<{
  (e: 'toggle-pause'): void
  (e: 'prev-video'): void
  (e: 'next-video'): void
  (e: 'seek', seconds: number): void
  (e: 'update-speed', speed: number): void
  (e: 'update-volume', vol: number): void
  (e: 'toggle-mute'): void
  (e: 'toggle-fullscreen'): void
  (e: 'select-video', path: string): void
  (e: 'update-playlist', list: PlaylistItem[]): void
  (e: 'update-play-mode', mode: PlayMode): void
  (e: 'update-aspect-ratio', ratio: AspectRatio): void
  (e: 'toggle-mirror'): void
  (e: 'rotate-clockwise'): void
  (e: 'update-hwdec', mode: HwdecMode): void
  (e: 'popup-state-change', active: boolean): void
  (e: 'dragging-state-change', dragging: boolean): void
}>()

// 进度条相关
const progressContainerRef = ref<HTMLElement | null>(null)
const isDraggingProgress = ref(false)
const dragRatio = ref(0)
const hoveringProgress = ref(false)
const hoverTime = ref(0)
const hoverTooltipLeft = ref(0)

// 拖拽时由本地位置驱动滑块，避免 IPC 往返竞争跳动
const progressPercent = computed(() => {
  if (isDraggingProgress.value) return dragRatio.value * 100
  if (props.duration <= 0) return 0
  return Math.min(100, Math.max(0, (props.currentTime / props.duration) * 100))
})

const calculateProgressRatio = (e: MouseEvent) => {
  if (!progressContainerRef.value) return 0
  const rect = progressContainerRef.value.getBoundingClientRect()
  const x = Math.max(0, Math.min(e.clientX - rect.left, rect.width))
  return rect.width > 0 ? x / rect.width : 0
}

// 拖拽过程中节流 Seek (120ms)，松手时执行最终精确 Seek
const throttledSeek = throttle((targetSec: number) => {
  emit('seek', targetSec)
}, 120)

const startSeek = (e: MouseEvent) => {
  isDraggingProgress.value = true
  emit('dragging-state-change', true)
  const ratio = calculateProgressRatio(e)
  dragRatio.value = ratio
  throttledSeek(ratio * props.duration)

  const onMouseMove = (moveEvent: MouseEvent) => {
    if (isDraggingProgress.value) {
      const r = calculateProgressRatio(moveEvent)
      dragRatio.value = r
      throttledSeek(r * props.duration)
    }
  }

  const onMouseUp = (upEvent: MouseEvent) => {
    if (isDraggingProgress.value) {
      isDraggingProgress.value = false
      emit('dragging-state-change', false)
      throttledSeek.cancel() // 关键：取消拖拽中排队的中间态 Seek，杜绝旧状态覆盖
      const finalRatio = calculateProgressRatio(upEvent)
      emit('seek', finalRatio * props.duration) // 松手 Seek 机制
    }
    window.removeEventListener('mousemove', onMouseMove)
    window.removeEventListener('mouseup', onMouseUp)
  }

  window.addEventListener('mousemove', onMouseMove)
  window.addEventListener('mouseup', onMouseUp)
}

const onProgressHover = (e: MouseEvent) => {
  hoveringProgress.value = true
  if (!progressContainerRef.value) return
  const rect = progressContainerRef.value.getBoundingClientRect()
  const x = Math.max(0, Math.min(e.clientX - rect.left, rect.width))
  hoverTime.value = (x / rect.width) * props.duration
  // 气泡边缘夹紧保护：防止最左端或最右端时间气泡溢出视口
  hoverTooltipLeft.value = Math.max(28, Math.min(x, rect.width - 28))
}

// 浮层卡片管理 (选集 / 倍速 / 设置)
type PopupType = 'playlist' | 'speed' | 'settings' | null
const activePopup = ref<PopupType>(null)
let hidePopupTimer: ReturnType<typeof setTimeout> | null = null

const triggerPopup = (p: PopupType) => {
  cancelHidePopup()
  if (showVolumePopup.value) {
    if (hideVolumeTimer) clearTimeout(hideVolumeTimer)
    showVolumePopup.value = false
  }
  activePopup.value = p
  emit('popup-state-change', true)
}

const togglePopup = (p: PopupType) => {
  cancelHidePopup()
  if (showVolumePopup.value) {
    if (hideVolumeTimer) clearTimeout(hideVolumeTimer)
    showVolumePopup.value = false
  }
  activePopup.value = activePopup.value === p ? null : p
  emit('popup-state-change', activePopup.value !== null)
}

// 移出卡片后 300ms 离开缓冲防抖自动平滑淡出，绝不需要用户二次点击按钮
const scheduleHidePopup = () => {
  if (hidePopupTimer) clearTimeout(hidePopupTimer)
  hidePopupTimer = setTimeout(() => {
    activePopup.value = null
    emit('popup-state-change', false)
  }, 300)
}

const cancelHidePopup = () => {
  if (hidePopupTimer) {
    clearTimeout(hidePopupTimer)
    hidePopupTimer = null
  }
}

// 当控制栏渐变隐藏时，自动收起任何可能打开的浮层卡片与音量面板
watch(
  () => props.visible,
  (newVisible) => {
    if (newVisible === false) {
      if (activePopup.value !== null) {
        cancelHidePopup()
        activePopup.value = null
      }
      showVolumePopup.value = false
      emit('popup-state-change', false)
    }
  }
)

// 垂直音量控制 (复刻 B 站手感：向上垂直弹出、动态百分比、圆形滑块、滚轮与拖拽)
const showVolumePopup = ref(false)
const isDraggingVolume = ref(false)
const volumeGroupRef = ref<HTMLElement | null>(null)
const volumePopupWrapperRef = ref<HTMLElement | null>(null)
const volumeTrackRef = ref<HTMLElement | null>(null)
const localDragVolume = ref<number | null>(null)
let localWheelTargetVol: number | null = null
let hideVolumeTimer: ReturnType<typeof setTimeout> | null = null

const displayVolume = computed(() => {
  if (localDragVolume.value !== null) return localDragVolume.value
  if (localWheelTargetVol !== null) return localWheelTargetVol
  if (props.isMuted) return 0
  return Math.round(props.volume)
})

const throttledVolume = throttle((val: number) => {
  emit('update-volume', val)
}, 60)

const throttledWheelVolume = throttle((nextVol: number) => {
  localWheelTargetVol = null
  emit('update-volume', nextVol)
}, 60)

const onVolumeMouseEnter = () => {
  if (activePopup.value !== null) {
    cancelHidePopup()
    activePopup.value = null
  }
  if (hideVolumeTimer) {
    clearTimeout(hideVolumeTimer)
    hideVolumeTimer = null
  }
  showVolumePopup.value = true
  emit('popup-state-change', true)
}

const onVolumeMouseLeave = () => {
  if (isDraggingVolume.value) return
  if (hideVolumeTimer) clearTimeout(hideVolumeTimer)
  hideVolumeTimer = setTimeout(() => {
    showVolumePopup.value = false
    emit('popup-state-change', false)
  }, 200)
}

// 滚轮调节音量 (向上 +step，向下 -step，采用累加值消除快速滚轮丢步)
const handleVolumeWheel = (e: WheelEvent) => {
  const step = currentVideoConfig.wheelVolumeStep || 5
  const delta = e.deltaY < 0 ? step : -step
  const current = localWheelTargetVol !== null ? localWheelTargetVol : props.volume
  const next = Math.max(0, Math.min(100, Math.round(current + delta)))
  localWheelTargetVol = next
  throttledWheelVolume(next)
}

// 依据鼠标 clientY 计算垂直音量百分比
const updateVolumeFromY = (clientY: number) => {
  if (!volumeTrackRef.value) return
  const rect = volumeTrackRef.value.getBoundingClientRect()
  if (rect.height <= 0) return
  const ratio = (rect.bottom - clientY) / rect.height
  const val = Math.round(Math.max(0, Math.min(100, ratio * 100)))
  localDragVolume.value = val
  throttledVolume(val)
}

// 拖拽垂直音量条
const startVolumeDrag = (e: MouseEvent) => {
  isDraggingVolume.value = true
  emit('dragging-state-change', true)
  updateVolumeFromY(e.clientY)

  const onMouseMove = (moveEvent: MouseEvent) => {
    if (isDraggingVolume.value) {
      updateVolumeFromY(moveEvent.clientY)
    }
  }

  const onMouseUp = (upEvent: MouseEvent) => {
    if (isDraggingVolume.value) {
      isDraggingVolume.value = false
      emit('dragging-state-change', false)
      updateVolumeFromY(upEvent.clientY)
      localDragVolume.value = null

      // 释放鼠标时：若仍处于音量按钮或音量弹层内部则保持展示；若已移出则平滑关闭
      let isInside = false
      if (volumeGroupRef.value) {
        const gRect = volumeGroupRef.value.getBoundingClientRect()
        if (
          upEvent.clientX >= gRect.left &&
          upEvent.clientX <= gRect.right &&
          upEvent.clientY >= gRect.top &&
          upEvent.clientY <= gRect.bottom
        ) {
          isInside = true
        }
      }
      if (!isInside && volumePopupWrapperRef.value) {
        const pRect = volumePopupWrapperRef.value.getBoundingClientRect()
        if (
          upEvent.clientX >= pRect.left &&
          upEvent.clientX <= pRect.right &&
          upEvent.clientY >= pRect.top &&
          upEvent.clientY <= pRect.bottom
        ) {
          isInside = true
        }
      }

      if (!isInside) {
        onVolumeMouseLeave()
      }
    }
    window.removeEventListener('mousemove', onMouseMove)
    window.removeEventListener('mouseup', onMouseUp)
  }

  window.addEventListener('mousemove', onMouseMove)
  window.addEventListener('mouseup', onMouseUp)
}

// 时间格式化 00:27 / 05:57 或 01:20:15
const formatTime = (secs: number) => {
  if (!secs || isNaN(secs) || secs < 0) secs = 0
  const s = Math.floor(secs)
  const hours = Math.floor(s / 3600)
  const minutes = Math.floor((s % 3600) / 60)
  const seconds = s % 60

  const pad = (n: number) => n.toString().padStart(2, '0')
  if (hours > 0) {
    return `${pad(hours)}:${pad(minutes)}:${pad(seconds)}`
  }
  return `${pad(minutes)}:${pad(seconds)}`
}

const handleKeyDown = (e: KeyboardEvent) => {
  if (e.key === 'Escape') {
    if (activePopup.value !== null || showVolumePopup.value) {
      e.stopPropagation()
      e.preventDefault()
      activePopup.value = null
      showVolumePopup.value = false
      cancelHidePopup()
      emit('popup-state-change', false)
    }
  }
}

onMounted(() => {
  window.addEventListener('keydown', handleKeyDown)
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown)
  cancelHidePopup()
  if (hideVolumeTimer) {
    clearTimeout(hideVolumeTimer)
    hideVolumeTimer = null
  }
  throttledSeek.cancel()
  throttledVolume.cancel()
  throttledWheelVolume.cancel()
})
</script>

<style scoped>
.controlbar {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  padding: 0 16px 12px 16px;
  background: var(--control-gradient);
  z-index: 100;
  display: flex;
  flex-direction: column;
  transition: opacity 0.25s ease;
}

.time-tooltip {
  position: absolute;
  bottom: 18px;
  transform: translateX(-50%);
  background: rgba(0, 0, 0, 0.85);
  padding: 3px 8px;
  border-radius: 4px;
  font-size: 11.5px;
  font-weight: 500;
  color: #FFFFFF;
  pointer-events: none;
  white-space: nowrap;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.4);
}

.control-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 42px;
}

.control-left,
.control-right {
  display: flex;
  align-items: center;
  gap: 12px;
}

.icon-btn {
  width: 32px;
  height: 32px;
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: rgba(255, 255, 255, 0.88);
  transition: all 0.15s ease;
}

.icon-btn:hover {
  color: #FFFFFF;
  background: rgba(255, 255, 255, 0.15);
}

.icon-btn.active {
  color: var(--pulse-pink);
}

.text-btn {
  padding: 4px 10px;
  border-radius: 6px;
  font-size: 13.5px;
  font-weight: 500;
  color: rgba(255, 255, 255, 0.88);
  transition: all 0.15s ease;
}

.text-btn:hover,
.text-btn.active {
  color: var(--pulse-pink);
  background: rgba(255, 255, 255, 0.12);
}

.time-label {
  font-size: 12.5px;
  color: rgba(255, 255, 255, 0.7);
  display: flex;
  align-items: center;
  gap: 4px;
  margin-left: 4px;
  font-variant-numeric: tabular-nums;
}

.curr-time {
  color: #FFFFFF;
  font-weight: 500;
}

.popup-anchor {
  position: relative;
}

.popup-pos {
  position: absolute;
  bottom: 100%;
  right: 50%;
  transform: translateX(50%);
  z-index: 110;
}

/* 设置面板专属定位：偏左对齐，彻底杜绝在窗口右边界裁切溢出 */
.popup-pos-settings {
  position: absolute;
  bottom: 100%;
  right: -40px;
  z-index: 110;
}

/* 垂直向上弹出音量条 (高质感对齐 B 站) */
.volume-group {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
}

.volume-popup-wrapper {
  position: absolute;
  bottom: 100%;
  left: -3px;
  width: 38px;
  padding-bottom: 12px; /* 悬停缓冲区间，防止鼠标从图标移入面板时中断 */
  z-index: 110;
}

.volume-popup {
  width: 38px;
  height: 136px;
  background: rgba(24, 25, 28, 0.94);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  padding: 10px 0 12px 0;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.55);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  display: flex;
  flex-direction: column;
  align-items: center;
  user-select: none;
}

.volume-num {
  font-size: 12px;
  font-weight: 600;
  color: #FFFFFF;
  text-align: center;
  font-variant-numeric: tabular-nums;
  line-height: 1;
  margin-bottom: 10px;
}

.volume-track-container {
  position: relative;
  width: 100%;
  flex: 1;
  display: flex;
  justify-content: center;
  cursor: pointer;
}

.volume-track-groove {
  position: absolute;
  top: 0;
  bottom: 0;
  width: 3.5px;
  background: rgba(255, 255, 255, 0.75);
  border-radius: 2px;
}

.volume-track-fill {
  position: absolute;
  bottom: 0;
  width: 3.5px;
  background: var(--pulse-pink);
  border-radius: 2px;
}

.volume-thumb {
  position: absolute;
  width: 12px;
  height: 12px;
  border-radius: 50%;
  background: var(--pulse-pink);
  left: 50%;
  margin-left: -6px;
  box-shadow: 0 0 5px rgba(251, 114, 153, 0.75);
  pointer-events: none;
  transition: transform 0.12s ease;
}

.volume-track-container:hover .volume-thumb,
.volume-popup:hover .volume-thumb {
  transform: scale(1.18);
}
</style>
