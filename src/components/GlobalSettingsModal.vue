<template>
  <transition name="modal-fade">
    <div v-if="visible" class="modal-backdrop" @click="emit('close')">
      <div class="settings-modal" @click.stop>
        <!-- 头部与 Tab 导航 -->
        <div class="modal-header">
          <div class="header-left">
            <SvgIcon name="settings" size="18" />
            <span class="header-title">系统偏好设置</span>
          </div>

          <!-- 导航选项卡 -->
          <div class="nav-tabs">
            <button
              class="tab-btn"
              :class="{ active: currentTab === 'playback' }"
              @click="currentTab = 'playback'"
            >
              播放偏好
            </button>
            <button
              class="tab-btn"
              :class="{ active: currentTab === 'association' }"
              @click="handleSwitchToAssociation"
            >
              文件关联
            </button>
            <button
              class="tab-btn"
              :class="{ active: currentTab === 'about' }"
              @click="currentTab = 'about'"
            >
              关于与更新
            </button>
          </div>

          <button class="close-btn" @click="emit('close')" title="关闭 (Esc)">
            <SvgIcon name="close" size="14" />
          </button>
        </div>

        <!-- 内容区域 -->
        <div class="modal-body custom-scrollbar">
          <!-- ================= TAB 1: 播放偏好 ================= -->
          <div v-if="currentTab === 'playback'" class="tab-pane">
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

          <!-- ================= TAB 2: 文件关联 ================= -->
          <div v-else-if="currentTab === 'association'" class="tab-pane">
            <!-- 友好成功气泡 / 提示条 -->
            <transition name="toast-slide">
              <div v-if="toastVisible" class="assoc-toast">
                <SvgIcon name="check" size="14" />
                <span>{{ toastMessage }}</span>
              </div>
            </transition>

            <!-- 顶栏：说明与快捷操作工具栏 -->
            <div class="assoc-top-card">
              <div class="assoc-intro">
                <span class="assoc-intro-title">媒体文件关联管理</span>
                <span class="assoc-intro-desc">
                  自由勾选需由 Pulse Player 关联的媒体格式。点击【立即应用】将批量写入 Windows 规范应用程序能力模型及打开方式推荐列表。
                </span>
              </div>
              <div class="assoc-actions-bar">
                <div class="btn-group-left">
                  <button class="action-btn secondary" @click="handleSelectAll" title="勾选全部媒体格式">
                    全选
                  </button>
                  <button class="action-btn secondary" @click="handleDeselectAll" title="清空全部勾选">
                    全不选
                  </button>
                  <button class="action-btn secondary" @click="handleSelectRecommended" title="勾选常用视频格式与主流无损音频">
                    推荐选择
                  </button>
                  <button class="action-btn secondary" @click="handleOpenSystemSettings" title="打开 Windows 系统设置中的默认应用面板">
                    <SvgIcon name="external-link" size="12" />
                    <span>系统设置</span>
                  </button>
                </div>
                <div class="btn-group-right">
                  <button
                    class="action-btn primary apply-btn"
                    :disabled="isApplyingAssoc"
                    @click="handleApplyAssociations"
                  >
                    <SvgIcon v-if="isApplyingAssoc" name="refresh" size="13" class="spin-icon" />
                    <SvgIcon v-else name="check" size="13" />
                    <span>{{ isApplyingAssoc ? '正在应用...' : '立即应用' }}</span>
                  </button>
                </div>
              </div>
            </div>

            <!-- 视频格式列表 -->
            <div class="assoc-group">
              <div class="group-title-row">
                <span class="group-title">
                  视频格式
                  <span class="count-badge">({{ selectedVideoCount }} / {{ videoAssocList.length }})</span>
                </span>
                <span class="group-badge">支持原生 GPU 硬件直解加速</span>
              </div>
              <div class="ext-grid">
                <div
                  v-for="item in videoAssocList"
                  :key="item.ext"
                  class="ext-card"
                  :class="{
                    'is-checked': selectedExts.has(item.ext),
                    'is-default': item.is_default,
                  }"
                  @click="toggleExt(item.ext)"
                >
                  <div class="card-left">
                    <label class="custom-checkbox" @click.stop>
                      <input
                        type="checkbox"
                        :checked="selectedExts.has(item.ext)"
                        @change="toggleExt(item.ext)"
                      />
                      <span class="checkmark"></span>
                    </label>
                    <span class="ext-name">.{{ item.ext.toUpperCase() }}</span>
                  </div>

                  <div class="card-right">
                    <span
                      class="ext-status"
                      :class="{ 'is-pulse': item.is_default }"
                      :title="item.is_default ? '已是系统默认播放器' : `当前系统默认: ${item.current_app} (点击呼出设置向导)`"
                      @click="handlePromptAssociate(item.ext, $event)"
                    >
                      {{ item.is_default ? '默认' : item.current_app }}
                    </span>
                  </div>
                </div>
              </div>
            </div>

            <!-- 音频格式列表 -->
            <div class="assoc-group">
              <div class="group-title-row">
                <span class="group-title">
                  音频格式
                  <span class="count-badge">({{ selectedAudioCount }} / {{ audioAssocList.length }})</span>
                </span>
                <span class="group-badge">高保真无损音质原生解码</span>
              </div>
              <div class="ext-grid">
                <div
                  v-for="item in audioAssocList"
                  :key="item.ext"
                  class="ext-card"
                  :class="{
                    'is-checked': selectedExts.has(item.ext),
                    'is-default': item.is_default,
                  }"
                  @click="toggleExt(item.ext)"
                >
                  <div class="card-left">
                    <label class="custom-checkbox" @click.stop>
                      <input
                        type="checkbox"
                        :checked="selectedExts.has(item.ext)"
                        @change="toggleExt(item.ext)"
                      />
                      <span class="checkmark"></span>
                    </label>
                    <span class="ext-name">.{{ item.ext.toUpperCase() }}</span>
                  </div>

                  <div class="card-right">
                    <span
                      class="ext-status"
                      :class="{ 'is-pulse': item.is_default }"
                      :title="item.is_default ? '已是系统默认播放器' : `当前系统默认: ${item.current_app} (点击呼出设置向导)`"
                      @click="handlePromptAssociate(item.ext, $event)"
                    >
                      {{ item.is_default ? '默认' : item.current_app }}
                    </span>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <!-- ================= TAB 3: 关于与更新 ================= -->
          <div v-else-if="currentTab === 'about'" class="tab-pane about-pane">
            <div class="about-hero">
              <div class="about-logo-box">
                <SvgIcon name="pulse-logo" size="58" />
              </div>
              <div class="about-title">Pulse Player</div>
              <div class="about-slogan">极简 · 原生硬件直解 · 离线媒体播放器</div>
              <div class="about-version-badge">版本 {{ currentAppVersion }}</div>
            </div>

            <!-- 更新检测交互区 -->
            <div class="update-check-card">
              <div class="update-info-col">
                <span class="update-card-title">软件版本更新</span>
                <span class="update-card-status">{{ updateStatusText }}</span>
              </div>

              <div class="update-actions">
                <button
                  class="pill-btn check-update-btn"
                  :disabled="isCheckingUpdate"
                  @click="handleCheckUpdate"
                >
                  <SvgIcon v-if="isCheckingUpdate" name="refresh" size="12" class="spin-icon" />
                  <span>{{ isCheckingUpdate ? '正在检查...' : '检查更新' }}</span>
                </button>
              </div>
            </div>

            <!-- 发现新版本时展示的更新卡片 -->
            <div v-if="updateInfo && updateInfo.hasUpdate" class="new-version-box">
              <div class="new-version-header">
                <span class="new-version-tag">发现新版本</span>
                <span class="new-version-name">{{ updateInfo.latestVersion }}</span>
              </div>
              <div v-if="updateInfo.releaseNotes" class="new-version-notes">
                {{ updateInfo.releaseNotes }}
              </div>
              <div class="new-version-footer">
                <button class="action-btn primary" @click="handleGoDownload(updateInfo.releaseUrl)">
                  <SvgIcon name="external-link" size="13" />
                  <span>前往下载最新安装包</span>
                </button>
                <button class="action-btn secondary" @click="updateInfo = null">
                  暂不更新
                </button>
              </div>
            </div>

            <!-- 关于软件说明与开源协议 -->
            <div class="about-footer-info">
              <p>采用 Tauri 2.0 (Rust) + libmpv 原生硬件直解内核 + Vue 3 构建</p>
              <button class="github-link-btn" @click="handleOpenGithub">
                <SvgIcon name="external-link" size="12" />
                <span>访问官方开源项目主页 (GitHub)</span>
              </button>
            </div>
          </div>
        </div>

        <!-- 底部说明与完成按钮 -->
        <div class="modal-footer">
          <span class="footer-tip">
            {{ currentTab === 'playback' ? '全局配置修改即时保存' : currentTab === 'association' ? '关联状态以当前 Windows 注册表为准' : 'Pulse Player 遵循 KISS 原则与硬件直解哲学' }}
          </span>
          <button class="done-btn" @click="emit('close')">完成</button>
        </div>
      </div>
    </div>
  </transition>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import SvgIcon from './SvgIcon.vue'
