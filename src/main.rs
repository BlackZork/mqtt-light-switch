use rumqttc::{MqttOptions, Client, Packet};
use rumqttc::Event;


mod switch;
use switch::Switch;

fn main() {
    let mqttoptions = MqttOptions::new("mqtt-light-switches", "zork.pl", 1883);

    let (mut client, mut connection) = Client::new(mqttoptions, 10);
    
    let mut switches: Vec<Switch> = Vec::new();
    
    let switch1 = Switch::new(
        "bedroom/switch/ceiling_door/state",
        "bedroom/light/ceiling_door/state",
        "bedroom/light/ceiling_door/set"
    );

    switches.push(switch1);

    for switch in switches {
        switch.add_subscriptions(&mut client);
    }

    for (_i, notification) in connection.iter().enumerate() {
        //println!("Notification = {:?}", notification);
        let event = notification.unwrap();
        if let Event::Incoming(evt) = event {
            //println!("Incoming event = {:?}", evt);
            if let Packet::Publish(data) = evt {
                println!("publish = {:?}", data.topic);
            }
        }
    }    
}
