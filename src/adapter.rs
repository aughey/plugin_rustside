// void rust_initialize();
// void plugin_constructor(plugin::IPlugin *);
// void plugin_destructor(plugin::IPlugin *);
// void plugin_on_initialize(plugin::IPlugin *);
// void plugin_on_frame(plugin::IPlugin *, plugin::IInterface *);
// void plugin_on_exit(plugin::IPlugin *);

use std::collections::HashMap;

use crate::{
    bindings,
    plugin::{Plugin, RustPlugin}, Context,
};

use lazy_static::lazy_static;

lazy_static! {
    static ref PLUGIN_SINGLETON: std::sync::Mutex<HashMap<usize, Box<dyn Plugin + Send + Sync>>> =
        std::sync::Mutex::new(HashMap::new());
}
lazy_static! {
    static ref CONTEXT_SINGLETON: std::sync::Mutex<Context> = 
        std::sync::Mutex::new(Context::new());
}

fn pointer_to_integer<T>(ptr: *mut T) -> usize {
    ptr as usize
}

#[no_mangle]
pub extern "C" fn rust_initialize() {
    println!("Initializing rust...");
    tracing_subscriber::fmt::init();
}

#[no_mangle]
pub extern "C" fn plugin_constructor(plugin_ptr: *mut bindings::plugin_IPlugin) {
    // Create an instance of RustPlugin
    let context = CONTEXT_SINGLETON.lock().unwrap();
    let plugin = Box::new(RustPlugin::new(&context).unwrap());
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
    let context = CONTEXT_SINGLETON.lock().unwrap();

    PLUGIN_SINGLETON
        .lock()
        .unwrap()
        .get_mut(&pointer_to_integer(plugin_ptr))
        .unwrap()
        .on_frame(&context, &crate::Interface { wrapper: interface });
}

#[no_mangle]
pub extern "C" fn plugin_on_exit(_plugin_ptr: *mut bindings::plugin_IPlugin) {
    // do nothing
}
