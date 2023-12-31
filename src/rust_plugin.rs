use std::fmt::Debug;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

use crate::{mqtt::MQTT, Context, FramesPerSecond, Interface, plugin::Plugin};


pub struct RustPlugin {
    mqtt: MQTT,
    speedtest: bool,
    fps: FramesPerSecond,
}
impl Debug for RustPlugin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("RustPlugin")
    }
}
impl RustPlugin {
    pub fn new(context: &Context) -> Result<Self> {
        info!("Constructing Rust Plugin");
        let id = format!("rusty_bind-{}", uuid::Uuid::new_v4());
        let host = std::env::var("MQTT_HOST").unwrap_or_else(|_| "localhost".to_string());

        // run blocking async calls for init
        let (mqtt, event_loop) = context.block_on_async(async move {
            info!("Connecting to mqtt host: {} with id: {}", host, id);
            let (mut mqtt, event_loop) = MQTT::new(&id, &host, 1883).await?;
            mqtt.subscribe("rusty_bind/command").await?;
            Ok::<_, anyhow::Error>((mqtt, event_loop))
        })?;

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
        if let Some(fps) = self.fps.tick() {
            // format fps to have , separator
            use num_format::{Locale, ToFormattedString};
            let fps = (fps as u32).to_formatted_string(&Locale::en);
            info!("RustPlugin::on_frame fps: {}", fps);
        }

        // Handle incoming messages
        while let Some((topic, payload)) = self.mqtt.try_recv() {
            debug!("  Received message on {topic}: {payload}");
            let command = serde_json::from_str::<Commands>(&payload)?;
            match command {
                Commands::Shutdown(shutdown) => shutdown.handle(context, interface),
                Commands::SpeedTest(speedtest) => self.speedtest = speedtest.speed_test,
            }
        }

        if self.speedtest {
            return Ok(());
        }

        debug!(
            "RustPlugin::on_frame {name} {frame} {position}",
            name = interface.name(),
            frame = interface.frame(),
            position = format!("{:?}", interface.position())
        );

        // For funzies, publish a message to mqtt
        let data = FrameData {
            frame: interface.frame(),
            position: interface.position(),
        };
        let json = serde_json::to_string(&data)?;
        self.mqtt.try_publish("rusty_bind/frame", json)?;

        // Spawn a task just for the fun of it
        context.spawn_task(async move {
            debug!("  Spawning a task and sleeping");
            tokio::time::sleep(std::time::Duration::from_micros(1000)).await;
            debug!("  Task completed");
        });

        // block the task with sleep
        std::thread::sleep(std::time::Duration::from_millis(100));

        Ok(())
    }
}

#[derive(Serialize)]
struct FrameData {
    frame: u64,
    position: (f64, f64, f64),
}

#[derive(Deserialize)]
struct ShutdownCommand {
    shutdown: bool,
}
impl ShutdownCommand {
    fn handle(&self, _context: &Context, interface: &Interface) {
        if self.shutdown {
            info!("Shutting down from remote command");
            interface.shutdown();
        }
    }
}

#[derive(Deserialize)]
struct SpeedTest {
    speed_test: bool,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Commands {
    Shutdown(ShutdownCommand),
    SpeedTest(SpeedTest),
}