<template>
  <div
    class="titlebar"
    data-tauri-drag-region
    @mousedown="handleMouseDown"
    @dblclick="emit('toggle-maximize')"
  >
    <!-- 左侧功能区：回到主界面 -->
    <div class="titlebar-left" @mousedown.stop>
      <button class="nav-btn" @click="emit('go-home')" title="回到主界面">
        <SvgIcon name="home" size="18" />
        <span class="nav-text">回到主界面</span>
      </button>
    </div>

    <!-- 居中标题区：[本地] 完整文件名 -->
    <div class="titlebar-center" data-tauri-drag-region>
      <span class="title-text" :title="fileName">
        {{ fileName ? `[本地] ${fileName}` : 'Pulse Player' }}
      </span>
    </div>

    <!-- 右侧窗口控制区 -->
    <div class="titlebar-right" @mousedown.stop>
      <!-- 置顶图钉 -->
      <button
        class="window-btn pin-btn"
        :class="{ 'is-pinned': isPinned }"
        @click="emit('toggle-pin')"
        :title="isPinned ? '取消置顶' : '窗口置顶'"
      >
        <SvgIcon :name="isPinned ? 'pin-active' : 'pin'" size="16" />
      </button>

      <!-- 最小化 -->
      <button class="window-btn" @click="emit('minimize')" title="最小化">
        <SvgIcon name="minimize" size="14" />
      </button>

      <!-- 最大化 / 还原 -->
      <button class="window-btn" @click="emit('toggle-maximize')" :title="isMaximized ? '还原' : '最大化'">
        <SvgIcon :name="isMaximized ? 'restore' : 'maximize'" size="13" />
      </button>

      <!-- 关闭 -->
      <button class="window-btn close-btn" @click="emit('close')" title="关闭">
        <SvgIcon name="close" size="14" />
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import SvgIcon from './SvgIcon.vue'

defineProps<{
  fileName: string
  isPinned: boolean
  isMaximized: boolean
}>()

const emit = defineEmits<{
  (e: 'go-home'): void
  (e: 'toggle-pin'): void
  (e: 'minimize'): void
  (e: 'toggle-maximize'): void
  (e: 'close'): void
}>()

const handleMouseDown = (e: MouseEvent) => {
  // 左键点击空白区域或标题时触发原生窗口拖拽
  if (e.buttons === 1) {
    invoke('start_dragging')
  }
}
</script>

<style scoped>
.titlebar {
  height: 40px;
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 10px;
  background: rgba(18, 19, 21, 0.98);
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  z-index: 100;
  position: relative;
  flex-shrink: 0;
  user-select: none;
  cursor: default;
}

.titlebar-left,
.titlebar-right {
  display: flex;
  align-items: center;
  gap: 5px;
  z-index: 101;
}

.nav-btn {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 4px 10px;
  border-radius: 5px;
  color: #FFFFFF;
  background: rgba(255, 255, 255, 0.1);
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: background 0.2s ease, transform 0.1s ease;
}

.nav-btn:hover {
  background: rgba(255, 255, 255, 0.2);
}

.nav-btn:active {
  transform: scale(0.96);
}

.titlebar-center {
  flex: 1;
  text-align: center;
  padding: 0 16px;
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
  cursor: default;
}

.title-text {
  font-size: 13px;
  font-weight: 500;
  color: rgba(255, 255, 255, 0.88);
  letter-spacing: 0.3px;
  pointer-events: none;
  user-select: none;
}

.window-btn {
  width: 28px;
  height: 28px;
  border-radius: 5px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  color: rgba(255, 255, 255, 0.85);
  cursor: pointer;
  transition: background-color 0.15s ease, color 0.15s ease;
}

.window-btn:hover {
  background-color: rgba(255, 255, 255, 0.15);
  color: #FFFFFF;
}

.pin-btn.is-pinned {
  color: var(--pulse-pink);
  background-color: var(--pulse-pink-light);
}

.close-btn:hover {
  background-color: #E81123 !important;
  color: #FFFFFF !important;
}
</style>