import { useConfig, GlobalConfig } from '../composables/useConfig'

export interface FileAssociationItem {
  ext: string
  is_video: boolean
  is_default: boolean
  current_app: string
  is_registered?: boolean
}

interface UpdateCheckResult {
  hasUpdate: boolean
  latestVersion: string
  releaseNotes: string
  releaseUrl: string
}

const props = defineProps<{
  visible: boolean
}>()

const emit = defineEmits<{
  (e: 'close'): void
}>()

const { globalConfig, saveGlobalConfig } = useConfig()

// Tab 状态: 'playback' | 'association' | 'about'
const currentTab = ref<'playback' | 'association' | 'about'>('playback')

// 播放偏好选项
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

// =================== 文件关联逻辑 (对标 Bandizip 体验) ===================
const assocList = ref<FileAssociationItem[]>([])
const selectedExts = ref<Set<string>>(new Set())
const isLoadingAssoc = ref(false)
const isApplyingAssoc = ref(false)
const toastMessage = ref('')
const toastVisible = ref(false)
let toastTimer: ReturnType<typeof setTimeout> | null = null

const showToast = (msg: string) => {
  if (toastTimer) clearTimeout(toastTimer)
  toastMessage.value = msg
  toastVisible.value = true
  toastTimer = setTimeout(() => {
    toastVisible.value = false
  }, 3200)
}

