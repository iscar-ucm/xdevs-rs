use xdevs_utils::{
    dmt::{Coupling, DevsModel},
    mqtt::MqttCoupled,
};

#[tokio::main]
async fn main() {
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    // First, let's create the DEVS Model Tree (DMT) to describe the structure of the EFP model
    let mut p = DevsModel::new("p");
    p.add_input("input_req");
    p.add_output("output_res");
    let mut ef = DevsModel::new("ef");
    ef.add_input("input_res");
    ef.add_output("output_req");
    let mut efp = DevsModel::new("efp");
    efp.add_component(p);
    efp.add_component(ef);
    efp.add_coupling(Coupling::ic("ef", "output_req", "p", "input_req"))
        .unwrap();
    efp.add_coupling(Coupling::ic("p", "output_res", "ef", "input_res"))
        .unwrap();

    // Next, we will create the MQTT client that propagates events through the DEVS model
    let mqtt_root = "xdevs/efp";
    let mqtt_host = "0.0.0.0";
    let mqtt_port = 1883;
    let coupled = MqttCoupled::new(mqtt_root, "efp", mqtt_host, mqtt_port, efp);

    for handle in coupled.spawn() {
        handle.await.unwrap();
    }
}
