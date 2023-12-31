use std::future::Future;

use anyhow::Result;
use tracing::info;

/// A little wrapper around the stock mqtt to handle initialization
/// and to make the user simple.
pub struct MQTT {
    client: rumqttc::AsyncClient,
    receiver: tokio::sync::mpsc::UnboundedReceiver<(String, String)>,
}
impl MQTT {
    pub async fn new(
        id: &str,
        host: &str,
        port: u16,
    ) -> Result<(Self, impl Future<Output = Result<()>>)> {
        let mut option = rumqttc::MqttOptions::new(id, host, port);
        option.set_keep_alive(std::time::Duration::from_secs(5));
        option.set_max_packet_size(1_000_000, 1_000_000);
        let (client, mut eventloop) = rumqttc::AsyncClient::new(option, 10);
        // Wait for a connection

        // loop until the connection is complete
        loop {
            let notification = eventloop.poll().await?;
            match notification {
                rumqttc::Event::Incoming(rumqttc::Packet::ConnAck(c)) => {
                    if c.code == rumqttc::ConnectReturnCode::Success {
                        info!("node: connected to mqtt");
                        break;
                    } else {
                        anyhow::bail!("Could not connect to mqtt");
                    }
                }
                _ => {}
            }
        }

        // Create a channel to handle incoming messages
        let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();

        let event_loop = async move {
            println!("Spawning event loop for mqtt messages");
            while let Ok(msg) = eventloop.poll().await {
                match msg {
                    rumqttc::Event::Incoming(rumqttc::Packet::Publish(p)) => {
                        let payload_as_str = String::from_utf8(p.payload.to_vec())?;
                        sender.send((p.topic.to_string(), payload_as_str))?
                    }
                    _ => {}
                }
            }
            Ok::<_, anyhow::Error>(())
        };

        Ok((Self { client, receiver }, event_loop))
    }

    pub async fn subscribe(&mut self, topic: &str) -> Result<()> {
        self.client
            .subscribe(topic, rumqttc::QoS::AtLeastOnce)
            .await?;
        Ok(())
    }

    pub fn try_publish<P>(&mut self, topic: &str, payload: P) -> Result<()>
    where
        P: Into<Vec<u8>>,
    {
        self.client
            .try_publish(topic, rumqttc::QoS::AtLeastOnce, false, payload)?;
        Ok(())
    }

    pub fn try_recv(&mut self) -> Option<(String, String)> {
        self.receiver.try_recv().ok()
    }
}
