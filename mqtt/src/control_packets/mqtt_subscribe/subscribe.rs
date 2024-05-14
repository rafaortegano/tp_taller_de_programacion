use crate::control_packets::mqtt_packet::fixed_header::PacketFixedHeader;
use crate::control_packets::mqtt_packet::fixed_header::_SUBSCRIBE_PACKET;
use crate::control_packets::mqtt_packet::packet::generic_packet::PacketReceived;
use crate::control_packets::mqtt_packet::packet_properties::PacketProperties;
use crate::control_packets::{
    mqtt_packet::packet::generic_packet::Serialization,
    mqtt_subscribe::subscribe_properties::SubscribeProperties,
};
use std::io::Error;
use std::io::Read;
use std::io::Write;

/// ## SUBSCRIBE PACKET (Enviado por el cliente al servidor)
///
/// ### FIXED HEADER: 2 BYTES
/// Primer Byte:
/// 4 bits mas significativos: MQTT Control Packet type
///
/// Segundo Byte:
/// Remaining Length
/// El remaining length es el numero de bytes que quedan en el paquete despues del Fixed Header y que contienen el Variable Header y el Payload
///
///
/// ### VARIABLE HEADER:
/// Packet Identifier: 2 bytes
///
/// Property Length: Variable Byte Integer
/// PROPERTIES: Subscribe
/// 11 - 0x0B - Subscription Identifier - Variable Byte Integer (valor entre 1 y 268,435,455)
/// 38 - 0x26 - User Property - UTF-8 String Pair
///
///
/// ### PAYLOAD:
/// Contiene una lista de Topic Filters indicando los Topics a los cuales el cliente se quiere subscribir
/// Los Topic Filters DEBEN ser Strings UTF-8 validos
/// Cada Topic Filter debe ser seguido por el Subscriptions Options Byte
///
/// El packet SUBSCRIBE debe contener al menos un par Topic Filter + Subscriptions Options
///
/// El byte de Subscription Options contiene los siguientes bits:
/// Bits 0 y 1: QoS Level (Maximo QoS level el cual el server puede enviar mensajes de aplicacion al cliente)
/// Bit 2: No Local (Si es 1, mensajes de aplicacion no deben ser enviados a una conexion con el client ID igual al client ID de la conexion que publica)
/// Bit 3: Retain As Published (Si es 1, mensajes de aplicacion enviados mediante esta subscripcion mantienen el RETAIN flag con el que fueron publicados)
///     (Si es 0, el RETAIN flag es seteado a 0)
/// Bits 4 y 5: Retain Handling (Esta opcion especifica el envio de los mensajes de aplicacion retenidos cuando *se establece la subscripcion*)
///     0 - Send retained messages at the time of the subscribe
///     1 - Send retained messages at subscribe only if the subscription does not already exist
///     2 - Do not send retained messages at the time of the subscribe
/// Bits 6 y 7: Reserved (deben ser 0)
///
///
/// ### Consideraciones:
/// Cuando el servidor recibe un SUBSCRIBE PACKET, debe responder con un SUBACK PACKET con el mismo packet identifier
/// El servidor puede enviar PUBLISH PACKETS a los clientes antes de enviar el SUBACK PACKET
///
pub struct _Subscribe {
    pub properties: SubscribeProperties,
}

impl Serialization for _Subscribe {
    fn read_from(stream: &mut dyn Read, remaining_length: u16) -> Result<Self, Error> {
        let mut aux_buffer = vec![0; remaining_length as usize];
        stream.read_exact(&mut aux_buffer)?;
        let mut buffer = aux_buffer.as_slice();

        let properties = SubscribeProperties::read_from(&mut buffer)?;

        Ok(_Subscribe { properties })
    }

    fn write_to(&self, stream: &mut dyn Write) -> Result<(), Error> {
        let remaining_length = self.properties.size_of();

        let fixed_header = PacketFixedHeader::new(_SUBSCRIBE_PACKET, remaining_length);
        let fixed_header_bytes = fixed_header.as_bytes();

        stream.write_all(&fixed_header_bytes)?;

        let properties = self.properties.as_bytes()?;
        stream.write_all(&properties)?;

        Ok(())
    }

    fn packed_package(package: Self) -> PacketReceived {
        PacketReceived::Subscribe(Box::new(package))
    }
}

impl _Subscribe {
    pub fn _new(properties: SubscribeProperties) -> _Subscribe {
        _Subscribe { properties }
    }
}

#[cfg(test)]

mod test {
    use super::*;
    use crate::control_packets::mqtt_subscribe::subscribe_properties::SubscriptionType;

