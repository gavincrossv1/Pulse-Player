<template>
  <div class="app-root" :class="currentView">
    <!-- 主页态 -->
    <HomeView
      v-if="currentView === 'home'"
      :history-list="historyList"
      :is-maximized="isMaximized"
      :is-dragging-over="isDraggingOver"
      @select-local-video="handleSelectLocalVideo"
      @resume-history="handleResumeHistory"
      @delete-history-item="handleDeleteHistoryItem"
      @clear-history="handleClearHistory"
      @minimize="handleMinimize"
      @toggle-maximize="handleToggleMaximize"
      @close="handleClose"
    />

    <!-- 播放态 -->
    <PlayerView
      v-else
      ref="playerViewRef"
      :current-video-path="currentVideoPath"
      :current-time="currentTime"
      :duration="duration"
      :is-paused="isPaused"
      :speed="speed"
      :volume="volume"
      :is-muted="isMuted"
      :is-fullscreen="isFullscreen"
      :is-pinned="isPinned"
      :is-maximized="isMaximized"
      :playlist="playlist"
      :play-mode="playMode"
      :aspect-ratio="aspectRatio"
      :is-mirrored="isMirrored"
      :rotation="rotation"
      :hwdec-mode="hwdecMode"
      @go-home="handleGoHome"
      @toggle-pin="handleTogglePin"
      @minimize="handleMinimize"
      @toggle-maximize="handleToggleMaximize"
      @close="handleClose"
      @toggle-pause="handleTogglePause"
      @prev-video="handlePrevVideo"
      @next-video="handleNextVideo"
      @seek="handleSeek"
      @update-speed="handleUpdateSpeed"
      @update-volume="handleUpdateVolume"
      @toggle-mute="handleToggleMute"
      @toggle-fullscreen="handleToggleFullscreen"
      @select-video="handleSelectVideoFromPlaylist"
      @update-playlist="(list) => playlist = list"
      @update-play-mode="setPlayMode"
      @update-aspect-ratio="handleUpdateAspectRatio"
      @toggle-mirror="handleToggleMirror"
      @rotate-clockwise="handleRotateClockwise"
      @update-hwdec="handleUpdateHwdec"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, UnlistenFn } from '@tauri-apps/api/event'
import { getCurrentWebview } from '@tauri-apps/api/webview'
import HomeView, { HistoryItem } from './views/HomeView.vue'
import PlayerView from './views/PlayerView.vue'
import { PlaylistItem } from './components/PlaylistDrawer.vue'
import { PlayMode, AspectRatio, HwdecMode } from './components/SettingsPopup.vue'
import { useKeyboard } from './composables/useKeyboard'
import { useConfig } from './composables/useConfig'
import { isSamePath } from './utils/path'
import { throttle } from './utils/throttle'

const { globalConfig, resetCurrentVideoConfig } = useConfig()

// 双态路由状态 ('home' | 'player')
const currentView = ref<'home' | 'player'>('home')
const playerViewRef = ref<InstanceType<typeof PlayerView> | null>(null)

// 播放核心状态
const currentVideoPath = ref('')
const currentTime = ref(0)
const duration = ref(0)
const isPaused = ref(false)
const speed = ref(1.0)
let savedSpeedBeforeLongPress = 1.0 // 用于长按 3.0x 恢复原倍速
const volume = ref(80)
const isMuted = ref(false)
let savedVolumeBeforeMute = 80

// 窗口与设置状态
const isFullscreen = ref(false)
const isPinned = ref(false)
const isMaximized = ref(false)
const isDraggingOver = ref(false)
const PLAY_MODE_STORAGE_KEY = 'pulse_player_play_mode'
const LEGACY_PLAY_MODE_KEY = 'bi_player_play_mode'
const legacyStoredMode = localStorage.getItem(LEGACY_PLAY_MODE_KEY) as PlayMode | null
if (legacyStoredMode) {
  try {
    localStorage.setItem(PLAY_MODE_STORAGE_KEY, legacyStoredMode)
    localStorage.removeItem(LEGACY_PLAY_MODE_KEY)
  } catch (e) {
    console.error('迁移历史播放模式配置异常:', e)
  }
}
const initialPlayMode =
  (localStorage.getItem(PLAY_MODE_STORAGE_KEY) as PlayMode) ||
  'auto-next'
