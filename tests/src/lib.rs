use parse::brc::BrcTransition;

#[test]
pub fn encodedecode() {
    let transition = BrcTransition::new(0, 0, 1, 0, 0);
    assert_eq!(transition.mult_input_by_to_add_current_measurement(), 0u8);

    let transition = BrcTransition::new(0, 1, 0, 0, 0);
    assert_eq!(transition.mult_input_by_to_add_current_measurement(), 1u8);

    let transition = BrcTransition::new(0, 10, 0, 0, 0);
    assert_eq!(transition.mult_input_by_to_add_current_measurement(), 10u8);

    let transition = BrcTransition::new(0, 100, 0, 0, 0);
    assert_eq!(transition.mult_input_by_to_add_current_measurement(), 100u8);
}