    #[test]
    fn test_subscribe_to_one_topic() {
        let properties = SubscribeProperties {
            packet_identifier: 1,
            subscription_identifier: Some(1),
            user_property: Some(("key".to_string(), "value".to_string())),
            topic_filters: vec![SubscriptionType {
                topic_filter: "topico1".to_string(),
                subscription_options: 0,
            }],
        };

        let subscribe = _Subscribe::_new(properties);

        //ESCRIBE EL PACKET EN EL BUFFER
        let mut bytes = Vec::new();
        subscribe.write_to(&mut bytes).unwrap();

        //LEE EL PACKET DEL BUFFER
        let mut buffer = bytes.as_slice();
        let subscribe_fixed_header = PacketFixedHeader::read_from(&mut buffer).unwrap();
        let subscribe =
            _Subscribe::read_from(&mut buffer, subscribe_fixed_header.remaining_length).unwrap();

        assert_eq!(subscribe.properties.packet_identifier, 1);
        assert_eq!(subscribe.properties.subscription_identifier, Some(1));
        assert_eq!(
            subscribe.properties.user_property,
            Some(("key".to_string(), "value".to_string()))
        );
        assert_eq!(subscribe.properties.topic_filters.len(), 1);
        assert_eq!(
            subscribe.properties.topic_filters[0].topic_filter,
            "topico1"
        );
        assert_eq!(
            subscribe.properties.topic_filters[0].subscription_options,
            0
        );
    }

    #[test]
    fn test_subscribe_to_multiple_topics() {
        let properties = SubscribeProperties {
            packet_identifier: 1,
            subscription_identifier: Some(1),
            user_property: Some(("key".to_string(), "value".to_string())),
            topic_filters: vec![
                SubscriptionType {
                    topic_filter: "topico1".to_string(),
                    subscription_options: 0,
                },
                SubscriptionType {
                    topic_filter: "topico2".to_string(),
                    subscription_options: 1,
                },
                SubscriptionType {
                    topic_filter: "topico3".to_string(),
                    subscription_options: 2,
                },
            ],
        };

        let subscribe = _Subscribe::_new(properties);

        //ESCRIBE EL PACKET EN EL BUFFER
        let mut bytes = Vec::new();
        subscribe.write_to(&mut bytes).unwrap();

        //LEE EL PACKET DEL BUFFER
        let mut buffer = bytes.as_slice();
        let subscribe_fixed_header = PacketFixedHeader::read_from(&mut buffer).unwrap();
        let subscribe =
            _Subscribe::read_from(&mut buffer, subscribe_fixed_header.remaining_length).unwrap();

        assert_eq!(subscribe.properties.packet_identifier, 1);
        assert_eq!(subscribe.properties.subscription_identifier, Some(1));
        assert_eq!(
            subscribe.properties.user_property,
            Some(("key".to_string(), "value".to_string()))
        );
        assert_eq!(subscribe.properties.topic_filters.len(), 3);
        assert_eq!(
            subscribe.properties.topic_filters[0].topic_filter,
            "topico1"
        );
        assert_eq!(
            subscribe.properties.topic_filters[0].subscription_options,
            0
        );
        assert_eq!(
            subscribe.properties.topic_filters[1].topic_filter,
            "topico2"
        );
        assert_eq!(
            subscribe.properties.topic_filters[1].subscription_options,
            1
        );
        assert_eq!(
            subscribe.properties.topic_filters[2].topic_filter,
            "topico3"
        );
        assert_eq!(
            subscribe.properties.topic_filters[2].subscription_options,
            2
        );
    }
    #[test]
    fn test_subscribe_with_empty_optional_fields() {
        let properties = SubscribeProperties {
            packet_identifier: 1,
            subscription_identifier: None,
            user_property: None,
            topic_filters: vec![
                SubscriptionType {
                    topic_filter: "topico1".to_string(),
                    subscription_options: 0,
                },
                SubscriptionType {
                    topic_filter: "topico2".to_string(),
                    subscription_options: 1,
                },
                SubscriptionType {
                    topic_filter: "topico3".to_string(),
                    subscription_options: 2,
                },
            ],
        };

        let subscribe = _Subscribe::_new(properties);

        //ESCRIBE EL PACKET EN EL BUFFER
        let mut bytes = Vec::new();
        subscribe.write_to(&mut bytes).unwrap();

        //LEE EL PACKET DEL BUFFER
        let mut buffer = bytes.as_slice();
        let subscribe_fixed_header = PacketFixedHeader::read_from(&mut buffer).unwrap();
        let subscribe =
            _Subscribe::read_from(&mut buffer, subscribe_fixed_header.remaining_length).unwrap();

        assert_eq!(subscribe.properties.packet_identifier, 1);
        assert_eq!(subscribe.properties.subscription_identifier, None);
        assert_eq!(subscribe.properties.user_property, None);
        assert_eq!(subscribe.properties.topic_filters.len(), 3);
        assert_eq!(
            subscribe.properties.topic_filters[0].topic_filter,
            "topico1"
        );
        assert_eq!(
            subscribe.properties.topic_filters[0].subscription_options,
            0
        );
        assert_eq!(
            subscribe.properties.topic_filters[1].topic_filter,
            "topico2"
        );
        assert_eq!(
            subscribe.properties.topic_filters[1].subscription_options,
            1
        );
        assert_eq!(
            subscribe.properties.topic_filters[2].topic_filter,
            "topico3"
        );
        assert_eq!(
            subscribe.properties.topic_filters[2].subscription_options,
            2
        );
    }
}