const playMode = ref<PlayMode>(initialPlayMode)

const setPlayMode = (m: PlayMode) => {
  playMode.value = m
  try {
    localStorage.setItem(PLAY_MODE_STORAGE_KEY, m)
    localStorage.removeItem(LEGACY_PLAY_MODE_KEY)
  } catch (e) {
    console.error('保存播放模式失败:', e)
  }
}
const aspectRatio = ref<AspectRatio>('auto')
const isMirrored = ref(false)
const rotation = ref(0)
const hwdecMode = ref<HwdecMode>('auto-safe')

// 播放列表与历史记录
const playlist = ref<PlaylistItem[]>([])
const historyList = ref<HistoryItem[]>([])

let unlistenList: UnlistenFn[] = []

// 启动与初始化
onMounted(async () => {
  // 1. 刷新历史记录
  await fetchHistory()

  // 2. 监听 mpv 派发的实时事件
  const u1 = await listen<number>('player:time', (e) => {
    currentTime.value = e.payload
  })
  const u2 = await listen<number>('player:duration', (e) => {
    duration.value = e.payload
  })
  const u3 = await listen<boolean>('player:pause', (e) => {
    isPaused.value = e.payload
  })
  const u4 = await listen('player:ended', () => {
    handleVideoEnded()
  })

  // 3. 监听单实例唤起与视频切换事件 (双击新视频直接切流播放，过滤关联测试探测文件)
  const u5 = await listen<string>('open-file', async (e) => {
    if (e.payload && e.payload.trim()) {
      const p = e.payload.trim()
      if (p.toLowerCase().includes('associate_sample')) return
      await startPlay(p)
    }
  })
  unlistenList.push(u1, u2, u3, u4, u5)

  // 4. 检查启动入参：若用户双击视频关联启动，直接直达播放态！
  try {
    const launchFile = await invoke<string | null>('get_launch_file')
    if (launchFile && !launchFile.toLowerCase().includes('associate_sample')) {
      await startPlay(launchFile)
    }
  } catch (err) {
    console.error('检查启动参数失败:', err)
  }

  // 5. 监听全局文件拖拽进入窗口事件 (支持主页态/播放态任意时刻拖入即播)
  try {
    const u6 = await getCurrentWebview().onDragDropEvent(async (event) => {
      if (event.payload.type === 'enter' || event.payload.type === 'over') {
        isDraggingOver.value = true
      } else if (event.payload.type === 'leave') {
        isDraggingOver.value = false
      } else if (event.payload.type === 'drop') {
        isDraggingOver.value = false
        const paths = event.payload.paths
        if (paths && paths.length > 0) {
          const target = paths[0]
          const ext = target.split('.').pop()?.toLowerCase() || ''
          const supported = [
            'mp4', 'mkv', 'avi', 'flv', 'mov', 'wmv', 'webm', 'ts',
            'm4v', 'rmvb', 'rm', 'm2ts', 'vob', 'mpg', 'mpeg', '3gp',
            'f4v', 'ogv',
            // 常见纯音频格式
            'mp3', 'flac', 'wav', 'm4a', 'aac', 'ogg', 'ape', 'opus', 'wma',
          ]
          if (supported.includes(ext)) {
            await startPlay(target)
          } else if (playerViewRef.value) {
            playerViewRef.value.showOsd('不支持的媒体格式', 1000)
          }
        }
      }
    })
    unlistenList.push(u6)
  } catch (err) {
    console.error('注册拖放监听失败:', err)
  }

  // 6. 双向严格同步全屏状态 (监听原生窗口事件与 DOM fullscreenchange)
  const u7 = await listen<boolean>('window:fullscreen', (e) => {
    isFullscreen.value = e.payload
    syncVideoMargin()
    setTimeout(syncVideoMargin, 150)
  })
  unlistenList.push(u7)

  document.addEventListener('fullscreenchange', onDomFullscreenChange)
  window.addEventListener('resize', throttledSyncVideoMargin)
})

