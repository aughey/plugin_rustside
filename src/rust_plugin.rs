//! This is a sample plugin written in rust.
//!
//! It is a simple plugin that will print out the frame number and position of the interface.
//!
//! It also will publish a message to the mqtt broker with the frame number and position.
//!
//! It can read mqtt messages with the following commands:
//! - shutdown - shutdown the interface
//! - speedtest - toggle speedtest mode.  In speedtest, it does the minimal amount of work and returns.
//!
//! The commands are defined in the Commands enum.
use crate::{mqtt::MQTT, plugin::Plugin, Context, FramesPerSecond, Interface};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use tracing::{debug, error, info};

/// The rust-side plugin that will be called by the host.
pub struct RustPlugin {
    /// The mqtt helper to send and receive messages
    mqtt: MQTT,
    /// A flag to toggle speedtest mode
    speedtest: bool,
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
        let id = format!("rusty_bind-{}", uuid::Uuid::new_v4());
        let host = std::env::var("MQTT_HOST").unwrap_or_else(|_| "localhost".to_string());

        // We create mqtt in an async context because it's going to do some async stuff
        // internally to connect.  We block here because we want the mqtt connection to be
        // established before we start processing frames.
        let (mqtt, event_loop) = context.block_on_async(async move {
            info!("Connecting to mqtt host: {} with id: {}", host, id);
            let (mut mqtt, event_loop) = MQTT::new(&id, &host, 1883).await?;
            mqtt.subscribe("rusty_bind/command").await?;
            Ok::<_, anyhow::Error>((mqtt, event_loop))
        })?;

        // mqtt proves an event_loop future that we need to run in a spawned task to handle the events.
        context.spawn_task(async move {
            if let Err(e) = event_loop.await {
                error!("Error in mqtt event loop: {}", e);
            }
        });

        Ok(RustPlugin {
            mqtt,
            speedtest: false,
            fps: FramesPerSecond::new(),
        })
    }
}

impl Plugin for RustPlugin {
    fn on_frame(&mut self, context: &Context, interface: &Interface) -> Result<()> {
        // Count/print the frames per second
        if let Some(fps) = self.fps.tick() {
            // format fps to have , separator for when the numbers get big
            use num_format::{Locale, ToFormattedString};
            let fps = (fps as u32).to_formatted_string(&Locale::en);
            info!("RustPlugin::on_frame fps: {}", fps);
        }

        // Handle incoming messages
        while let Some((topic, payload)) = self.mqtt.try_recv() {
            debug!("  Received message on {topic}: {payload}");
            // Parse the payload into a Commands enum.
            // Note: a failure of parsing will stop any further processing of this frame
            let command = serde_json::from_str::<Commands>(&payload)?;

            // Depending on the command, do something
            match command {
                Commands::Shutdown(shutdown) => if shutdown.shutdown { interface.shutdown(); },
                Commands::SpeedTest(speedtest) => self.speedtest = speedtest.speed_test,
            }
        }

        // in speed test mode, we return early before doing the silly stuff below
        if self.speedtest {
            return Ok(());
        }

        debug!(
            "RustPlugin::on_frame {name} {frame} {position}",
            name = interface.name(),
            frame = interface.frame(),
            position = format!("{:?}", interface.position())
        );

        // For funzies, publish a message to mqtt.  This is a bit of a flex, but shows
        // how we might send information to an external service for debugging or logging.
        let data = FrameData {
            frame: interface.frame(),
            position: interface.position(),
        };
        let json = serde_json::to_string(&data)?;
        self.mqtt.try_publish("rusty_bind/frame", json)?;

        // Spawn a task just for the fun of it.  This might represent a longer running operation
        // that doesn't happen all the time and we don't want to block the on_frame call.
        const SLEEP_TIME: u64 = 100;
        context.spawn_task(async {
            debug!("  Spawning a task and sleeping");
            tokio::time::sleep(std::time::Duration::from_micros(SLEEP_TIME/10)).await;
            debug!("  Task completed");
        });

        // block the task with sleep
        std::thread::sleep(std::time::Duration::from_millis(SLEEP_TIME));

        Ok(())
    }
}

/// A struct to hold the data we want to send to mqtt.
/// This is primarily created to allow serde to serialize the data into json.
#[derive(Serialize)]
struct FrameData {
    frame: u64,
    position: (f64, f64, f64),
}

/// The shutdown command can remotely tell our processing to stop.
#[derive(Deserialize)]
struct ShutdownCommand {
    shutdown: bool,
}

/// The speedtest command will tell our plugin to do the minimal amount of work and return.
#[derive(Deserialize)]
struct SpeedTest {
    speed_test: bool,
}

/// Wrap all commands in an enum.  These commands are untagged, so the first
/// one to parse correctly is the "right" command.  We could use serde enum
/// tagging to make this more explicit, but this is a simple example.
#[derive(Deserialize)]
#[serde(untagged)]
enum Commands {
    Shutdown(ShutdownCommand),
    SpeedTest(SpeedTest),
}
