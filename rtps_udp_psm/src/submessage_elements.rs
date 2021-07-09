use rust_rtps_pim::{
    messages::types::{Count, FragmentNumber, ParameterId, SubmessageFlag, Time},
    structure::types::ProtocolVersion,
};
use serde::ser::SerializeStruct;

#[derive(Clone, Copy, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub struct Octet(pub(crate) u8);

impl Octet {
    pub fn is_bit_set(&self, index: usize) -> bool {
        self.0 & (0b_0000_0001 << index) != 0
    }
}

impl<const N: usize> From<[SubmessageFlag; N]> for Octet {
    fn from(value: [SubmessageFlag; N]) -> Self {
        let mut flags = 0b_0000_0000;
        for (i, &item) in value.iter().enumerate() {
            if item {
                flags |= 0b_0000_0001 << i
            }
        }
        Self(flags)
    }
}
impl<const N: usize> From<Octet> for [SubmessageFlag; N] {
    fn from(_value: Octet) -> Self {
        todo!()
    }
}
impl From<Octet> for u8 {
    fn from(value: Octet) -> Self {
        value.0
    }
}
impl From<u8> for Octet {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct UShortUdp(pub(crate) u16);

impl rust_rtps_pim::messages::submessage_elements::UShortSubmessageElementType for UShortUdp {
    fn new(value: &u16) -> Self {
        Self(*value)
    }

