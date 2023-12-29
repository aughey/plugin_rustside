pub mod bindings;
mod adapter;
mod plugin;

pub struct Interface {
    wrapper: *mut bindings::plugin_IInterface,
}
impl Interface {
    fn name(&self) -> &str {
        unsafe {
            let name = bindings::interface_get_name(self.wrapper);
            std::ffi::CStr::from_ptr(name).to_str().unwrap()
        }
    }
    fn frame(&self) -> u64 {
        unsafe {
            bindings::interface_get_frame(self.wrapper)
        }
    }
    fn position(&self) -> (f64, f64, f64) {
        unsafe {
            let x = bindings::interface_get_position_x(self.wrapper);
            let y = bindings::interface_get_position_y(self.wrapper);
            let z = bindings::interface_get_position_z(self.wrapper);
            (x, y, z)
        }
    }
}