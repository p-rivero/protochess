import { threads } from 'wasm-feature-detect'
import * as Comlink from 'comlink'

async function initWasm() {
  let wasmObject
  let memoryBuffer
  let supportsThreads = await threads()
  if (supportsThreads) {
    const multiThread = await import('../pkg-parallel/protochess_engine_wasm.js')
    const init = await multiThread.default()
    // Initialize the thread pool with the number of logical cores
    await multiThread.initThreadPool(navigator.hardwareConcurrency)
    wasmObject = new multiThread.Protochess()
    memoryBuffer = init.memory.buffer
  } else {
    const singleThread = await import('../pkg/protochess_engine_wasm.js')
    const init = await singleThread.default()
    wasmObject = new singleThread.Protochess()
    memoryBuffer = init.memory.buffer
  }

  return Comlink.proxy({
    wasmObject,
    supportsThreads,
    memoryBuffer,
  })
}

Comlink.expose({
  wasm: initWasm()
})
