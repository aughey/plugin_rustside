use tokio::runtime::Builder;

pub mod bindings;
mod adapter;
mod plugin;

pub struct Context {
    tokio_runtime: tokio::runtime::Runtime,
}
impl Context {
    pub fn new() -> Self {
        let tokio_runtime = Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        Context {
            tokio_runtime,
        }
    }
    pub fn spawn_task<F>(&self, task: F)
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        self.tokio_runtime.spawn(task);
    }
    pub fn block_on_async<F, T>(&self, task: F) -> T
    where
        F: std::future::Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        self.tokio_runtime.block_on(task)
    }
}

pub struct Interface {
    wrapper: *mut bindings::plugin_IInterface,
}
impl Interface {
    pub fn name(&self) -> &str {
        unsafe {
            let name = bindings::interface_get_name(self.wrapper);
            std::ffi::CStr::from_ptr(name).to_str().unwrap()
        }
    }
    pub fn frame(&self) -> u64 {
        unsafe {
            bindings::interface_get_frame(self.wrapper)
        }
    }
    pub fn position(&self) -> (f64, f64, f64) {
        unsafe {
            let x = bindings::interface_get_position_x(self.wrapper);
            let y = bindings::interface_get_position_y(self.wrapper);
            let z = bindings::interface_get_position_z(self.wrapper);
            (x, y, z)
        }
    }
    pub fn shutdown(&self) {
        unsafe {
            bindings::interface_shutdown(self.wrapper);
        }
    }
}