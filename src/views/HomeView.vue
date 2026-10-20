<template>
  <div class="home-view">
    <!-- 背景暗域微光辉光 (Ambient Glow) -->
    <div class="ambient-glow"></div>

    <!-- 顶部极简窗口控制条 -->
    <div
      class="home-titlebar"
      data-tauri-drag-region
      @mousedown="handleMouseDown"
      @dblclick="emit('toggle-maximize')"
    >
      <div class="brand">
        <SvgIcon name="pulse-logo" size="22" />
        <span class="brand-title">Pulse Player</span>
      </div>
      <div class="win-controls" @mousedown.stop>
        <button class="win-btn settings-btn" @click="showGlobalSettings = true" title="全局偏好设置">
          <SvgIcon name="settings" size="14" />
        </button>
        <button class="win-btn" @click="emit('minimize')" title="最小化">
          <SvgIcon name="minimize" size="14" />
        </button>
        <button class="win-btn" @click="emit('toggle-maximize')" :title="isMaximized ? '还原' : '最大化'">
          <SvgIcon :name="isMaximized ? 'restore' : 'maximize'" size="13" />
        </button>
        <button class="win-btn close-btn" @click="emit('close')" title="关闭">
          <SvgIcon name="close" size="14" />
        </button>
      </div>
    </div>

    <!-- 主体居中容器 -->
    <div class="home-content">
      <!-- Logo 与产品标语 -->
      <div class="hero-section">
        <div class="hero-logo-box">
          <div class="logo-back-glow"></div>
          <SvgIcon name="pulse-logo" size="76" class="hero-logo" />
        </div>
        <h1 class="hero-title">Pulse Player</h1>
        <p class="hero-desc">现代极简 · 硬件直解 · 极速离线媒体播放器</p>
      </div>

      <!-- 交互式投落胶囊 (Drop Capsule: 一体化选择与拖拽唤起) -->
      <div
        class="drop-capsule"
        :class="{ 'is-dragging': isDraggingOver }"
        @click="emit('select-local-video')"
      >
        <div class="capsule-inner">
          <div class="capsule-icon-row">
            <span class="pulse-spark">✦</span>
            <SvgIcon name="play" size="22" color="#FF2E88" />
            <span class="pulse-spark">✦</span>
          </div>
          <div class="capsule-title">打开媒体文件或拖拽至此处</div>
          <div class="capsule-hint">
            支持 MP4, MKV, FLV, TS 及 MP3, FLAC, WAV 等 25+ 格式原生硬解
          </div>
        </div>
        <div class="capsule-border-glow"></div>
      </div>

      <!-- 历史记录与断点续播 -->
      <div v-if="historyList.length > 0" class="history-section">
        <div class="history-header">
          <div class="history-title-wrap">
            <SvgIcon name="history" size="14" />
            <span class="history-title">最近播放 (断点记忆)</span>
          </div>
          <button class="clear-history-btn" @click.stop="emit('clear-history')" title="清空全部最近播放记录">
            <SvgIcon name="trash" size="13" />
            <span>清空记录</span>
          </button>
        </div>
        <div class="history-list">
          <div
            v-for="item in historyList"
            :key="item.path"
            class="history-item"
            @click="emit('resume-history', item)"
            :title="item.path"
          >
            <div class="history-info">
              <span class="item-name">{{ item.name }}</span>
              <span class="item-path">{{ item.path }}</span>
            </div>
            <div class="history-actions">
              <span class="progress-tag">{{ formatProgress(item.timestamp, item.duration) }}</span>
              <button
                class="delete-item-btn"
                @click.stop="emit('delete-history-item', item.path)"
                title="删除此条记录"
              >
                <SvgIcon name="close" size="12" />
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 全局偏好设置弹窗 -->
    <GlobalSettingsModal
      :visible="showGlobalSettings"
      @close="showGlobalSettings = false"
    />
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import SvgIcon from '../components/SvgIcon.vue'
import GlobalSettingsModal from '../components/GlobalSettingsModal.vue'