const videoAssocList = computed(() => assocList.value.filter((i) => i.is_video))
const audioAssocList = computed(() => assocList.value.filter((i) => !i.is_video))

const selectedVideoCount = computed(() =>
  videoAssocList.value.filter((i) => selectedExts.value.has(i.ext)).length
)
const selectedAudioCount = computed(() =>
  audioAssocList.value.filter((i) => selectedExts.value.has(i.ext)).length
)

// 默认推荐勾选的媒体扩展名（覆盖全量常用视频与主流高保真音频）
const RECOMMENDED_EXTS = [
  'mp4', 'mkv', 'avi', 'flv', 'mov', 'wmv', 'webm', 'ts', 'm4v', 'rmvb',
  'mp3', 'flac', 'wav', 'm4a', 'aac'
]

const toggleExt = (ext: string) => {
  const newSet = new Set(selectedExts.value)
  if (newSet.has(ext)) {
    newSet.delete(ext)
  } else {
    newSet.add(ext)
  }
  selectedExts.value = newSet
}

const handleSelectAll = () => {
  selectedExts.value = new Set(assocList.value.map((i) => i.ext))
}

const handleDeselectAll = () => {
  selectedExts.value = new Set()
}

const handleSelectRecommended = () => {
  selectedExts.value = new Set(RECOMMENDED_EXTS)
}

const fetchAssociations = async () => {
  isLoadingAssoc.value = true
  try {
    const res = await invoke<FileAssociationItem[]>('get_file_associations')
    assocList.value = res || []

    // 初始化勾选状态：优先同步系统注册表已登记关联的格式；若初次运行无任何登记，默认载入推荐格式
    if (selectedExts.value.size === 0) {
      const registered = (res || []).filter((i) => i.is_registered || i.is_default).map((i) => i.ext)
      if (registered.length > 0) {
        selectedExts.value = new Set(registered)
      } else {
        selectedExts.value = new Set(RECOMMENDED_EXTS)
      }
    }
  } catch (err) {
    console.error('获取文件关联状态失败:', err)
  } finally {
    isLoadingAssoc.value = false
  }
}

const handleApplyAssociations = async () => {
  if (isApplyingAssoc.value) return
  isApplyingAssoc.value = true
  try {
    await invoke('apply_file_associations', {
      selectedExts: Array.from(selectedExts.value),
    })
    showToast('✓ 文件关联已成功应用！已写入系统注册表并通知 Windows 外壳刷新。')
    await fetchAssociations()
  } catch (err) {
    console.error('应用文件关联失败:', err)
    showToast('✕ 应用文件关联失败: ' + String(err))
  } finally {
    isApplyingAssoc.value = false
  }
}

