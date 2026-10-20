<template>
  <div
    class="settings-popup-wrapper"
    @mouseenter="onMouseEnter"
    @mouseleave="onMouseLeave"
  >
    <div class="settings-popup">
      <!-- 播放方式 -->
      <div class="setting-group">
        <div class="group-title">播放方式</div>
        <div class="group-options">
          <button
            v-for="item in playModes"
            :key="item.id"
            class="option-btn"
            :class="{ active: playMode === item.id }"
            @click="emit('update-play-mode', item.id)"
          >
            {{ item.label }}
          </button>
        </div>
      </div>

      <!-- 视频比例 -->
      <div class="setting-group">
        <div class="group-title">视频比例</div>
        <div class="group-options">
          <button
            v-for="item in aspectRatios"
            :key="item.id"
            class="option-btn"
            :class="{ active: aspectRatio === item.id }"
            @click="emit('update-aspect-ratio', item.id)"
          >
            {{ item.label }}
          </button>
        </div>
      </div>

      <!-- 画面变换 -->
      <div class="setting-group">
        <div class="group-title">画面变换</div>
        <div class="group-options">
          <button
            class="option-btn"
            :class="{ active: isMirrored }"
            @click="emit('toggle-mirror')"
          >
            水平镜像: {{ isMirrored ? '开启' : '关闭' }}
          </button>
          <button
            class="option-btn"
            @click="emit('rotate-clockwise')"
          >
            画面旋转: {{ rotation }}°
          </button>
        </div>
      </div>

      <!-- 解码策略 -->
      <div class="setting-group">
        <div class="group-title">解码模式</div>
        <div class="group-options">
          <button
            v-for="item in hwdecModes"
            :key="item.id"
            class="option-btn"
            :class="{ active: hwdecMode === item.id }"
            @click="emit('update-hwdec', item.id)"
          >
            {{ item.label }}
          </button>
        </div>
      </div>

      <!-- 快捷微调 (仅当前视频临时生效，切集后自动恢复全局默认) -->
      <div class="setting-group quick-adjust-group">
        <div class="group-title flex-title">
          <span>快捷微调</span>
          <span class="sub-hint">仅当前视频生效</span>
        </div>
        <div class="sub-setting-row">
          <span class="sub-label">快进步长</span>
          <div class="mini-options">
            <button
              v-for="s in [3, 5, 10, 15, 30]"
              :key="s"
              class="mini-btn"
              :class="{ active: currentVideoConfig.seekStep === s }"
              @click="currentVideoConfig.seekStep = s"
            >
              {{ s }}s
            </button>
          </div>
        </div>
        <div class="sub-setting-row">
          <span class="sub-label">长按倍速</span>
          <div class="mini-options">
            <button
              v-for="sp in [1.5, 2.0, 2.5, 3.0, 4.0]"
              :key="sp"
              class="mini-btn"
              :class="{ active: Math.abs(currentVideoConfig.longPressSpeed - sp) < 0.01 }"
              @click="currentVideoConfig.longPressSpeed = sp"
            >
              {{ sp.toFixed(1) }}x
            </button>
          </div>
        </div>
        <div class="sub-setting-row">
          <span class="sub-label">滚轮步长</span>
          <div class="mini-options">
            <button
              v-for="v in [2, 5, 10]"
              :key="v"
              class="mini-btn"
              :class="{ active: currentVideoConfig.wheelVolumeStep === v }"
              @click="currentVideoConfig.wheelVolumeStep = v"
            >
              {{ v }}%
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { currentVideoConfig } from '../composables/useConfig'
export type PlayMode = 'auto-next' | 'pause-on-end' | 'single-loop' | 'list-loop'
export type AspectRatio = 'auto' | '4:3' | '16:9' | 'fill'
export type HwdecMode = 'auto-safe' | 'auto-copy' | 'no'

defineProps<{
  playMode: PlayMode
  aspectRatio: AspectRatio
  isMirrored: boolean
  rotation: number
  hwdecMode: HwdecMode
}>()

const emit = defineEmits<{
  (e: 'update-play-mode', mode: PlayMode): void
  (e: 'update-aspect-ratio', ratio: AspectRatio): void
  (e: 'toggle-mirror'): void
  (e: 'rotate-clockwise'): void
  (e: 'update-hwdec', mode: HwdecMode): void
  (e: 'mouse-enter'): void
  (e: 'mouse-leave'): void
}>()

const playModes: { id: PlayMode; label: string }[] = [
  { id: 'auto-next', label: '自动切集' },
  { id: 'pause-on-end', label: '播完暂停' },
  { id: 'single-loop', label: '单集循环' },
  { id: 'list-loop', label: '列表循环' },
]

const aspectRatios: { id: AspectRatio; label: string }[] = [
  { id: 'auto', label: '自动' },
  { id: '4:3', label: '4:3' },
  { id: '16:9', label: '16:9' },
  { id: 'fill', label: '铺满' },
]

const hwdecModes: { id: HwdecMode; label: string }[] = [
  { id: 'auto-safe', label: '推荐硬解' },
  { id: 'auto-copy', label: '兼容硬解' },
  { id: 'no', label: '兼容软解' },
]

const onMouseEnter = () => emit('mouse-enter')
const onMouseLeave = () => emit('mouse-leave')
</script>

<style scoped>
.settings-popup-wrapper {
  padding-bottom: 12px;
}

.settings-popup {
  width: 290px;
  background: rgba(24, 25, 28, 0.94);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  padding: 14px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.55);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.setting-group {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.group-title {
  font-size: 11.5px;
  font-weight: 600;
  color: var(--text-secondary);
}

.group-options {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 6px;
}

.option-btn {
  padding: 6px 8px;
  border-radius: 4px;
  background: var(--bg-card);
  color: var(--text-primary);
  font-size: 12px;
  text-align: center;
  white-space: nowrap;
  transition: all 0.15s ease;
}

.option-btn:hover {
  background: var(--bg-card-hover);
}

.option-btn.active {
  background: var(--pulse-pink);
  color: #FFFFFF;
}

.quick-adjust-group {
  border-top: 1px solid rgba(255, 255, 255, 0.08);
  padding-top: 8px;
}

.flex-title {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.sub-hint {
  font-size: 10px;
  font-weight: normal;
  color: var(--text-muted);
}

.sub-setting-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  margin-top: 2px;
}

.sub-label {
  font-size: 11px;
  color: rgba(255, 255, 255, 0.7);
  flex-shrink: 0;
}

.mini-options {
  display: flex;
  gap: 4px;
}

.mini-btn {
  padding: 3px 6px;
  border-radius: 4px;
  background: rgba(255, 255, 255, 0.06);
  color: rgba(255, 255, 255, 0.75);
  font-size: 11px;
  font-weight: 500;
  transition: all 0.15s ease;
}

.mini-btn:hover {
  background: rgba(255, 255, 255, 0.12);
  color: #FFFFFF;
}

.mini-btn.active {
  background: var(--pulse-pink);
  color: #FFFFFF;
}
</style>
