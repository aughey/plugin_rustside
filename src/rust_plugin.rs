use crate::{plugin::Plugin, Context, FramesPerSecond, Interface};
use anyhow::Result;
use std::fmt::Debug;
use tracing::info;

/// The rust-side plugin that will be called by the host.
pub struct RustPlugin {
    /// A helper to count frames per second
    fps: FramesPerSecond,
}

// Implement Debug for tracing
impl Debug for RustPlugin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("RustPlugin")
    }
}

impl RustPlugin {
    /// Create a new instance of the plugin.
    /// 
    /// The plugin might block while connecting to the mqtt server.
    pub fn new(context: &Context) -> Result<Self> {
        info!("Constructing Rust Plugin");
 
        // spawn a task to run asynchronously and do something
        context.spawn_task(async {
            // do something
            // sleep 1 second
            loop {
            tokio::time::sleep(tokio::time::Duration::from_micros(250000)).await;
                let current_time_of_day = chrono::Local::now().time();
                info!("in an asynchronous loop {current_time_of_day}");
            }
        });
 
        Ok(RustPlugin {
            fps: FramesPerSecond::new(),
        })
    }
}

impl Plugin for RustPlugin {
    fn on_frame(&mut self, _context: &Context, _interface: &Interface) -> Result<()> {
        //info!("RustPlugin::on_frame");
        // Count/print the frames per second
        if let Some(fps) = self.fps.tick() {
            // format fps to have , separator for when the numbers get big
            use num_format::{Locale, ToFormattedString};
            let fps = (fps as u32).to_formatted_string(&Locale::en);
            info!("RustPlugin::on_frame fps: {}", fps);
        }

      
        Ok(())
    }
}

