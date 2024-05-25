use std::collections::HashMap;

use fnmatch_regex::glob_to_regex;
use tracing::warn;

use crate::{
    data_representation_builtin_endpoints::discovered_reader_data::DiscoveredReaderData,
    dds_async::{
        domain_participant::DomainParticipantAsync, publisher::PublisherAsync,
        publisher_listener::PublisherListenerAsync,
    },
    implementation::actor::{Actor, ActorAddress, Mail, MailHandler, DEFAULT_ACTOR_BUFFER_SIZE},
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DataWriterQos, PublisherQos, QosKind},
        qos_policy::PartitionQosPolicy,
        status::StatusKind,
        time::Duration,
    },
    rtps::{
        behavior_types::DURATION_ZERO,
        endpoint::RtpsEndpoint,
        group::RtpsGroup,
        messages::overall_structure::RtpsMessageRead,
        types::{
            EntityId, Guid, Locator, TopicKind, USER_DEFINED_WRITER_NO_KEY,
            USER_DEFINED_WRITER_WITH_KEY,
        },
        writer::RtpsWriter,
    },
};

use super::{
    any_data_writer_listener::AnyDataWriterListener,
    data_writer_actor::{self, DataWriterActor},
    domain_participant_listener_actor::DomainParticipantListenerActor,
    message_sender_actor::MessageSenderActor,
    publisher_listener_actor::PublisherListenerActor,
    status_condition_actor::StatusConditionActor,
    topic_actor::TopicActor,
};

pub struct PublisherActor {
    qos: PublisherQos,
    rtps_group: RtpsGroup,
    data_writer_list: HashMap<InstanceHandle, Actor<DataWriterActor>>,
    enabled: bool,
    user_defined_data_writer_counter: u8,
    default_datawriter_qos: DataWriterQos,
    listener: Actor<PublisherListenerActor>,
    status_kind: Vec<StatusKind>,
    status_condition: Actor<StatusConditionActor>,
}

impl PublisherActor {
    pub fn new(
        qos: PublisherQos,
        rtps_group: RtpsGroup,
        listener: Option<Box<dyn PublisherListenerAsync + Send>>,
        status_kind: Vec<StatusKind>,
        data_writer_list: Vec<DataWriterActor>,
        handle: &tokio::runtime::Handle,
    ) -> Self {
        let data_writer_list = data_writer_list
            .into_iter()
            .map(|dw| {
                (
                    dw.get_instance_handle(),
                    Actor::spawn(dw, handle, DEFAULT_ACTOR_BUFFER_SIZE),
                )
            })
            .collect();
        Self {
            qos,
            rtps_group,
            data_writer_list,
            enabled: false,
            user_defined_data_writer_counter: 0,
            default_datawriter_qos: DataWriterQos::default(),
            listener: Actor::spawn(
                PublisherListenerActor::new(listener),
                handle,
                DEFAULT_ACTOR_BUFFER_SIZE,
            ),
            status_kind,
            status_condition: Actor::spawn(
                StatusConditionActor::default(),
                handle,
                DEFAULT_ACTOR_BUFFER_SIZE,
            ),
        }
    }

    fn get_unique_writer_id(&mut self) -> u8 {
        let counter = self.user_defined_data_writer_counter;
        self.user_defined_data_writer_counter += 1;
        counter
    }

    fn is_partition_matched(&self, discovered_partition_qos_policy: &PartitionQosPolicy) -> bool {
        let is_any_name_matched = discovered_partition_qos_policy
            .name
            .iter()
            .any(|n| self.qos.partition.name.contains(n));

        let is_any_received_regex_matched_with_partition_qos = discovered_partition_qos_policy
            .name
            .iter()
            .filter_map(|n| match glob_to_regex(n) {
                Ok(regex) => Some(regex),
                Err(e) => {
                    warn!(
                        "Received invalid partition regex name {:?}. Error {:?}",
                        n, e
                    );
                    None
                }
            })
            .any(|regex| self.qos.partition.name.iter().any(|n| regex.is_match(n)));

        let is_any_local_regex_matched_with_received_partition_qos = self
            .qos
            .partition
            .name
            .iter()
            .filter_map(|n| match glob_to_regex(n) {
                Ok(regex) => Some(regex),
                Err(e) => {
                    warn!(
                        "Invalid partition regex name on publisher qos {:?}. Error {:?}",
                        n, e
                    );
                    None
                }
            })
            .any(|regex| {
                discovered_partition_qos_policy
                    .name
                    .iter()
                    .any(|n| regex.is_match(n))
            });

        discovered_partition_qos_policy == &self.qos.partition
            || is_any_name_matched
            || is_any_received_regex_matched_with_partition_qos
            || is_any_local_regex_matched_with_received_partition_qos
    }
}

