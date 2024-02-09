use std::fmt::Debug;

use tokio::runtime::Builder;

mod adapter;
pub mod bindings;
// Enable if feature mqtt is enabled
#[cfg(feature = "mqtt")]
pub mod mqtt;
mod plugin;
mod rust_plugin;

/// A context object that gives plugins access to runtime features like an async context.
pub struct Context {
    tokio_runtime: tokio::runtime::Runtime,
}
impl Context {
    pub fn new() -> Self {
        //let tokio_runtime = Builder::new_multi_thread().enable_all().build().unwrap();
        let tokio_runtime = Builder::new_current_thread().enable_all().build().unwrap();

        Context { tokio_runtime }
    }
    pub fn poll(&self) {
        self.tokio_runtime.block_on(async {tokio::time::sleep(std::time::Duration::ZERO).await});
    }

    /// Spawn a task in the async runtime that is held by the context.
    pub fn spawn_task<F>(&self, task: F)
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        self.tokio_runtime.spawn(task);
    }

    /// Run an async task in the async runtime that is held by the context, but block
    /// until the task is complete.
    /// Returns the output of the task.
    pub fn block_on_async<F, T>(&self, task: F) -> T
    where
        F: std::future::Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        self.tokio_runtime.block_on(task)
    }
}

/// An interface wrapper to make calls back to the host through the C interface.
pub struct Interface {
}
impl Debug for Interface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Interface")
    }
}
// impl Interface {
//     pub fn name(&self) -> &str {
//         unsafe {
//             let name = bindings::interface_get_name(self.wrapper);
//             std::ffi::CStr::from_ptr(name)
//                 .to_str()
//                 .expect("a valid string from the interface_get_name call")
//         }
//     }
//     pub fn frame(&self) -> u64 {
//         unsafe { bindings::interface_get_frame(self.wrapper) }
//     }
//     pub fn position(&self) -> (f64, f64, f64) {
//         unsafe {
//             let x = bindings::interface_get_position_x(self.wrapper);
//             let y = bindings::interface_get_position_y(self.wrapper);
//             let z = bindings::interface_get_position_z(self.wrapper);
//             (x, y, z)
//         }
//     }
//     pub fn shutdown(&self) {
//         unsafe {
//             bindings::interface_shutdown(self.wrapper);
//         }
//     }
// }

/// A helper struct to count frames per second.
/// Every second tick() will return the current frames per second.
pub struct FramesPerSecond {
    count: u64,
    last_print: std::time::Instant,
}
impl FramesPerSecond {
    /// Construct the value.
    pub fn new() -> Self {
        FramesPerSecond {
            count: 0,
            last_print: std::time::Instant::now(),
        }
    }
    /// Tick the counter and return the current frames per second if a second has passed.
    pub fn tick(&mut self) -> Option<f64> {
        self.count += 1;
        let now = std::time::Instant::now();
        let since = now.duration_since(self.last_print);
        if since.as_secs() >= 1 {
            let ret = Some(self.count as f64 / since.as_secs_f64());
            self.count = 0;
            self.last_print = now;
            ret
        } else {
            None
        }
    }
}
