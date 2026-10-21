<template>
  <div
    class="player-view-container"
    :class="{ 'is-fullscreen': isFullscreen }"
  >
    <!-- 顶部独立物理标题栏 (窗口与最大化常驻高 40px，与视频物理隔离绝对不挡画面，全屏时隐去) -->
    <TitleBar
      v-if="!isFullscreen"
      :file-name="currentVideoName"
      :is-pinned="isPinned"
      :is-maximized="isMaximized"
      @mousemove="onMouseMove"
      @go-home="emit('go-home')"
      @toggle-pin="emit('toggle-pin')"
      @minimize="emit('minimize')"
      @toggle-maximize="emit('toggle-maximize')"
      @close="emit('close')"
    />

    <!-- 视频/音频视口主体 (窗口模式下自动占满剩余 calc(100vh - 40px)，全屏占满 100vh) -->
    <div
      class="player-viewport"
      :class="{ 'hide-cursor': !controlsVisible }"
      @mousemove="onMouseMove"
      @mouseenter="onMouseMove"
      @mouseleave="onMouseLeaveWindow"
    >
      <!-- 穿透式居中 OSD 提示层 (顶部紧凑居中 20px，完全避开人像面部，对齐 B 站) -->
      <OsdCapsule ref="osdRef" />

      <!-- 纯音频/音乐文件播放时的沉浸式视觉卡片 (Pulse 动态黑胶能量核心与格式徽章) -->
      <div v-if="isAudio" class="audio-visual-container">
        <div class="audio-disc" :class="{ playing: !isPaused }">
          <div class="disc-inner">
            <SvgIcon name="pulse-logo" size="52" />
          </div>
          <div class="disc-glow"></div>
        </div>
        <div class="audio-meta">
          <div class="audio-title">{{ currentVideoName }}</div>
          <div class="audio-badge-row">
            <span class="audio-tag">{{ audioFormatTag }}</span>
            <span class="audio-hint">本地音频播放中</span>
          </div>
        </div>
      </div>

      <!-- 视口手势交互透明层 (用于响应单击防抖播放/暂停、双击全屏、滚轮音量) -->
      <div
        class="video-gesture-layer"
        @click="onViewportClick"
        @dblclick="onViewportDblClick"
        @wheel="onViewportWheel"
      ></div>

      <!-- 底部悬浮控制台 (悬浮淡入淡出，不占用视口物理高度) -->
      <transition name="bar-fade">
        <ControlBar
          v-show="controlsVisible"
          :current-time="currentTime"
          :duration="duration"
          :is-paused="isPaused"
          :speed="speed"
          :volume="volume"
          :is-muted="isMuted"
          :is-fullscreen="isFullscreen"
          :playlist="playlist"
          :current-path="currentVideoPath"
          :play-mode="playMode"
          :aspect-ratio="aspectRatio"
          :is-mirrored="isMirrored"
          :rotation="rotation"
          :hwdec-mode="hwdecMode"
          :visible="controlsVisible"
          @toggle-pause="emit('toggle-pause')"
          @prev-video="emit('prev-video')"
          @next-video="emit('next-video')"
          @seek="(s) => emit('seek', s)"
          @update-speed="(s) => emit('update-speed', s)"
          @update-volume="(v) => emit('update-volume', v)"
          @toggle-mute="emit('toggle-mute')"
          @toggle-fullscreen="emit('toggle-fullscreen')"
          @select-video="(p) => emit('select-video', p)"
          @update-playlist="(list) => emit('update-playlist', list)"
          @update-play-mode="(m) => emit('update-play-mode', m)"
          @update-aspect-ratio="(r) => emit('update-aspect-ratio', r)"
          @toggle-mirror="emit('toggle-mirror')"
          @rotate-clockwise="emit('rotate-clockwise')"
          @update-hwdec="(h) => emit('update-hwdec', h)"
          @popup-state-change="(active) => hasActivePopup = active"
          @dragging-state-change="(dragging) => isDraggingProgress = dragging"
        />
      </transition>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import TitleBar from '../components/TitleBar.vue'
import ControlBar from '../components/ControlBar.vue'
import OsdCapsule from '../components/OsdCapsule.vue'
import SvgIcon from '../components/SvgIcon.vue'
import { PlaylistItem } from '../components/PlaylistDrawer.vue'
import { PlayMode, AspectRatio, HwdecMode } from '../components/SettingsPopup.vue'
import { useAutoHide } from '../composables/useAutoHide'
import { currentVideoConfig } from '../composables/useConfig'
import { throttle } from '../utils/throttle'

const props = defineProps<{
  currentVideoPath: string
  currentTime: number
  duration: number
  isPaused: boolean
  speed: number
  volume: number
  isMuted: boolean
  isFullscreen: boolean
  isPinned: boolean
  isMaximized: boolean
  playlist: PlaylistItem[]
  playMode: PlayMode
  aspectRatio: AspectRatio
  isMirrored: boolean
  rotation: number
  hwdecMode: HwdecMode
}>()

const emit = defineEmits<{
  (e: 'go-home'): void
  (e: 'toggle-pin'): void
  (e: 'minimize'): void
  (e: 'toggle-maximize'): void
  (e: 'close'): void
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
}>()

const osdRef = ref<InstanceType<typeof OsdCapsule> | null>(null)

// 纯音频媒体识别
const isAudio = computed(() => {
  if (!props.currentVideoPath) return false
  const ext = props.currentVideoPath.split('.').pop()?.toLowerCase() || ''
  return ['mp3', 'flac', 'wav', 'm4a', 'aac', 'ogg', 'ape', 'opus', 'wma'].includes(ext)
})

const audioFormatTag = computed(() => {
  const ext = props.currentVideoPath.split('.').pop()?.toUpperCase() || 'AUDIO'
  return ext
})

