use std::io::{BufRead, Error, Write};

use byteorder::LittleEndian;

use crate::implementation::{
    rtps::messages::{
        overall_structure::RtpsSubmessageHeader, RtpsMessageRead, RtpsMessageWrite,
        RtpsSubmessageReadKind, RtpsSubmessageWriteKind,
    },
    rtps_udp_psm::mapping_traits::{
        MappingReadByteOrderInfoInData, MappingReadByteOrdered, MappingWriteByteOrderInfoInData,
        MappingWriteByteOrdered,
    },
};

use super::submessages::submessage_header::{
    ACKNACK, DATA, DATA_FRAG, GAP, HEARTBEAT, HEARTBEAT_FRAG, INFO_DST, INFO_REPLY, INFO_SRC,
    INFO_TS, NACK_FRAG, PAD,
};

impl MappingWriteByteOrderInfoInData for RtpsSubmessageWriteKind<'_> {
    fn mapping_write_byte_order_info_in_data<W: Write>(&self, mut writer: W) -> Result<(), Error> {
        match self {
            RtpsSubmessageWriteKind::AckNack(s) => {
                s.mapping_write_byte_order_info_in_data(&mut writer)?
            }
            RtpsSubmessageWriteKind::Data(s) => {
                s.mapping_write_byte_order_info_in_data(&mut writer)?
            }
            RtpsSubmessageWriteKind::DataFrag(s) => {
                s.mapping_write_byte_order_info_in_data(&mut writer)?
            }
            RtpsSubmessageWriteKind::Gap(s) => {
                s.mapping_write_byte_order_info_in_data(&mut writer)?
            }
            RtpsSubmessageWriteKind::Heartbeat(s) => {
                s.mapping_write_byte_order_info_in_data(&mut writer)?
            }
            RtpsSubmessageWriteKind::HeartbeatFrag(s) => {
                s.mapping_write_byte_order_info_in_data(&mut writer)?
            }
            RtpsSubmessageWriteKind::InfoDestination(s) => {
                s.mapping_write_byte_order_info_in_data(&mut writer)?
            }
            RtpsSubmessageWriteKind::InfoReply(s) => {
                s.mapping_write_byte_order_info_in_data(&mut writer)?
            }
            RtpsSubmessageWriteKind::InfoSource(s) => {
                s.mapping_write_byte_order_info_in_data(&mut writer)?
            }
            RtpsSubmessageWriteKind::InfoTimestamp(s) => {
                s.mapping_write_byte_order_info_in_data(&mut writer)?
            }
            RtpsSubmessageWriteKind::NackFrag(s) => {
                s.mapping_write_byte_order_info_in_data(&mut writer)?
            }
            RtpsSubmessageWriteKind::Pad(s) => {
                s.mapping_write_byte_order_info_in_data(&mut writer)?
            }
        };
        Ok(())
    }
}

impl MappingWriteByteOrderInfoInData for RtpsMessageWrite<'_> {
    fn mapping_write_byte_order_info_in_data<W: Write>(&self, mut writer: W) -> Result<(), Error> {
        // The byteorder is determined by each submessage individually. Hence
        // decide here for a byteorder for the header
        self.header()
            .mapping_write_byte_ordered::<_, LittleEndian>(&mut writer)?;
        for submessage in self.submessages() {
            submessage.mapping_write_byte_order_info_in_data(&mut writer)?;
        }
        Ok(())
    }
}