    fn value(&self) -> u16 {
        self.0
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
pub struct LongUdp(pub(crate) i32);

impl rust_rtps_pim::messages::submessage_elements::LongSubmessageElementType for LongUdp {
    fn new(value: &i32) -> Self {
        Self(*value)
    }

    fn value(&self) -> i32 {
        self.0
    }
}

impl From<[u8; 4]> for LongUdp {
    fn from(value: [u8; 4]) -> Self {
        Self(i32::from_le_bytes(value))
    }
}

impl Into<[u8; 4]> for LongUdp {
    fn into(self) -> [u8; 4] {
        self.0.to_le_bytes()
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub struct ULongUdp(pub(crate) u32);

impl rust_rtps_pim::messages::submessage_elements::ULongSubmessageElementType for ULongUdp {
    fn new(value: &u32) -> Self {
        Self(*value)
    }

    fn value(&self) -> u32 {
        self.0
    }
}

impl From<[u8; 4]> for ULongUdp {
    fn from(value: [u8; 4]) -> Self {
        Self(u32::from_le_bytes(value))
    }
}

impl Into<[u8; 4]> for ULongUdp {
    fn into(self) -> [u8; 4] {
        self.0.to_le_bytes()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize)]
pub struct GuidPrefixUdp(pub(crate) [u8; 12]);

impl rust_rtps_pim::messages::submessage_elements::GuidPrefixSubmessageElementType
    for GuidPrefixUdp
{
    fn new(value: &rust_rtps_pim::structure::types::GuidPrefix) -> Self {
        Self(value.clone())
    }

    fn value(&self) -> rust_rtps_pim::structure::types::GuidPrefix {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct EntityIdUdp(pub(crate) rust_rtps_pim::structure::types::EntityId);

impl rust_rtps_pim::messages::submessage_elements::EntityIdSubmessageElementType for EntityIdUdp {
    fn new(value: &rust_rtps_pim::structure::types::EntityId) -> Self {
        Self(value.clone())
    }

    fn value(&self) -> rust_rtps_pim::structure::types::EntityId {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SequenceNumberUdp {
    pub(crate) high: i32,
    pub(crate) low: u32,
}

impl From<SequenceNumberUdp> for i64 {
    fn from(value: SequenceNumberUdp) -> Self {
        ((value.high as i64) << 32) + value.low as i64
    }
}
impl From<&i64> for SequenceNumberUdp {
    fn from(value: &i64) -> Self {
        Self {
            high: (value >> 32) as i32,
            low: *value as u32,
        }
    }
}

impl rust_rtps_pim::messages::submessage_elements::SequenceNumberSubmessageElementType
    for SequenceNumberUdp
{
    fn new(value: &rust_rtps_pim::structure::types::SequenceNumber) -> Self {
        value.into()
    }

    fn value(&self) -> rust_rtps_pim::structure::types::SequenceNumber {
        (*self).into()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SequenceNumberSetUdp {
    base: SequenceNumberUdp,
    num_bits: ULongUdp,
    bitmap: [i32; 8],
}

impl SequenceNumberSetUdp {
    pub fn len(&self) -> u16 {
        let number_of_bitmap_elements = ((self.num_bits.0 + 31) / 32) as usize; // aka "M"
        12 /*bitmapBase + numBits */ + 4 * number_of_bitmap_elements /* bitmap[0] .. bitmap[M-1] */ as u16
    }
}

impl serde::Serialize for SequenceNumberSetUdp {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let len = 2 + self.bitmap.len();

        let mut state = serializer.serialize_struct("SequenceNumberSet", len)?;
        state.serialize_field("bitmapBase", &self.base)?;
        state.serialize_field("numBits", &self.num_bits)?;
        const BITMAP_NAMES: [&str; 8] = [
            "bitmap[0]",
            "bitmap[1]",
            "bitmap[2]",
            "bitmap[3]",
            "bitmap[4]",
            "bitmap[5]",
            "bitmap[6]",
            "bitmap[7]",
        ];
        let number_of_bitmap_elements = ((self.num_bits.0 + 31) / 32) as usize; // aka "M"
        for i in 0..number_of_bitmap_elements {
            state.serialize_field(BITMAP_NAMES[i], &self.bitmap[i])?;
        }
        state.end()
    }
}

struct SequenceNumberSetVisitor;

impl<'de> serde::de::Visitor<'de> for SequenceNumberSetVisitor {
    type Value = SequenceNumberSetUdp;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("SequenceNumberSet Submessage Element")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let base: SequenceNumberUdp = seq
            .next_element()?
            .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;
        let num_bits: ULongUdp = seq
            .next_element()?
            .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;
        let num_bitmaps = (num_bits.0 + 31) / 32; //In standard refered to as "M"
        let mut bitmap = [0; 8];
        for i in 0..num_bitmaps as usize {
            let bitmap_i = seq
                .next_element()?
                .ok_or_else(|| serde::de::Error::invalid_length(i + 2, &self))?;
            bitmap[i] = bitmap_i;
        }
        Ok(SequenceNumberSetUdp {
            base,
            num_bits,
            bitmap,
        })
    }
}

impl<'de> serde::Deserialize<'de> for SequenceNumberSetUdp {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const MAX_BITMAPS: usize = 8;
        const OTHER_FIELDS: usize = 2; /* base + num_bits */
        const MAX_FIELDS: usize = MAX_BITMAPS + OTHER_FIELDS;
        deserializer.deserialize_tuple(MAX_FIELDS, SequenceNumberSetVisitor)
    }
}

impl rust_rtps_pim::messages::submessage_elements::SequenceNumberSetSubmessageElementType
    for SequenceNumberSetUdp
{
    type IntoIter = std::vec::IntoIter<rust_rtps_pim::structure::types::SequenceNumber>;

    fn new(
        base: &rust_rtps_pim::structure::types::SequenceNumber,
        set: &[rust_rtps_pim::structure::types::SequenceNumber],
    ) -> Self {
        let mut bitmap = [0; 8];
        let mut num_bits = 0;
        for sequence_number in set.iter() {
            let delta_n = (sequence_number - base) as u32;
            let bitmap_num = delta_n / 32;
            bitmap[bitmap_num as usize] |= 1 << (31 - delta_n % 32);
            if delta_n + 1 > num_bits {
                num_bits = delta_n + 1;
            }
        }
        Self {
            base: base.into(),
            num_bits: ULongUdp(num_bits),
            bitmap,
        }
    }

    fn base(&self) -> rust_rtps_pim::structure::types::SequenceNumber {
        self.base.into()
    }

    fn set(&self) -> Self::IntoIter {
        let mut set = vec![];
        for delta_n in 0..self.num_bits.0 as usize {
            if (self.bitmap[delta_n / 32] & (1 << (31 - delta_n % 32)))
                == (1 << (31 - delta_n % 32))
            {
                let seq_num =
                    Into::<rust_rtps_pim::structure::types::SequenceNumber>::into(self.base)
                        + delta_n as rust_rtps_pim::structure::types::SequenceNumber;
                set.push(seq_num);
            }
        }
        set.into_iter()
    }
}

pub type InstanceHandleUdp = i32;

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ProtocolVersionUdp {
    pub major: u8,
    pub minor: u8,
}

impl rust_rtps_pim::messages::submessage_elements::ProtocolVersionSubmessageElementType
    for ProtocolVersionUdp
{
    fn new(_value: &ProtocolVersion) -> Self {
        todo!()
    }

    fn value(&self) -> ProtocolVersion {
        todo!()
    }
}

#[derive(Debug, PartialEq, serde::Deserialize)]
pub struct SerializedDataUdp<'a>(pub &'a [u8]);

impl<'a> SerializedDataUdp<'a> {
    pub fn len(&self) -> u16 {
        self.0.len() as u16
    }
}

impl<'a> rust_rtps_pim::messages::submessage_elements::SerializedDataSubmessageElementType<'a>
    for SerializedDataUdp<'_>
{
    type Value = &'a [u8];
    type Constructed = SerializedDataUdp<'a>;

    fn new(value: &Self::Value) -> Self::Constructed {
        SerializedDataUdp(value)
    }

    fn value(&self) -> &Self::Value {
        todo!()
        // &self.0
    }
}

impl<'a> serde::Serialize for SerializedDataUdp<'a> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_bytes(self.0)
    }
}

impl<'a> rust_rtps_pim::messages::submessage_elements::SerializedDataFragmentSubmessageElementType
    for SerializedDataUdp<'a>
{
    type Value = &'a [u8];

    fn new<T: Into<Self::Value>>(value: T) -> Self {
        Self(value.into())
    }

    fn value(&self) -> &[u8] {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct VendorIdUdp(pub(crate) [u8; 2]);

impl rust_rtps_pim::messages::submessage_elements::VendorIdSubmessageElementType for VendorIdUdp {
    fn new(value: &rust_rtps_pim::structure::types::VendorId) -> Self {
        Self(value.clone())
    }

    fn value(&self) -> rust_rtps_pim::structure::types::VendorId {
        self.0
    }
}

#[derive(Clone, Copy)]
pub struct TimeUdp {
    pub seconds: u32,
    pub fraction: u32,
}

impl<'a> rust_rtps_pim::messages::submessage_elements::TimestampSubmessageElementType for TimeUdp {
    fn new(value: &Time) -> Self {
        // value.clone()
        todo!()
    }

    fn value(&self) -> Time {
        // self
        todo!()
    }
}

#[derive(Debug, PartialEq, Clone, Copy, serde::Serialize)]
pub struct CountUdp(pub(crate) i32);

impl<'a> rust_rtps_pim::messages::submessage_elements::CountSubmessageElementType for CountUdp {
    fn new(value: &Count) -> Self {
        // value.clone()
        todo!()
    }

    fn value(&self) -> Count {
        // self
        todo!()
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct FragmentNumberUdp(pub(crate) u32);

impl rust_rtps_pim::messages::submessage_elements::FragmentNumberSubmessageElementType
    for FragmentNumberUdp
{
    fn new(value: &FragmentNumber) -> Self {
        Self(value.0)
    }

    fn value(&self) -> FragmentNumber {
        FragmentNumber(self.0)
    }
}

impl From<u32> for FragmentNumberUdp {
    fn from(_: u32) -> Self {
        todo!()
    }
}

impl Into<u32> for FragmentNumberUdp {
    fn into(self) -> u32 {
        todo!()
    }
}

pub struct FragmentNumberSetUdp(Vec<FragmentNumberUdp>);

impl rust_rtps_pim::messages::submessage_elements::FragmentNumberSetSubmessageElementType
    for FragmentNumberSetUdp
{
    type IntoIter = Vec<FragmentNumber>;
    fn new(_base: &FragmentNumber, _set: &[FragmentNumber]) -> Self {
        todo!()
    }

    fn base(&self) -> FragmentNumber {
        // &0
        todo!()
    }

    fn set(&self) -> Self::IntoIter {
        todo!()
        // self
    }
}

pub type GroupDigestUdp = [u8; 4];

#[derive(Clone, Copy)]
pub struct DurationUdp {
    pub seconds: i32,
    pub fraction: u32,
}

#[derive(Debug, PartialEq, Clone)]
pub struct VectorUdp(Vec<u8>);
impl serde::Serialize for VectorUdp {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_bytes(self.0.as_slice())
    }
}

impl From<Vec<u8>> for VectorUdp {
    fn from(value: Vec<u8>) -> Self {
        Self(value)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ParameterUdp {
    pub parameter_id: u16,
    pub length: i16,
    pub value: VectorUdp,
}

impl ParameterUdp {
    pub fn new(
        parameter_id: u16,
        value: VectorUdp,
    ) -> Self {
        Self {
            parameter_id,
            length: value.0.len() as i16,
            value,
        }
    }

    pub fn len(&self) -> u16 {
        4 + self.value.0.len() as u16
    }
}

impl serde::Serialize for ParameterUdp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Parameter", 3)?;
        state.serialize_field("ParameterId", &self.parameter_id)?;
        state.serialize_field("length", &self.length)?;
        state.serialize_field("value", &self.value)?;
        state.end()
    }
}

struct ParameterVisitor;

impl<'de> serde::de::Visitor<'de> for ParameterVisitor {
    type Value = ParameterUdp;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Parameter of the ParameterList Submessage Element")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let parameter_id: u16 = seq
            .next_element()?
            .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;
        let length: i16 = seq
            .next_element()?
            .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;
        let mut data = vec![];
        for _ in 0..length {
            data.push(
                seq.next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(3, &self))?,
            );
        }
        Ok(ParameterUdp {
            parameter_id,
            length,
            value: data.into(),
        })
    }
}

impl<'de> serde::Deserialize<'de> for ParameterUdp {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const MAX_BYTES: usize = 2 ^ 16;
        deserializer.deserialize_tuple(MAX_BYTES, ParameterVisitor {})
    }
}
const PID_SENTINEL: u16 = 1;
static SENTINEL: ParameterUdp = ParameterUdp {
    parameter_id: PID_SENTINEL,
    length: 0,
    value: VectorUdp(vec![]),
};

#[derive(Debug, PartialEq, Clone)]
pub struct ParameterListUdp {
    pub(crate) parameter: Vec<ParameterUdp>,
}
impl serde::Serialize for ParameterListUdp {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let len = self.parameter.len();
        let mut state = serializer.serialize_struct("ParameterList", len)?;
        for parameter in &self.parameter {
            state.serialize_field("parameter", &parameter)?;
        }
        state.serialize_field("sentinel", &SENTINEL)?;
        state.end()
    }
}

struct ParameterListVisitor;

impl<'de> serde::de::Visitor<'de> for ParameterListVisitor {
    type Value = ParameterListUdp;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("ParameterList Submessage Element")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut parameters = vec![];
        for _ in 0..seq.size_hint().unwrap() {
            let parameter: ParameterUdp = seq
                .next_element()?
                .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;
            if parameter == SENTINEL {
                return Ok(ParameterListUdp {
                    parameter: parameters.into(),
                });
            } else {
                parameters.push(parameter);
            }
        }
        todo!()
    }
}

impl<'de, 'a> serde::Deserialize<'de> for ParameterListUdp {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const MAX_PARAMETERS: usize = 2 ^ 16;
        deserializer.deserialize_tuple(MAX_PARAMETERS, ParameterListVisitor {})
    }
}

