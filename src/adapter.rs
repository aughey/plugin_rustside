// void rust_initialize();
// void plugin_constructor(plugin::IPlugin *);
// void plugin_destructor(plugin::IPlugin *);
// void plugin_on_initialize(plugin::IPlugin *);
// void plugin_on_frame(plugin::IPlugin *, plugin::IInterface *);
// void plugin_on_exit(plugin::IPlugin *);

use std::collections::HashMap;

use crate::{
    bindings,
    plugin::{Plugin, RustPlugin},
};

use lazy_static::lazy_static;

lazy_static! {
    static ref PLUGIN_SINGLETON: std::sync::Mutex<HashMap<usize, Box<dyn Plugin + Send + Sync>>> =
        std::sync::Mutex::new(HashMap::new());
}

fn pointer_to_integer<T>(ptr: *mut T) -> usize {
    ptr as usize
}

#[no_mangle]
pub extern "C" fn rust_initialize() {
    println!("Initializing rust...");
}

#[no_mangle]
pub extern "C" fn plugin_constructor(plugin_ptr: *mut bindings::plugin_IPlugin) {
    // Create an instance of RustPlugin
    let plugin = Box::new(RustPlugin {});
    // Store the instance in the singleton
    PLUGIN_SINGLETON
        .lock()
        .unwrap()
        .insert(pointer_to_integer(plugin_ptr), plugin);
}

#[no_mangle]
pub extern "C" fn plugin_destructor(plugin_ptr: *mut bindings::plugin_IPlugin) {
    // Remove the instance from the singleton
    PLUGIN_SINGLETON
        .lock()
        .unwrap()
        .remove(&pointer_to_integer(plugin_ptr));
}

#[no_mangle]
pub extern "C" fn plugin_on_initialize(_plugin_ptr: *mut bindings::plugin_IPlugin) {
    // do nothing
}

#[no_mangle]
pub extern "C" fn plugin_on_frame(
    plugin_ptr: *mut bindings::plugin_IPlugin,
    interface: *mut bindings::plugin_IInterface,
) {
    PLUGIN_SINGLETON
        .lock()
        .unwrap()
        .get_mut(&pointer_to_integer(plugin_ptr))
        .unwrap()
        .on_frame(&crate::Interface { wrapper: interface });
}

#[no_mangle]
pub extern "C" fn plugin_on_exit(_plugin_ptr: *mut bindings::plugin_IPlugin) {
    // do nothing
}
