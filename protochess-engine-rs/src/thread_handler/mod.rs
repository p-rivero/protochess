use std::thread;

mod wasm_thread;

#[derive(Clone, Debug)]
pub struct ThreadHandler {
    use_std: bool,
    num_threads: u32,
}

impl ThreadHandler {
    pub fn std_threads() -> Self {
        Self { use_std: true, num_threads: 1 }
    }
    pub fn wasm_threads() -> Self {
        Self { use_std: false, num_threads: 1 }
    }
    
    pub fn spawn<F, T>(&self, f: F) -> JoinHandle<T>
    where
        F: FnOnce() -> T,
        F: Send + 'static,
        T: Send + 'static,
    {
        if self.use_std {
            JoinHandle { std: Some(thread::spawn(f)), wasm: None }
        } else {
            JoinHandle { std: None, wasm: Some(wasm_thread::spawn(f)) }
        }
    }
        
    pub fn sleep(&self, dur: std::time::Duration) {
        if self.use_std {
            thread::sleep(dur)
        } else {
            wasm_thread::sleep(dur)
        }
    }
    
    pub fn set_num_threads(&mut self, num_threads: u32) {
        if num_threads == 0 {
            panic!("Cannot set number of threads to 0");
        }
        if !self.use_std {
            // TODO: If wasm threads are implemented, remove this check
            panic!("Cannot set number of threads for wasm");
        }
        self.num_threads = num_threads;
    }
    pub fn num_threads(&self) -> u32 {
        self.num_threads
    }
}


pub struct JoinHandle<T> {
    std: Option<thread::JoinHandle<T>>,
    wasm: Option<wasm_thread::WasmJoinHandle<T>>,
}
impl<T> JoinHandle<T> {
    pub fn join(self) -> thread::Result<T> {
        if let Some(handle) = self.std {
            handle.join()
        } else if let Some(handle) = self.wasm {
            handle.join()
        } else {
            panic!("JoinHandle is empty");
        }
    }
    pub fn is_finished(&self) -> bool {
        if let Some(handle) = &self.std {
            handle.is_finished()
        } else if let Some(handle) = &self.wasm {
            handle.is_finished()
        } else {
            panic!("JoinHandle is empty");
        }
    }
}


pub trait JoinHandleTrait<T> {
    fn join(self) -> thread::Result<T>;
    fn is_finished(&self) -> bool;
}
impl<T> JoinHandleTrait<T> for thread::JoinHandle<T> {
    fn join(self) -> thread::Result<T> {
        self.join()
    }
    fn is_finished(&self) -> bool {
        self.is_finished()
    }
}
