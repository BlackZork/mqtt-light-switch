use rumqttc::{MqttOptions, Client, Packet};
use rumqttc::Event;
use yaml_rust::YamlLoader;
use std::fs::File;
use std::io::Read;


mod switch;
use switch::Switch;

fn main() {
    let mqttoptions = MqttOptions::new("mqtt-light-switches", "zork.pl", 1883);

    let (mut client, mut connection) = Client::new(mqttoptions, 10);
    
    let mut switches: Vec<Switch> = read_config();

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
                        println!("Error converting payload from {:?}: {:?}", packet.topic, e);
                    }
                }
                // println!("publish = {:?}", data.topic);
                // println!("publish = {:?}", String::from_utf8(data.payload.to_vec()));
            }
        }
    }    
}


fn read_config() -> Vec<Switch> {
    let mut config_file = File::open("/etc/mqtt-light-switch.yaml").expect("Unable to open config file");
    let mut config_data = String::new();
    config_file.read_to_string(&mut config_data).expect("Cannot read config file");
    let docs = YamlLoader::load_from_str(&config_data).unwrap();

    let doc = &docs[0];
    let cf_switches = doc["switches"].as_vec().unwrap();


    let mut switches: Vec<Switch> = Vec::new();
    for cf_switch in cf_switches {
        let s = Switch::new(
            cf_switch["counter_topic"].as_str().unwrap(),
            cf_switch["light_state_topic"].as_str().unwrap(),
            cf_switch["light_command_topic"].as_str().unwrap()
        );
        println!("Configured light switch {:?}", s.light_state_topic);
        switches.push(s);
    }

    return switches;
}
