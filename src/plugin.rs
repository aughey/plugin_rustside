use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::{Context, Interface};

pub trait Plugin {
    fn on_frame(&mut self, context: &Context, interface: &Interface);
}
pub struct RustPlugin {
    client: rumqttc::AsyncClient,
    receiver: tokio::sync::mpsc::UnboundedReceiver<(String, String)>,
}
impl RustPlugin {
    pub fn new(context: &Context) -> Result<Self> {
        println!("Constructing Rust Plugin");
        let id = format!("rusty_bind-{}", uuid::Uuid::new_v4());

        let host = std::env::var("MQTT_HOST").unwrap_or_else(|_| "localhost".to_string());

        println!("connecting to mqtt host");
        let mut option = rumqttc::MqttOptions::new(id, host, 1883);
        option.set_keep_alive(std::time::Duration::from_secs(5));
        option.set_max_packet_size(1_000_000, 1_000_000);
        let (client, mut eventloop) = rumqttc::AsyncClient::new(option, 10);
        // Wait for a connection

        let mut eventloop = context.block_on_async(async move {
            loop {
                let notification = eventloop.poll().await?;
                match notification {
                    rumqttc::Event::Incoming(rumqttc::Packet::ConnAck(c)) => {
                        if c.code == rumqttc::ConnectReturnCode::Success {
                            info!("node: connected to mqtt");
                            break Ok(eventloop);
                        } else {
                            anyhow::bail!("Could not connect to mqtt");
                        }
                    }
                    _ => {}
                }
            }
        })?;

        // Create a channel to handle incoming messages
        let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();

        println!("Spawning event loop for mqtt messages");
        context.spawn_task(async move {
            while let Ok(msg) = eventloop.poll().await {
                match msg {
                    rumqttc::Event::Incoming(rumqttc::Packet::Publish(p)) => {
                        let payload_as_str = String::from_utf8(p.payload.to_vec())
                            .expect("couldn't convert payload");
                        if let Err(e) = sender.send((p.topic.to_string(), payload_as_str)) {
                            error!("mqtt Error sending message: {:?}", e);
                            break;
                        }
                    }
                    _ => {}
                }
            }
        });

        // Subscribe to a topic called "rusty_bind_command"
        let client = context.block_on_async(async move {
            client
                .subscribe("rusty_bind/command", rumqttc::QoS::AtLeastOnce)
                .await?;
            Ok::<_,anyhow::Error>(client)
        })?;

        Ok(RustPlugin { client, receiver })
    }
}
impl Plugin for RustPlugin {
    fn on_frame(&mut self, context: &Context, interface: &Interface) {
        println!("RustPlugin::on_frame");
        println!("  name: {}", interface.name());
        println!("  frame: {}", interface.frame());
        println!("  position: {:?}", interface.position());

        // Handle incoming messages
        while let Ok((topic, payload)) = self.receiver.try_recv() {
            println!("  Received message on {}: {}", topic, payload);
            let command = serde_json::from_str::<ShutdownCommand>(&payload);
            if let Ok(command) = command {
                command.handle(context, interface);
            } else {
                println!("  Could not parse message as command");
            }
        }

        // For funzies, publish a message to mqtt
        let data = FrameData {
            frame: interface.frame(),
            position: interface.position(),
        };
        let json = serde_json::to_string(&data).unwrap();
        self.client
            .try_publish("rusty_bind/frame", rumqttc::QoS::AtLeastOnce, false, json)
            .unwrap();

        // Spawn a task just for the fun of it
        context.spawn_task(async move {
            println!("  Spawning a task and sleeping");
            tokio::time::sleep(std::time::Duration::from_micros(1000)).await;
            println!("  Task completed");
        });
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
            println!("Shutting down from remote command");
            interface.shutdown();
        }
    }
}
