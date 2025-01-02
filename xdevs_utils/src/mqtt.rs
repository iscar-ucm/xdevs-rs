use crate::dmt::DevsModel;
use rumqttc::{AsyncClient, EventLoop, Packet, Publish};
pub use rumqttc::{LastWill, MqttOptions, QoS};
use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use xdevs::{
    simulation::rt::{input::InputSender, output::OutputReceiver, Handler},
    Event,
};

#[derive(Clone)]
pub struct MqttHandler {
    root_topic: String,
    pub mqtt_options: MqttOptions,
    pub client_cap: usize,
    pub input_qos: QoS,
    pub output_qos: QoS,
    pub output_retain: bool,
}

impl MqttHandler {
    pub fn new<R: Into<String>, S: Into<String>, T: Into<String>>(
        root_topic: R,
        id: S,
        host: T,
        port: u16,
    ) -> Self {
        Self {
            root_topic: root_topic.into(),
            mqtt_options: MqttOptions::new(id, host, port),
            client_cap: 10,
            input_qos: QoS::AtMostOnce,
            output_qos: QoS::AtLeastOnce,
            output_retain: true,
        }
    }
}

impl Handler for MqttHandler {
    fn spawn(
        self,
        input_tx: Option<InputSender>,
        output_rx: Option<OutputReceiver>,
    ) -> Vec<tokio::task::JoinHandle<()>> {
        let mut handles = Vec::new();
        if input_tx.is_none() && output_rx.is_none() {
            tracing::warn!("no input or output handler provided. Exiting.");
        } else {
            let root_topic = self.root_topic;
            let (client, eventloop) = AsyncClient::new(self.mqtt_options, self.client_cap);

            let client_config = ClientConfig {
                input: if input_tx.is_some() {
                    Some(self.input_qos)
                } else {
                    None
                },
                output: output_rx.map(|rx| (self.output_qos, self.output_retain, rx)),
            };
            handles.push(tokio::task::spawn(client_thread(
                client,
                root_topic.clone(),
                client_config,
            )));
            handles.push(tokio::task::spawn(eventloop_thread(eventloop, input_tx)));
        };

        handles
    }
}

struct ClientConfig {
    input: Option<QoS>,
    output: Option<(QoS, bool, OutputReceiver)>,
}

async fn client_thread(client: AsyncClient, root_topic: String, config: ClientConfig) {
    let input_topic = format!("{root_topic}/input");
    let output_topic = format!("{root_topic}/output");

    if let Some(input_qos) = config.input {
        tracing::info!("subscribing to MQTT topic {input_topic}/+");
        if let Err(e) = client
            .subscribe(format!("{input_topic}/+"), input_qos)
            .await
        {
            tracing::error!("failed to subscribe to MQTT topic {input_topic}/+: {e:?}");
            return;
        }
    }

    if let Some((output_qos, output_retain, mut output_rx)) = config.output {
        loop {
            match output_rx.recv().await {
                Ok(event) => {
                    let port = event.port();
                    let value = event.value();
                    tracing::info!("publishing value {value} to MQTT topic {output_topic}/{port}");

                    if let Err(e) = client
                        .publish(
                            format!("{output_topic}/{port}"),
                            output_qos,
                            output_retain,
                            value,
                        )
                        .await
                    {
                        tracing::warn!("failed to publish to MQTT topic: {e:?}.");
                    }
                }
                Err(e) => {
                    tracing::error!("output handler dropped: {e:?}. Disconnecting MQTT client.");
                    client.disconnect().await.ok();
                    break;
                }
            }
        }
    }
}

async fn eventloop_thread(mut eventloop: EventLoop, input_tx: Option<InputSender>) {
    loop {
        match eventloop.poll().await {
            Ok(notif) => {
                tracing::debug!("MQTT event notification: {notif:?}");
                if let Some(input_tx) = &input_tx {
                    if let rumqttc::Event::Incoming(Packet::Publish(packet)) = notif {
                        let port = packet.topic.split('/').last().unwrap().to_string();

                        let value = match String::from_utf8(packet.payload.to_vec()) {
                            Ok(string) => string,
                            Err(e) => {
                                tracing::warn!("Failed to convert payload to UTF8 String: {e}.");
                                continue;
                            }
                        };
                        match input_tx.send(Event::new(port, value)).await {
                            Ok(_) => {}
                            Err(e) => {
                                tracing::error!(
                                    "input handler dropped: {e:?}. Disconnecting MQTT client."
                                );
                                break;
                            }
                        }
                    }
                }
            }
            Err(e) => {
                tracing::error!("MQTT eventloop error: {:?}", e);
                break;
            }
        }
    }
}

