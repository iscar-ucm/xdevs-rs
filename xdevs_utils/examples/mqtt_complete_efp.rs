use std::time::Duration;
use xdevs::{
    gpt::{ExperimentalFrame, Processor},
    simulation::rt::RootCoordinator,
};
use xdevs_utils::{
    dmt::{Coupling, DevsModel},
    mqtt::{LastWill, MqttCoupled, MqttHandler, QoS},
};

fn mqtt_coupled(
    root_topic: &str,
    id: &str,
    host: &str,
    port: u16,
) -> Vec<tokio::task::JoinHandle<()>> {
    // First, let's create the DEVS Model Tree (DMT) for the EFP model
    let mut p = DevsModel::new("processor");
    p.add_input("input_req");
    p.add_output("output_res");

    let mut ef = DevsModel::new("ef");
    ef.add_input("input_res");
    ef.add_output("output_req");

    let mut efp = DevsModel::new("efp");
    efp.add_component(p);
    efp.add_component(ef);

    efp.add_coupling(Coupling::ic("ef", "output_req", "processor", "input_req"))
        .unwrap();
    efp.add_coupling(Coupling::ic("processor", "output_res", "ef", "input_res"))
        .unwrap();

    let coupled = MqttCoupled::new(root_topic, id, host, port, efp);
    coupled.spawn()
}

#[tokio::main]
async fn main() {
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .finish();

    // use that subscriber to process traces emitted after this point
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let req_period = 3.;
    let proc_time = 1.;
    let obst_time = 50.;

    let time_scale = 1.;
    let max_jitter = None;
    let queue_size = 10;
    let window = None;
    let rt_config = RootCoordinatorConfig::new(
        time_scale,
        max_jitter,
        Some(queue_size),
        Some(queue_size),
        window,
    );

    let mqtt_root = "xdevs/efp";
    let mqtt_host = "0.0.0.0";
    let mqtt_port = 1883;

    let mut handles = mqtt_coupled(mqtt_root, "efp", mqtt_host, mqtt_port);

    let processor = Processor::new("processor", proc_time);

    let mut proc_sim = RootCoordinator::new(processor, rt_config);

    let mut proc_mqtt = MqttHandler::new(
        "xdevs/efp/components/processor",
        "processor",
        mqtt_host,
        mqtt_port,
    );
    let will = LastWill::new(
        "efp/components/processor",
        "good bye",
        QoS::AtMostOnce,
        false,
    );
    proc_mqtt
        .mqtt_options
        .set_keep_alive(Duration::from_secs(5))
        .set_last_will(will);
    handles.extend(proc_sim.spawn_handler(proc_mqtt));
    handles.push(tokio::task::spawn(proc_sim.simulate(obst_time + 10.)));

    let ef = ExperimentalFrame::new("ef", req_period, obst_time);
    let mut ef_sim = RootCoordinator::new(ef, rt_config);

    let mut ef_mqtt = MqttHandler::new("xdevs/efp/components/ef", "ef", mqtt_host, mqtt_port);
    let will = LastWill::new("efp/components/ef", "good bye", QoS::AtMostOnce, false);
    ef_mqtt
        .mqtt_options
        .set_keep_alive(Duration::from_secs(5))
        .set_last_will(will);
    handles.extend(ef_sim.spawn_handler(ef_mqtt));
    handles.push(tokio::task::spawn(ef_sim.simulate(obst_time + 10.)));

    for handle in handles {
        handle.await.unwrap();
    }
}
