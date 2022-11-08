use crate::infrastructure::qos_policy::ReliabilityQosPolicyKind;

use super::{
    history_cache::RtpsWriterCacheChange, messages::RtpsSubmessageType,
    reader_locator::RtpsReaderLocator, types::Count, writer::RtpsWriter,
};

pub struct RtpsStatelessWriter {
    writer: RtpsWriter,
    reader_locators: Vec<RtpsReaderLocator>,
    _heartbeat_count: Count,
}

impl RtpsStatelessWriter {
    pub fn writer(&self) -> &RtpsWriter {
        &self.writer
    }

    pub fn writer_mut(&mut self) -> &mut RtpsWriter {
        &mut self.writer
    }
}

impl RtpsStatelessWriter {
    pub fn new(writer: RtpsWriter) -> Self {
        Self {
            writer,
            reader_locators: Vec::new(),
            _heartbeat_count: Count(0),
        }
    }
}

impl RtpsStatelessWriter {
    pub fn reader_locator_add(&mut self, mut a_locator: RtpsReaderLocator) {
        *a_locator.unsent_changes_mut() = self
            .writer
            .writer_cache()
            .changes()
            .iter()
            .map(|c| c.sequence_number())
            .collect();
        self.reader_locators.push(a_locator);
    }
}

impl RtpsStatelessWriter {
    pub fn add_change(&mut self, change: RtpsWriterCacheChange) {
        for reader_locator in &mut self.reader_locators {
            reader_locator
                .unsent_changes_mut()
                .push(change.sequence_number());
        }
        self.writer.writer_cache_mut().add_change(change);
    }
}

impl RtpsStatelessWriter {
    pub fn produce_submessages(&mut self) -> Vec<(&RtpsReaderLocator, Vec<RtpsSubmessageType>)> {
        let mut destined_submessages = Vec::new();
        let reliability_kind = &self.writer.get_qos().reliability.kind;
        let writer_cache = self.writer.writer_cache();
        match reliability_kind {
            ReliabilityQosPolicyKind::BestEffort => {
                for rl in self.reader_locators.iter_mut() {
                    let mut submessages = Vec::new();
                    while !rl.unsent_changes().is_empty() {
                        let change = rl.next_unsent_change(writer_cache);
                        // The post-condition:
                        // "( a_change BELONGS-TO the_reader_locator.unsent_changes() ) == FALSE"
                        // should be full-filled by next_unsent_change()
                        if change.is_in_cache() {
                            let (info_ts_submessage, data_submessage) = change.into();
                            submessages.push(RtpsSubmessageType::InfoTimestamp(info_ts_submessage));
                            submessages.push(RtpsSubmessageType::Data(data_submessage));
                        } else {
                            let gap_submessage = change.into();
                            submessages.push(RtpsSubmessageType::Gap(gap_submessage));
                        }
                    }
                    if !submessages.is_empty() {
                        destined_submessages.push((&*rl, submessages));
                    }
                }
            }
            ReliabilityQosPolicyKind::Reliable => todo!(),
        }

        destined_submessages
    }
}
