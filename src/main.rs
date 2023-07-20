use rumqttc::{MqttOptions, Client, Packet};
use rumqttc::Event;


mod switch;
use switch::Switch;

fn main() {
    let mqttoptions = MqttOptions::new("mqtt-light-switches", "localhost", 1883);

    let (mut client, mut connection) = Client::new(mqttoptions, 10);
    
    let mut switches: Vec<Switch> = Vec::new();
    
    let switch1 = Switch::new(
        "bedroom/switch/ceiling_door/state",
        "bedroom/light/ceiling_door/state",
        "bedroom/light/ceiling_door/set"
    );

    let switch2 = Switch::new(
        "bedroom/switch/ceiling_window/state",
        "bedroom/light/ceiling_window/state",
        "bedroom/light/ceiling_window/set"
    );


    switches.push(switch1);
    switches.push(switch2);

    for switch in &switches {
        switch.add_subscriptions(&mut client);
    }

    for (_i, notification) in connection.iter().enumerate() {
        println!("Notification = {:?}", notification);
        let event = notification.unwrap();
        if let Event::Incoming(evt) = event {
            //println!("Incoming event = {:?}", evt);
            if let Packet::Publish(packet) = evt {
                let data_result = String::from_utf8(packet.payload.to_vec());
                match data_result {
                    Ok(data) => {
                        //println!("{:?}: {:?}", packet.topic, data);
                        for switch in switches.iter_mut() {
                            switch.process(&mut client, &packet, &data);
                        }    
                    },
                    Err(e) => {
                        println!("Error converting payload from {:?}", packet.topic);
                    }
                }
                // println!("publish = {:?}", data.topic);
                // println!("publish = {:?}", String::from_utf8(data.payload.to_vec()));
            }
        }
    }    
}

