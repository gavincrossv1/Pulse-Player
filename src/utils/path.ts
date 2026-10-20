// src/utils/path.ts
// 路径归一化工具：消除 Windows 系统下的正反斜杠、大小写及首尾空白差异

/**
 * 规范化文件路径 (统一小写、转为正斜杠、去除两端空格)
 */
export function normalizePath(path: string): string {
  if (!path) return ''
  return path.replace(/\\/g, '/').toLowerCase().trim()
}

/**
 * 判断两个路径是否指向同一文件
 */
export function isSamePath(path1: string, path2: string): boolean {
  if (!path1 || !path2) return false
  return normalizePath(path1) === normalizePath(path2)
}
