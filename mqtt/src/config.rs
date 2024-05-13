use std::{fs::File, io::Error};

use crate::{
    common::utils::*,
    control_packets::{
        mqtt_connack::connack_properties::ConnackProperties,
        mqtt_connect::{connect_properties::ConnectProperties, payload::ConnectPayload},
        mqtt_packet::flags::flags_handler::*,
    },
};

pub struct ClientConfig {
    pub port: u16,
    pub ip: String,
    pub connect_properties: ConnectProperties,
    pub connect_payload: ConnectPayload,
}

impl ClientConfig {
    pub fn get_address(&self) -> String {
        let adress = format!("{}:{}", self.ip, self.port);
        adress
    }

    fn set_params(params: &[(String, String)]) -> Result<Self, Error> {
        // seteo los parametros del cliente:
        let mut port = 0;
        let mut ip = String::new();

        // Corroborar que le pasen los campos obligatorios
        let mut connect_properties = ConnectProperties::default();

        for param in params.iter() {
            match param.0.as_str() {
                "port" => {
                    port = match param.1.parse::<u16>() {
                        Ok(p) => p,
                        Err(_) => {
                            return Err(Error::new(
                                std::io::ErrorKind::InvalidData,
                                "Invalid parameter: Port",
                            ))
                        }
                    }
                }
                "ip" => ip = param.1.clone(),
                "protocol_name" => connect_properties.protocol_name = param.1.clone(),
                "protocol_version" => {
                    connect_properties.protocol_version = match param.1.parse::<u8>() {
                        Ok(p) => p,
                        Err(_) => {
                            return Err(Error::new(
                                std::io::ErrorKind::InvalidData,
                                "Invalid parameter: Protocol Version",
                            ))
                        }
                    }
                }
                "flag_clean_start" => {
                    connect_properties.connect_flags = match add_connect_flag_clean_start(
                        connect_properties.connect_flags,
                        param.1.clone(),
                    ) {
                        Ok(p) => p,
                        Err(e) => return Err(e),
                    }
                }
                "flag_will_flag" => {
                    connect_properties.connect_flags = match add_connect_flag_will_flag(
                        connect_properties.connect_flags,
                        param.1.clone(),
                    ) {
                        Ok(p) => p,
                        Err(e) => return Err(e),
                    }
                }
                "flag_will_qos" => {
                    connect_properties.connect_flags = match add_connect_flag_will_qos(
                        connect_properties.connect_flags,
                        param.1.clone(),
                    ) {
                        Ok(p) => p,
                        Err(e) => return Err(e),
                    }
                }
                "flag_will_retain" => {
                    connect_properties.connect_flags = match add_connect_flag_will_retain(
                        connect_properties.connect_flags,
                        param.1.clone(),
                    ) {
                        Ok(p) => p,
                        Err(e) => return Err(e),
                    }
                }
                "flag_password" => {
                    connect_properties.connect_flags = match add_connect_flag_password(
                        connect_properties.connect_flags,
                        param.1.clone(),
                    ) {
                        Ok(p) => p,
                        Err(e) => return Err(e),
                    }
                }
                "flag_username" => {
                    connect_properties.connect_flags = match add_connect_flag_username(
                        connect_properties.connect_flags,
                        param.1.clone(),
                    ) {
                        Ok(p) => p,
                        Err(e) => return Err(e),
                    }
                }
                "keep_alive" => {
                    connect_properties.keep_alive = match param.1.parse::<u16>() {
                        Ok(p) => p,
                        Err(_) => {
                            return Err(Error::new(
                                std::io::ErrorKind::InvalidData,
                                "Invalid parameter: Keep Alive",
                            ))
                        }
                    }
                }
                "session_expiry_interval" => {
                    connect_properties.session_expiry_interval = match param.1.parse::<u32>() {
                        Ok(p) => Some(p),
                        Err(_) => {
                            return Err(Error::new(
                                std::io::ErrorKind::InvalidData,
                                "Invalid parameter: Session Expiry Interval",
                            ))
                        }
                    }
                }
                "receive_maximum" => {
                    connect_properties.receive_maximum = match param.1.parse::<u16>() {
                        Ok(p) => Some(p),
                        Err(_) => {
                            return Err(Error::new(
                                std::io::ErrorKind::InvalidData,
                                "Invalid parameter: Receive Maximum",
                            ))
                        }
                    }
                }
                "maximum_packet_size" => {
                    connect_properties.maximum_packet_size = match param.1.parse::<u32>() {
                        Ok(p) => Some(p),
                        Err(_) => {
                            return Err(Error::new(
                                std::io::ErrorKind::InvalidData,
                                "Invalid parameter:  Maximum Packet Size",
                            ))
                        }
                    }
                }
                "topic_alias_maximum" => {
                    connect_properties.topic_alias_maximum = match param.1.parse::<u16>() {
                        Ok(p) => Some(p),
                        Err(_) => {
                            return Err(Error::new(
                                std::io::ErrorKind::InvalidData,
                                "Invalid parameter: Topic Alias Maximum",
                            ))
                        }
                    }
                }
                "request_response_information" => {
                    connect_properties.request_response_information =
                        match catch_true_false(&param.1) {
                            Ok(p) => Some(p),
                            Err(e) => return Err(e),
                        }
                }
                "request_problem_information" => {
                    connect_properties.request_problem_information =
                        match catch_true_false(&param.1) {
                            Ok(p) => Some(p),
                            Err(e) => return Err(e),
                        }
                }
                "authentication_method" => {
                    connect_properties.authentication_method = Some(param.1.clone())
                }
                "authentication_data" => {
                    connect_properties.authentication_data = Some(param.1.clone())
                }

                _ => {
                    return Err(Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Invalid parameter: Parameter not found",
                    ))
                }
            }
        }

