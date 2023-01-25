import { threads } from 'wasm-feature-detect';
import * as Comlink from 'comlink';

const BINARY = 'protochess_engine_wasm';
const OBJECT_NAME = 'Protochess';

async function initWasm() {
  let wasmObject
  let supportsThreads = await threads()
  if (supportsThreads) {
    const multiThread = await import('../pkg-parallel/' + BINARY + '.js');
    await multiThread.default();
    await multiThread.initThreadPool(navigator.hardwareConcurrency);
    wasmObject = await new multiThread[OBJECT_NAME]();
  } else {
    const singleThread = await import('../pkg/' + BINARY + '.js');
    await singleThread.default();
    wasmObject = await new singleThread[OBJECT_NAME]();
  }

  return Comlink.proxy({
    wasmObject,
    supportsThreads
  });
}

Comlink.expose({
  wasm: initWasm()
});