impl ParameterListUdp {
    pub fn len(&self) -> u16 {
        self.parameter.iter().map(|p| p.len()).sum()
    }
}

impl<'a> rust_rtps_pim::messages::submessage_elements::ParameterListSubmessageElementType<'a>
    for ParameterListUdp
{
    type IntoIter = Vec<rust_rtps_pim::messages::submessage_elements::Parameter<'a>>;

    fn new(parameter: &[rust_rtps_pim::messages::submessage_elements::Parameter]) -> Self {
        let mut parameter_list = vec![];
        for parameter_i in parameter {
            let parameter_i_udp = ParameterUdp{
                parameter_id: parameter_i.parameter_id.0,
                length: parameter_i.length,
                value: VectorUdp(parameter_i.value.iter().cloned().collect()),
            };
            parameter_list.push(parameter_i_udp);
        }
        Self {
            parameter: parameter_list,
        }
    }

    fn parameter(&'a self) -> Self::IntoIter {
        self.parameter.iter().map(|x|rust_rtps_pim::messages::submessage_elements::Parameter{
            parameter_id: ParameterId(x.parameter_id),
            length: x.value.0.len() as i16,
            value: x.value.0.as_ref(),
        }).collect()
    }
}

pub struct LocatorListUdp(Vec<rust_rtps_pim::structure::types::Locator>);