        let connect_payload = ConnectPayload::default();
        // Faltan agregar los campos del payload desde el archivo de configuracion!

        Ok(ClientConfig {
            port,
            ip,
            connect_properties,
            connect_payload,
        })
    }

    pub fn from_file(file_path: String) -> Result<Self, Error> {
        let archivo_abierto: Option<File> = abrir_archivo(&file_path);
        let mut parametros = Vec::new();

        archivo_abierto.map(|archivo| match leer_archivo(&archivo) {
            None => None,
            Some(lineas_leidas) => {
                parametros = obtener_parametros_archivo(lineas_leidas, 2);
                Some(())
            }
        });

        ClientConfig::set_params(&parametros)
    }
}

pub struct ServerConfig {
    pub port: u16,
    pub ip: String,
    pub connack_properties: ConnackProperties,
}

impl ServerConfig {
    pub fn get_address(&self) -> String {
        let adress = format!("{}:{}", self.ip, self.port);
        adress
    }

    fn set_params(params: &[(String, String)]) -> Result<Self, Error> {
        // seteo los parametros del cliente:
        let mut port = 0;
        let mut ip = String::new();

        //chequear que tipo de parametros se le pasan

        let connack_properties = ConnackProperties::default();

        for param in params.iter() {
            match param.0.as_str() {
                "port" => {
                    port = match param.1.parse::<u16>() {
                        Ok(p) => p,
                        Err(_) => {
                            return Err(Error::new(
                                std::io::ErrorKind::InvalidData,
                                "Invalid parameter",
                            ))
                        }
                    }
                }
                "ip" => ip = param.1.clone(),

                _ => {
                    return Err(Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Invalid parameter",
                    ))
                }
            }
        }

        Ok(ServerConfig {
            port,
            ip,
            connack_properties,
        })
    }

    pub fn from_file(file_path: String) -> Result<Self, Error> {
        let archivo_abierto: Option<File> = abrir_archivo(&file_path);
        let mut parametros = Vec::new();

        archivo_abierto.map(|archivo| match leer_archivo(&archivo) {
            None => None,
            Some(lineas_leidas) => {
                parametros = obtener_parametros_archivo(lineas_leidas, 2);
                Some(())
            }
        });

        ServerConfig::set_params(&parametros)
    }
}