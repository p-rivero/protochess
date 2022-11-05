use super::JoinHandleTrait;

// This is a dummy implementation of a wasm thread, which does not actually spawn a thread
// Instead, it just runs the function in the current thread and returns a handle that is immediately finished

// If in the future there is an easy way to spawn a thread in wasm, this can be replaced with a real implementation

// TODO: Implement JoinHandleTrait for a wasm thread
pub struct WasmJoinHandle<T> { result: T, }

impl<T> JoinHandleTrait<T> for WasmJoinHandle<T> {
    fn join(self) -> std::thread::Result<T> {
        Ok(self.result)
    }
    fn is_finished(&self) -> bool {
        true
    }
}


pub fn spawn<F, T>(f: F) -> WasmJoinHandle<T>
    where
        F: FnOnce() -> T,
        F: Send + 'static,
        T: Send + 'static,
{
    // TODO: Implement spawning a wasm thread
    // For now, just run the function in the current thread
    WasmJoinHandle { result: f() }
}


pub fn sleep(_dur: std::time::Duration) {
    // TODO: Implement sleep for wasm
    // For now, just do nothing and return immediately
}