const handleSwitchToAssociation = () => {
  currentTab.value = 'association'
  fetchAssociations()
}

const handleOpenSystemSettings = async () => {
  try {
    await invoke('open_default_apps_settings')
  } catch (err) {
    console.error('打开系统设置失败:', err)
  }
}

const handlePromptAssociate = async (ext: string, event?: Event) => {
  if (event) event.stopPropagation()
  try {
    await invoke('prompt_associate_extension', { ext })
  } catch (err) {
    console.error('唤起打开方式弹窗失败:', err)
  }
}

// =================== 检查更新逻辑 ===================
const currentAppVersion = 'v0.2.0'
const isCheckingUpdate = ref(false)
const updateStatusText = ref('点击右侧按钮手动连接官方清单检查')
const updateInfo = ref<UpdateCheckResult | null>(null)

// 版本号字符串比对 (如 'v0.1.0' 与 'v0.2.0')
const compareSemver = (v1: string, v2: string): number => {
  const parse = (s: string) =>
    s.replace(/^v/i, '').split('.').map((n) => parseInt(n, 10) || 0)
  const [a1, b1, c1] = parse(v1)
  const [a2, b2, c2] = parse(v2)
  if (a1 !== a2) return a1 - a2
  if (b1 !== b2) return b1 - b2
  return c1 - c2
}

const handleCheckUpdate = async () => {
  if (isCheckingUpdate.value) return
  isCheckingUpdate.value = true
  updateStatusText.value = '正在连接 GitHub 获取最新版本信息...'
  updateInfo.value = null

  try {
    const res = await fetch('https://api.github.com/repos/gavincrossv1/Pulse-Player/releases/latest', {
      headers: {
        Accept: 'application/vnd.github.v3+json',
      },
    })

    if (!res.ok) {
      throw new Error(`HTTP ${res.status}`)
    }

    const data = await res.json()
    const latestTag = data.tag_name || ''
    const releaseUrl = data.html_url || 'https://github.com/gavincrossv1/Pulse-Player/releases'
    const releaseNotes = (data.body || '').trim()

    if (compareSemver(latestTag, currentAppVersion) > 0) {
      // 存在更新
      updateStatusText.value = `已检测到新版本: ${latestTag}`
      updateInfo.value = {
        hasUpdate: true,
        latestVersion: latestTag,
        releaseNotes,
        releaseUrl,
      }
    } else {
      updateStatusText.value = `当前已是最新版本 (${currentAppVersion})，无需更新`
    }
  } catch (err) {
    console.error('检查更新失败:', err)
    updateStatusText.value = '检查更新失败，请检查网络连接后重试'
  } finally {
    isCheckingUpdate.value = false
  }
}

const handleGoDownload = async (url: string) => {
  try {
    await invoke('open_external_url', { url })
  } catch (err) {
    console.error('打开更新页面异常:', err)
  }
}

const handleOpenGithub = async () => {
  try {
    await invoke('open_external_url', { url: 'https://github.com/gavincrossv1/Pulse-Player' })
  } catch (err) {
    console.error('打开 GitHub 异常:', err)
  }
}

// 弹窗显隐与键盘按键
const handleKeyDown = (e: KeyboardEvent) => {
  if (e.key === 'Escape' && props.visible) {
    emit('close')
  }
}

watch(
  () => props.visible,
  (val) => {
    if (val && currentTab.value === 'association') {
      fetchAssociations()
    }
  }
)

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
  width: 580px;
  max-width: 92vw;
  max-height: 85vh;
  background: rgba(26, 27, 30, 0.98);
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 12px;
  box-shadow: 0 16px 40px rgba(0, 0, 0, 0.65);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.modal-header {
  height: 52px;
  padding: 0 16px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  background: rgba(255, 255, 255, 0.02);
  flex-shrink: 0;
  gap: 12px;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 8px;
  color: var(--text-primary);
  flex-shrink: 0;
}

.header-title {
  font-size: 14.5px;
  font-weight: 600;
}

.nav-tabs {
  display: flex;
  align-items: center;
  gap: 4px;
  background: rgba(255, 255, 255, 0.04);
  padding: 3px;
  border-radius: 8px;
}

.tab-btn {
  padding: 4px 12px;
  font-size: 12px;
  font-weight: 500;
  border-radius: 6px;
  color: rgba(255, 255, 255, 0.7);
  background: transparent;
  transition: all 0.15s ease;
}