pub struct MqttCoupled {
    config: MqttHandler,
    model: DevsModel,
}

impl MqttCoupled {
    pub fn new<R: Into<String>, S: Into<String>, T: Into<String>>(
        root_topic: R,
        id: S,
        host: T,
        port: u16,
        model: DevsModel,
    ) -> Self {
        assert!(model.is_coupled());
        let config = MqttHandler::new(root_topic, id, host, port);
        Self { config, model }
    }
}

impl MqttCoupled {
    pub fn spawn(self) -> Vec<tokio::task::JoinHandle<()>> {
        let mut handles = Vec::new();
        let topic_map = self.build_topic_map();
        let (config, _) = (self.config, self.model);
        let (sender, receiver) = channel(10);
        let (client, eventloop) = AsyncClient::new(config.mqtt_options, config.client_cap);

        handles.push(tokio::task::spawn(coupled_client_thread(
            client,
            config.input_qos,
            config.output_qos,
            config.output_retain,
            receiver,
            topic_map,
        )));
        handles.push(tokio::task::spawn(coupled_eventloop_thread(
            eventloop, sender,
        )));
        handles
    }

    fn build_topic_map(&self) -> HashMap<String, Vec<String>> {
        let mut topic_map = HashMap::new();
        let root_topic = self.root_topic.clone();
        for (src, dst) in self.model.components.as_ref().unwrap().couplings.iter() {
            let port = &src.port;
            let src_topic = match src.component.as_ref() {
                Some(component) => format!("{root_topic}/components/{component}/output/{port}"),
                None => format!("{root_topic}/input/{port}"),
            };
            let set: Vec<_> = dst
                .iter()
                .map(|dst| {
                    let port = &dst.port;
                    match dst.component.as_ref() {
                        Some(component) => {
                            format!("{root_topic}/components/{component}/input/{port}")
                        }
                        None => format!("{root_topic}/output/{port}"),
                    }
                })
                .collect();
            topic_map.insert(src_topic, set);
        }
        topic_map
    }
}

impl Deref for MqttCoupled {
    type Target = MqttHandler;

    fn deref(&self) -> &Self::Target {
        &self.config
    }
}

impl DerefMut for MqttCoupled {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.config
    }
}

async fn coupled_client_thread(
    client: AsyncClient,
    input_qos: QoS,
    output_qos: QoS,
    output_retain: bool,
    mut input_rx: Receiver<Publish>,
    topic_map: HashMap<String, Vec<String>>,
) {
    for (topic, _) in topic_map.iter() {
        tracing::info!("subscribing to MQTT topic {topic}.");
        if let Err(e) = client.subscribe(topic.clone(), input_qos).await {
            tracing::error!("failed to subscribe to MQTT topic {topic}: {e:?}.");
            return;
        }
    }

    loop {
        match input_rx.recv().await {
            Some(publish) => {
                let topic = publish.topic;
                let payload = publish.payload;
                tracing::info!("received value from MQTT topic {topic}.");
                match topic_map.get(&topic) {
                    Some(topics) => {
                        for topic in topics.iter() {
                            tracing::info!("publishing value to MQTT topic {topic}");
                            if let Err(e) = client
                                .publish(topic, output_qos, output_retain, payload.clone())
                                .await
                            {
                                tracing::warn!("failed to publish to MQTT topic: {e:?}.");
                            }
                        }
                    }
                    None => {
                        tracing::error!("no mapping found for MQTT topic {topic}.");
                    }
                }
            }
            None => {
                tracing::error!("input handler dropped. Disconnecting MQTT client.");
                client.disconnect().await.ok();
                break;
            }
        }
    }
}

async fn coupled_eventloop_thread(mut eventloop: EventLoop, sender: Sender<Publish>) {
    loop {
        match eventloop.poll().await {
            Ok(notif) => {
                tracing::debug!("MQTT event notification: {notif:?}");
                if let rumqttc::Event::Incoming(Packet::Publish(packet)) = notif {
                    match sender.send(packet).await {
                        Ok(_) => {}
                        Err(e) => {
                            tracing::error!(
                                "output handler dropped: {e:?}. Disconnecting MQTT client."
                            );
                            break;
                        }
                    }
                }
            }
            Err(e) => {
                tracing::error!("MQTT eventloop error: {:?}", e);
                break;
            }
        }
    }
}
