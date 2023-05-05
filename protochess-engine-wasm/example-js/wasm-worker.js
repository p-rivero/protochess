import { threads } from 'wasm-feature-detect'
import * as Comlink from 'comlink'

async function initWasm() {
  let wasmObject
  let supportsThreads = await threads()
  if (supportsThreads) {
    const multiThread = await import('../pkg-parallel/protochess_engine_wasm.js')
    await multiThread.default()
    // Initialize the thread pool with the number of logical cores
    await multiThread.initThreadPool(navigator.hardwareConcurrency)
    wasmObject = new multiThread.Protochess()
  } else {
    const singleThread = await import('../pkg/protochess_engine_wasm.js')
    await singleThread.default()
    wasmObject = new singleThread.Protochess()
  }

  return Comlink.proxy({
    wasmObject,
    supportsThreads
  })
}

Comlink.expose({
  wasm: initWasm()
})