pub struct CreateDatawriter {
    pub topic_address: ActorAddress<TopicActor>,
    pub has_key: bool,
    pub data_max_size_serialized: usize,
    pub qos: QosKind<DataWriterQos>,
    pub a_listener: Option<Box<dyn AnyDataWriterListener + Send>>,
    pub mask: Vec<StatusKind>,
    pub default_unicast_locator_list: Vec<Locator>,
    pub default_multicast_locator_list: Vec<Locator>,
    pub runtime_handle: tokio::runtime::Handle,
}
impl Mail for CreateDatawriter {
    type Result = DdsResult<ActorAddress<DataWriterActor>>;
}
impl MailHandler<CreateDatawriter> for PublisherActor {
    async fn handle(&mut self, message: CreateDatawriter) -> <CreateDatawriter as Mail>::Result {
        let qos = match message.qos {
            QosKind::Default => self.default_datawriter_qos.clone(),
            QosKind::Specific(q) => {
                q.is_consistent()?;
                q
            }
        };

        let guid_prefix = self.rtps_group.guid().prefix();
        let (entity_kind, topic_kind) = match message.has_key {
            true => (USER_DEFINED_WRITER_WITH_KEY, TopicKind::WithKey),
            false => (USER_DEFINED_WRITER_NO_KEY, TopicKind::NoKey),
        };
        let entity_key = [
            self.rtps_group.guid().entity_id().entity_key()[0],
            self.get_unique_writer_id(),
            0,
        ];
        let entity_id = EntityId::new(entity_key, entity_kind);
        let guid = Guid::new(guid_prefix, entity_id);

        let rtps_writer_impl = RtpsWriter::new(
            RtpsEndpoint::new(
                guid,
                topic_kind,
                &message.default_unicast_locator_list,
                &message.default_multicast_locator_list,
            ),
            true,
            Duration::new(0, 200_000_000).into(),
            DURATION_ZERO,
            DURATION_ZERO,
            message.data_max_size_serialized,
        );

        let data_writer = DataWriterActor::new(
            rtps_writer_impl,
            message.topic_address,
            message.a_listener,
            message.mask,
            qos,
            &message.runtime_handle,
        );
        let data_writer_actor = Actor::spawn(
            data_writer,
            &message.runtime_handle,
            DEFAULT_ACTOR_BUFFER_SIZE,
        );
        let data_writer_address = data_writer_actor.address();
        self.data_writer_list
            .insert(InstanceHandle::new(guid.into()), data_writer_actor);

        Ok(data_writer_address)
    }
}

pub struct DeleteDatawriter {
    pub handle: InstanceHandle,
}
impl Mail for DeleteDatawriter {
    type Result = DdsResult<Actor<DataWriterActor>>;
}
impl MailHandler<DeleteDatawriter> for PublisherActor {
    async fn handle(&mut self, message: DeleteDatawriter) -> <DeleteDatawriter as Mail>::Result {
        if let Some(removed_writer) = self.data_writer_list.remove(&message.handle) {
            Ok(removed_writer)
        } else {
            Err(DdsError::PreconditionNotMet(
                "Data writer can only be deleted from its parent publisher".to_string(),
            ))
        }
    }
}

pub struct LookupDatawriter {
    pub topic_name: String,
}
impl Mail for LookupDatawriter {
    type Result = DdsResult<Option<ActorAddress<DataWriterActor>>>;
}
impl MailHandler<LookupDatawriter> for PublisherActor {
    async fn handle(&mut self, message: LookupDatawriter) -> <LookupDatawriter as Mail>::Result {
        for dw in self.data_writer_list.values() {
            if dw
                .send_actor_mail(data_writer_actor::GetTopicName)
                .await
                .receive_reply()
                .await
                .as_ref()
                == Ok(&message.topic_name)
            {
                return Ok(Some(dw.address()));
            }
        }
        Ok(None)
    }
}

pub struct Enable;
impl Mail for Enable {
    type Result = ();
}
impl MailHandler<Enable> for PublisherActor {
    async fn handle(&mut self, _: Enable) -> <Enable as Mail>::Result {
        self.enabled = true;
    }
}

pub struct IsEnabled;
impl Mail for IsEnabled {
    type Result = bool;
}
impl MailHandler<IsEnabled> for PublisherActor {
    async fn handle(&mut self, _: IsEnabled) -> <IsEnabled as Mail>::Result {
        self.enabled
    }
}

