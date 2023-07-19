use rumqttc::{MqttOptions, Client};
use rumqttc::Event::{Incoming, Outgoing};

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
        println!("Notification = {:?}", notification);
        let event = notification.unwrap();
        match event {
            Incoming(evt) => { println!("Incoming event = {:?}", evt) }
            Outgoing(evt) => { println!("Outgoing event = {:?}", evt) }
        }
    }    
}
