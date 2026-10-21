<template>
  <svg
    :class="['svg-icon', iconClass]"
    :width="size"
    :height="size"
    viewBox="0 0 24 24"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
    :style="{ color: color || 'currentColor' }"
  >
    <!-- 回到主界面 -->
    <path
      v-if="name === 'home'"
      d="M3 10.5L12 3L21 10.5V20C21 20.5523 20.5523 21 20 21H15V14H9V21H4C3.44772 21 3 20.5523 3 20V10.5Z"
      stroke="currentColor"
      stroke-width="1.8"
      stroke-linejoin="round"
      fill="currentColor"
      fill-opacity="0.15"
    />

    <!-- 图钉 (未置顶 45度倾斜) -->
    <g v-else-if="name === 'pin'" transform="rotate(-45 12 12)">
      <path
        d="M16 4H8V6L10 8V14L8 16V18H11V22L12 23L13 22V18H16V16L14 14V8L16 6V4Z"
        stroke="currentColor"
        stroke-width="1.8"
        stroke-linejoin="round"
      />
    </g>

    <!-- 图钉 (置顶态 90度挺立高亮) -->
    <g v-else-if="name === 'pin-active'">
      <path
        d="M16 3H8V5L10 7V13L8 15V17H11V21L12 22L13 21V17H16V15L14 13V7L16 5V3Z"
        fill="#FB7299"
        stroke="#FB7299"
        stroke-width="1.5"
        stroke-linejoin="round"
      />
    </g>

    <!-- 最小化 -->
    <path
      v-else-if="name === 'minimize'"
      d="M5 12H19"
      stroke="currentColor"
      stroke-width="1.8"
      stroke-linecap="round"
    />

    <!-- 最大化 -->
    <rect
      v-else-if="name === 'maximize'"
      x="5"
      y="5"
      width="14"
      height="14"
      rx="1.5"
      stroke="currentColor"
      stroke-width="1.8"
    />

    <!-- 还原 -->
    <g v-else-if="name === 'restore'">
      <rect x="7" y="7" width="12" height="12" rx="1.5" stroke="currentColor" stroke-width="1.6" />
      <path d="M5 15V5H15" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" />
    </g>

    <!-- 关闭 -->
    <path
      v-else-if="name === 'close'"
      d="M6 6L18 18M6 18L18 6"
      stroke="currentColor"
      stroke-width="1.8"
      stroke-linecap="round"
    />

    <!-- 播放 -->
    <path
      v-else-if="name === 'play'"
      d="M7 4.5V19.5L19 12L7 4.5Z"
      fill="currentColor"
    />

    <!-- 暂停 -->
    <path
      v-else-if="name === 'pause'"
      d="M6 5H10V19H6V5ZM14 5H18V19H14V5Z"
      fill="currentColor"
    />

    <!-- 上一集 -->
    <path
      v-else-if="name === 'prev'"
      d="M6 5H8V19H6V5ZM19 5.5L10 12L19 18.5V5.5Z"
      fill="currentColor"
    />

    <!-- 下一集 -->
    <path
      v-else-if="name === 'next'"
      d="M18 5H16V19H18V5ZM5 5.5L14 12L5 18.5V5.5Z"
      fill="currentColor"
    />

    <!-- 音量 (正常) -->
    <g v-else-if="name === 'volume'">
      <path
        d="M11 5L6 9H2V15H6L11 19V5Z"
        fill="currentColor"
      />
      <path
        d="M15.54 8.46C16.48 9.4 17 10.66 17 12C17 13.34 16.48 14.6 15.54 15.54"
        stroke="currentColor"
        stroke-width="1.8"
        stroke-linecap="round"
      />
      <path
        d="M19.07 4.93C20.94 6.8 22 9.33 22 12C22 14.67 20.94 17.2 19.07 19.07"
        stroke="currentColor"
        stroke-width="1.8"
        stroke-linecap="round"
      />
    </g>

    <!-- 音量 (静音) -->
    <g v-else-if="name === 'volume-mute'">
      <path
        d="M11 5L6 9H2V15H6L11 19V5Z"
        fill="currentColor"
      />
      <path
        d="M22 9L16 15M16 9L22 15"
        stroke="#FB7299"
        stroke-width="2"
        stroke-linecap="round"
      />
    </g>

    <!-- 齿轮 (设置) -->
    <g v-else-if="name === 'settings'">
      <path
        d="M19.14 12.94C19.18 12.63 19.2 12.32 19.2 12C19.2 11.68 19.18 11.37 19.14 11.06L21.4 9.3C21.6 9.14 21.66 8.86 21.53 8.64L19.39 4.93C19.26 4.71 18.99 4.62 18.76 4.71L16.1 5.78C15.55 5.36 14.94 5.01 14.28 4.74L13.88 1.9C13.85 1.65 13.63 1.46 13.38 1.46H9.08C8.83 1.46 8.62 1.65 8.58 1.9L8.18 4.74C7.52 5.01 6.92 5.36 6.36 5.78L3.7 4.71C3.47 4.62 3.2 4.71 3.07 4.93L0.93 8.64C0.8 8.86 0.86 9.14 1.06 9.3L3.32 11.06C3.28 11.37 3.26 11.68 3.26 12C3.26 12.32 3.28 12.63 3.32 12.94L1.06 14.7C0.86 14.86 0.8 15.14 0.93 15.36L3.07 19.07C3.2 19.29 3.47 19.38 3.7 19.29L6.36 18.22C6.92 18.64 7.52 18.99 8.18 19.26L8.58 22.1C8.62 22.35 8.83 22.54 9.08 22.54H13.38C13.63 22.54 13.85 22.35 13.88 22.1L14.28 19.26C14.94 18.99 15.55 18.64 16.1 18.22L18.76 19.29C18.99 19.38 19.26 19.29 19.39 19.07L21.53 15.36C21.66 15.14 21.6 14.86 21.4 14.7L19.14 12.94ZM11.23 15.5C9.3 15.5 7.73 13.93 7.73 12C7.73 10.07 9.3 8.5 11.23 8.5C13.16 8.5 14.73 10.07 14.73 12C14.73 13.93 13.16 15.5 11.23 15.5Z"
        fill="currentColor"
      />
    </g>

    <!-- 全屏 -->
    <path
      v-else-if="name === 'fullscreen'"
      d="M7 3H3V7M21 7V3H17M3 17V21H7M17 21H21V17"
      stroke="currentColor"
      stroke-width="2"
      stroke-linecap="round"
      stroke-linejoin="round"
    />

    <!-- 退出全屏 -->
    <path
      v-else-if="name === 'fullscreen-exit'"
      d="M3 7H7V3M21 7H17V3M3 17H7V21M21 17H17V21"
      stroke="currentColor"
      stroke-width="2"
      stroke-linecap="round"
      stroke-linejoin="round"
    />

    <!-- Pulse Player 品牌标 (脉冲光棱视窗与双切面播放核，四位一体核心视觉) -->
    <g v-else-if="name === 'pulse-logo'">
      <defs>
        <!-- 脉冲高能主渐变 (自左上洋红向右下霓虹粉辐射) -->
        <linearGradient :id="'pulse-grad-' + uid" x1="2" y1="2" x2="22" y2="22" gradientUnits="userSpaceOnUse">
          <stop stop-color="#FF2E88" />
          <stop offset="0.5" stop-color="#FB7299" />
          <stop offset="1" stop-color="#FF85AD" />
        </linearGradient>
        <!-- 视窗底座微渐变 -->
        <linearGradient :id="'pulse-bg-' + uid" x1="0" y1="0" x2="24" y2="24" gradientUnits="userSpaceOnUse">
          <stop stop-color="#241B26" stop-opacity="0.9" />
          <stop offset="1" stop-color="#141118" stop-opacity="0.95" />
        </linearGradient>
      </defs>

      <!-- 外部圆角晶体视窗外框 (Squircle 连续曲率) -->
      <rect
        x="2.5"
        y="2.5"
        width="19"
        height="19"
        rx="5.5"
        :fill="'url(#pulse-bg-' + uid + ')'"
        :stroke="'url(#pulse-grad-' + uid + ')'"
        stroke-width="1.8"
      />

      <!-- 内部核心：脉冲双光刃播放箭头 (中缝呈音波脉冲光隙，剪影锐利辨识度极高) -->
      <!-- 上光刃：主导播放动势与右侧锐角 -->
      <path
        d="M8.2 6.8C8.2 6.1 9.0 5.6 9.6 6.0L16.8 10.9C17.4 11.3 17.4 12.2 16.7 12.6L14.6 13.9L9.8 9.9C8.8 9.1 8.2 7.9 8.2 6.8Z"
        :fill="'url(#pulse-grad-' + uid + ')'"
      />
      <!-- 下光刃：音律支撑与负空间脉冲槽 -->
      <path
        d="M8.2 12.2L13.6 15.8L10.8 17.6C10.2 18.0 9.4 17.6 9.4 16.8V13.8C9.4 13.2 8.9 12.7 8.3 12.4L8.2 12.2Z"
        :fill="'url(#pulse-grad-' + uid + ')'"
        opacity="0.9"
      />
    </g>

    <!-- 垃圾桶 / 删除 (清空历史记录) -->
    <g v-else-if="name === 'trash'">
      <path
        d="M3 6H5H21"
        stroke="currentColor"
        stroke-width="1.8"
        stroke-linecap="round"
        stroke-linejoin="round"
      />
      <path
        d="M8 6V4C8 3.44772 8.44772 3 9 3H15C15.5523 3 16 3.44772 16 4V6M19 6V20C19 20.5523 18.5523 21 18 21H6C5.44772 21 5 20.5523 5 20V6H19Z"
        stroke="currentColor"
        stroke-width="1.8"
        stroke-linecap="round"
        stroke-linejoin="round"
      />
      <path
        d="M10 11V16M14 11V16"
        stroke="currentColor"
        stroke-width="1.8"
        stroke-linecap="round"
      />
    </g>

    <!-- 随机播放 / 洗牌 -->
    <g v-else-if="name === 'shuffle'">
      <path
        d="M16 3H21V8"
        stroke="currentColor"
        stroke-width="1.8"
        stroke-linecap="round"
        stroke-linejoin="round"
      />
      <path
        d="M4 20L21 3"
        stroke="currentColor"
        stroke-width="1.8"
        stroke-linecap="round"
        stroke-linejoin="round"
      />
      <path
        d="M21 16V21H16"
        stroke="currentColor"
        stroke-width="1.8"
        stroke-linecap="round"
        stroke-linejoin="round"
      />
      <path
        d="M15 15L21 21"
        stroke="currentColor"
        stroke-width="1.8"
        stroke-linecap="round"
        stroke-linejoin="round"
      />
      <path
        d="M4 4L9 9"
        stroke="currentColor"
        stroke-width="1.8"
        stroke-linecap="round"
        stroke-linejoin="round"
      />
    </g>

    <!-- 排序 / 倒序 -->
    <g v-else-if="name === 'sort'">
      <path
        d="M3 6H15"
        stroke="currentColor"
        stroke-width="1.8"
        stroke-linecap="round"
      />
      <path
        d="M3 12H11"
        stroke="currentColor"
        stroke-width="1.8"
        stroke-linecap="round"
      />
      <path
        d="M3 18H8"
        stroke="currentColor"
        stroke-width="1.8"
        stroke-linecap="round"
      />
      <path
        d="M18 9V20M18 20L15 17M18 20L21 17"
        stroke="currentColor"
        stroke-width="1.8"
        stroke-linecap="round"
        stroke-linejoin="round"
      />
    </g>

    <!-- 音乐 / 音频 -->
    <g v-else-if="name === 'music'">
      <path
        d="M9 18V5L21 3V16"
        stroke="currentColor"
        stroke-width="1.8"
        stroke-linecap="round"
        stroke-linejoin="round"
      />
      <circle cx="6" cy="18" r="3" stroke="currentColor" stroke-width="1.8" />
      <circle cx="18" cy="16" r="3" stroke="currentColor" stroke-width="1.8" />
    </g>

    <!-- 刷新 -->
    <g v-else-if="name === 'refresh'">
      <path
        d="M4 12C4 7.58172 7.58172 4 12 4C15.1585 4 17.8817 5.82869 19.1672 8.5M20 12C20 16.4183 16.4183 20 12 20C8.84153 20 6.11828 18.1713 4.83279 15.5"
        stroke="currentColor"
        stroke-width="1.8"
        stroke-linecap="round"
      />
      <path d="M20 4V9H15" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" />
      <path d="M4 20V15H9" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" />
    </g>

    <!-- 勾选 / 成功 -->
    <path
      v-else-if="name === 'check'"
      d="M5 13L9 17L19 7"
      stroke="currentColor"
      stroke-width="2"
      stroke-linecap="round"
      stroke-linejoin="round"
    />

    <!-- 外链 / 箭头 -->
    <g v-else-if="name === 'external-link'">
      <path d="M18 13V19C18 20.1046 17.1046 21 16 21H5C3.89543 21 3 20.1046 3 19V8C3 6.89543 3.89543 6 5 6H11" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" />
      <path d="M15 3H21V9" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" />
      <path d="M10 14L21 3" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" />
    </g>

    <!-- 默认兜底 -->
    <circle v-else cx="12" cy="12" r="9" stroke="currentColor" stroke-width="1.5" />
  </svg>
</template>

<script setup lang="ts">
defineProps<{
  name: string
  size?: number | string
  color?: string
  iconClass?: string
}>()

// 为每个实例分配独立 UID，彻底消除 SVG 内联 Gradient ID 冲突
const uid = Math.random().toString(36).substring(2, 8)
</script>

<style scoped>
.svg-icon {
  display: inline-block;
  vertical-align: middle;
  flex-shrink: 0;
}
</style>
