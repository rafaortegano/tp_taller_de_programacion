use std::{
    env::args,
    fs,
    io::{self, Error},
    process,
    sync::{mpsc::Receiver, Arc, Mutex},
    thread::{self, JoinHandle}, time::Duration,
};

use logger::logger_handler::{create_logger_handler, Logger};
use mqtt::{
    client::{client_message::MqttClientMessage, mqtt_client::MqttClient},
    config::{client_config::ClientConfig, mqtt_config::Config},
};
use shared::models::{drone_model::drone::Drone, inc_model::incident::Incident};
use walkers::Position;

pub fn process_messages(
    client: &mut MqttClient,
    receiver: Receiver<MqttClientMessage>,
    drone: Arc<Mutex<Drone>>,
    logger: Logger,
) -> Result<JoinHandle<()>, Error> {
    let mut client = client.clone();
    let handler = thread::spawn(move || loop {
        for message_received in receiver.try_iter() {
            if message_received.topic.as_str() == "inc" {
                let incident = Incident::from_be_bytes(message_received.data);
                drone
                    .lock()
                    .unwrap()
                    .process_incident(&mut client, incident.clone(), &logger);
            } else if message_received.topic.as_str() == "drone" {
                let drone_received = Drone::from_be_bytes(&message_received.data);
                if drone_received.id == drone.lock().unwrap().id {
                    continue;
                }
                drone
                    .lock()
                    .unwrap()
                    .process_drone_message(&mut client, drone_received, &logger);
            }
        }
    });
    Ok(handler)
}

fn main() -> Result<(), Error> {
    let args: Vec<String> = args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <path_to_config_file>", args[0]);
        process::exit(1);
    }

    let contents = fs::read_to_string(&args[1])?;
    let mut config_path = String::new();

    let mut distancia_maxima_alcance: f64 = 0.0;
    let mut duracion_de_bateria: f64 = 0.0;
    let mut initial_lat: f64 = 0.0;
    let mut initial_lon: f64 = 0.0;
    let mut charging_station_lat: f64 = 0.0;
    let mut charging_station_lon: f64 = 0.0;
    let mut id = 0;
    let mut db_path = String::new();
    for line in contents.lines() {
        let parts: Vec<&str> = line.split(':').collect();
        match parts[0].trim() {
            "distancia_maxima_alcance" => {
                distancia_maxima_alcance = parts[1].trim().parse().map_err(|_| {
                    io::Error::new(io::ErrorKind::InvalidData, "Invalid range_alert value")
                })?
            }
            "duracion_de_bateria" => {
                duracion_de_bateria = parts[1].trim().parse().map_err(|_| {
                    io::Error::new(io::ErrorKind::InvalidData, "Invalid battery duration value")
                })?
            }
            "initial_lat" => {
                initial_lat = parts[1]
                    .trim()
                    .parse()
                    .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid latitude"))?
            }
            "initial_lon" => {
                initial_lon = parts[1]
                    .trim()
                    .parse()
                    .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid longitude"))?
            }
            "charging_station_lat" => {
                charging_station_lat = parts[1]
                    .trim()
                    .parse()
                    .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid latitude"))?
            }
            "charging_station_lon" => {
                charging_station_lon = parts[1]
                    .trim()
                    .parse()
                    .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid longitude"))?
            }
            "id" => {
                id = parts[1]
                    .trim()
                    .parse()
                    .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid id"))?
            }
            "db_path" => {
                db_path = parts[1].trim().to_string();
            }
            "mqtt_drone_config" => {
                config_path = parts[1].trim().to_string();
            }
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Invalid configuration file",
                ))
            }
        }
    }

    let initial_pos = Position::from_lat_lon(initial_lat, initial_lon);
    let charging_station_pos = Position::from_lat_lon(charging_station_lat, charging_station_lon);

    let drone = Drone::init(
        id,
        distancia_maxima_alcance,
        duracion_de_bateria,
        initial_pos,
        charging_station_pos,
        db_path,
    )?;

    let config = ClientConfig::from_file(String::from(config_path))?;

    let logger_handler = create_logger_handler(&config.general.log_path)?;
    let logger = logger_handler.get_logger();

    let mut client = match MqttClient::init(config) {
        Ok(r) => r,
        Err(e) => {
            logger.close();
            logger_handler.close();
            return Err(e);
        }
    };

    match client.subscribe(vec!["inc"], &logger) {
        Ok(r) => r,
        Err(e) => {
            logger.close();
            logger_handler.close();
            return Err(e);
        }
    };

    match client.subscribe(vec!["drone"], &logger) {
        Ok(r) => r,
        Err(e) => {
            logger.close();
            logger_handler.close();
            return Err(e);
        }
    };

    client
        .publish(drone.as_bytes(), "drone".to_string(), &logger)
        .unwrap();
    let drone_ref = Arc::new(Mutex::new(drone));

    let listener = match client.run_listener() {
        Ok(r) => r,
        Err(e) => {
            logger.close();
            logger_handler.close();
            return Err(e);
        }
    };

    let process_message_handler: JoinHandle<()> =
        match process_messages(&mut client, listener.receiver, drone_ref.clone(), logger.clone()) {
            Ok(r) => r,
            Err(e) => {
                logger.close();
                logger_handler.close();
                return Err(e);
            }
        };
    
        let _ = {
            let drone_ref = drone_ref.clone();
            let logger = logger.clone();
            thread::spawn(move || loop {
                thread::sleep(Duration::from_secs(10));
                let mut drone = drone_ref.lock().unwrap();
                drone.discharge(&mut client, logger.clone());
                println!("Drone battery: {}", drone.nivel_de_bateria);
            })
        };
    logger.close();
    logger_handler.close();

    listener.handler.join().unwrap()?;
    process_message_handler.join().unwrap();
    Ok(())
}
