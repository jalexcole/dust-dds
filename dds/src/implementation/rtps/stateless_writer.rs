use crate::{
    infrastructure::{
        error::DdsResult,
        instance::{InstanceHandle, HANDLE_NIL},
        qos_policy::ReliabilityQosPolicyKind,
        time::{Duration, Time},
    },
    topic_definition::type_support::DdsSerializedKey,
};

use super::{
    history_cache::{RtpsParameter, RtpsWriterCacheChange},
    messages::overall_structure::RtpsMessageHeader,
    reader_locator::RtpsReaderLocator,
    transport::TransportWrite,
    types::{ChangeKind, Guid, Locator},
    writer::RtpsWriter,
};

pub struct RtpsStatelessWriter {
    writer: RtpsWriter,
    reader_locators: Vec<RtpsReaderLocator>,
}

impl RtpsStatelessWriter {
    pub fn new(writer: RtpsWriter) -> Self {
        Self {
            writer,
            reader_locators: Vec::new(),
        }
    }

    pub fn _guid(&self) -> Guid {
        self.writer.guid()
    }

    pub fn _unicast_locator_list(&self) -> &[Locator] {
        self.writer.unicast_locator_list()
    }

    pub fn _multicast_locator_list(&self) -> &[Locator] {
        self.writer.multicast_locator_list()
    }

    pub fn _push_mode(&self) -> bool {
        self.writer.push_mode()
    }

    pub fn _heartbeat_period(&self) -> Duration {
        self.writer.heartbeat_period()
    }

    pub fn _data_max_size_serialized(&self) -> usize {
        self.writer.data_max_size_serialized()
    }

    pub fn _new_change(
        &mut self,
        kind: ChangeKind,
        data: Vec<u8>,
        inline_qos: Vec<RtpsParameter>,
        handle: InstanceHandle,
        timestamp: Time,
    ) -> RtpsWriterCacheChange {
        self.writer
            .new_change(kind, data, inline_qos, handle, timestamp)
    }

    pub fn _change_list(&self) -> &[RtpsWriterCacheChange] {
        self.writer.change_list()
    }

    pub fn add_change(&mut self, change: RtpsWriterCacheChange) {
        for reader_locator in &mut self.reader_locators {
            reader_locator
                .unsent_changes_mut()
                .push(change.sequence_number());
        }
        self.writer.add_change(change);
    }

    pub fn _remove_change<F>(&mut self, f: F)
    where
        F: FnMut(&RtpsWriterCacheChange) -> bool,
    {
        self.writer.remove_change(f)
    }

    pub fn reader_locator_add(&mut self, mut a_locator: RtpsReaderLocator) {
        *a_locator.unsent_changes_mut() = self
            .writer
            .change_list()
            .iter()
            .map(|c| c.sequence_number())
            .collect();
        self.reader_locators.push(a_locator);
    }

    pub fn _reader_locator_remove(&mut self, a_locator: Locator) {
        self.reader_locators
            .retain(|l| !(l._locator() == a_locator))
    }

    pub fn write_w_timestamp(
        &mut self,
        serialized_data: Vec<u8>,
        instance_serialized_key: DdsSerializedKey,
        _handle: Option<InstanceHandle>,
        timestamp: Time,
    ) -> DdsResult<()> {
        let handle = self
            .writer
            .register_instance_w_timestamp(instance_serialized_key, timestamp)?
            .unwrap_or(HANDLE_NIL);
        let change = self.writer.new_change(
            ChangeKind::Alive,
            serialized_data,
            vec![],
            handle,
            timestamp,
        );

        self.add_change(change);

        Ok(())
    }

    pub fn send_message(&mut self, header: RtpsMessageHeader, transport: &mut impl TransportWrite) {
        match self.writer.get_qos().reliability.kind {
            ReliabilityQosPolicyKind::BestEffort => {
                for rl in self.reader_locators.iter_mut() {
                    rl.send_message(
                        self.writer.writer_cache(),
                        self.writer.guid().entity_id(),
                        header,
                        transport,
                    );
                }
            }
            ReliabilityQosPolicyKind::Reliable => unimplemented!(),
        }
    }
}
