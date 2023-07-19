use rumqttc::{Client, QoS};
pub struct Switch {
    pub counter_topic: String,
    pub light_state_topic: String,
    pub light_command_topic: String,

    is_light_on: bool,
    counter_value: i32
}

impl Switch {
    pub fn new(counter_topic: &str, light_state_topic: &str, light_command_topic: &str) -> Self {
        return Switch {
            counter_topic: String::from(counter_topic),
            light_state_topic: String::from(light_state_topic),
            light_command_topic: String::from(light_command_topic),
            is_light_on: false,
            counter_value: 0
        }
    }

    pub fn add_subscriptions(&self, client: &mut Client) {
        client.subscribe(self.counter_topic.as_str(), QoS::AtLeastOnce).unwrap();
        client.subscribe(self.light_state_topic.as_str(), QoS::AtLeastOnce).unwrap();
    }
}