// 自动隐藏状态机 (静止淡出，移出窗口立即淡出)
const {
  visible: controlsVisible,
  isDraggingProgress,
  hasActivePopup,
  resetTimer,
  onMouseMove,
  onMouseLeaveWindow,
} = useAutoHide()

const handleMouseOut = (e: MouseEvent) => {
  // 当鼠标离开当前窗口（relatedTarget 为 null 或移出窗口）时立即渐变隐藏
  if (!e.relatedTarget) {
    onMouseLeaveWindow()
  }
}

onMounted(() => {
  resetTimer()
  document.addEventListener('mouseleave', onMouseLeaveWindow)
  document.addEventListener('mouseout', handleMouseOut)
  document.addEventListener('mouseenter', onMouseMove)
  window.addEventListener('blur', onMouseLeaveWindow)
})

onUnmounted(() => {
  if (clickTimer) {
    clearTimeout(clickTimer)
    clickTimer = null
  }
  throttledWheelVolume.cancel()
  document.removeEventListener('mouseleave', onMouseLeaveWindow)
  document.removeEventListener('mouseout', handleMouseOut)
  document.removeEventListener('mouseenter', onMouseMove)
  window.removeEventListener('blur', onMouseLeaveWindow)
})

const currentVideoName = computed(() => {
  if (!props.currentVideoPath) return ''
  const parts = props.currentVideoPath.split(/[/\\]/)
  return parts[parts.length - 1] || props.currentVideoPath
})

// 单击 250ms 防抖消除与双击全屏冲突
let clickTimer: ReturnType<typeof setTimeout> | null = null

const onViewportClick = () => {
  if (clickTimer) {
    clearTimeout(clickTimer)
    clickTimer = null
    return
  }
  clickTimer = setTimeout(() => {
    emit('toggle-pause')
    clickTimer = null
  }, 250)
}

const onViewportDblClick = () => {
  if (clickTimer) {
    clearTimeout(clickTimer)
    clickTimer = null
  }
  emit('toggle-fullscreen')
}

// 画面滚轮无级调节音量 (60ms 节流，结合本地累加驱动，步长受配置管控)
let localWheelTargetVol: number | null = null

const throttledWheelVolume = throttle((nextVol: number) => {
  localWheelTargetVol = null
  emit('update-volume', nextVol)
}, 60)

const onViewportWheel = (e: WheelEvent) => {
  e.preventDefault()
  const step = currentVideoConfig.wheelVolumeStep || 5
  const delta = e.deltaY < 0 ? step : -step
  const currentBase = localWheelTargetVol !== null ? localWheelTargetVol : props.volume
  const nextVol = Math.min(100, Math.max(0, currentBase + delta))
  localWheelTargetVol = nextVol
  throttledWheelVolume(nextVol)
}

// 向外暴露 OSD 接口供全局按键调用
const showOsd = (text: string, duration?: number) => {
  osdRef.value?.show(text, duration)
}
const hideOsd = () => {
  osdRef.value?.hide()
}

defineExpose({
  showOsd,
  hideOsd,
})
</script>

<style scoped>
.player-view-container {
  width: 100vw;
  height: 100vh;
  display: flex;
  flex-direction: column;
  background-color: transparent;
  overflow: hidden;
  position: relative;
}

.player-viewport.hide-cursor,
.player-viewport.hide-cursor * {
  cursor: none !important;
}

.player-viewport {
  flex: 1;
  min-height: 0;
  width: 100%;
  position: relative;
  overflow: hidden;
}

.video-gesture-layer {
  position: absolute;
  top: 0;
  bottom: 0;
  left: 0;
  right: 0;
  z-index: 10;
  cursor: default;
}

/* 纯音频沉浸卡片 */
.audio-visual-container {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -55%);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 20px;
  z-index: 5;
  user-select: none;
  pointer-events: none;
}

.audio-disc {
  width: 130px;
  height: 130px;
  border-radius: 50%;
  background: radial-gradient(circle, #25262b 0%, #151619 70%, #0d0e10 100%);
  border: 4px solid rgba(255, 255, 255, 0.12);
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
  box-shadow: 0 12px 36px rgba(0, 0, 0, 0.6);
  transition: transform 0.3s ease;
}

.audio-disc.playing {
  animation: disc-rotate 16s linear infinite;
}

.disc-inner {
  width: 68px;
  height: 68px;
  border-radius: 50%;
  background: rgba(251, 114, 153, 0.12);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--pulse-pink);
}

.disc-glow {
  position: absolute;
  inset: -10px;
  border-radius: 50%;
  background: radial-gradient(circle, rgba(251, 114, 153, 0.25) 0%, transparent 70%);
  opacity: 0.6;
  filter: blur(10px);
}

@keyframes disc-rotate {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

.audio-meta {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  max-width: 480px;
  text-align: center;
}

.audio-title {
  font-size: 15px;
  font-weight: 600;
  color: #FFFFFF;
  text-shadow: 0 2px 8px rgba(0, 0, 0, 0.8);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 100%;
}

.audio-badge-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.audio-tag {
  font-size: 10.5px;
  font-weight: 700;
  padding: 2px 6px;
  border-radius: 4px;
  background: var(--pulse-pink);
  color: #FFFFFF;
  letter-spacing: 0.5px;
}

.audio-hint {
  font-size: 11.5px;
  color: rgba(255, 255, 255, 0.6);
}

.bar-fade-enter-active,
.bar-fade-leave-active {
  transition: opacity 0.25s cubic-bezier(0.25, 0.8, 0.25, 1);
}

.bar-fade-enter-from,
.bar-fade-leave-to {
  opacity: 0;
}
</style>
