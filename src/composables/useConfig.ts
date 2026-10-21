// src/composables/useConfig.ts
// Pulse Player 全局偏好配置与当前视频临时微调状态机 (继承与覆盖模型)
import { reactive } from 'vue'

export interface GlobalConfig {
  seekStep: number // 左右方向键快进/快退步长 (秒): 3, 5, 10, 15, 30
  longPressSpeed: number // 长按快进倍速: 1.5, 2.0, 2.5, 3.0, 4.0
  wheelVolumeStep: number // 滚轮音量步长 (%): 2, 5, 10
  autoHideDelay: number // 底栏静止淡出时间 (秒): 1.5, 2.5, 4.0, 0 (0 为从不隐藏)
  autoResume: boolean // 断点记忆自动续播: true / false
}

const STORAGE_KEY = 'pulse_player_global_config_v1'
const LEGACY_STORAGE_KEY = 'bi_player_global_config_v1'

const DEFAULT_GLOBAL_CONFIG: GlobalConfig = {
  seekStep: 5,
  longPressSpeed: 3.0,
  wheelVolumeStep: 5,
  autoHideDelay: 2.5,
  autoResume: true,
}

// 1. 全局持久配置 (存储于 localStorage)
export const globalConfig = reactive<GlobalConfig>({ ...DEFAULT_GLOBAL_CONFIG })

export function loadGlobalConfig(): GlobalConfig {
  try {
    const raw = localStorage.getItem(STORAGE_KEY) || localStorage.getItem(LEGACY_STORAGE_KEY)
    if (raw) {
      const parsed = JSON.parse(raw)
      Object.assign(globalConfig, { ...DEFAULT_GLOBAL_CONFIG, ...parsed })
      if (localStorage.getItem(LEGACY_STORAGE_KEY)) {
        localStorage.removeItem(LEGACY_STORAGE_KEY)
        localStorage.setItem(STORAGE_KEY, JSON.stringify(globalConfig))
      }
    }
  } catch (e) {
    console.error('加载全局配置失败:', e)
  }
  return globalConfig
}

export function saveGlobalConfig(newConfig: Partial<GlobalConfig>) {
  Object.assign(globalConfig, newConfig)
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(globalConfig))
    localStorage.removeItem(LEGACY_STORAGE_KEY)
  } catch (e) {
    console.error('保存全局配置失败:', e)
  }
}

// 恢复出厂默认偏好配置
export function resetGlobalConfigToDefault() {
  Object.assign(globalConfig, DEFAULT_GLOBAL_CONFIG)
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(DEFAULT_GLOBAL_CONFIG))
    localStorage.removeItem(LEGACY_STORAGE_KEY)
  } catch (e) {
    console.error('重置全局配置失败:', e)
  }
}

// 2. 当前视频的临时配置 (内存级，打开新视频时从全局默认克隆，不污染全局)
export const currentVideoConfig = reactive({
  seekStep: 5,
  longPressSpeed: 3.0,
  wheelVolumeStep: 5,
})

// 当起播一个新视频时，初始化/重置当前视频的临时配置为全局默认值
export function resetCurrentVideoConfig() {
  currentVideoConfig.seekStep = globalConfig.seekStep
  currentVideoConfig.longPressSpeed = globalConfig.longPressSpeed
  currentVideoConfig.wheelVolumeStep = globalConfig.wheelVolumeStep
}

// 预先执行一次初始加载
loadGlobalConfig()
resetCurrentVideoConfig()

export function useConfig() {
  return {
    globalConfig,
    currentVideoConfig,
    loadGlobalConfig,
    saveGlobalConfig,
    resetGlobalConfigToDefault,
    resetCurrentVideoConfig,
  }
}