const onDomFullscreenChange = () => {
  if (!document.fullscreenElement && isFullscreen.value) {
    isFullscreen.value = false
    invoke('set_fullscreen', { fullscreen: false }).catch(() => {})
    syncVideoMargin()
    setTimeout(syncVideoMargin, 150)
  }
}

onUnmounted(() => {
  unlistenList.forEach((fn) => fn())
  document.removeEventListener('fullscreenchange', onDomFullscreenChange)
  window.removeEventListener('resize', throttledSyncVideoMargin)
})

// 同步视频顶部物理安全边距 (窗口模式避开 40px 独立物理顶栏，全屏模式 0 边距无缝铺满)
const syncVideoMargin = () => {
  if (currentView.value !== 'player') return
  const ratio = isFullscreen.value ? 0 : 40.0 / Math.max(window.innerHeight, 100)
  invoke('set_video_margin', { ratio }).catch((err) => {
    console.error('设置视频安全边距失败:', err)
  })
}
const throttledSyncVideoMargin = throttle(syncVideoMargin, 50)

// 读取历史记录
const fetchHistory = async () => {
  try {
    const res = await invoke<HistoryItem[]>('get_history')
    historyList.value = res || []
  } catch (e) {
    console.error('获取历史记录失败:', e)
  }
}

// 刷新同目录下播放列表 (若当前列表已包含该文件，则保持当前列表顺序避免重置用户排序与重复磁盘 I/O)
const refreshPlaylist = async (videoPath: string) => {
  if (playlist.value.some((item) => isSamePath(item.path, videoPath))) {
    return
  }
  try {
    const list = await invoke<PlaylistItem[]>('scan_playlist', { path: videoPath })
    playlist.value = list || []
  } catch (e) {
    console.error('扫描播放列表失败:', e)
  }
}

// 启动播放视频
const startPlay = async (filePath: string, startTime?: number) => {
  currentVideoPath.value = filePath
  currentView.value = 'player'
  isPaused.value = false
  currentTime.value = startTime || 0
  duration.value = 0

  // 每次起播新视频时，初始化当前视频的临时配置为全局默认值 (继承模型)
  resetCurrentVideoConfig()

  // 启动播放与同目录扫描并行执行，消除大目录 I/O 带来的画面起播等待
  const playPromise = invoke('play_video', { path: filePath, startTime: startTime || 0 })
  const playlistPromise = refreshPlaylist(filePath)
  await Promise.all([playPromise, playlistPromise])

  // 显式确保前端与后端暂停状态完全对齐 (起播即播，杜绝画面不动却显示暂停符号)
  await invoke('set_pause', { paused: false }).catch(() => {})

  // 严格同步视频顶边距，杜绝非全屏窗口模式下画面被 40px TitleBar 遮挡
  syncVideoMargin()

  // 若为断点续播，触发提示
  if (startTime && startTime > 1) {
    setTimeout(() => {
      playerViewRef.value?.showOsd('已为您恢复上次播放进度', 800)
    }, 300)
  }

  // 设定默认音量
  await invoke('set_volume', { volume: isMuted.value ? 0 : volume.value })
}

// 记录当前播放进度至历史记录
const recordHistory = async () => {
  if (!currentVideoPath.value) return
  const parts = currentVideoPath.value.split(/[/\\]/)
  const name = parts[parts.length - 1] || currentVideoPath.value

  // 若进度已处于片尾 1.5 秒内，则断点时间归零，避免关闭窗口或返回主页时覆盖为片尾时间戳
  let ts = currentTime.value
  if (duration.value > 0 && ts >= duration.value - 1.5) {
    ts = 0
  }

  const record: HistoryItem = {
    name,
    path: currentVideoPath.value,
    timestamp: ts,
    duration: duration.value,
  }
  await invoke('save_history', { record: { ...record, last_played_at: Date.now() } })
}

