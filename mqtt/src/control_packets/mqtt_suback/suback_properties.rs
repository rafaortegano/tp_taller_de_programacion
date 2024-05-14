use crate::control_packets::mqtt_packet::{packet_property::*, packet_properties::PacketProperties, variable_header_properties::VariableHeaderProperties};

use std::io::Error;

pub struct _SubackProperties {
    pub packet_identifier: u16,
    pub reason_string: Option<String>,
    pub user_property: Option<(String, String)>,

    pub reason_codes: Vec<u8>, //Payload
}

impl Default for _SubackProperties {
    fn default() -> _SubackProperties {
        _SubackProperties {
            packet_identifier: 0,
            reason_string: None,
            user_property: None,
            reason_codes: Vec::new(),
        }
    }
}

impl Clone for _SubackProperties {
    fn clone(&self) -> Self {
        _SubackProperties {
            packet_identifier: self.packet_identifier,
            reason_string: self.reason_string.clone(),
            user_property: self.user_property.clone(),
            
            reason_codes: self.reason_codes.clone(),
        }
    }
}

impl PacketProperties for _SubackProperties {
   
    fn variable_props_size(&self) -> u16 {
        let header = self.as_variable_header_properties().unwrap();
        header.properties.len() as u16
    }    
    fn size_of(&self) -> u16 {
        let variable_props = self.as_variable_header_properties().unwrap();
        let fixed_props_size = std::mem::size_of::<u16>();

        let mut payload_size = 0;

        for _ in &self.reason_codes {
            payload_size += std::mem::size_of::<u8>();
        }
        fixed_props_size as u16 + variable_props.bytes_length + payload_size as u16
    }

    fn as_variable_header_properties(&self) -> Result<VariableHeaderProperties, Error> {
        let mut variable_props = VariableHeaderProperties::new();

        if let Some(value) = self.reason_string.clone() {
            variable_props.add_utf8_string_property(REASON_STRING, value)?;
        }
        if let Some(user_property) = self.user_property.clone() {
            variable_props.add_utf8_pair_string_property(
                USER_PROPERTY,
                user_property.0,
                user_property.1,
            )?;
        }

        Ok(variable_props)
    }
    fn as_bytes(&self) -> Result<Vec<u8>, std::io::Error> {
        todo!()
    }

    fn read_from(stream: &mut dyn std::io::Read) -> Result<Self, std::io::Error> {
        todo!()
    }
}