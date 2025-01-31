use tracing::warn;

use super::{
    condition::StatusConditionAsync, data_writer_listener::DataWriterListenerAsync,
    publisher::PublisherAsync, topic::TopicAsync,
};
use crate::{
    builtin_topics::SubscriptionBuiltinTopicData,
    implementation::{
        any_data_writer_listener::AnyDataWriterListener,
        domain_participant_backend::{
            domain_participant_actor::DomainParticipantActor, services::data_writer_service,
        },
        status_condition::status_condition_actor::StatusConditionActor,
    },
    infrastructure::{
        error::DdsResult,
        instance::InstanceHandle,
        qos::{DataWriterQos, QosKind},
        status::{
            LivelinessLostStatus, OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus,
            PublicationMatchedStatus, StatusKind,
        },
        time::{Duration, Time},
    },
    runtime::actor::ActorAddress,
    topic_definition::type_support::DdsSerialize,
};
use std::marker::PhantomData;

/// Async version of [`DataWriter`](crate::publication::data_writer::DataWriter).
pub struct DataWriterAsync<Foo> {
    handle: InstanceHandle,
    status_condition_address: ActorAddress<StatusConditionActor>,
    publisher: PublisherAsync,
    topic: TopicAsync,
    phantom: PhantomData<Foo>,
}

impl<Foo> Clone for DataWriterAsync<Foo> {
    fn clone(&self) -> Self {
        Self {
            handle: self.handle,
            status_condition_address: self.status_condition_address.clone(),
            publisher: self.publisher.clone(),
            topic: self.topic.clone(),
            phantom: self.phantom,
        }
    }
}

impl<Foo> DataWriterAsync<Foo> {
    pub(crate) fn new(
        handle: InstanceHandle,
        status_condition_address: ActorAddress<StatusConditionActor>,
        publisher: PublisherAsync,
        topic: TopicAsync,
    ) -> Self {
        Self {
            handle,
            status_condition_address,
            publisher,
            topic,
            phantom: PhantomData,
        }
    }

    pub(crate) fn participant_address(&self) -> &ActorAddress<DomainParticipantActor> {
        self.publisher.participant_address()
    }

    pub(crate) fn change_foo_type<T>(self) -> DataWriterAsync<T> {
        DataWriterAsync {
            handle: self.handle,
            status_condition_address: self.status_condition_address,
            publisher: self.publisher,
            topic: self.topic,
            phantom: PhantomData,
        }
    }
}

impl<Foo> DataWriterAsync<Foo>
where
    Foo: DdsSerialize,
{
    /// Async version of [`register_instance`](crate::publication::data_writer::DataWriter::register_instance).
    #[tracing::instrument(skip(self, instance))]
    pub async fn register_instance(&self, instance: &Foo) -> DdsResult<Option<InstanceHandle>> {
        let timestamp = self
            .get_publisher()
            .get_participant()
            .get_current_time()
            .await?;
        self.register_instance_w_timestamp(instance, timestamp)
            .await
    }

    /// Async version of [`register_instance_w_timestamp`](crate::publication::data_writer::DataWriter::register_instance_w_timestamp).
    #[tracing::instrument(skip(self, _instance))]
    pub async fn register_instance_w_timestamp(
        &self,
        _instance: &Foo,
        timestamp: Time,
    ) -> DdsResult<Option<InstanceHandle>> {
        todo!()
    }

    /// Async version of [`unregister_instance`](crate::publication::data_writer::DataWriter::unregister_instance).
    #[tracing::instrument(skip(self, instance))]
    pub async fn unregister_instance(
        &self,
        instance: &Foo,
        handle: Option<InstanceHandle>,
    ) -> DdsResult<()> {
        let timestamp = self
            .get_publisher()
            .get_participant()
            .get_current_time()
            .await?;
        self.unregister_instance_w_timestamp(instance, handle, timestamp)
            .await
    }

    /// Async version of [`unregister_instance_w_timestamp`](crate::publication::data_writer::DataWriter::unregister_instance_w_timestamp).
    #[tracing::instrument(skip(self, instance))]
    pub async fn unregister_instance_w_timestamp(
        &self,
        instance: &Foo,
        handle: Option<InstanceHandle>,
        timestamp: Time,
    ) -> DdsResult<()> {
        let serialized_data = instance.serialize_data()?;
        self.participant_address()
            .send_actor_mail(data_writer_service::UnregisterInstance {
                publisher_handle: self.publisher.get_instance_handle().await,
                data_writer_handle: self.handle,
                serialized_data,
                timestamp,
            })?
            .receive_reply()
            .await
    }

    /// Async version of [`get_key_value`](crate::publication::data_writer::DataWriter::get_key_value).
    #[tracing::instrument(skip(self, _key_holder))]
    pub async fn get_key_value(
        &self,
        _key_holder: &mut Foo,
        _handle: InstanceHandle,
    ) -> DdsResult<()> {
        todo!()
    }

    /// Async version of [`lookup_instance`](crate::publication::data_writer::DataWriter::lookup_instance).
    #[tracing::instrument(skip(self, instance))]
    pub async fn lookup_instance(&self, instance: &Foo) -> DdsResult<Option<InstanceHandle>> {
        let serialized_data = instance.serialize_data()?;
        self.participant_address()
            .send_actor_mail(data_writer_service::LookupInstance {
                publisher_handle: self.publisher.get_instance_handle().await,
                data_writer_handle: self.handle,
                serialized_data,
            })?
            .receive_reply()
            .await
    }

    /// Async version of [`write`](crate::publication::data_writer::DataWriter::write).
    #[tracing::instrument(skip(self, data))]
    pub async fn write(&self, data: &Foo, handle: Option<InstanceHandle>) -> DdsResult<()> {
        let timestamp = self
            .get_publisher()
            .get_participant()
            .get_current_time()
            .await?;
        self.write_w_timestamp(data, handle, timestamp).await
    }

    /// Async version of [`write_w_timestamp`](crate::publication::data_writer::DataWriter::write_w_timestamp).
    #[tracing::instrument(skip(self, data))]
    pub async fn write_w_timestamp(
        &self,
        data: &Foo,
        handle: Option<InstanceHandle>,
        timestamp: Time,
    ) -> DdsResult<()> {
        let serialized_data = data.serialize_data()?;
        self.participant_address()
            .send_actor_mail(data_writer_service::WriteWTimestamp {
                participant_address: self.participant_address().clone(),
                publisher_handle: self.publisher.get_instance_handle().await,
                data_writer_handle: self.handle,
                serialized_data,
                timestamp,
            })?
            .receive_reply()
            .await
    }

    /// Async version of [`dispose`](crate::publication::data_writer::DataWriter::dispose).
    #[tracing::instrument(skip(self, data))]
    pub async fn dispose(&self, data: &Foo, handle: Option<InstanceHandle>) -> DdsResult<()> {
        let timestamp = self
            .get_publisher()
            .get_participant()
            .get_current_time()
            .await?;
        self.dispose_w_timestamp(data, handle, timestamp).await
    }

    /// Async version of [`dispose_w_timestamp`](crate::publication::data_writer::DataWriter::dispose_w_timestamp).
    #[tracing::instrument(skip(self, data))]
    pub async fn dispose_w_timestamp(
        &self,
        data: &Foo,
        handle: Option<InstanceHandle>,
        timestamp: Time,
    ) -> DdsResult<()> {
        let serialized_data = data.serialize_data()?;
        self.participant_address()
            .send_actor_mail(data_writer_service::DisposeWTimestamp {
                publisher_handle: self.publisher.get_instance_handle().await,
                data_writer_handle: self.handle,
                serialized_data,
                timestamp,
            })?
            .receive_reply()
            .await
    }
}