impl<'a, 'de: 'a> MappingReadByteOrderInfoInData<'de> for RtpsMessageRead<'a> {
    fn mapping_read_byte_order_info_in_data(buf: &mut &'de [u8]) -> Result<Self, Error> {
        // The byteorder is determined by each submessage individually. Hence
        // decide here for a byteorder for the header
        let header = MappingReadByteOrdered::mapping_read_byte_ordered::<LittleEndian>(buf)?;
        const MAX_SUBMESSAGES: usize = 2_usize.pow(16);
        let mut submessages = vec![];
        for _ in 0..MAX_SUBMESSAGES {
            if buf.len() < 4 {
                break;
            }
            // Preview byte only (to allow full deserialization of submessage header)
            let submessage_id = buf[0];
            let submessage = match submessage_id {
                ACKNACK => RtpsSubmessageReadKind::AckNack(
                    MappingReadByteOrderInfoInData::mapping_read_byte_order_info_in_data(buf)?,
                ),
                DATA => RtpsSubmessageReadKind::Data(
                    MappingReadByteOrderInfoInData::mapping_read_byte_order_info_in_data(buf)?,
                ),
                DATA_FRAG => RtpsSubmessageReadKind::DataFrag(
                    MappingReadByteOrderInfoInData::mapping_read_byte_order_info_in_data(buf)?,
                ),
                GAP => RtpsSubmessageReadKind::Gap(
                    MappingReadByteOrderInfoInData::mapping_read_byte_order_info_in_data(buf)?,
                ),
                HEARTBEAT => RtpsSubmessageReadKind::Heartbeat(
                    MappingReadByteOrderInfoInData::mapping_read_byte_order_info_in_data(buf)?,
                ),
                HEARTBEAT_FRAG => RtpsSubmessageReadKind::HeartbeatFrag(
                    MappingReadByteOrderInfoInData::mapping_read_byte_order_info_in_data(buf)?,
                ),
                INFO_DST => RtpsSubmessageReadKind::InfoDestination(
                    MappingReadByteOrderInfoInData::mapping_read_byte_order_info_in_data(buf)?,
                ),
                INFO_REPLY => RtpsSubmessageReadKind::InfoReply(
                    MappingReadByteOrderInfoInData::mapping_read_byte_order_info_in_data(buf)?,
                ),
                INFO_SRC => RtpsSubmessageReadKind::InfoSource(
                    MappingReadByteOrderInfoInData::mapping_read_byte_order_info_in_data(buf)?,
                ),
                INFO_TS => RtpsSubmessageReadKind::InfoTimestamp(
                    MappingReadByteOrderInfoInData::mapping_read_byte_order_info_in_data(buf)?,
                ),
                NACK_FRAG => RtpsSubmessageReadKind::NackFrag(
                    MappingReadByteOrderInfoInData::mapping_read_byte_order_info_in_data(buf)?,
                ),
                PAD => RtpsSubmessageReadKind::Pad(
                    MappingReadByteOrderInfoInData::mapping_read_byte_order_info_in_data(buf)?,
                ),
                _ => {
                    let submessage_header: RtpsSubmessageHeader =
                        MappingReadByteOrderInfoInData::mapping_read_byte_order_info_in_data(buf)?;
                    buf.consume(submessage_header.submessage_length as usize);
                    continue;
                }
            };
            submessages.push(submessage);
        }
        Ok(RtpsMessageRead::new(header, submessages))
    }
}

#[cfg(test)]
mod tests {

    use crate::implementation::{
        rtps::{
            messages::{
                overall_structure::RtpsMessageHeader,
                submessage_elements::{Parameter, ParameterList},
                submessages::{DataSubmessageRead, DataSubmessageWrite, HeartbeatSubmessage},
                types::{ParameterId, ProtocolId, SerializedPayload},
            },
            types::{
                Count, EntityId, EntityKey, GuidPrefix, ProtocolVersion, SequenceNumber, VendorId,
                USER_DEFINED_READER_GROUP, USER_DEFINED_READER_NO_KEY,
            },
        },
        rtps_udp_psm::mapping_traits::{from_bytes, to_bytes},
    };

    use super::*;

    #[test]
    fn serialize_rtps_message_no_submessage() {
        let header = RtpsMessageHeader {
            protocol: ProtocolId::PROTOCOL_RTPS,
            version: ProtocolVersion::new(2, 3),
            vendor_id: VendorId::new([9, 8]),
            guid_prefix: GuidPrefix::new([3; 12]),
        };
        let value = RtpsMessageWrite::new(header, Vec::new());
        #[rustfmt::skip]
        assert_eq!(to_bytes(&value).unwrap(), vec![
            b'R', b'T', b'P', b'S', // Protocol
            2, 3, 9, 8, // ProtocolVersion | VendorId
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
        ]);
    }

