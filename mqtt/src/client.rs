use std::{io::Error, net::TcpStream};

use crate::{config::ClientConfig, control_packets::mqtt_connect::connect::Connect};

pub struct MqttClient {
    _id: String,
    _config: ClientConfig,
}

impl MqttClient {
    pub fn new(client_id: String, config: ClientConfig) -> Result<Self, Error> {
        let mut socket = TcpStream::connect(config.get_address())?;

        let connection = Connect::new(&client_id, &config.connect_properties)?;

        connection.write_to(&mut socket)?;

        Ok(MqttClient {
            _id: client_id,
            _config: config,
        })
    }
}
