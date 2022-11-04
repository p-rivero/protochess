use std::thread;

#[derive(Clone, Debug)]
pub struct ThreadHandler {
    use_std: bool,
}

impl ThreadHandler {
    pub fn std_threads() -> Self {
        Self { use_std: true }
    }
    pub fn wasm_threads() -> Self {
        Self { use_std: false }
    }
    
    pub fn spawn<F, T>(&self, f: F) -> thread::JoinHandle<T>
    where
        F: FnOnce() -> T,
        F: Send + 'static,
        T: Send + 'static,
    {
        if self.use_std {
            thread::spawn(f)
        } else {
            panic!("Not implemented");
        }
    }
        
    pub fn sleep(&self, dur: std::time::Duration) {
        if self.use_std {
            thread::sleep(dur)
        } else {
            panic!("Not implemented");
        }
    }
}


