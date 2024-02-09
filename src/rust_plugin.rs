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

async fn get_current_time() -> String {
    // Use chrono to get the current time
    let now = chrono::Local::now();
    let current_thread = std::thread::current();
    let current_thread_id = current_thread.id();
    format!(
        "Hello from Axum in rust thread {current_thread_id:?}, Current time: {}",
        now.to_rfc2822()
    )
}

impl RustPlugin {
    /// Create a new instance of the plugin.
    ///
    /// The plugin might block while connecting to the mqtt server.
    pub fn new(context: &Context) -> Result<Self> {
        info!("Constructing Rust Plugin");

        // spawn an async task to run synchronous with our frame and do something
        context.spawn_task_synchronous(async {
            // do something
            // sleep 1 second
            loop {
                tokio::time::sleep(tokio::time::Duration::from_micros(250000)).await;
                let current_time_of_day = chrono::Local::now().time();
                info!("in an asynchronous loop {current_time_of_day}");
            }
        });

        // Create/spawn an axum server
        context.spawn_task_threaded(async {
            use axum::{routing::get, Router};
            // Build our application with a single route
            let app = Router::new().route("/", get(get_current_time));

            // run our app with hyper, listening globally on port 3000
            let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
            axum::serve(listener, app).await.unwrap();
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
