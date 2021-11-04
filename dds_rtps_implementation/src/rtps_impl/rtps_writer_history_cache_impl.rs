use rust_dds_api::dcps_psm::{InstanceStateKind, ViewStateKind};
use rust_rtps_pim::{
    messages::{submessage_elements::Parameter, types::Time},
    structure::{
        cache_change::RtpsCacheChange,
        history_cache::{
            RtpsHistoryCacheAddChange, RtpsHistoryCacheConstructor, RtpsHistoryCacheGetChange,
            RtpsHistoryCacheOperations,
        },
        types::{ChangeKind, Guid, InstanceHandle, SequenceNumber},
    },
};

use crate::dds_type::{BigEndian, DdsSerialize};

struct WriterCacheChange<T> {
    kind: ChangeKind,
    writer_guid: Guid,
    sequence_number: SequenceNumber,
    instance_handle: InstanceHandle,
    data: T,
    _source_timestamp: Option<Time>,
    _view_state_kind: ViewStateKind,
    _instance_state_kind: InstanceStateKind,
}

pub struct WriterHistoryCache<T> {
    changes: Vec<WriterCacheChange<T>>,
    source_timestamp: Option<Time>,
}

impl<T> WriterHistoryCache<T> {
    /// Set the Rtps history cache impl's info.
    pub fn set_source_timestamp(&mut self, info: Option<Time>) {
        self.source_timestamp = info;
    }
}

impl<T> RtpsHistoryCacheConstructor for WriterHistoryCache<T> {
    fn new() -> Self {
        Self {
            changes: Vec::new(),
            source_timestamp: None,
        }
    }
}

impl<T> RtpsHistoryCacheAddChange<Vec<Parameter<Vec<u8>>>, T> for WriterHistoryCache<T> {
    fn add_change(&mut self, change: RtpsCacheChange<Vec<Parameter<Vec<u8>>>, T>) {
        let instance_state_kind = match change.kind {
            ChangeKind::Alive => InstanceStateKind::Alive,
            ChangeKind::AliveFiltered => InstanceStateKind::Alive,
            ChangeKind::NotAliveDisposed => InstanceStateKind::NotAliveDisposed,
            ChangeKind::NotAliveUnregistered => todo!(),
        };

        let local_change = WriterCacheChange {
            kind: change.kind,
            writer_guid: change.writer_guid,
            sequence_number: change.sequence_number,
            instance_handle: change.instance_handle,
            data: change.data_value,
            _source_timestamp: self.source_timestamp,
            _view_state_kind: ViewStateKind::New,
            _instance_state_kind: instance_state_kind,
        };

        self.changes.push(local_change)
    }
}

impl<T> RtpsHistoryCacheGetChange<'_, Vec<Parameter<Vec<u8>>>, Vec<u8>> for WriterHistoryCache<T>
where
    T: DdsSerialize,
{
    fn get_change(
        &'_ self,
        seq_num: &SequenceNumber,
    ) -> Option<RtpsCacheChange<Vec<Parameter<Vec<u8>>>, Vec<u8>>> {
        let local_change = self
            .changes
            .iter()
            .find(|&cc| &cc.sequence_number == seq_num)?;

        let mut data_value = Vec::new();
        local_change
            .data
            .serialize::<_, BigEndian>(&mut data_value)
            .ok()?;

        Some(RtpsCacheChange {
            kind: local_change.kind,
            writer_guid: local_change.writer_guid,
            instance_handle: local_change.instance_handle,
            sequence_number: local_change.sequence_number,
            data_value,
            inline_qos: vec![],
        })
    }
}

impl<T> RtpsHistoryCacheOperations for WriterHistoryCache<T> {
    fn remove_change(&mut self, seq_num: &SequenceNumber) {
        self.changes.retain(|cc| &cc.sequence_number != seq_num)
    }

