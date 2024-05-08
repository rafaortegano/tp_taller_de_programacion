use std::io::{Error, Read};

use crate::common::data_types::data_representation::{read_byte, read_two_byte_integer};

pub static CONNECT_PACKET: u8 = 0x10;
pub static CONNACK_PACKET: u8 = 0x20;
pub static _PUBLISH_PACKET: u8 = 0x30;
pub static _PUBACK_PACKET: u8 = 0x40;
pub static _PINGREQ_PACKET: u8 = 0xC0;
pub static _PINGRESP_PACKET: u8 = 0xD0;
pub static _DISCONNECT_PACKET: u8 = 0xE0;

pub struct PacketFixedHeader {
    pub packet_type: u8,
    pub remaining_length: u16, // This is the length of the Variable Header plus the length of the Payload. It is encoded as a Variable Byte Integer.
}

pub fn _create_publish_header_flags(dup_flag: u8, qos_level: u8, retain: u8) -> u8 {
    let mut type_and_flags = _PUBLISH_PACKET;
    type_and_flags |= dup_flag << 3;
    type_and_flags |= qos_level << 1;
    type_and_flags |= retain;
    type_and_flags
}

impl PacketFixedHeader {
    pub fn new(packet_type: u8, remaining_length: u16) -> Self {
        PacketFixedHeader {
            packet_type,
            remaining_length,
        }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();

        bytes.push(self.packet_type);
        bytes.extend_from_slice(&self.remaining_length.to_be_bytes());

        bytes
    }

    pub fn read_from(stream: &mut dyn Read) -> Result<Self, Error> {
        let packet_type = read_byte(stream)?;
        let remaining_length = read_two_byte_integer(stream)?;

        Ok(PacketFixedHeader::new(packet_type, remaining_length))
    }
}
