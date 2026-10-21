// src/composables/useUpdater.ts
// Pulse Player 原生静默更新 (Tauri Updater) 状态机与资源生命周期管理
import { ref } from 'vue'
import { check, type Update } from '@tauri-apps/plugin-updater'
import { relaunch } from '@tauri-apps/plugin-process'

export interface UpdateInfo {
  version: string
  currentVersion: string
  date?: string
  body?: string
}

export type DownloadState = 'idle' | 'downloading' | 'ready' | 'error'

export function useUpdater() {
  const currentAppVersion = 'v0.3.1'
  const isCheckingUpdate = ref(false)
  const updateStatusText = ref('点击右侧按钮连接官方服务器检查更新')
  const updateInfo = ref<UpdateInfo | null>(null)

  // 下载与安装阶段状态: 'idle' | 'downloading' | 'ready' | 'error'
  const downloadState = ref<DownloadState>('idle')
  const downloadedBytes = ref(0)
  const totalBytes = ref(0)
  const downloadPercent = ref(0)
  const downloadErrorMsg = ref('')
  const isRestarting = ref(false)

  // 关键架构机制：将 Rust/libmpv/Tauri 原生 FFI 资源对象严格隔离在 Vue 响应式 Proxy 之外。
  // Tauri 的 Update / Resource 类内部采用原生私有字段及 WeakMap 映射，
  // 一旦进入 Vue 的深层 Proxy 包装，this 上下文会退化为 Proxy 导致
  // "Cannot read private member from an object whose class did not declare it" 运行时崩溃。
  // 保持 activeUpdate 为纯 JS 内部引用，仅向 UI 暴露纯数据快照 (UpdateInfo)。
  let activeUpdate: Update | null = null

  const formatBytes = (bytes: number): string => {
    if (!bytes || bytes <= 0) return '0 B'
    const units = ['B', 'KB', 'MB', 'GB']
    const i = Math.floor(Math.log(bytes) / Math.log(1024))
    const val = (bytes / Math.pow(1024, i)).toFixed(1)
    return `${val} ${units[i]}`
  }

  const formatDate = (dateStr?: string): string => {
    if (!dateStr) return ''
    try {
      const d = new Date(dateStr)
      return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`
    } catch {
      return dateStr
    }
  }

  // 安全释放底层 SDK 资源句柄
  const cleanupActiveUpdate = async () => {
    if (activeUpdate) {
      try {
        await activeUpdate.close()
      } catch (_) {}
      activeUpdate = null
    }
  }

  // 检查更新
  const handleCheckUpdate = async () => {
    if (isCheckingUpdate.value) return
    isCheckingUpdate.value = true
    updateStatusText.value = '正在连接官方更新服务器检测最新版本...'

    await cleanupActiveUpdate()
    updateInfo.value = null
    downloadState.value = 'idle'
    downloadedBytes.value = 0
    totalBytes.value = 0
    downloadPercent.value = 0
    downloadErrorMsg.value = ''

    try {
      const update = await check()
      if (update) {
        activeUpdate = update
        updateInfo.value = {
          version: update.version,
          currentVersion: update.currentVersion,
          date: update.date,
          body: update.body,
        }
        updateStatusText.value = `已检测到新版本: v${update.version}`
      } else {
        updateStatusText.value = `当前已是最新版本 (${currentAppVersion})，无需更新`
      }
    } catch (err: unknown) {
      console.error('检查更新失败:', err)
      const errText = (err as Error)?.message || String(err)
      if (errText.includes('Could not fetch a valid release') || errText.includes('404')) {
        updateStatusText.value = `当前已是最新版本 (${currentAppVersion})，官方暂无新版本发布`
      } else {
        updateStatusText.value = `检查更新失败: ${errText}`
      }
    } finally {
      isCheckingUpdate.value = false
    }
  }

  // 开始下载更新并校验数字签名
  const handleStartDownload = async () => {
    if (!activeUpdate || downloadState.value === 'downloading') return
    downloadState.value = 'downloading'
    downloadedBytes.value = 0
    totalBytes.value = 0
    downloadPercent.value = 0
    downloadErrorMsg.value = ''

    try {
      let accumulated = 0
      await activeUpdate.download((event) => {
        if (event.event === 'Started') {
          totalBytes.value = event.data.contentLength ?? 0
        } else if (event.event === 'Progress') {
          accumulated += event.data.chunkLength
          downloadedBytes.value = accumulated
          if (totalBytes.value > 0) {
            downloadPercent.value = Math.min(
              100,
              Math.round((accumulated / totalBytes.value) * 100)
            )
          }
        } else if (event.event === 'Finished') {
          downloadPercent.value = 100
        }
      })

      // 下载完成并通过 Minisign 数字签名校验
      downloadState.value = 'ready'
    } catch (err: unknown) {
      console.error('下载或验证更新包失败:', err)
      downloadState.value = 'error'
      downloadErrorMsg.value = `下载更新失败: ${(err as Error)?.message || String(err)}`
    }
  }

  // 重启并安装
  const handleRestartAndInstall = async () => {
    if (!activeUpdate || isRestarting.value) return
    isRestarting.value = true
    try {
      // Windows 下调用 install 会启动静默安装程序并退出主进程释放文件锁
      await activeUpdate.install({ restartAfterInstall: true })
      // 降级兜底：若进程未退出则主动调用 relaunch
      await relaunch()
    } catch (err: unknown) {
      console.error('拉起安装更新异常:', err)
      downloadState.value = 'error'
      downloadErrorMsg.value = `拉起安装程序失败: ${(err as Error)?.message || String(err)}`
      isRestarting.value = false
    }
  }

  // 取消/忽略本次更新
  const handleDismissUpdate = async () => {
    await cleanupActiveUpdate()
    updateInfo.value = null
    downloadState.value = 'idle'
  }

  // 组件卸载时释放
  const cleanupUpdater = async () => {
    if (activeUpdate && downloadState.value !== 'downloading') {
      await cleanupActiveUpdate()
    }
  }

  return {
    currentAppVersion,
    isCheckingUpdate,
    updateStatusText,
    updateInfo,
    downloadState,
    downloadedBytes,
    totalBytes,
    downloadPercent,
    downloadErrorMsg,
    isRestarting,
    formatBytes,
    formatDate,
    handleCheckUpdate,
    handleStartDownload,
    handleRestartAndInstall,
    handleDismissUpdate,
    cleanupUpdater,
  }
}
