<template>
  <transition name="modal-fade">
    <div v-if="visible" class="modal-backdrop" @click="emit('close')">
      <div class="settings-modal" @click.stop>
        <!-- 头部 -->
        <div class="modal-header">
          <div class="header-left">
            <SvgIcon name="settings" size="18" />
            <span class="header-title">播放器全局偏好设置</span>
          </div>
          <button class="close-btn" @click="emit('close')" title="关闭">
            <SvgIcon name="close" size="14" />
          </button>
        </div>

        <!-- 内容表单 -->
        <div class="modal-body">
          <!-- 左右键快进/快退步长 -->
          <div class="setting-item">
            <div class="setting-label-col">
              <span class="item-title">快进 / 快退步长</span>
              <span class="item-desc">左右方向键短按单次步进时间</span>
            </div>
            <div class="option-pills">
              <button
                v-for="val in seekStepOptions"
                :key="val"
                class="pill-btn"
                :class="{ active: globalConfig.seekStep === val }"
                @click="updateConfig({ seekStep: val })"
              >
                {{ val }}秒
              </button>
            </div>
          </div>

          <!-- 长按快进倍速 -->
          <div class="setting-item">
            <div class="setting-label-col">
              <span class="item-title">长按快进倍速</span>
              <span class="item-desc">长按右方向键或画面触发的极速倍率</span>
            </div>
            <div class="option-pills">
              <button
                v-for="val in speedOptions"
                :key="val"
                class="pill-btn"
                :class="{ active: globalConfig.longPressSpeed === val }"
                @click="updateConfig({ longPressSpeed: val })"
              >
                {{ val.toFixed(1) }}x
              </button>
            </div>
          </div>

          <!-- 滚轮音量步长 -->
          <div class="setting-item">
            <div class="setting-label-col">
              <span class="item-title">滚轮音量步长</span>
              <span class="item-desc">画面或控制栏滚轮单格调节幅度</span>
            </div>
            <div class="option-pills">
              <button
                v-for="val in volumeStepOptions"
                :key="val"
                class="pill-btn"
                :class="{ active: globalConfig.wheelVolumeStep === val }"
                @click="updateConfig({ wheelVolumeStep: val })"
              >
                {{ val }}%
              </button>
            </div>
          </div>

          <!-- 底栏静止隐藏时间 -->
          <div class="setting-item">
            <div class="setting-label-col">
              <span class="item-title">底栏自动隐藏时间</span>
              <span class="item-desc">光标在画面静止后平滑隐藏控制栏延时</span>
            </div>
            <div class="option-pills">
              <button
                v-for="opt in hideDelayOptions"
                :key="opt.value"
                class="pill-btn"
                :class="{ active: globalConfig.autoHideDelay === opt.value }"
                @click="updateConfig({ autoHideDelay: opt.value })"
              >
                {{ opt.label }}
              </button>
            </div>
          </div>

          <!-- 自动断点记忆续播 -->
          <div class="setting-item">
            <div class="setting-label-col">
              <span class="item-title">断点记忆自动续播</span>
              <span class="item-desc">打开近期看过的视频时自动恢复上次播放进度</span>
            </div>
            <div class="option-pills">
              <button
                class="pill-btn"
                :class="{ active: globalConfig.autoResume }"
                @click="updateConfig({ autoResume: true })"
              >
                开启
              </button>
              <button
                class="pill-btn"
                :class="{ active: !globalConfig.autoResume }"
                @click="updateConfig({ autoResume: false })"
              >
                关闭
              </button>
            </div>
          </div>
        </div>

        <!-- 底部说明 -->
        <div class="modal-footer">
          <span class="footer-tip">全局配置修改即时保存，作为每个新视频的默认偏好生效</span>
          <button class="done-btn" @click="emit('close')">完成</button>
        </div>
      </div>
    </div>
  </transition>
</template>
<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue'
import SvgIcon from './SvgIcon.vue'
import { useConfig, GlobalConfig } from '../composables/useConfig'

const props = defineProps<{
  visible: boolean
}>()

const emit = defineEmits<{
  (e: 'close'): void
}>()

const { globalConfig, saveGlobalConfig } = useConfig()

const seekStepOptions = [3, 5, 10, 15, 30]
const speedOptions = [1.5, 2.0, 2.5, 3.0, 4.0]
const volumeStepOptions = [2, 5, 10]
const hideDelayOptions = [
  { label: '1.5 秒', value: 1.5 },
  { label: '2.5 秒', value: 2.5 },
  { label: '4.0 秒', value: 4.0 },
  { label: '从不隐藏', value: 0 },
]

const updateConfig = (patch: Partial<GlobalConfig>) => {
  saveGlobalConfig(patch)
}

const handleKeyDown = (e: KeyboardEvent) => {
  if (e.key === 'Escape' && props.visible) {
    emit('close')
  }
}

onMounted(() => {
  window.addEventListener('keydown', handleKeyDown)
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown)
})
</script>

<style scoped>
.modal-backdrop {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.65);
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.settings-modal {
  width: 520px;
  max-width: 90vw;
  background: rgba(26, 27, 30, 0.96);
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 12px;
  box-shadow: 0 16px 40px rgba(0, 0, 0, 0.65);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.modal-header {
  height: 48px;
  padding: 0 18px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  background: rgba(255, 255, 255, 0.02);
}

.header-left {
  display: flex;
  align-items: center;
  gap: 8px;
  color: var(--text-primary);
}

.header-title {
  font-size: 14.5px;
  font-weight: 600;
}

.close-btn {
  width: 28px;
  height: 28px;
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-secondary);
  transition: all 0.15s ease;
}

.close-btn:hover {
  background: rgba(255, 255, 255, 0.1);
  color: #FFFFFF;
}

.modal-body {
  padding: 16px 20px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.setting-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
}

.setting-label-col {
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.item-title {
  font-size: 13.5px;
  font-weight: 500;
  color: var(--text-primary);
}

.item-desc {
  font-size: 11.5px;
  color: var(--text-muted);
}

.option-pills {
  display: flex;
  gap: 6px;
  flex-shrink: 0;
}

.pill-btn {
  padding: 4px 10px;
  font-size: 12px;
  font-weight: 500;
  border-radius: 6px;
  background: rgba(255, 255, 255, 0.06);
  color: rgba(255, 255, 255, 0.75);
  border: 1px solid transparent;
  transition: all 0.15s ease;
}

.pill-btn:hover {
  background: rgba(255, 255, 255, 0.12);
  color: #FFFFFF;
}

.pill-btn.active {
  background: var(--pulse-pink);
  color: #FFFFFF;
  box-shadow: 0 2px 8px rgba(251, 114, 153, 0.35);
}

.modal-footer {
  height: 52px;
  padding: 0 20px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  border-top: 1px solid rgba(255, 255, 255, 0.08);
  background: rgba(0, 0, 0, 0.2);
}

.footer-tip {
  font-size: 11.5px;
  color: var(--text-muted);
}

.done-btn {
  padding: 6px 18px;
  font-size: 13px;
  font-weight: 500;
  border-radius: 6px;
  background: var(--pulse-pink);
  color: #FFFFFF;
  transition: background 0.15s ease;
}

.done-btn:hover {
  background: var(--pulse-pink-hover);
}

.modal-fade-enter-active,
.modal-fade-leave-active {
  transition: opacity 0.2s ease;
}

.modal-fade-enter-from,
.modal-fade-leave-to {
  opacity: 0;
}
</style>