    fn get_seq_num_min(&self) -> Option<SequenceNumber> {
        self.changes
            .iter()
            .map(|cc| cc.sequence_number)
            .min()
            .clone()
    }

    fn get_seq_num_max(&self) -> Option<SequenceNumber> {
        self.changes
            .iter()
            .map(|cc| cc.sequence_number)
            .max()
            .clone()
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use crate::dds_type::Endianness;

    use super::*;
    use rust_dds_api::return_type::DDSResult;
    use rust_rtps_pim::structure::types::GUID_UNKNOWN;

    struct MockDdsSerialize;

    impl DdsSerialize for MockDdsSerialize {
        fn serialize<W: Write, E: Endianness>(&self, _writer: W) -> DDSResult<()> {
            Ok(())
        }
    }

    #[test]
    fn add_change() {
        let mut hc = WriterHistoryCache::new();
        let change = RtpsCacheChange {
            kind: rust_rtps_pim::structure::types::ChangeKind::Alive,
            writer_guid: GUID_UNKNOWN,
            instance_handle: 0,
            sequence_number: 1,
            data_value: &MockDdsSerialize,
            inline_qos: vec![],
        };
        hc.add_change(change);
        assert!(hc.get_change(&1).is_some());
    }

    #[test]
    fn remove_change() {
        let mut hc = WriterHistoryCache::new();
        let change = RtpsCacheChange {
            kind: rust_rtps_pim::structure::types::ChangeKind::Alive,
            writer_guid: GUID_UNKNOWN,
            instance_handle: 0,
            sequence_number: 1,
            data_value: &MockDdsSerialize,
            inline_qos: vec![],
        };
        hc.add_change(change);
        hc.remove_change(&1);
        assert!(hc.get_change(&1).is_none());
    }

    #[test]
    fn get_change() {
        let mut hc = WriterHistoryCache::new();
        let change = RtpsCacheChange {
            kind: rust_rtps_pim::structure::types::ChangeKind::Alive,
            writer_guid: GUID_UNKNOWN,
            instance_handle: 0,
            sequence_number: 1,
            data_value: &MockDdsSerialize,
            inline_qos: vec![],
        };
        hc.add_change(change);
        assert!(hc.get_change(&1).is_some());
        assert!(hc.get_change(&2).is_none());
    }

    #[test]
    fn get_seq_num_min() {
        let mut hc = WriterHistoryCache::new();
        let change1 = RtpsCacheChange {
            kind: rust_rtps_pim::structure::types::ChangeKind::Alive,
            writer_guid: GUID_UNKNOWN,
            instance_handle: 0,
            sequence_number: 1,
            data_value: &MockDdsSerialize,
            inline_qos: vec![],
        };
        let change2 = RtpsCacheChange {
            kind: rust_rtps_pim::structure::types::ChangeKind::Alive,
            writer_guid: GUID_UNKNOWN,
            instance_handle: 0,
            sequence_number: 2,
            data_value: &MockDdsSerialize,
            inline_qos: vec![],
        };
        hc.add_change(change1);
        hc.add_change(change2);
        assert_eq!(hc.get_seq_num_min(), Some(1));
    }

    #[test]
    fn get_seq_num_max() {
        let mut hc = WriterHistoryCache::new();
        let change1 = RtpsCacheChange {
            kind: rust_rtps_pim::structure::types::ChangeKind::Alive,
            writer_guid: GUID_UNKNOWN,
            instance_handle: 0,
            sequence_number: 1,
            data_value: &MockDdsSerialize,
            inline_qos: vec![],
        };
        let change2 = RtpsCacheChange {
            kind: rust_rtps_pim::structure::types::ChangeKind::Alive,
            writer_guid: GUID_UNKNOWN,
            instance_handle: 0,
            sequence_number: 2,
            data_value: &MockDdsSerialize,
            inline_qos: vec![],
        };
        hc.add_change(change1);
        hc.add_change(change2);
        assert_eq!(hc.get_seq_num_max(), Some(2));
    }
}