// 删除单项播放历史记录
const handleDeleteHistoryItem = async (path: string) => {
  try {
    await invoke('delete_history_item', { path })
    await fetchHistory()
  } catch (e) {
    console.error('删除历史记录失败:', e)
  }
}

// 清空全部播放历史记录
const handleClearHistory = async () => {
  try {
    await invoke('clear_all_history')
    await fetchHistory()
  } catch (e) {
    console.error('清空历史记录失败:', e)
  }
}

// 主页态：选择本地视频
const handleSelectLocalVideo = async () => {
  try {
    const selected = await invoke<string | null>('open_file_dialog')
    if (selected) {
      await startPlay(selected)
    }
  } catch (e) {
    console.error('选择文件异常:', e)
  }
}

// 点击历史记录续播
const handleResumeHistory = async (item: HistoryItem) => {
  let resumeTime = item.timestamp
  // 若用户在全局偏好中关闭了自动断点续播，或历史进度已在视频最后 1.5 秒内，则从头开始播放
  if (!globalConfig.autoResume || (item.duration > 0 && resumeTime >= item.duration - 1.5)) {
    resumeTime = 0
  }
  await startPlay(item.path, resumeTime)
}

// 回到主界面
const handleGoHome = async () => {
  // 明确暂停播放并保存当前进度
  if (!isPaused.value) {
    isPaused.value = true
    await invoke('set_pause', { paused: true })
  }
  await recordHistory()
  await fetchHistory()
  currentView.value = 'home'
}

// 播放/暂停：乐观更新 (Optimistic UI 瞬时 0ms 感知，使用 set_pause 明确目标状态同步后端，杜绝交错反转)
const handleTogglePause = () => {
  const target = !isPaused.value
  isPaused.value = target
  invoke('set_pause', { paused: target }).catch((err) => {
    console.error('set_pause 失败:', err)
    isPaused.value = !target
  })
}

// 上一集 (人性化判定: >3s 重播当前集开头，<=3s 切上一个视频)
const handlePrevVideo = async () => {
  if (currentTime.value > 3.0) {
    await invoke('seek', { seconds: 0, relative: false })
    playerViewRef.value?.showOsd('重新播放本集', 800)
    return
  }

  if (playlist.value.length === 0) return
  const currIdx = playlist.value.findIndex((v) => isSamePath(v.path, currentVideoPath.value))
  if (currIdx > 0) {
    const prevItem = playlist.value[currIdx - 1]
    await recordHistory()
    await startPlay(prevItem.path)
  } else {
    playerViewRef.value?.showOsd('已经是第一个视频了', 800)
  }
}

// 下一集
const handleNextVideo = async () => {
  if (playlist.value.length === 0) return
  const currIdx = playlist.value.findIndex((v) => isSamePath(v.path, currentVideoPath.value))
  if (currIdx >= 0 && currIdx < playlist.value.length - 1) {
    const nextItem = playlist.value[currIdx + 1]
    await recordHistory()
    await startPlay(nextItem.path)
  } else {
    playerViewRef.value?.showOsd('已经是最后一个视频了', 800)
  }
}

// 从选集列表直达 (统一归一化比对)
const handleSelectVideoFromPlaylist = async (path: string) => {
  if (isSamePath(path, currentVideoPath.value)) return
  await recordHistory()
  await startPlay(path)
}

// 视频播完切集逻辑与单集循环重置 (带防重入互斥)
let isHandlingEnded = false

