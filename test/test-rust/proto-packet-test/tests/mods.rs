use proto_packet_test::mods::inception::inception::Inception;
use proto_packet_test::mods::parent::Parent;
use proto_packet_test::mods::parent::child::Child;

#[test]
fn inception_mods() {
    assert_eq!(Inception::new(1).value(), 1);
}

#[test]
fn mods_with_types_and_child_mods() {
    let value: Parent = Parent::new(Child::new(1));
    assert_eq!(value.child(), Child::new(1));
}
