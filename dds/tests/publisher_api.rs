use dust_dds::{
    domain::domain_participant_factory::DomainParticipantFactory,
    infrastructure::{qos::Qos, status::NO_STATUS},
};

#[test]
fn get_publisher_parent_participant() {
    let domain_participant_factory = DomainParticipantFactory::get_instance();
    let participant = domain_participant_factory
        .create_participant(0, Qos::Default, None, NO_STATUS)
        .unwrap();

    let publisher = participant
        .create_publisher(Qos::Default, None, NO_STATUS)
        .unwrap();

    let publisher_parent_participant = publisher.get_participant().unwrap();

    assert_eq!(participant, publisher_parent_participant);
}
