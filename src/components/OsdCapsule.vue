<template>
  <transition name="osd-fade">
    <div v-if="visible" class="osd-capsule-container">
      <div class="osd-capsule">
        <span class="osd-text">{{ text }}</span>
      </div>
    </div>
  </transition>
</template>

<script setup lang="ts">
import { ref } from 'vue'

const visible = ref(false)
const text = ref('')
let timer: ReturnType<typeof setTimeout> | null = null

const show = (msg: string, duration = 800) => {
  text.value = msg
  visible.value = true

  if (timer) {
    clearTimeout(timer)
    timer = null
  }

  // duration 为 0 时表示常驻（如长按加速），等待外部调用 hide()
  if (duration > 0) {
    timer = setTimeout(() => {
      visible.value = false
      timer = null
    }, duration)
  }
}

const hide = () => {
  if (timer) {
    clearTimeout(timer)
    timer = null
  }
  visible.value = false
}

defineExpose({
  show,
  hide,
})
</script>

<style scoped>
.osd-capsule-container {
  position: absolute;
  top: 20px; /* 顶部紧凑居中，完全避开人像面部与核心视觉焦点 (对齐 B 站) */
  left: 50%;
  transform: translateX(-50%);
  pointer-events: none; /* 穿透式交互，点击滚动直达底层视口 */
  z-index: 90;
}

.osd-capsule {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 8px 22px;
  background: rgba(0, 0, 0, 0.78);
  border-radius: 18px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.35);
  backdrop-filter: blur(6px);
  -webkit-backdrop-filter: blur(6px);
  border: 1px solid rgba(255, 255, 255, 0.08);
}

.osd-text {
  color: #FFFFFF;
  font-size: 15px;
  font-weight: 600;
  letter-spacing: 0.5px;
  white-space: nowrap;
}

.osd-fade-enter-active,
.osd-fade-leave-active {
  transition: opacity 0.18s ease, transform 0.18s ease;
}

.osd-fade-enter-from,
.osd-fade-leave-to {
  opacity: 0;
  transform: scale(0.92);
}
</style>