.tab-btn:hover {
  color: #FFFFFF;
}

.tab-btn.active {
  background: rgba(255, 255, 255, 0.12);
  color: #FFFFFF;
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.3);
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
  flex-shrink: 0;
}

.close-btn:hover {
  background: rgba(255, 255, 255, 0.1);
  color: #FFFFFF;
}

.modal-body {
  padding: 16px 20px;
  display: flex;
  flex-direction: column;
  overflow-y: auto;
  flex: 1;
}

.tab-pane {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

/* 播放偏好项 */
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
  display: flex;
  align-items: center;
  gap: 5px;
}

.pill-btn:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.12);
  color: #FFFFFF;
}

.pill-btn.active {
  background: var(--pulse-pink);
  color: #FFFFFF;
  box-shadow: 0 2px 8px rgba(251, 114, 153, 0.35);
}

/* 文件关联专区 (对标 Bandizip 体验) */
.assoc-toast {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 14px;
  background: rgba(46, 204, 113, 0.15);
  border: 1px solid rgba(46, 204, 113, 0.35);
  color: #2ECC71;
  border-radius: 8px;
  font-size: 12px;
  font-weight: 500;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.25);
}

.toast-slide-enter-active,
.toast-slide-leave-active {
  transition: all 0.25s ease;
}

.toast-slide-enter-from,
.toast-slide-leave-to {
  opacity: 0;
  transform: translateY(-8px);
}

.assoc-top-card {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 12px 14px;
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid rgba(255, 255, 255, 0.07);
  border-radius: 8px;
}

.assoc-intro {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.assoc-intro-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
}

.assoc-intro-desc {
  font-size: 11.5px;
  color: var(--text-muted);
  line-height: 1.5;
}

.assoc-actions-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-wrap: wrap;
  gap: 8px;
  padding-top: 4px;
  border-top: 1px solid rgba(255, 255, 255, 0.05);
}

.btn-group-left {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-wrap: wrap;
}

.btn-group-right {
  display: flex;
  align-items: center;
}

.apply-btn {
  padding: 6px 16px;
  font-weight: 600;
  letter-spacing: 0.3px;
  box-shadow: 0 2px 10px rgba(251, 114, 153, 0.3);
}

.action-btn {
  padding: 5px 12px;
  font-size: 11.5px;
  font-weight: 500;
  border-radius: 6px;
  display: flex;
  align-items: center;
  gap: 6px;
  transition: all 0.15s ease;
  cursor: pointer;
}

.action-btn.primary {
  background: var(--pulse-pink);
  color: #FFFFFF;
}

.action-btn.primary:hover:not(:disabled) {
  background: var(--pulse-pink-hover);
}

.action-btn.secondary {
  background: rgba(255, 255, 255, 0.07);
  color: var(--text-primary);
  border: 1px solid rgba(255, 255, 255, 0.06);
}

.action-btn.secondary:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.13);
}

.assoc-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.group-title-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.group-title {
  font-size: 12.5px;
  font-weight: 600;
  color: var(--text-primary);
  display: flex;
  align-items: center;
}

.count-badge {
  font-size: 11.5px;
  font-weight: 500;
  color: var(--pulse-pink);
  margin-left: 4px;
}

.group-badge {
  font-size: 11px;
  color: var(--text-muted);
}

.ext-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
  gap: 8px;
}

.ext-card {
  padding: 8px 10px;
  background: rgba(255, 255, 255, 0.035);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 6px;
  transition: all 0.15s ease;
  cursor: pointer;
  user-select: none;
}

.ext-card:hover {
  background: rgba(255, 255, 255, 0.07);
  border-color: rgba(255, 255, 255, 0.18);
  transform: translateY(-1px);
}

.ext-card.is-checked {
  border-color: rgba(251, 114, 153, 0.35);
  background: rgba(251, 114, 153, 0.06);
}

.ext-card.is-checked:hover {
  border-color: rgba(251, 114, 153, 0.55);
  background: rgba(251, 114, 153, 0.1);
}

.card-left {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
}

.ext-name {
  font-size: 12px;
  font-weight: 700;
  color: var(--text-primary);
  letter-spacing: 0.5px;
  white-space: nowrap;
}

.card-right {
  display: flex;
  align-items: center;
  min-width: 0;
}

.ext-status {
  font-size: 10px;
  color: var(--text-muted);
  background: rgba(255, 255, 255, 0.05);
  padding: 2px 6px;
  border-radius: 4px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 75px;
  transition: all 0.15s ease;
}

