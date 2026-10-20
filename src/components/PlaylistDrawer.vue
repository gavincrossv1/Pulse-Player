<template>
  <div
    class="playlist-drawer-wrapper"
    @mouseenter="emit('mouse-enter')"
    @mouseleave="emit('mouse-leave')"
  >
    <div class="playlist-drawer">
      <div class="drawer-header">
        <div class="header-info">
          <span class="drawer-title">选集列表</span>
          <span class="drawer-count">({{ playlist.length }})</span>
        </div>
        <div class="header-actions">
          <button class="action-btn" @click.stop="handleReverse" title="一键倒序排列">
            <SvgIcon name="sort" size="12" />
            <span>倒序</span>
          </button>
          <button class="action-btn" @click.stop="handleShuffle" title="一键随机洗牌">
            <SvgIcon name="shuffle" size="12" />
            <span>随机</span>
          </button>
        </div>
      </div>

      <div class="drawer-list">
        <div
          v-for="(item, idx) in playlist"
          :key="item.path"
          class="playlist-item"
          :class="{ active: isSamePath(currentPath, item.path) }"
          @click="emit('select-video', item.path)"
          :title="item.name"
        >
          <span class="item-index">{{ idx + 1 }}</span>
          <span class="item-name">{{ item.name }}</span>
          <span v-if="isSamePath(currentPath, item.path)" class="playing-badge">播放中</span>

          <!-- 临时内存微调上移/下移工具 -->
          <div class="reorder-tools" @click.stop>
            <button
              class="reorder-btn"
              :disabled="idx === 0"
              @click="handleMoveItem(idx, idx - 1)"
              title="上移本条"
            >
              ▲
            </button>
            <button
              class="reorder-btn"
              :disabled="idx === playlist.length - 1"
              @click="handleMoveItem(idx, idx + 1)"
              title="下移本条"
            >
              ▼
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { nextTick, onMounted } from 'vue'
import SvgIcon from './SvgIcon.vue'
import { isSamePath } from '../utils/path'

export interface PlaylistItem {
  name: string
  path: string
}

const props = defineProps<{
  playlist: PlaylistItem[]
  currentPath: string
}>()

const emit = defineEmits<{
  (e: 'select-video', path: string): void
  (e: 'update-playlist', list: PlaylistItem[]): void
  (e: 'mouse-enter'): void
  (e: 'mouse-leave'): void
}>()

const handleReverse = () => {
  const reversed = [...props.playlist].reverse()
  emit('update-playlist', reversed)
}

const handleShuffle = () => {
  const shuffled = [...props.playlist]
  for (let i = shuffled.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1));
    [shuffled[i], shuffled[j]] = [shuffled[j], shuffled[i]]
  }
  emit('update-playlist', shuffled)
}

const handleMoveItem = (fromIdx: number, toIdx: number) => {
  if (toIdx < 0 || toIdx >= props.playlist.length) return
  const list = [...props.playlist]
  const [moved] = list.splice(fromIdx, 1)
  list.splice(toIdx, 0, moved)
  emit('update-playlist', list)
}

// 展开选集面板时自动定位至当前正在播放集数
onMounted(() => {
  nextTick(() => {
    const activeEl = document.querySelector('.drawer-list .playlist-item.active')
    if (activeEl) {
      activeEl.scrollIntoView({ block: 'nearest' })
    }
  })
})
</script>

<style scoped>
.playlist-drawer-wrapper {
  padding-bottom: 12px;
}

.playlist-drawer {
  width: 330px;
  max-height: 400px;
  background: rgba(24, 25, 28, 0.94);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  padding: 12px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.55);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  display: flex;
  flex-direction: column;
}

.drawer-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-bottom: 8px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  margin-bottom: 8px;
}

.header-info {
  display: flex;
  align-items: center;
  gap: 6px;
}

.drawer-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
}

.drawer-count {
  font-size: 11.5px;
  color: var(--text-muted);
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 6px;
}

.action-btn {
  display: flex;
  align-items: center;
  gap: 3px;
  padding: 2px 7px;
  border-radius: 4px;
  background: rgba(255, 255, 255, 0.06);
  color: rgba(255, 255, 255, 0.75);
  font-size: 11px;
  transition: all 0.15s ease;
}

.action-btn:hover {
  background: rgba(255, 255, 255, 0.12);
  color: #FFFFFF;
}

.drawer-list {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding-right: 2px;
}

.playlist-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 7px 10px;
  border-radius: 6px;
  cursor: pointer;
  background: var(--bg-card);
  transition: all 0.15s ease;
  position: relative;
}

.playlist-item:hover {
  background: var(--bg-card-hover);
}

.playlist-item.active {
  background: rgba(251, 114, 153, 0.16);
  border-left: 3px solid var(--pulse-pink);
}

.item-index {
  font-size: 11px;
  font-weight: 600;
  color: var(--text-muted);
  min-width: 18px;
  text-align: center;
}

.playlist-item.active .item-index {
  color: var(--pulse-pink);
}

.item-name {
  flex: 1;
  font-size: 12.5px;
  color: var(--text-primary);
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.playlist-item.active .item-name {
  color: var(--pulse-pink);
  font-weight: 500;
}

.playing-badge {
  font-size: 10px;
  padding: 1px 5px;
  border-radius: 3px;
  background: var(--pulse-pink);
  color: #FFFFFF;
  flex-shrink: 0;
}

.reorder-tools {
  display: flex;
  align-items: center;
  gap: 2px;
  opacity: 0;
  transition: opacity 0.15s ease;
  margin-left: 4px;
  flex-shrink: 0;
}

.playlist-item:hover .reorder-tools {
  opacity: 1;
}

.reorder-btn {
  width: 18px;
  height: 18px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 8px;
  color: rgba(255, 255, 255, 0.6);
  border-radius: 3px;
  background: rgba(255, 255, 255, 0.08);
  transition: all 0.15s ease;
}

.reorder-btn:hover:not(:disabled) {
  background: var(--pulse-pink);
  color: #FFFFFF;
}

.reorder-btn:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}
</style>
