use std::{
    io::Error,
    sync::{mpsc::Receiver, Arc, Mutex},
    thread::{self, JoinHandle},
};

use logger::logger_handler::Logger;
use mqtt::client::{client_message::MqttClientMessage, mqtt_client::MqttClient};
use shared::{
    interfaces::incident_interface::IncidentInterface, models::cam_model::{cam::{Cam, CamState}, cam_list::CamList},
};

use crate::interface::run_interface;

/// ## MonitoringApp
///
/// Estructura que representa la aplicación de monitoreo
///
/// ### Atributos
/// - `client`: cliente MQTT
/// - `cam_list`: lista de cámaras
/// - `inc_interface`: interfaz de incidentes
/// - `log_path`: ruta del archivo de log
///
pub struct MonitoringApp {
    pub client: MqttClient,
    pub cam_list: Arc<Mutex<CamList>>,
    pub inc_interface: IncidentInterface,
    //pub log_path: String,
    pub logger: Logger,
    /* tiles: Tiles,
    map_memory: MapMemory, */
}

/// ## MonitoringHandler
///
/// Estructura que representa los manejadores de la aplicación de monitoreo
///
/// ### Atributos
/// - `broker_listener`: manejador del broker
/// - `message_handler`: manejador de mensajes
///
pub struct MonitoringHandler {
    pub broker_listener: JoinHandle<Result<(), Error>>,
    pub message_handler: JoinHandle<()>,
}

/// ### process_messages
///
/// Procesa los mensajes recibidos por el cliente MQTT
///
/// ### Parametros
/// - `receiver`: receptor de mensajes
/// - `system`: sistema de cámaras del monitoreo
///
fn process_messages(
    receiver: Receiver<MqttClientMessage>,
    system: Arc<Mutex<CamList>>,
) -> Result<JoinHandle<()>, Error> {
    let handler = thread::spawn(move || loop {
        for message_received in receiver.try_iter() {
            match message_received.topic.as_str() {
                "camaras" => {
                    let data = Cam::from_be_bytes(message_received.data);
                    let mut system_lock = system.lock().unwrap();
                    if let Some(cam) = system_lock.cams.iter_mut().find(|c| c.id == data.id) {
                        if data.state != CamState::Removed {
                            *cam = data;
                        } else {
                            system_lock.cams.retain(|c| c.id != data.id);
                        }
                    } else {
                        system_lock.cams.push(data);
                    }
                }
                "dron" => {
                    // cambiar estado
                }
                _ => {}
            }
            // leer el mensaje recibido y cambiar estados según corresponda
        }
    });

    Ok(handler)
}

impl MonitoringApp {
    /// ### new
    ///    
    /// Crea una nueva aplicación de monitoreo
    ///
    /// #### Parametros
    /// - `client`: cliente MQTT
    /// - `log_path`: ruta del archivo de log
    ///     
    pub fn new(client: MqttClient, logger_cpy: Logger) -> Self {
        let cam_list = Arc::new(Mutex::new(CamList::default()));

        Self {
            client,
            cam_list,
            inc_interface: IncidentInterface {
                editable: true,
                ..Default::default()
            },
            logger: logger_cpy,
            /* tiles: Tiles::new(OpenStreetMap, egui_ctx),
            map_memory: MapMemory::default(), */
        }
    }

    /// ### init
    ///
    /// Inicializa la aplicación de monitoreo
    ///
    /// #### Retorno
    /// Resultado de la inicialización
    ///
    pub fn init(mut self, logger: &Logger) -> Result<MonitoringHandler, Error> {
        let listener = self.client.run_listener()?;

        let handler = process_messages(listener.receiver, self.cam_list.clone())?;

        self.client.subscribe(vec!["camaras"], logger)?;

        match run_interface(self) {
            Ok(_) => Ok(MonitoringHandler {
                broker_listener: listener.handler,
                message_handler: handler,
            }),
            Err(e) => Err(Error::new(std::io::ErrorKind::Other, e.to_string())),
        }
    }
}
