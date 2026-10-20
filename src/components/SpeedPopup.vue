<template>
  <div
    class="speed-popup-wrapper"
    @mouseenter="onMouseEnter"
    @mouseleave="onMouseLeave"
  >
    <div class="speed-popup">
      <div class="popup-title">播放速度 (0.1x - 10.0x)</div>

      <!-- 预设快捷倍速 -->
      <div class="preset-speeds">
        <button
          v-for="s in presetList"
          :key="s"
          class="preset-btn"
          :class="{ active: Math.abs(currentSpeed - s) < 0.01 }"
          @click="selectSpeed(s)"
        >
          {{ s === 1 ? '1.0x' : s === 2 ? '2.0x' : s === 3 ? '3.0x' : `${s}x` }}
        </button>
      </div>

      <!-- 无级微调区 -->
      <div class="slider-section">
        <button class="step-btn" @click="stepSpeed(-0.1)" title="减速 0.1x">-</button>
        <div class="slider-container">
          <input
            type="range"
            min="0.1"
            max="10.0"
            step="0.1"
            :value="currentSpeed"
            @input="onSliderInput"
            class="speed-slider"
          />
        </div>
        <button class="step-btn" @click="stepSpeed(0.1)" title="加速 0.1x">+</button>
        <div class="speed-val">{{ currentSpeed.toFixed(1) }}x</div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">

const props = defineProps<{
  currentSpeed: number
}>()

const emit = defineEmits<{
  (e: 'change-speed', speed: number): void
  (e: 'mouse-enter'): void
  (e: 'mouse-leave'): void
}>()

const presetList = [0.5, 0.75, 1.0, 1.25, 1.5, 2.0, 3.0]

const selectSpeed = (s: number) => {
  emit('change-speed', s)
}

const stepSpeed = (delta: number) => {
  let next = Math.round((props.currentSpeed + delta) * 10) / 10
  if (next < 0.1) next = 0.1
  if (next > 10.0) next = 10.0
  emit('change-speed', next)
}

const onSliderInput = (e: Event) => {
  const val = parseFloat((e.target as HTMLInputElement).value)
  emit('change-speed', val)
}

const onMouseEnter = () => {
  emit('mouse-enter')
}

const onMouseLeave = () => {
  emit('mouse-leave')
}
</script>

<style scoped>
.speed-popup-wrapper {
  padding-bottom: 12px; /* 缓冲空隙，保证鼠标在移至按钮与面板间不断触 */
}

.speed-popup {
  width: 240px;
  background: rgba(24, 25, 28, 0.94);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  padding: 12px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.55);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
}

.popup-title {
  font-size: 12px;
  color: var(--text-secondary);
  margin-bottom: 10px;
  text-align: center;
}

.preset-speeds {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 6px;
  margin-bottom: 12px;
}

.preset-btn {
  padding: 6px 0;
  border-radius: 4px;
  font-size: 12.5px;
  font-weight: 500;
  color: var(--text-primary);
  background: var(--bg-card);
  transition: all 0.15s ease;
}

.preset-btn:hover {
  background: var(--bg-card-hover);
}

.preset-btn.active {
  background: var(--pulse-pink);
  color: #FFFFFF;
}

.slider-section {
  display: flex;
  align-items: center;
  gap: 8px;
  padding-top: 4px;
  border-top: 1px solid rgba(255, 255, 255, 0.08);
}

.step-btn {
  width: 24px;
  height: 24px;
  border-radius: 4px;
  background: var(--bg-card);
  color: #FFFFFF;
  font-size: 14px;
  font-weight: bold;
  display: flex;
  align-items: center;
  justify-content: center;
}

.step-btn:hover {
  background: var(--bg-card-hover);
}

.slider-container {
  flex: 1;
  display: flex;
  align-items: center;
}

.speed-slider {
  width: 100%;
  height: 4px;
  border-radius: 2px;
  accent-color: var(--pulse-pink);
  background: rgba(255, 255, 255, 0.2);
  outline: none;
  cursor: pointer;
}

.speed-val {
  min-width: 42px;
  font-size: 12.5px;
  font-weight: 600;
  color: var(--pulse-pink);
  text-align: right;
}
</style>