.ext-status:hover {
  background: rgba(255, 255, 255, 0.1);
  color: var(--text-primary);
}

.ext-status.is-pulse {
  color: #2ECC71;
  background: rgba(46, 204, 113, 0.12);
  border: 1px solid rgba(46, 204, 113, 0.25);
  font-weight: 500;
}

/* 自定义复选框样式 */
.custom-checkbox {
  position: relative;
  width: 15px;
  height: 15px;
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  user-select: none;
  flex-shrink: 0;
}

.custom-checkbox input {
  opacity: 0;
  width: 0;
  height: 0;
  position: absolute;
}

.checkmark {
  position: absolute;
  top: 0;
  left: 0;
  height: 15px;
  width: 15px;
  background-color: rgba(255, 255, 255, 0.08);
  border: 1px solid rgba(255, 255, 255, 0.25);
  border-radius: 4px;
  transition: all 0.15s ease;
}

.custom-checkbox:hover input ~ .checkmark {
  border-color: rgba(255, 255, 255, 0.5);
  background-color: rgba(255, 255, 255, 0.12);
}

.custom-checkbox input:checked ~ .checkmark {
  background-color: var(--pulse-pink);
  border-color: var(--pulse-pink);
}

.checkmark:after {
  content: "";
  position: absolute;
  display: none;
  left: 4.5px;
  top: 1.5px;
  width: 4px;
  height: 8px;
  border: solid white;
  border-width: 0 2px 2px 0;
  transform: rotate(45deg);
}

.custom-checkbox input:checked ~ .checkmark:after {
  display: block;
}

/* 关于与更新页面 */
.about-pane {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
  padding: 8px 0;
}

.about-hero {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
}

.about-logo-box {
  width: 72px;
  height: 72px;
  border-radius: 18px;
  background: radial-gradient(circle, rgba(255, 46, 136, 0.2) 0%, rgba(20, 21, 24, 0.8) 100%);
  border: 1px solid rgba(255, 46, 136, 0.3);
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: 0 8px 24px rgba(255, 46, 136, 0.2);
}

.about-title {
  font-size: 18px;
  font-weight: 700;
  color: #FFFFFF;
  margin-top: 4px;
}

.about-slogan {
  font-size: 12px;
  color: var(--text-secondary);
}

.about-version-badge {
  font-size: 11px;
  font-weight: 600;
  color: var(--pulse-pink);
  background: rgba(255, 46, 136, 0.12);
  padding: 2px 8px;
  border-radius: 10px;
  border: 1px solid rgba(255, 46, 136, 0.2);
}

.update-check-card {
  width: 100%;
  padding: 12px 16px;
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.update-info-col {
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.update-card-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
}

.update-card-status {
  font-size: 11.5px;
  color: var(--text-muted);
}

.check-update-btn {
  padding: 6px 14px;
  font-size: 12px;
}

.new-version-box {
  width: 100%;
  padding: 12px 14px;
  background: rgba(255, 46, 136, 0.06);
  border: 1px solid rgba(255, 46, 136, 0.3);
  border-radius: 8px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.new-version-header {
  display: flex;
  align-items: center;
  gap: 8px;
}

.new-version-tag {
  font-size: 11px;
  font-weight: 700;
  padding: 1px 6px;
  border-radius: 4px;
  background: var(--pulse-pink);
  color: #FFFFFF;
}

.new-version-name {
  font-size: 13.5px;
  font-weight: 600;
  color: #FFFFFF;
}

.new-version-notes {
  font-size: 11.5px;
  color: rgba(255, 255, 255, 0.75);
  line-height: 1.5;
  white-space: pre-wrap;
  max-height: 100px;
  overflow-y: auto;
  background: rgba(0, 0, 0, 0.2);
  padding: 8px;
  border-radius: 4px;
}

.new-version-footer {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 4px;
}

.about-footer-info {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  font-size: 11px;
  color: var(--text-muted);
  margin-top: 4px;
}

.github-link-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 11.5px;
  color: var(--pulse-pink);
  background: transparent;
  transition: opacity 0.15s ease;
}

.github-link-btn:hover {
  opacity: 0.8;
  text-decoration: underline;
}

.spin-icon {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

.modal-footer {
  height: 52px;
  padding: 0 20px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  border-top: 1px solid rgba(255, 255, 255, 0.08);
  background: rgba(0, 0, 0, 0.2);
  flex-shrink: 0;
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