impl<Foo> DataWriterAsync<Foo> {
    /// Async version of [`wait_for_acknowledgments`](crate::publication::data_writer::DataWriter::wait_for_acknowledgments).
    #[tracing::instrument(skip(self))]
    pub async fn wait_for_acknowledgments(&self, max_wait: Duration) -> DdsResult<()> {
        self.participant_address()
            .send_actor_mail(data_writer_service::WaitForAcknowledgments {
                participant_address: self.participant_address().clone(),
                publisher_handle: self.publisher.get_instance_handle().await,
                data_writer_handle: self.handle,
                timeout: max_wait,
            })?
            .receive_reply()
            .await
            .await
    }

    /// Async version of [`get_liveliness_lost_status`](crate::publication::data_writer::DataWriter::get_liveliness_lost_status).
    #[tracing::instrument(skip(self))]
    pub async fn get_liveliness_lost_status(&self) -> DdsResult<LivelinessLostStatus> {
        todo!()
    }

    /// Async version of [`get_offered_deadline_missed_status`](crate::publication::data_writer::DataWriter::get_offered_deadline_missed_status).
    #[tracing::instrument(skip(self))]
    pub async fn get_offered_deadline_missed_status(
        &self,
    ) -> DdsResult<OfferedDeadlineMissedStatus> {
        self.participant_address()
            .send_actor_mail(data_writer_service::GetOfferedDeadlineMissedStatus {
                publisher_handle: self.publisher.get_instance_handle().await,
                data_writer_handle: self.handle,
            })?
            .receive_reply()
            .await
    }

    /// Async version of [`get_offered_incompatible_qos_status`](crate::publication::data_writer::DataWriter::get_offered_incompatible_qos_status).
    #[tracing::instrument(skip(self))]
    pub async fn get_offered_incompatible_qos_status(
        &self,
    ) -> DdsResult<OfferedIncompatibleQosStatus> {
        todo!()
    }

    /// Async version of [`get_publication_matched_status`](crate::publication::data_writer::DataWriter::get_publication_matched_status).
    #[tracing::instrument(skip(self))]
    pub async fn get_publication_matched_status(&self) -> DdsResult<PublicationMatchedStatus> {
        self.participant_address()
            .send_actor_mail(data_writer_service::GetPublicationMatchedStatus {
                publisher_handle: self.publisher.get_instance_handle().await,
                data_writer_handle: self.handle,
            })?
            .receive_reply()
            .await
    }