pub struct IsEmpty;
impl Mail for IsEmpty {
    type Result = bool;
}
impl MailHandler<IsEmpty> for PublisherActor {
    async fn handle(&mut self, _: IsEmpty) -> <IsEmpty as Mail>::Result {
        self.data_writer_list.is_empty()
    }
}

pub struct DrainDataWriterList;
impl Mail for DrainDataWriterList {
    type Result = Vec<Actor<DataWriterActor>>;
}
impl MailHandler<DrainDataWriterList> for PublisherActor {
    async fn handle(&mut self, _: DrainDataWriterList) -> <DrainDataWriterList as Mail>::Result {
        self.data_writer_list.drain().map(|(_, a)| a).collect()
    }
}

pub struct SetDefaultDatawriterQos {
    pub qos: DataWriterQos,
}
impl Mail for SetDefaultDatawriterQos {
    type Result = ();
}
impl MailHandler<SetDefaultDatawriterQos> for PublisherActor {
    async fn handle(
        &mut self,
        message: SetDefaultDatawriterQos,
    ) -> <SetDefaultDatawriterQos as Mail>::Result {
        self.default_datawriter_qos = message.qos;
    }
}

pub struct GetDefaultDatawriterQos;
impl Mail for GetDefaultDatawriterQos {
    type Result = DataWriterQos;
}
impl MailHandler<GetDefaultDatawriterQos> for PublisherActor {
    async fn handle(
        &mut self,
        _: GetDefaultDatawriterQos,
    ) -> <GetDefaultDatawriterQos as Mail>::Result {
        self.default_datawriter_qos.clone()
    }
}

pub struct SetQos {
    pub qos: QosKind<PublisherQos>,
}
impl Mail for SetQos {
    type Result = DdsResult<()>;
}
impl MailHandler<SetQos> for PublisherActor {
    async fn handle(&mut self, message: SetQos) -> <SetQos as Mail>::Result {
        let qos = match message.qos {
            QosKind::Default => Default::default(),
            QosKind::Specific(q) => q,
        };

        if self.enabled {
            self.qos.check_immutability(&qos)?;
        }

        self.qos = qos;

        Ok(())
    }
}

pub struct GetGuid;
impl Mail for GetGuid {
    type Result = Guid;
}
impl MailHandler<GetGuid> for PublisherActor {
    async fn handle(&mut self, _: GetGuid) -> <GetGuid as Mail>::Result {
        self.rtps_group.guid()
    }
}

pub struct GetInstanceHandle;
impl Mail for GetInstanceHandle {
    type Result = InstanceHandle;
}
impl MailHandler<GetInstanceHandle> for PublisherActor {
    async fn handle(&mut self, _: GetInstanceHandle) -> <GetInstanceHandle as Mail>::Result {
        InstanceHandle::new(self.rtps_group.guid().into())
    }
}

pub struct GetStatusKind;
impl Mail for GetStatusKind {
    type Result = Vec<StatusKind>;
}
impl MailHandler<GetStatusKind> for PublisherActor {
    async fn handle(&mut self, _: GetStatusKind) -> <GetStatusKind as Mail>::Result {
        self.status_kind.clone()
    }
}

pub struct GetQos;
impl Mail for GetQos {
    type Result = PublisherQos;
}
impl MailHandler<GetQos> for PublisherActor {
    async fn handle(&mut self, _: GetQos) -> <GetQos as Mail>::Result {
        self.qos.clone()
    }
}

pub struct GetDataWriterList;
impl Mail for GetDataWriterList {
    type Result = Vec<ActorAddress<DataWriterActor>>;
}
impl MailHandler<GetDataWriterList> for PublisherActor {
    async fn handle(&mut self, _: GetDataWriterList) -> <GetDataWriterList as Mail>::Result {
        self.data_writer_list
            .values()
            .map(|x| x.address())
            .collect()
    }
}

pub struct ProcessRtpsMessage {
    pub rtps_message: RtpsMessageRead,
    pub message_sender_actor: ActorAddress<MessageSenderActor>,
}
impl Mail for ProcessRtpsMessage {
    type Result = ();
}
impl MailHandler<ProcessRtpsMessage> for PublisherActor {
    async fn handle(
        &mut self,
        message: ProcessRtpsMessage,
    ) -> <ProcessRtpsMessage as Mail>::Result {
        for data_writer_address in self.data_writer_list.values() {
            data_writer_address
                .send_actor_mail(data_writer_actor::ProcessRtpsMessage {
                    rtps_message: message.rtps_message.clone(),
                    message_sender_actor: message.message_sender_actor.clone(),
                })
                .await;
        }
    }
}