const showGlobalSettings = ref(false)

export interface HistoryItem {
  name: string
  path: string
  timestamp: number
  duration: number
}

defineProps<{
  historyList: HistoryItem[]
  isMaximized: boolean
  isDraggingOver?: boolean
}>()

const emit = defineEmits<{
  (e: 'select-local-video'): void
  (e: 'resume-history', item: HistoryItem): void
  (e: 'delete-history-item', path: string): void
  (e: 'clear-history'): void
  (e: 'minimize'): void
  (e: 'toggle-maximize'): void
  (e: 'close'): void
}>()

const handleMouseDown = (e: MouseEvent) => {
  if (e.buttons === 1) {
    invoke('start_dragging')
  }
}

const formatProgress = (timestamp: number, duration: number) => {
  if (!duration || duration <= 0 || !timestamp || timestamp <= 0) return '从头开始'
  const pad = (n: number) => Math.floor(n).toString().padStart(2, '0')
  const s = Math.floor(timestamp)
  const hours = Math.floor(s / 3600)
  const minutes = Math.floor((s % 3600) / 60)
  const seconds = s % 60
  const timeStr = hours > 0 ? `${pad(hours)}:${pad(minutes)}:${pad(seconds)}` : `${pad(minutes)}:${pad(seconds)}`
  const percent = Math.min(100, Math.round((timestamp / duration) * 100))
  return `上次播至 ${timeStr} (${percent}%)`
}
</script>

<style scoped>
.home-view {
  width: 100vw;
  height: 100vh;
  background-color: #0E0F12;
  display: flex;
  flex-direction: column;
  position: relative;
  overflow: hidden;
  user-select: none;
}

/* 背景中心径向微光 (沉浸暗夜微光氛围) */
.ambient-glow {
  position: absolute;
  top: 30%;
  left: 50%;
  width: 500px;
  height: 500px;
  transform: translate(-50%, -50%);
  background: radial-gradient(circle, rgba(255, 46, 136, 0.08) 0%, rgba(251, 114, 153, 0.03) 40%, transparent 70%);
  pointer-events: none;
  filter: blur(40px);
}

.home-titlebar {
  height: 44px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 14px;
  background: rgba(14, 15, 18, 0.85);
  backdrop-filter: blur(12px);
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  z-index: 10;
}

.brand {
  display: flex;
  align-items: center;
  gap: 8px;
  color: #FFFFFF;
  font-weight: 600;
  font-size: 14px;
  letter-spacing: 0.3px;
  pointer-events: none;
}

