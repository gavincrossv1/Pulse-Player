// tests/updater_proxy_isolation.test.mjs
// 针对 Tauri Updater 原生 FFI 资源对象与 Vue 3 Proxy 隔离特性的单元测试
import test from 'node:test'
import assert from 'node:assert/strict'

// 模拟 Tauri Resource / Update 类的私有字段与 rid 契约
class MockResource {
  #rid
  constructor(rid) {
    this.#rid = rid
  }
  get rid() {
    return this.#rid
  }
}

class MockUpdate extends MockResource {
  constructor(rid, metadata) {
    super(rid)
    this.version = metadata.version
    this.date = metadata.date
    this.body = metadata.body
    this.downloaded = false
    this.installed = false
  }

  async download(onProgress) {
    // 关键点：底层必须能安全访问 this.rid 派发 FFI 调用
    const currentRid = this.rid
    if (onProgress) {
      onProgress({ event: 'Started', data: { contentLength: 1024 } })
      onProgress({ event: 'Progress', data: { chunkLength: 1024 } })
      onProgress({ event: 'Finished' })
    }
    this.downloaded = true
    return currentRid
  }

  async install() {
    const currentRid = this.rid
    this.installed = true
    return currentRid
  }
}

test('原生 ES 私有成员在进入 Proxy 后调用必定抛出 TypeError', async () => {
  const update = new MockUpdate(101, { version: '0.3.1' })
  // 模拟 Vue ref/reactive 内部的 Proxy 包装行为
  const proxyUpdate = new Proxy(update, {
    get(target, prop, receiver) {
      return Reflect.get(target, prop, receiver)
    }
  })

  // 验证在 Proxy 上直接调用方法会由于 receiver 为 Proxy 而导致私有字段报错
  await assert.rejects(
    async () => {
      await proxyUpdate.download()
    },
    {
      name: 'TypeError',
      message: /Cannot read private member/
    }
  )
})

test('纯内部变量隔离架构下调用方法完全正常', async () => {
  let activeUpdate = null
  let uiSnapshot = null

  // 模拟 useUpdater 的核心机制：仅向 UI 暴露纯数据，SDK 原生实例留存内部
  const checkUpdate = () => {
    const rawInstance = new MockUpdate(202, {
      version: '0.4.0',
      date: '2026-10-21',
      body: '测试更新内容'
    })
    activeUpdate = rawInstance
    uiSnapshot = {
      version: rawInstance.version,
      date: rawInstance.date,
      body: rawInstance.body
    }
  }

  checkUpdate()

  // 1. UI 获得的是纯数据对象，无任何私有字段
  assert.equal(uiSnapshot.version, '0.4.0')
  assert.equal(uiSnapshot.date, '2026-10-21')

  // 2. 执行下载操作，直接作用于 activeUpdate，私有字段正常访问
  let progressEventCount = 0
  await activeUpdate.download((event) => {
    if (event.event) progressEventCount++
  })

  assert.equal(activeUpdate.downloaded, true)
  assert.equal(progressEventCount, 3)

  // 3. 执行安装操作，私有字段正常访问
  await activeUpdate.install()
  assert.equal(activeUpdate.installed, true)
})
