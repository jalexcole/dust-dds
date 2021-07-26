use rust_rtps_pim::structure::{types::SequenceNumber, RTPSCacheChange, RTPSHistoryCache};

use super::rtps_cache_change_impl::RTPSCacheChangeImpl;

pub struct RTPSHistoryCacheImpl {
    changes: Vec<RTPSCacheChangeImpl>,
}

impl RTPSHistoryCache for RTPSHistoryCacheImpl {
    type CacheChange = RTPSCacheChangeImpl;

    fn new() -> Self
    where
        Self: Sized,
    {
        Self {
            changes: Vec::new(),
        }
    }

    fn add_change(&mut self, change: Self::CacheChange) {
        self.changes.push(change)
    }

    fn remove_change(&mut self, seq_num: &SequenceNumber) {
        self.changes.retain(|cc| cc.sequence_number() != seq_num)
    }

    fn get_change(&self, seq_num: &SequenceNumber) -> Option<&Self::CacheChange> {
        self.changes
            .iter()
            .find(|&cc| cc.sequence_number() == seq_num)
    }

    fn get_seq_num_min(&self) -> Option<SequenceNumber> {
        self.changes.iter().map(|cc| cc.sequence_number()).min().cloned()
    }

    fn get_seq_num_max(&self) -> Option<SequenceNumber> {
        self.changes.iter().map(|cc| cc.sequence_number()).max().cloned()
    }
}

#[cfg(test)]
mod tests {
    use rust_rtps_pim::structure::{types::GUID_UNKNOWN, RTPSCacheChangeOperations};

    use super::*;

    #[test]
    fn add_change() {
        let mut hc: RTPSHistoryCacheImpl = RTPSHistoryCacheImpl::new();
        let change = RTPSCacheChangeImpl::new(
            rust_rtps_pim::structure::types::ChangeKind::Alive,
            GUID_UNKNOWN,
            0,
            1,
            &[],
            (),
        );
        hc.add_change(change);
        assert!(hc.get_change(&1).is_some());
    }

    #[test]
    fn remove_change() {
        let mut hc: RTPSHistoryCacheImpl = RTPSHistoryCacheImpl::new();
        let change = RTPSCacheChangeImpl::new(
            rust_rtps_pim::structure::types::ChangeKind::Alive,
            GUID_UNKNOWN,
            0,
            1,
            &[],
            (),
        );
        hc.add_change(change);
        hc.remove_change(&1);
        assert!(hc.get_change(&1).is_none());
    }

    #[test]
    fn get_change() {
        let mut hc: RTPSHistoryCacheImpl = RTPSHistoryCacheImpl::new();
        let change = RTPSCacheChangeImpl::new(
            rust_rtps_pim::structure::types::ChangeKind::Alive,
            GUID_UNKNOWN,
            0,
            1,
            &[],
            (),
        );
        hc.add_change(change);
        assert!(hc.get_change(&1).is_some());
        assert!(hc.get_change(&2).is_none());
    }

    #[test]
    fn get_seq_num_min() {
        let mut hc: RTPSHistoryCacheImpl = RTPSHistoryCacheImpl::new();
        let change1 = RTPSCacheChangeImpl::new(
            rust_rtps_pim::structure::types::ChangeKind::Alive,
            GUID_UNKNOWN,
            0,
            1,
            &[],
            (),
        );
        let change2 = RTPSCacheChangeImpl::new(
            rust_rtps_pim::structure::types::ChangeKind::Alive,
            GUID_UNKNOWN,
            0,
            2,
            &[],
            (),
        );
        hc.add_change(change1);
        hc.add_change(change2);
        assert_eq!(hc.get_seq_num_min(), Some(1));
    }

    #[test]
    fn get_seq_num_max() {
        let mut hc: RTPSHistoryCacheImpl = RTPSHistoryCacheImpl::new();
        let change1 = RTPSCacheChangeImpl::new(
            rust_rtps_pim::structure::types::ChangeKind::Alive,
            GUID_UNKNOWN,
            0,
            1,
            &[],
            (),
        );
        let change2 = RTPSCacheChangeImpl::new(
            rust_rtps_pim::structure::types::ChangeKind::Alive,
            GUID_UNKNOWN,
            0,
            2,
            &[],
            (),
        );
        hc.add_change(change1);
        hc.add_change(change2);
        assert_eq!(hc.get_seq_num_max(), Some(2));
    }

    // #[test]
    // fn get_seq_num_max() {
    //     let mut hc: RTPSHistoryCacheImpl<RtpsUdpPsm> = RTPSHistoryCacheImpl::new();
    //     assert_eq!(hc.get_seq_num_max(), None);
    //     let guid = GUID::new(
    //         [1; 12],
    //         EntityId {
    //             entity_key: [1; 3],
    //             entity_kind: 1,
    //         },
    //     );

    //     hc.add_change(RTPSCacheChange {
    //         kind: ChangeKind::Alive,
    //         writer_guid: guid,
    //         instance_handle: 1,
    //         sequence_number: 5.into(),
    //         data_value: vec![],
    //         // inline_qos: vec![],
    //     });
    //     assert_eq!(hc.get_seq_num_max().unwrap(), 5.into());

    //     hc.add_change(RTPSCacheChange {
    //         kind: ChangeKind::Alive,
    //         writer_guid: guid,
    //         instance_handle: 1,
    //         sequence_number: 3.into(),
    //         data_value: vec![],
    //         // inline_qos: vec![],
    //     });
    //     assert_eq!(hc.get_seq_num_max().unwrap(), 5.into());
    // }

    // #[test]
    // fn get_seq_num_min() {
    //     let mut hc: RTPSHistoryCacheImpl<RtpsUdpPsm> = RTPSHistoryCacheImpl::new();
    //     assert_eq!(hc.get_seq_num_min(), None);
    //     let guid = GUID::new(
    //         [1; 12],
    //         EntityId {
    //             entity_key: [1; 3],
    //             entity_kind: 1,
    //         },
    //     );

    //     hc.add_change(RTPSCacheChange {
    //         kind: ChangeKind::Alive,
    //         writer_guid: guid,
    //         instance_handle: 1,
    //         sequence_number: 5.into(),
    //         data_value: vec![],
    //         // inline_qos: vec![],
    //     });
    //     assert_eq!(hc.get_seq_num_max().unwrap(), 5.into());

    //     hc.add_change(RTPSCacheChange {
    //         kind: ChangeKind::Alive,
    //         writer_guid: guid,
    //         instance_handle: 1,
    //         sequence_number: 3.into(),
    //         data_value: vec![],
    //         // inline_qos: vec![],
    //     });
    //     assert_eq!(hc.get_seq_num_min().unwrap(), 3.into());
    // }
}
