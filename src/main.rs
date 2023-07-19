use rumqttc::{MqttOptions, Client, QoS};

fn main() {
    let mqttoptions = MqttOptions::new("mqtt-light-switches", "zork.pl", 1883);

    let (mut client, mut connection) = Client::new(mqttoptions, 10);
    
    client.subscribe("bedroom/switch/ceiling_door/state", QoS::AtLeastOnce).unwrap();

    for (_i, notification) in connection.iter().enumerate() {
        println!("Notification = {:?}", notification);
    }    
}