const handleVideoEnded = async () => {
  if (isHandlingEnded) return
  isHandlingEnded = true

  try {
    // 必须确保当前处于有效播放态且已获取总时长，杜绝加载过程中的虚假结束事件
    if (!currentVideoPath.value || duration.value <= 0) {
      return
    }

    // 播完重置历史时间戳为 0，避免下次续播时卡在视频最后一帧
    if (currentVideoPath.value) {
      const parts = currentVideoPath.value.split(/[/\\]/)
      const name = parts[parts.length - 1] || currentVideoPath.value
      await invoke('save_history', {
        record: {
          name,
          path: currentVideoPath.value,
          timestamp: 0,
          duration: duration.value,
          last_played_at: Date.now(),
        },
      })
    }

    if (playMode.value === 'single-loop') {
      // 单集循环：重回 0 秒并明确恢复播放状态
      await invoke('seek', { seconds: 0, relative: false })
      isPaused.value = false
      await invoke('set_pause', { paused: false })
      return
    }

    if (playMode.value === 'pause-on-end') {
      // 自动暂停在最后一帧
      isPaused.value = true
      await invoke('set_pause', { paused: true })
      return
    }

    if (playlist.value.length === 0) {
      isPaused.value = true
      await invoke('set_pause', { paused: true })
      return
    }

    const currIdx = playlist.value.findIndex((v) => isSamePath(v.path, currentVideoPath.value))

    if (currIdx >= 0 && currIdx < playlist.value.length - 1) {
      // 播放下一个
      const nextItem = playlist.value[currIdx + 1]
      await startPlay(nextItem.path)
    } else if (playMode.value === 'list-loop') {
      // 列表循环：回到第一个
      const firstItem = playlist.value[0]
      await startPlay(firstItem.path)
    } else {
      // 默认 auto-next 模式到达列表最后一集：停止播放并恢复播放图标
      isPaused.value = true
      await invoke('set_pause', { paused: true })
    }
  } finally {
    setTimeout(() => {
      isHandlingEnded = false
    }, 500)
  }
}

// 跳转 Seek (由 ControlBar 前端节流驱动拖拽，松手后精确即时 Seek)
const handleSeek = (seconds: number) => {
  currentTime.value = seconds
  invoke('seek', { seconds, relative: false }).catch(console.error)
}

// 更新倍速
const handleUpdateSpeed = async (s: number) => {
  speed.value = s
  await invoke('set_speed', { speed: s })
}

// 更新独立音量 (节流 60ms 杜绝 IPC 拥堵)
const throttledVolumeIpc = throttle((v: number) => {
  invoke('set_volume', { volume: v }).catch(console.error)
}, 60)

const handleUpdateVolume = (v: number) => {
  volume.value = v
  isMuted.value = false
  throttledVolumeIpc(v)
  playerViewRef.value?.showOsd(`音量: ${Math.round(v)}%`, 800)
}

// 静音切换
const handleToggleMute = async () => {
  if (isMuted.value) {
    isMuted.value = false
    volume.value = savedVolumeBeforeMute || 80
    await invoke('set_volume', { volume: volume.value })
    playerViewRef.value?.showOsd(`音量: ${Math.round(volume.value)}%`, 800)
  } else {
    savedVolumeBeforeMute = volume.value
    isMuted.value = true
    await invoke('set_volume', { volume: 0 })
    playerViewRef.value?.showOsd('静音', 800)
  }
}

// 画面比例 (GPU 硬件无缝缩放)
const handleUpdateAspectRatio = async (ratio: AspectRatio) => {
  aspectRatio.value = ratio
  await invoke('set_aspect_ratio', { ratio })
  const textMap: Record<AspectRatio, string> = {
    auto: '自动比例',
    '4:3': '4:3 比例',
    '16:9': '16:9 比例',
    fill: '画面铺满',
  }
  playerViewRef.value?.showOsd(`视频比例: ${textMap[ratio]}`, 1000)
}

// 水平镜像 (GPU 零开销着色器)
const handleToggleMirror = async () => {
  isMirrored.value = !isMirrored.value
  await invoke('set_mirror', { enabled: isMirrored.value })
  playerViewRef.value?.showOsd(`水平镜像: ${isMirrored.value ? '开启' : '关闭'}`, 1000)
}