    #[test]
    fn serialize_rtps_message() {
        let header = RtpsMessageHeader {
            protocol: ProtocolId::PROTOCOL_RTPS,
            version: ProtocolVersion::new(2, 3),
            vendor_id: VendorId::new([9, 8]),
            guid_prefix: GuidPrefix::new([3; 12]),
        };
        let endianness_flag = true;
        let inline_qos_flag = true;
        let data_flag = false;
        let key_flag = false;
        let non_standard_payload_flag = false;
        let reader_id = EntityId::new(EntityKey::new([1, 2, 3]), USER_DEFINED_READER_NO_KEY);
        let writer_id = EntityId::new(EntityKey::new([6, 7, 8]), USER_DEFINED_READER_GROUP);
        let writer_sn = SequenceNumber::new(5);
        let parameter_1 = Parameter::new(ParameterId(6), vec![10, 11, 12, 13]);
        let parameter_2 = Parameter::new(ParameterId(7), vec![20, 21, 22, 23]);
        let inline_qos = &ParameterList::new(vec![parameter_1, parameter_2]);
        let serialized_payload = SerializedPayload::new(&[]);

        let submessage = RtpsSubmessageWriteKind::Data(DataSubmessageWrite {
            endianness_flag,
            inline_qos_flag,
            data_flag,
            key_flag,
            non_standard_payload_flag,
            reader_id,
            writer_id,
            writer_sn,
            inline_qos,
            serialized_payload,
        });
        let value = RtpsMessageWrite::new(header, vec![submessage]);
        #[rustfmt::skip]
        assert_eq!(to_bytes(&value).unwrap(), vec![
            b'R', b'T', b'P', b'S', // Protocol
            2, 3, 9, 8, // ProtocolVersion | VendorId
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
            0x15, 0b_0000_0011, 40, 0, // Submessage header
            0, 0, 16, 0, // extraFlags, octetsToInlineQos
            1, 2, 3, 4, // readerId: value[4]
            6, 7, 8, 9, // writerId: value[4]
            0, 0, 0, 0, // writerSN: high
            5, 0, 0, 0, // writerSN: low
            6, 0, 4, 0, // inlineQos: parameterId_1, length_1
            10, 11, 12, 13, // inlineQos: value_1[length_1]
            7, 0, 4, 0, // inlineQos: parameterId_2, length_2
            20, 21, 22, 23, // inlineQos: value_2[length_2]
            1, 0, 0, 0, // inlineQos: Sentinel
        ]);
    }