pub struct AddMatchedReader {
    pub discovered_reader_data: DiscoveredReaderData,
    pub default_unicast_locator_list: Vec<Locator>,
    pub default_multicast_locator_list: Vec<Locator>,
    pub publisher_address: ActorAddress<PublisherActor>,
    pub participant: DomainParticipantAsync,
    pub participant_mask_listener: (
        ActorAddress<DomainParticipantListenerActor>,
        Vec<StatusKind>,
    ),
    pub message_sender_actor: ActorAddress<MessageSenderActor>,
}
impl Mail for AddMatchedReader {
    type Result = DdsResult<()>;
}
impl MailHandler<AddMatchedReader> for PublisherActor {
    async fn handle(&mut self, message: AddMatchedReader) -> <AddMatchedReader as Mail>::Result {
        if self.is_partition_matched(
            message
                .discovered_reader_data
                .subscription_builtin_topic_data()
                .partition(),
        ) {
            for data_writer in self.data_writer_list.values() {
                let data_writer_address = data_writer.address();
                let publisher_mask_listener = (self.listener.address(), self.status_kind.clone());

                data_writer
                    .send_actor_mail(data_writer_actor::AddMatchedReader {
                        discovered_reader_data: message.discovered_reader_data.clone(),
                        default_unicast_locator_list: message.default_unicast_locator_list.clone(),
                        default_multicast_locator_list: message
                            .default_multicast_locator_list
                            .clone(),
                        data_writer_address,
                        publisher: PublisherAsync::new(
                            message.publisher_address.clone(),
                            self.status_condition.address(),
                            message.participant.clone(),
                        ),
                        publisher_qos: self.qos.clone(),
                        publisher_mask_listener,
                        participant_mask_listener: message.participant_mask_listener.clone(),
                        message_sender_actor: message.message_sender_actor.clone(),
                    })
                    .await
                    .receive_reply()
                    .await?;
            }
        }
        Ok(())
    }
}

pub struct RemoveMatchedReader {
    pub discovered_reader_handle: InstanceHandle,
    pub publisher_address: ActorAddress<PublisherActor>,
    pub participant: DomainParticipantAsync,
    pub participant_mask_listener: (
        ActorAddress<DomainParticipantListenerActor>,
        Vec<StatusKind>,
    ),
}
impl Mail for RemoveMatchedReader {
    type Result = DdsResult<()>;
}
impl MailHandler<RemoveMatchedReader> for PublisherActor {
    async fn handle(
        &mut self,
        message: RemoveMatchedReader,
    ) -> <RemoveMatchedReader as Mail>::Result {
        for data_writer in self.data_writer_list.values() {
            let data_writer_address = data_writer.address();
            let publisher_mask_listener = (self.listener.address(), self.status_kind.clone());
            data_writer
                .send_actor_mail(data_writer_actor::RemoveMatchedReader {
                    discovered_reader_handle: message.discovered_reader_handle,
                    data_writer_address,
                    publisher: PublisherAsync::new(
                        message.publisher_address.clone(),
                        self.status_condition.address(),
                        message.participant.clone(),
                    ),
                    publisher_mask_listener,
                    participant_mask_listener: message.participant_mask_listener.clone(),
                })
                .await
                .receive_reply()
                .await?;
        }
        Ok(())
    }
}

pub struct GetStatuscondition;
impl Mail for GetStatuscondition {
    type Result = ActorAddress<StatusConditionActor>;
}
impl MailHandler<GetStatuscondition> for PublisherActor {
    async fn handle(&mut self, _: GetStatuscondition) -> <GetStatuscondition as Mail>::Result {
        self.status_condition.address()
    }
}

pub struct SetListener {
    pub listener: Option<Box<dyn PublisherListenerAsync + Send>>,
    pub status_kind: Vec<StatusKind>,
    pub runtime_handle: tokio::runtime::Handle,
}
impl Mail for SetListener {
    type Result = ();
}
impl MailHandler<SetListener> for PublisherActor {
    async fn handle(&mut self, message: SetListener) -> <SetListener as Mail>::Result {
        self.listener = Actor::spawn(
            PublisherListenerActor::new(message.listener),
            &message.runtime_handle,
            DEFAULT_ACTOR_BUFFER_SIZE,
        );
        self.status_kind = message.status_kind;
    }
}

impl PublisherQos {
    fn check_immutability(&self, other: &Self) -> DdsResult<()> {
        if self.presentation != other.presentation {
            Err(DdsError::ImmutablePolicy)
        } else {
            Ok(())
        }
    }
}