impl rust_rtps_pim::messages::submessage_elements::LocatorListSubmessageElementType
    for LocatorListUdp
{
    type IntoIter = Vec<rust_rtps_pim::structure::types::Locator>;

    fn new(_value: &[rust_rtps_pim::structure::types::Locator]) -> Self {
        // Self(value)
        todo!()
    }

    fn value(&self) -> Self::IntoIter {
        self.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_rtps_pim::messages::submessage_elements::{
        SequenceNumberSetSubmessageElementType, SequenceNumberSubmessageElementType,
    };
    use rust_serde_cdr::{
        deserializer::RtpsMessageDeserializer, serializer::RtpsMessageSerializer,
    };

    fn serialize<T: serde::Serialize>(value: T) -> Vec<u8> {
        let mut serializer = RtpsMessageSerializer {
            writer: Vec::<u8>::new(),
        };
        value.serialize(&mut serializer).unwrap();
        serializer.writer
    }

    fn deserialize<'de, T: serde::Deserialize<'de>>(buffer: &'de [u8]) -> T {
        let mut de = RtpsMessageDeserializer { reader: buffer };
        serde::de::Deserialize::deserialize(&mut de).unwrap()
    }

    #[test]
    fn octet_from_submessage_flags() {
        let result: Octet = [true, false, true].into();
        assert_eq!(result, Octet(0b_0000_0101));
    }

    #[test]
    fn octet_from_submessage_flags_empty() {
        let result: Octet = [].into();
        assert_eq!(result, Octet(0b_0000_0000));
    }
    #[test]
    #[should_panic]
    fn octet_from_submessage_flags_overflow() {
        let _: Octet = [true; 9].into();
    }

    #[test]
    fn octet_is_set_bit() {
        let flags = Octet(0b_0000_0001);
        assert_eq!(flags.is_bit_set(0), true);

        let flags = Octet(0b_0000_0000);
        assert_eq!(flags.is_bit_set(0), false);

        let flags = Octet(0b_0000_0010);
        assert_eq!(flags.is_bit_set(1), true);

        let flags = Octet(0b_1000_0011);
        assert_eq!(flags.is_bit_set(7), true);
    }
    #[test]
    fn serialize_octet() {
        assert_eq!(serialize(Octet(5)), vec![5]);
    }
    #[test]
    fn deserialize_octet() {
        let result: Octet = deserialize(&[5]);
        assert_eq!(result, Octet(5));
    }

    #[test]
    fn serialize_parameter() {
        let parameter = ParameterUdp::new(2, vec![5, 6, 7, 8].into());
        #[rustfmt::skip]
        assert_eq!(serialize(parameter), vec![
            0x02, 0x00, 4, 0, // Parameter | length
            5, 6, 7, 8,       // value
        ]);
    }

    #[test]
    fn serialize_parameter_list() {
        let parameter = ParameterListUdp {
            parameter: vec![
                ParameterUdp::new(2, vec![51, 61, 71, 81].into()),
                ParameterUdp::new(3, vec![52, 62, 72, 82].into()),
            ]
            .into(),
        };
        #[rustfmt::skip]
        assert_eq!(serialize(parameter), vec![
            0x02, 0x00, 4, 0, // Parameter ID | length
            51, 61, 71, 81,   // value
            0x03, 0x00, 4, 0, // Parameter ID | length
            52, 62, 72, 82,   // value
            0x01, 0x00, 0, 0, // Sentinel: PID_SENTINEL | PID_PAD
        ]);
    }

    #[test]
    fn deserialize_parameter() {
        let expected = ParameterUdp::new(0x02, vec![5, 6, 7, 8].into());
        #[rustfmt::skip]
        let result = deserialize(&[
            0x02, 0x00, 4, 0, // Parameter | length
            5, 6, 7, 8,       // value
        ]);
        assert_eq!(expected, result);
    }

    #[test]
    fn deserialize_parameter_list() {
        let expected = ParameterListUdp {
            parameter: vec![
                ParameterUdp::new(0x02, vec![15, 16, 17, 18].into()),
                ParameterUdp::new(0x03, vec![25, 26, 27, 28].into()),
            ]
            .into(),
        };
        #[rustfmt::skip]
        let result: ParameterListUdp = deserialize(&[
            0x02, 0x00, 4, 0, // Parameter ID | length
            15, 16, 17, 18,        // value
            0x03, 0x00, 4, 0, // Parameter ID | length
            25, 26, 27, 28,        // value
            0x01, 0x00, 0, 0, // Sentinel: Parameter ID | length
            9, 9, 9,    // Following data
        ]);
        assert_eq!(expected, result);
    }

    #[test]
    fn serialize_serialized_data() {
        let data = SerializedDataUdp(&[1, 2]);
        assert_eq!(serialize(data), vec![1, 2]);
    }

    #[test]
    fn sequence_number_set_submessage_element_type_constructor() {
        let expected = SequenceNumberSetUdp {
            base: SequenceNumberUdp::new(&2),
            num_bits: ULongUdp(0),
            bitmap: [0; 8],
        };
        assert_eq!(SequenceNumberSetUdp::new(&2, &[]), expected);

        let expected = SequenceNumberSetUdp {
            base: SequenceNumberUdp::new(&2),
            num_bits: ULongUdp(1),
            bitmap: [
                0b_10000000_00000000_00000000_00000000_u32 as i32,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        };
        assert_eq!(SequenceNumberSetUdp::new(&2, &[2]), expected);

        let expected = SequenceNumberSetUdp {
            base: SequenceNumberUdp::new(&2),
            num_bits: ULongUdp(256),
            bitmap: [
                0b_10000000_00000000_00000000_00000000_u32 as i32,
                0,
                0,
                0,
                0,
                0,
                0,
                0b_00000000_00000000_00000000_00000001,
            ],
        };
        assert_eq!(SequenceNumberSetUdp::new(&2, &[2, 257]), expected);
    }

    #[test]
    fn sequence_number_set_submessage_element_type_getters() {
        let sequence_number_set = SequenceNumberSetUdp {
            base: SequenceNumberUdp::new(&2),
            num_bits: ULongUdp(0),
            bitmap: [0; 8],
        };
        assert_eq!(sequence_number_set.base(), 2);
        assert!(sequence_number_set.set().eq(Vec::<i64>::new()));

        let sequence_number_set = SequenceNumberSetUdp {
            base: SequenceNumberUdp::new(&2),
            num_bits: ULongUdp(100),
            bitmap: [
                0b_10000000_00000000_00000000_00000000_u32 as i32,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
        };
        assert_eq!(sequence_number_set.base(), 2);
        assert!(sequence_number_set.set().eq(vec![2]));

        let sequence_number_set = SequenceNumberSetUdp {
            base: SequenceNumberUdp::new(&2),
            num_bits: ULongUdp(256),
            bitmap: [
                0b_10000000_00000000_00000000_00000000_u32 as i32,
                0,
                0,
                0,
                0,
                0,
                0,
                0b_00000000_00000000_00000000_00000001,
            ],
        };
        assert_eq!(sequence_number_set.base(), 2);
        assert!(sequence_number_set.set().eq(vec![2, 257]));
    }

    #[test]
    fn serialize_sequence_number_max_gap() {
        let sequence_number_set = SequenceNumberSetUdp::new(&2, &[2, 257]);
        #[rustfmt::skip]
        assert_eq!(serialize(sequence_number_set), vec![
            0, 0, 0, 0, // bitmapBase: high (long)
            2, 0, 0, 0, // bitmapBase: low (unsigned long)
            0, 1, 0, 0, // numBits (ULong)
            0b_000_0000, 0b_0000_0000, 0b_0000_0000, 0b_1000_0000, // bitmap[0] (long)
            0b_000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[1] (long)
            0b_000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[2] (long)
            0b_000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[3] (long)
            0b_000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[4] (long)
            0b_000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[5] (long)
            0b_000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[6] (long)
            0b_000_0001, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[7] (long)
        ]);
    }

    #[test]
    fn serialize_sequence_number_set_empty() {
        let sequence_number_set = SequenceNumberSetUdp::new(&2, &[]);
        #[rustfmt::skip]
        assert_eq!(serialize(sequence_number_set), vec![
            0, 0, 0, 0, // bitmapBase: high (long)
            2, 0, 0, 0, // bitmapBase: low (unsigned long)
            0, 0, 0, 0, // numBits (ULong)
        ]);
    }

    #[test]
    fn deserialize_sequence_number_set_empty() {
        let expected = SequenceNumberSetUdp::new(&2, &[]);
        #[rustfmt::skip]
        let result = deserialize(&[
            0, 0, 0, 0, // bitmapBase: high (long)
            2, 0, 0, 0, // bitmapBase: low (unsigned long)
            0, 0, 0, 0, // numBits (ULong)
        ]);
        assert_eq!(expected, result);
    }

    #[test]
    fn deserialize_sequence_number_set_max_gap() {
        let expected = SequenceNumberSetUdp::new(&2, &[2, 257]);
        #[rustfmt::skip]
        let result = deserialize(&[
            0, 0, 0, 0, // bitmapBase: high (long)
            2, 0, 0, 0, // bitmapBase: low (unsigned long)
            0, 1, 0, 0, // numBits (ULong)
            0b_000_0000, 0b_0000_0000, 0b_0000_0000, 0b_1000_0000, // bitmap[0] (long)
            0b_000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[1] (long)
            0b_000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[2] (long)
            0b_000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[3] (long)
            0b_000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[4] (long)
            0b_000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[5] (long)
            0b_000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[6] (long)
            0b_000_0001, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[7] (long)

        ]);
        assert_eq!(expected, result);
    }
}
