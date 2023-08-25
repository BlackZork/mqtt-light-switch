use std::str::FromStr;
use rumqttc::{Client, QoS, Publish, Connection, Event};
use log::*;

pub struct Switch {
    pub counter_topic: String,
    pub light_state_topic: String,
    pub light_command_topic: String,

    is_light_on: bool,
    counter_value: String
}

impl Switch {
    pub fn new(counter_topic: &str, light_state_topic: &str, light_command_topic: &str) -> Self {
        return Switch {
            counter_topic: String::from(counter_topic),
            light_state_topic: String::from(light_state_topic),
            light_command_topic: String::from(light_command_topic),
            is_light_on: false,
            counter_value: String::from_str("").unwrap()
        }
    }

    pub fn subscribe(&self, topic: &str, client: &mut Client, connection: &mut Connection) {
        client.subscribe(topic, QoS::AtLeastOnce).unwrap();
        // without loop subscribe hangs after 10 calls to client.subscribe()
        for (_i, notification) in connection.iter().enumerate() {
            match notification {
                Ok(event) => {
                    if let Event::Outgoing(_evt) = event {
                        debug!("Subscribed to {:?} ", topic);
                        break;
                    }
                },
                Err(_) => { break; }
            }
        }
    }

    pub fn add_subscriptions(&self, client: &mut Client, connection: &mut Connection) {
        self.subscribe(self.counter_topic.as_str(), client, connection);
        self.subscribe(self.light_state_topic.as_str(), client, connection);
    }

    pub fn process(&mut self, client: &mut Client, packet: &Publish, data: &str) {
        trace!("{:?}: {:?}", packet.topic, data);
        if packet.topic == self.light_state_topic {
            self.is_light_on = data == "1";
            debug!("Light {:?} initial state changed to {:?}", self.light_state_topic, self.is_light_on);
        }
        if packet.topic == self.counter_topic {
            if packet.retain {
                debug!("Switch {:?} initial value is {:?}", packet.topic, data);
                self.counter_value = String::from_str(data).unwrap();
            } else if self.counter_value != data {
                let payload;
                if self.is_light_on {
                    debug!("Light off for {:?}", packet.topic);
                    payload = "0";
                } else {
                    debug!("Light on for {:?}", packet.topic);
                    payload = "1";
                }
                client.publish(&self.light_command_topic, QoS::AtLeastOnce, false, payload).unwrap();
                self.counter_value = String::from_str(data).unwrap();
            }
        }
    }
}
