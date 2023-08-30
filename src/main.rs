use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::path::Path;
use std::thread;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use rumqttc::{MqttOptions, Client, Packet};
use rumqttc::Event;
use yaml_rust::{Yaml, YamlLoader};
use clap::Parser;
use log::*;
use signal_hook::{consts::SIGTERM, iterator::Signals};



mod switch;
use switch::Switch;


#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8
}

struct MqttConfig {
    host: String,
    port: u16
}

struct LogConfig {
    level: String
}

fn main() {
    let cli = Cli::parse();

    let verbosity: usize = 2 + usize::from(cli.verbose);

    let mut log_handler = stderrlog::new();
    log_handler.module(module_path!());
    if cli.verbose != 0 {
        log_handler.verbosity(verbosity);
        log_handler.init().unwrap();
    }

    let mut config_path = Path::new("/etc/mqtt-light-switch.yaml");
    if let Some(arg_config_path) = cli.config.as_deref() {
        config_path = arg_config_path;
    }

    info!("Using configuration file {:?}", config_path);

    let config = read_config(config_path); 

    if cli.verbose == 0 {
        let log_config = read_log_config(&config);
        let new_level = log::LevelFilter::from_str(log_config.level.as_str()).unwrap();
        log_handler.verbosity(new_level);
        log_handler.init().unwrap();
    }

    let mqtt_config = read_mqtt_config(&config);
    let switches: Vec<Switch> = read_switch_config(&config);

    info!("Starting switch action loop for {:?} switches", switches.len());
    do_work(mqtt_config, switches);
    info!("Finished");
}

fn read_config(config_path: &Path) -> Yaml {
    let mut config_file = File::open(config_path).expect("Unable to open config file");
    let mut config_data = String::new();
    config_file.read_to_string(&mut config_data).expect("Cannot read config file");
    let docs = YamlLoader::load_from_str(&config_data).unwrap();

    let doc = &docs[0];
    return doc.clone();
}

fn read_mqtt_config(doc: &Yaml) -> MqttConfig {
    let conf = &doc["mqtt_config"];

    return MqttConfig {
        host: String::from(conf["host"].as_str().unwrap()),
        port: u16::try_from(conf["port"].as_i64().unwrap()).unwrap()
    }
}

fn read_log_config(doc: &Yaml) -> LogConfig {
    let conf = &doc["log"];

    return LogConfig { 
        level: String::from(conf["level"].as_str().unwrap())
    }
}

fn read_switch_config(doc: &Yaml) -> Vec<Switch> {
    let cf_switches = doc["switches"].as_vec().unwrap();

    let mut switches: Vec<Switch> = Vec::new();
    for cf_switch in cf_switches {
        let s = Switch::new(
            cf_switch["counter_topic"].as_str().unwrap(),
            cf_switch["light_state_topic"].as_str().unwrap(),
            cf_switch["light_command_topic"].as_str().unwrap()
        );
        trace!("Configured light switch {:?}", s.light_state_topic);
        switches.push(s);
    }

    return switches;
}

fn do_work(mqtt_config: MqttConfig, switches: Vec<Switch>) {
    let mqttoptions = MqttOptions::new("mqtt-light-switches", mqtt_config.host, mqtt_config.port);

    let (client, mut connection) = Client::new(mqttoptions, 10);
    

    let mut signals = Signals::new(&[SIGTERM]).unwrap();

    let client_arc = Arc::new(Mutex::new(client));
    let thread_client = client_arc.clone();

    let switches_arc = Arc::new(Mutex::new(switches));
    let thread_switches = switches_arc.clone();

    let mqtt_loop = thread::spawn(move || {
        for (_i, notification) in connection.iter().enumerate() {
            trace!("Notification = {:?}", notification);
            match notification {
                Ok(event) => {
                    if let Event::Incoming(evt) = event {
                        //println!("Incoming event = {:?}", evt);
                        if let Packet::Publish(packet) = evt {
                            let data_result = String::from_utf8(packet.payload.to_vec());
                            match data_result {
                                Ok(data) => {
                                    trace!("{:?}: {:?}", packet.topic, data);
                                    let mut switches_shared = thread_switches.lock().unwrap();
                                    for switch in switches_shared.iter_mut() {
                                        let mut client_shared = thread_client.lock().unwrap();
                                        switch.process(&mut client_shared, &packet, &data);
                                    }    
                                },
                                Err(e) => {
                                    error!("Error converting payload from {:?}: {:?}", packet.topic, e);
                                }
                            }
                        }
                    }
                },
                Err(_) => { break; }
            }
        }    
    });

    // must be done after main loop is started, because subscribe needs main loop
    // processing
    {
        let mut switches_shared = switches_arc.lock().unwrap();
        let mut client_shared = client_arc.lock().unwrap();
        for switch in switches_shared.iter_mut() {
            switch.add_subscriptions(&mut client_shared);
        }
    }


    for sig in signals.forever() {
        info!("Recevied signal {:?}", sig);
        if sig == SIGTERM {
            //TODO how to use moved value
            let mut client_shared = client_arc.lock().unwrap();
            client_shared.disconnect().unwrap();
            break;
        }
    }

    debug!("Joining processing thread");
    mqtt_loop.join().unwrap(); 
}