//! This is the adapter between the C interface and the Rust plugin.
//!
//! C Function we are expected to implement
//! - void rust_initialize();
//! - void plugin_constructor(plugin::IPlugin *);
//! - void plugin_destructor(plugin::IPlugin *);
//! - void plugin_on_initialize(plugin::IPlugin *);
//! - void plugin_on_frame(plugin::IPlugin *, plugin::IInterface *);
//! - void plugin_on_exit(plugin::IPlugin *);

use crate::{bindings, plugin::Plugin, rust_plugin::RustPlugin, Context};

use lazy_static::lazy_static;
use tracing::{error, info};

lazy_static! {
    /// Singleton for the plugin instances
    /// These are stored as a Vec of (key, value) pairs where key is the "this" pointer of the
    /// host plugin.  We use Vec instead of HashMap because the number of plugins is expected to
    /// be small and the overhead of a HashMap is more than searching a short list.
    static ref PLUGIN_SINGLETON: std::sync::Mutex<Vec<(usize, Box<dyn Plugin + Send + Sync>)>> =
        std::sync::Mutex::new(Vec::new());
}

lazy_static! {
    /// Singleton for the context
    /// This context creates an async executing context that the plugin can use to perform
    /// asynchronous operations.
    static ref CONTEXT_SINGLETON: std::sync::Mutex<Context> = std::sync::Mutex::new(Context::new());
}

/// Convert a point of whatever type into a number.
fn pointer_to_integer<T>(ptr: *mut T) -> usize {
    ptr as usize
}

/// Called when a plugin is created on the host side.  
/// We simply create a Rust represenative plugin to accept and handle the calls.
#[no_mangle]
pub extern "C" fn plugin_constructor(plugin_ptr: *mut bindings::plugin_CBoeingPackageModel) {
    tracing_subscriber::fmt::init();
    info!("Initialized rust");

    let context = CONTEXT_SINGLETON.lock().unwrap();
    // Create and box an instance of RustPlugin to be the recipient of the plugin calls through the C interface
    let plugin = Box::new(RustPlugin::new(&context).unwrap());
    // Store the instance in the singleton
    PLUGIN_SINGLETON
        .lock()
        .unwrap()
        .push((pointer_to_integer(plugin_ptr), plugin));
}

/// Called when a plugin is destroyed on the host side.
/// We simply remove the Rust represenative plugin from the singleton which will drop the instance.
#[no_mangle]
pub extern "C" fn plugin_destructor(plugin_ptr: *mut bindings::plugin_CBoeingPackageModel) {
    // Remove the instance from the singleton
    let mut plugins = PLUGIN_SINGLETON.lock().unwrap();
    let key = pointer_to_integer(plugin_ptr);
    if let Some(index) = plugins.iter().position(|(k, _)| *k == key) {
        plugins.remove(index);
    }
}

/// Called when a plugin is initialized on the host side.
/// This is an eroneous call and we do nothing.
#[no_mangle]
pub extern "C" fn plugin_on_initialize(_plugin_ptr: *mut bindings::plugin_CBoeingPackageModel) {
    // do nothing
}

/// Called when a plugin on_frame is called on the host side.
/// Pass the call to the Rust represenative plugin and wrap the instance
/// into a callable rust object.
#[no_mangle]
pub extern "C" fn plugin_on_run(
    plugin_ptr: *mut bindings::plugin_CBoeingPackageModel
) {
    // Grab our static context and plugin instances
    let context = CONTEXT_SINGLETON.lock().unwrap();
    let mut plugins = PLUGIN_SINGLETON.lock().unwrap();

    // info!("plugin_on_run (called from info)");

    // This is how we identify a plugin
    let key = pointer_to_integer(plugin_ptr);

    // assuming we find it, call the on_frame method
    if let Some((_, plugin)) = plugins.iter_mut().find(|(k, _)| *k == key) {
        // This is our error boundary, so log any errors and continue
        if let Err(e) = plugin.on_frame(&context, &crate::Interface { }) {
            error!("Error in plugin_on_frame: {}", e);
        }
    }
    // poll the context to allow async tasks to run
    context.poll();
}

/// Called when a plugin is exiting on the host side.
/// This is an eroneous call and we do nothing.
#[no_mangle]
pub extern "C" fn plugin_on_reset(_plugin_ptr: *mut bindings::plugin_CBoeingPackageModel) {
    // do nothing
}