    #[test]
    fn deserialize_rtps_message_no_submessage() {
        let header = RtpsMessageHeader {
            protocol: ProtocolId::PROTOCOL_RTPS,
            version: ProtocolVersion::new(2, 3),
            vendor_id: VendorId::new([9, 8]),
            guid_prefix: GuidPrefix::new([3; 12]),
        };

        let expected = RtpsMessageRead::new(header, Vec::new());
        #[rustfmt::skip]
        let result: RtpsMessageRead = from_bytes(&[
            b'R', b'T', b'P', b'S', // Protocol
            2, 3, 9, 8, // ProtocolVersion | VendorId
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
        ]).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn deserialize_rtps_message() {
        let header = RtpsMessageHeader {
            protocol: ProtocolId::PROTOCOL_RTPS,
            version: ProtocolVersion::new(2, 3),
            vendor_id: VendorId::new([9, 8]),
            guid_prefix: GuidPrefix::new([3; 12]),
        };
        let endianness_flag = true;
        let inline_qos_flag = true;
        let data_flag = false;
        let key_flag = false;
        let non_standard_payload_flag = false;
        let reader_id = EntityId::new(EntityKey::new([1, 2, 3]), USER_DEFINED_READER_NO_KEY);
        let writer_id = EntityId::new(EntityKey::new([6, 7, 8]), USER_DEFINED_READER_GROUP);
        let writer_sn = SequenceNumber::new(5);
        let inline_qos = &[
            6, 0, 4, 0, // inlineQos: parameterId_1, length_1
            10, 11, 12, 13, // inlineQos: value_1[length_1]
            7, 0, 4, 0, // inlineQos: parameterId_2, length_2
            20, 21, 22, 23, // inlineQos: value_2[length_2]
            1, 0, 1, 0, // inlineQos: Sentinel
        ];
        let serialized_payload = SerializedPayload::new(&[]);

        let data_submessage = RtpsSubmessageReadKind::Data(DataSubmessageRead {
            endianness_flag,
            inline_qos_flag,
            data_flag,
            key_flag,
            non_standard_payload_flag,
            reader_id,
            writer_id,
            writer_sn,
            inline_qos,
            serialized_payload,
        });
        let endianness_flag = true;
        let final_flag = false;
        let liveliness_flag = true;
        let reader_id = EntityId::new(EntityKey::new([1, 2, 3]), USER_DEFINED_READER_NO_KEY);
        let writer_id = EntityId::new(EntityKey::new([6, 7, 8]), USER_DEFINED_READER_GROUP);
        let first_sn = SequenceNumber::new(5);
        let last_sn = SequenceNumber::new(7);
        let count = Count::new(2);
        let heartbeat_submessage = RtpsSubmessageReadKind::Heartbeat(HeartbeatSubmessage {
            endianness_flag,
            final_flag,
            liveliness_flag,
            reader_id,
            writer_id,
            first_sn,
            last_sn,
            count,
        });
        let expected = RtpsMessageRead::new(header, vec![data_submessage, heartbeat_submessage]);
        #[rustfmt::skip]
        let result: RtpsMessageRead = from_bytes(&[
            b'R', b'T', b'P', b'S', // Protocol
            2, 3, 9, 8, // ProtocolVersion | VendorId
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
            0x15, 0b_0000_0011, 40, 0, // Submessage header
            0, 0, 16, 0, // extraFlags, octetsToInlineQos
            1, 2, 3, 4, // readerId: value[4]
            6, 7, 8, 9, // writerId: value[4]
            0, 0, 0, 0, // writerSN: high
            5, 0, 0, 0, // writerSN: low
            6, 0, 4, 0, // inlineQos: parameterId_1, length_1
            10, 11, 12, 13, // inlineQos: value_1[length_1]
            7, 0, 4, 0, // inlineQos: parameterId_2, length_2
            20, 21, 22, 23, // inlineQos: value_2[length_2]
            1, 0, 1, 0, // inlineQos: Sentinel
            0x07, 0b_0000_0101, 28, 0, // Submessage header
            1, 2, 3, 4, // readerId: value[4]
            6, 7, 8, 9, // writerId: value[4]
            0, 0, 0, 0, // firstSN: SequenceNumber: high
            5, 0, 0, 0, // firstSN: SequenceNumber: low
            0, 0, 0, 0, // lastSN: SequenceNumberSet: high
            7, 0, 0, 0, // lastSN: SequenceNumberSet: low
            2, 0, 0, 0, // count: Count: value (long)
        ]).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn deserialize_rtps_message_unknown_submessage() {
        let header = RtpsMessageHeader {
            protocol: ProtocolId::PROTOCOL_RTPS,
            version: ProtocolVersion::new(2, 3),
            vendor_id: VendorId::new([9, 8]),
            guid_prefix: GuidPrefix::new([3; 12]),
        };
        let endianness_flag = true;
        let inline_qos_flag = true;
        let data_flag = false;
        let key_flag = false;
        let non_standard_payload_flag = false;
        let reader_id = EntityId::new(EntityKey::new([1, 2, 3]), USER_DEFINED_READER_NO_KEY);
        let writer_id = EntityId::new(EntityKey::new([6, 7, 8]), USER_DEFINED_READER_GROUP);
        let writer_sn = SequenceNumber::new(5);
        let inline_qos = &[
            6, 0, 4, 0, // inlineQos: parameterId_1, length_1
            10, 11, 12, 13, // inlineQos: value_1[length_1]
            7, 0, 4, 0, // inlineQos: parameterId_2, length_2
            20, 21, 22, 23, // inlineQos: value_2[length_2]
            1, 0, 0, 0, // inlineQos: Sentinel
        ];
        let serialized_payload = SerializedPayload::new(&[]);

        let submessage = RtpsSubmessageReadKind::Data(DataSubmessageRead {
            endianness_flag,
            inline_qos_flag,
            data_flag,
            key_flag,
            non_standard_payload_flag,
            reader_id,
            writer_id,
            writer_sn,
            inline_qos,
            serialized_payload,
        });
        let expected = RtpsMessageRead::new(header, vec![submessage]);
        #[rustfmt::skip]
        let result: RtpsMessageRead = from_bytes(&[
            b'R', b'T', b'P', b'S', // Protocol
            2, 3, 9, 8, // ProtocolVersion | VendorId
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
            0x99, 0b_0101_0011, 4, 0, // Submessage header
            9, 9, 9, 9, // Unkown data
            0x15, 0b_0000_0011, 40, 0, // Submessage header
            0, 0, 16, 0, // extraFlags, octetsToInlineQos
            1, 2, 3, 4, // readerId: value[4]
            6, 7, 8, 9, // writerId: value[4]
            0, 0, 0, 0, // writerSN: high
            5, 0, 0, 0, // writerSN: low
            6, 0, 4, 0, // inlineQos: parameterId_1, length_1
            10, 11, 12, 13, // inlineQos: value_1[length_1]
            7, 0, 4, 0, // inlineQos: parameterId_2, length_2
            20, 21, 22, 23, // inlineQos: value_2[length_2]
            1, 0, 0, 0, // inlineQos: Sentinel
        ]).unwrap();
        assert_eq!(result, expected);
    }
}
