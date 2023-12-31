// Function we are expected to implement
// void rust_initialize();
// void plugin_constructor(plugin::IPlugin *);
// void plugin_destructor(plugin::IPlugin *);
// void plugin_on_initialize(plugin::IPlugin *);
// void plugin_on_frame(plugin::IPlugin *, plugin::IInterface *);
// void plugin_on_exit(plugin::IPlugin *);

use crate::{
    bindings,
    plugin::{Plugin, RustPlugin},
    Context,
};

use lazy_static::lazy_static;
use tracing::{error, info};

lazy_static! {
    static ref PLUGIN_SINGLETON: std::sync::Mutex<Vec<(usize, Box<dyn Plugin + Send + Sync>)>> =
        std::sync::Mutex::new(Vec::new());
}

lazy_static! {
    static ref CONTEXT_SINGLETON: std::sync::Mutex<Context> = std::sync::Mutex::new(Context::new());
}

/// Convert a point of whatever type into a number.
fn pointer_to_integer<T>(ptr: *mut T) -> usize {
    ptr as usize
}

#[no_mangle]
pub extern "C" fn rust_initialize() {
    tracing_subscriber::fmt::init();
    info!("Initialized rust");
}

#[no_mangle]
pub extern "C" fn plugin_constructor(plugin_ptr: *mut bindings::plugin_IPlugin) {
    let context = CONTEXT_SINGLETON.lock().unwrap();
    // Create and box an instance of RustPlugin
    let plugin = Box::new(RustPlugin::new(&context).unwrap());
    // Store the instance in the singleton
    PLUGIN_SINGLETON
        .lock()
        .unwrap()
        .push((pointer_to_integer(plugin_ptr), plugin));
}

#[no_mangle]
pub extern "C" fn plugin_destructor(plugin_ptr: *mut bindings::plugin_IPlugin) {
    // Remove the instance from the singleton
    let mut plugins = PLUGIN_SINGLETON.lock().unwrap();
    let key = pointer_to_integer(plugin_ptr);
    if let Some(index) = plugins.iter().position(|(k, _)| *k == key) {
        plugins.remove(index);
    }
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

    let mut plugins = PLUGIN_SINGLETON.lock().unwrap();

    let key = pointer_to_integer(plugin_ptr);

    if let Some((_, plugin)) = plugins.iter_mut().find(|(k, _)| *k == key) {
        if let Err(e) = plugin.on_frame(&context, &crate::Interface { wrapper: interface }) {
            error!("Error in plugin_on_frame: {}", e);
        }
    }
}

#[no_mangle]
pub extern "C" fn plugin_on_exit(_plugin_ptr: *mut bindings::plugin_IPlugin) {
    // do nothing
}
