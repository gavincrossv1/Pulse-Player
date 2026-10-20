// src/utils/throttle.ts
// 通用节流工具函数 (具备首触发与尾触发特性，保证最终状态必然生效，支持显式取消未完成的尾触发)

export interface ThrottledFunction<T extends (...args: any[]) => any> {
  (...args: Parameters<T>): void
  cancel: () => void
  isPending: () => boolean
}

export function throttle<T extends (...args: any[]) => any>(
  fn: T,
  wait: number
): ThrottledFunction<T> {
  let timer: ReturnType<typeof setTimeout> | null = null
  let lastArgs: Parameters<T> | null = null
  let lastExecTime = 0

  const throttled = (...args: Parameters<T>) => {
    const now = Date.now()
    lastArgs = args

    const execute = () => {
      lastExecTime = Date.now()
      const currentArgs = lastArgs
      lastArgs = null
      timer = null
      if (currentArgs) {
        fn(...currentArgs)
      }
    }

    if (now - lastExecTime >= wait) {
      if (timer) {
        clearTimeout(timer)
        timer = null
      }
      execute()
    } else if (!timer) {
      timer = setTimeout(execute, wait - (now - lastExecTime))
    }
  }

  throttled.cancel = () => {
    if (timer) {
      clearTimeout(timer)
      timer = null
    }
    lastArgs = null
  }

  throttled.isPending = () => {
    return timer !== null
  }

  return throttled
}