// 顺时针旋转 (GPU 顶点变换)
const handleRotateClockwise = async () => {
  rotation.value = (rotation.value + 90) % 360
  await invoke('set_rotation', { degree: rotation.value })
  playerViewRef.value?.showOsd(`画面旋转: ${rotation.value}°`, 1000)
}

// 解码策略热生效：在当前时间戳自动无缝热重载当前流
const handleUpdateHwdec = async (mode: HwdecMode) => {
  hwdecMode.value = mode
  await invoke('set_hwdec', { mode })
  if (currentVideoPath.value) {
    const resumeTime = currentTime.value
    await invoke('play_video', {
      path: currentVideoPath.value,
      startTime: resumeTime,
    })
  }
  const textMap: Record<HwdecMode, string> = {
    'auto-safe': '推荐硬解 (已生效)',
    'auto-copy': '兼容硬解 (已生效)',
    no: '兼容软解 (已生效)',
  }
  playerViewRef.value?.showOsd(textMap[mode], 1000)
}

// 窗口控制
const handleTogglePin = async () => {
  isPinned.value = !isPinned.value
  await invoke('set_pinned', { pinned: isPinned.value })
  playerViewRef.value?.showOsd(isPinned.value ? '窗口置顶: 开启' : '窗口置顶: 关闭', 800)
}

const handleMinimize = async () => {
  await invoke('minimize_window')
}

const handleToggleMaximize = async () => {
  const isMax = await invoke<boolean>('toggle_maximize')
  isMaximized.value = isMax
  syncVideoMargin()
  setTimeout(syncVideoMargin, 150)
}

// 原生窗口级全屏切换 (与 window.set_fullscreen 和 DOM fullscreenchange 双向严格同步)
const handleToggleFullscreen = async () => {
  const nextState = !isFullscreen.value
  isFullscreen.value = nextState
  syncVideoMargin()
  setTimeout(syncVideoMargin, 150)
  try {
    await invoke('set_fullscreen', { fullscreen: nextState })
  } catch (err) {
    console.error('set_fullscreen 失败:', err)
  }
  if (nextState) {
    if (!document.fullscreenElement) {
      document.documentElement.requestFullscreen().catch(() => {})
    }
  } else {
    if (document.fullscreenElement) {
      document.exitFullscreen().catch(() => {})
    }
  }
}

const handleClose = async () => {
  await recordHistory()
  await invoke('close_window')
}

// 全局键盘状态机绑定 (仅在播放态响应媒体播放控制快捷键)
useKeyboard({
  isEnabled: () => currentView.value === 'player',
  seekRelative: async (delta: number) => {
    await invoke('seek', { seconds: delta, relative: true })
  },
  togglePause: handleTogglePause,
  changeVolume: (delta: number) => {
    const nextVol = Math.min(100, Math.max(0, volume.value + delta))
    handleUpdateVolume(nextVol)
  },
  setTempSpeed: async (s: number | null) => {
    if (s !== null) {
      savedSpeedBeforeLongPress = speed.value
      await invoke('set_speed', { speed: s })
    } else {
      await invoke('set_speed', { speed: savedSpeedBeforeLongPress })
    }
  },
  exitFullscreen: () => {
    if (isFullscreen.value) {
      handleToggleFullscreen()
    }
  },
  isFullscreen: () => isFullscreen.value,
  showOsd: (text: string, duration?: number) => {
    playerViewRef.value?.showOsd(text, duration)
  },
  hideOsd: () => {
    playerViewRef.value?.hideOsd()
  },
})
</script>

<style scoped>
.app-root {
  width: 100vw;
  height: 100vh;
  position: relative;
  overflow: hidden;
}

.app-root.home {
  background-color: var(--bg-home);
}

.app-root.player {
  background-color: transparent;
}
</style>