.brand-title {
  background: linear-gradient(135deg, #FFFFFF 30%, rgba(255, 255, 255, 0.7) 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}

.win-controls {
  display: flex;
  align-items: center;
  gap: 4px;
}

.win-btn {
  width: 32px;
  height: 32px;
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: rgba(255, 255, 255, 0.8);
  transition: all 0.15s ease;
}

.win-btn:hover {
  background: rgba(255, 255, 255, 0.12);
  color: #FFFFFF;
}

.close-btn:hover {
  background-color: #E81123 !important;
  color: #FFFFFF !important;
}

.home-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 24px;
  max-width: 660px;
  margin: 0 auto;
  width: 100%;
  z-index: 5;
}

.hero-section {
  text-align: center;
  margin-bottom: 28px;
}

.hero-logo-box {
  position: relative;
  display: inline-flex;
  margin-bottom: 16px;
}

.logo-back-glow {
  position: absolute;
  inset: -12px;
  border-radius: 50%;
  background: radial-gradient(circle, rgba(255, 46, 136, 0.4) 0%, transparent 70%);
  filter: blur(12px);
  animation: pulse-glow 3s ease-in-out infinite;
}

.hero-logo {
  position: relative;
  filter: drop-shadow(0 8px 24px rgba(255, 46, 136, 0.35));
  animation: float 3s ease-in-out infinite;
}

@keyframes float {
  0%, 100% { transform: translateY(0); }
  50% { transform: translateY(-5px); }
}

@keyframes pulse-glow {
  0%, 100% { opacity: 0.5; transform: scale(1); }
  50% { opacity: 0.9; transform: scale(1.1); }
}

.hero-title {
  font-size: 30px;
  font-weight: 800;
  letter-spacing: -0.5px;
  color: #FFFFFF;
  margin-bottom: 8px;
  background: linear-gradient(180deg, #FFFFFF 0%, rgba(255, 255, 255, 0.85) 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}

.hero-desc {
  font-size: 13.5px;
  font-weight: 400;
  color: rgba(255, 255, 255, 0.55);
  letter-spacing: 0.5px;
}

/* 交互投落卡片 */
.drop-capsule {
  width: 100%;
  height: 124px;
  border-radius: 14px;
  background: rgba(255, 255, 255, 0.03);
  border: 1px dashed rgba(255, 255, 255, 0.15);
  backdrop-filter: blur(16px);
  cursor: pointer;
  position: relative;
  overflow: hidden;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.25s cubic-bezier(0.2, 0.8, 0.2, 1);
  margin-bottom: 24px;
}

.drop-capsule:hover {
  background: rgba(255, 46, 136, 0.05);
  border-color: rgba(255, 46, 136, 0.4);
  transform: translateY(-2px);
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.4), 0 0 24px rgba(255, 46, 136, 0.15);
}

.drop-capsule.is-dragging {
  background: rgba(255, 46, 136, 0.12);
  border-color: #FF2E88;
  border-style: solid;
  transform: scale(1.02);
  box-shadow: 0 0 32px rgba(255, 46, 136, 0.35);
}

.capsule-inner {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  text-align: center;
  pointer-events: none;
}

.capsule-icon-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.pulse-spark {
  color: #FF2E88;
  font-size: 11px;
  opacity: 0.6;
}

.capsule-title {
  font-size: 15px;
  font-weight: 600;
  color: #FFFFFF;
  letter-spacing: 0.2px;
}

.capsule-hint {
  font-size: 12px;
  color: rgba(255, 255, 255, 0.45);
}

/* 历史记录区 */
.history-section {
  width: 100%;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.history-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 4px;
}

.history-title-wrap {
  display: flex;
  align-items: center;
  gap: 6px;
  color: rgba(255, 255, 255, 0.85);
  font-size: 13px;
  font-weight: 600;
}

.clear-history-btn {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: 11.5px;
  color: rgba(255, 255, 255, 0.45);
  padding: 4px 8px;
  border-radius: 4px;
  transition: all 0.15s ease;
}

.clear-history-btn:hover {
  color: #E81123;
  background: rgba(232, 17, 35, 0.1);
}

.history-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
  max-height: 180px;
  overflow-y: auto;
  padding-right: 4px;
}

.history-list::-webkit-scrollbar {
  width: 4px;
}

.history-list::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.15);
  border-radius: 4px;
}

.history-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 14px;
  border-radius: 8px;
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid rgba(255, 255, 255, 0.04);
  cursor: pointer;
  transition: all 0.15s ease;
}

.history-item:hover {
  background: rgba(255, 255, 255, 0.07);
  border-color: rgba(255, 255, 255, 0.1);
  transform: translateX(2px);
}

.history-info {
  display: flex;
  flex-direction: column;
  gap: 3px;
  overflow: hidden;
  padding-right: 12px;
}

.item-name {
  font-size: 13px;
  font-weight: 500;
  color: #FFFFFF;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.item-path {
  font-size: 11px;
  color: rgba(255, 255, 255, 0.4);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.history-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}

.progress-tag {
  font-size: 11px;
  padding: 2px 7px;
  border-radius: 4px;
  background: rgba(255, 46, 136, 0.15);
  color: #FF62A5;
  font-weight: 500;
}

.delete-item-btn {
  width: 20px;
  height: 20px;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: rgba(255, 255, 255, 0.3);
  transition: all 0.15s ease;
}

.delete-item-btn:hover {
  background: rgba(255, 255, 255, 0.15);
  color: #FFFFFF;
}
</style>