    /// Async version of [`get_topic`](crate::publication::data_writer::DataWriter::get_topic).
    #[tracing::instrument(skip(self))]
    pub fn get_topic(&self) -> TopicAsync {
        self.topic.clone()
    }

    /// Async version of [`get_publisher`](crate::publication::data_writer::DataWriter::get_publisher).
    #[tracing::instrument(skip(self))]
    pub fn get_publisher(&self) -> PublisherAsync {
        self.publisher.clone()
    }

    /// Async version of [`assert_liveliness`](crate::publication::data_writer::DataWriter::assert_liveliness).
    #[tracing::instrument(skip(self))]
    pub async fn assert_liveliness(&self) -> DdsResult<()> {
        todo!()
    }

    /// Async version of [`get_matched_subscription_data`](crate::publication::data_writer::DataWriter::get_matched_subscription_data).
    #[tracing::instrument(skip(self))]
    pub async fn get_matched_subscription_data(
        &self,
        subscription_handle: InstanceHandle,
    ) -> DdsResult<SubscriptionBuiltinTopicData> {
        self.participant_address()
            .send_actor_mail(data_writer_service::GetMatchedSubscriptionData {
                publisher_handle: self.publisher.get_instance_handle().await,
                data_writer_handle: self.handle,
                subscription_handle,
            })?
            .receive_reply()
            .await
    }

    /// Async version of [`get_matched_subscriptions`](crate::publication::data_writer::DataWriter::get_matched_subscriptions).
    #[tracing::instrument(skip(self))]
    pub async fn get_matched_subscriptions(&self) -> DdsResult<Vec<InstanceHandle>> {
        self.participant_address()
            .send_actor_mail(data_writer_service::GetMatchedSubscriptions {
                publisher_handle: self.publisher.get_instance_handle().await,
                data_writer_handle: self.handle,
            })?
            .receive_reply()
            .await
    }
}

impl<Foo> DataWriterAsync<Foo> {
    /// Async version of [`set_qos`](crate::publication::data_writer::DataWriter::set_qos).
    #[tracing::instrument(skip(self))]
    pub async fn set_qos(&self, qos: QosKind<DataWriterQos>) -> DdsResult<()> {
        self.participant_address()
            .send_actor_mail(data_writer_service::SetDataWriterQos {
                publisher_handle: self.publisher.get_instance_handle().await,
                data_writer_handle: self.handle,
                qos,
                participant_address: self.participant_address().clone(),
            })?
            .receive_reply()
            .await
    }

    /// Async version of [`get_qos`](crate::publication::data_writer::DataWriter::get_qos).
    #[tracing::instrument(skip(self))]
    pub async fn get_qos(&self) -> DdsResult<DataWriterQos> {
        self.participant_address()
            .send_actor_mail(data_writer_service::GetDataWriterQos {
                publisher_handle: self.publisher.get_instance_handle().await,
                data_writer_handle: self.handle,
            })?
            .receive_reply()
            .await
    }

    /// Async version of [`get_statuscondition`](crate::publication::data_writer::DataWriter::get_statuscondition).
    #[tracing::instrument(skip(self))]
    pub fn get_statuscondition(&self) -> StatusConditionAsync {
        StatusConditionAsync::new(self.status_condition_address.clone())
    }

    /// Async version of [`get_status_changes`](crate::publication::data_writer::DataWriter::get_status_changes).
    #[tracing::instrument(skip(self))]
    pub async fn get_status_changes(&self) -> DdsResult<Vec<StatusKind>> {
        todo!()
    }

    /// Async version of [`enable`](crate::publication::data_writer::DataWriter::enable).
    #[tracing::instrument(skip(self))]
    pub async fn enable(&self) -> DdsResult<()> {
        self.participant_address()
            .send_actor_mail(data_writer_service::Enable {
                publisher_handle: self.publisher.get_instance_handle().await,
                data_writer_handle: self.handle,
                participant_address: self.participant_address().clone(),
            })?
            .receive_reply()
            .await
    }

    /// Async version of [`get_instance_handle`](crate::publication::data_writer::DataWriter::get_instance_handle).
    #[tracing::instrument(skip(self))]
    pub async fn get_instance_handle(&self) -> InstanceHandle {
        self.handle
    }
}
impl<'a, Foo> DataWriterAsync<Foo>
where
    Foo: 'a,
{
    /// Async version of [`set_listener`](crate::publication::data_writer::DataWriter::set_listener).
    #[tracing::instrument(skip(self, a_listener))]
    pub async fn set_listener(
        &self,
        a_listener: Option<Box<dyn DataWriterListenerAsync<'a, Foo = Foo> + Send + 'a>>,
        mask: &[StatusKind],
    ) -> DdsResult<()> {
        let listener = a_listener.map::<Box<dyn AnyDataWriterListener + Send>, _>(|b| Box::new(b));
        self.participant_address()
            .send_actor_mail(data_writer_service::SetListener {
                publisher_handle: self.publisher.get_instance_handle().await,
                data_writer_handle: self.handle,
                listener,
                listener_mask: mask.to_vec(),
            })?
            .receive_reply()
            .await
    }
}
