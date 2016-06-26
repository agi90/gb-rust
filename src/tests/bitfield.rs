use bitfield::Bitfield;

#[test]
pub fn test_bitfield_set() {
    let mut bitfield = Bitfield::new(0b11111111);
    bitfield.set_0(false);
    assert_eq!(bitfield.get(), 0b11111110);

    bitfield.set(0b11111111);
    bitfield.set_1(false);
    assert_eq!(bitfield.get(), 0b11111101);

    bitfield.set(0b11111111);
    bitfield.set_2(false);
    assert_eq!(bitfield.get(), 0b11111011);

    bitfield.set(0b11111111);
    bitfield.set_3(false);
    assert_eq!(bitfield.get(), 0b11110111);

    bitfield.set(0b11111111);
    bitfield.set_4(false);
    assert_eq!(bitfield.get(), 0b11101111);

    bitfield.set(0b11111111);
    bitfield.set_5(false);
    assert_eq!(bitfield.get(), 0b11011111);

    bitfield.set(0b11111111);
    bitfield.set_6(false);
    assert_eq!(bitfield.get(), 0b10111111);

    bitfield.set(0b11111111);
    bitfield.set_7(false);
    assert_eq!(bitfield.get(), 0b01111111);

    bitfield.set(0b00000000);
    bitfield.set_0(true);
    assert_eq!(bitfield.get(), 0b00000001);

    bitfield.set(0b00000000);
    bitfield.set_1(true);
    assert_eq!(bitfield.get(), 0b00000010);

    bitfield.set(0b00000000);
    bitfield.set_2(true);
    assert_eq!(bitfield.get(), 0b00000100);

    bitfield.set(0b00000000);
    bitfield.set_3(true);
    assert_eq!(bitfield.get(), 0b00001000);

    bitfield.set(0b00000000);
    bitfield.set_4(true);
    assert_eq!(bitfield.get(), 0b00010000);

    bitfield.set(0b00000000);
    bitfield.set_5(true);
    assert_eq!(bitfield.get(), 0b00100000);

    bitfield.set(0b00000000);
    bitfield.set_6(true);
    assert_eq!(bitfield.get(), 0b01000000);

    bitfield.set(0b00000000);
    bitfield.set_7(true);
    assert_eq!(bitfield.get(), 0b10000000);

    bitfield.set(0b00000000);
    bitfield.set_01(0b00);
    assert_eq!(bitfield.get(), 0b00000000);

    bitfield.set(0b00000000);
    bitfield.set_01(0b01);
    assert_eq!(bitfield.get(), 0b00000001);

    bitfield.set(0b00000000);
    bitfield.set_01(0b10);
    assert_eq!(bitfield.get(), 0b00000010);

    bitfield.set(0b00000000);
    bitfield.set_01(0b11);
    assert_eq!(bitfield.get(), 0b00000011);

    bitfield.set(0b00000000);
    bitfield.set_23(0b00);
    assert_eq!(bitfield.get(), 0b00000000);

    bitfield.set(0b00000000);
    bitfield.set_23(0b01);
    assert_eq!(bitfield.get(), 0b00000100);

    bitfield.set(0b00000000);
    bitfield.set_23(0b10);
    assert_eq!(bitfield.get(), 0b00001000);

    bitfield.set(0b00000000);
    bitfield.set_23(0b11);
    assert_eq!(bitfield.get(), 0b00001100);

    bitfield.set(0b00000000);
    bitfield.set_45(0b00);
    assert_eq!(bitfield.get(), 0b00000000);

    bitfield.set(0b00000000);
    bitfield.set_45(0b01);
    assert_eq!(bitfield.get(), 0b00010000);

    bitfield.set(0b00000000);
    bitfield.set_45(0b10);
    assert_eq!(bitfield.get(), 0b00100000);

    bitfield.set(0b00000000);
    bitfield.set_45(0b11);
    assert_eq!(bitfield.get(), 0b00110000);

    bitfield.set(0b00000000);
    bitfield.set_67(0b00);
    assert_eq!(bitfield.get(), 0b00000000);

    bitfield.set(0b00000000);
    bitfield.set_67(0b01);
    assert_eq!(bitfield.get(), 0b01000000);

    bitfield.set(0b00000000);
    bitfield.set_67(0b10);
    assert_eq!(bitfield.get(), 0b10000000);

    bitfield.set(0b00000000);
    bitfield.set_67(0b11);
    assert_eq!(bitfield.get(), 0b11000000);
}

#[test]
pub fn test_bitfield_get() {
    let mut bitfield = Bitfield::new(0b11111111);
    assert!(bitfield.get_0());
    assert!(bitfield.get_1());
    assert!(bitfield.get_2());
    assert!(bitfield.get_3());
    assert!(bitfield.get_4());
    assert!(bitfield.get_5());
    assert!(bitfield.get_6());
    assert!(bitfield.get_7());

    bitfield.set(0b11110000);
    assert_eq!(bitfield.get(), 0b11110000);

    bitfield.set(0b11111110);
    assert!(!bitfield.get_0());
    assert!(bitfield.get_1());
    assert!(bitfield.get_2());
    assert!(bitfield.get_3());
    assert!(bitfield.get_4());
    assert!(bitfield.get_5());
    assert!(bitfield.get_6());
    assert!(bitfield.get_7());

    bitfield.set(0b11111101);
    assert!(!bitfield.get_1());

    bitfield.set(0b11111011);
    assert!(!bitfield.get_2());

    bitfield.set(0b11110111);
    assert!(!bitfield.get_3());

    bitfield.set(0b11101111);
    assert!(!bitfield.get_4());

    bitfield.set(0b11011111);
    assert!(!bitfield.get_5());

    bitfield.set(0b10111111);
    assert!(!bitfield.get_6());

    bitfield.set(0b01111111);
    assert!(!bitfield.get_7());

    bitfield.set(0b11111111);
    assert_eq!(bitfield.get_01(), 0b11);
    assert_eq!(bitfield.get_23(), 0b11);
    assert_eq!(bitfield.get_45(), 0b11);
    assert_eq!(bitfield.get_67(), 0b11);

    bitfield.set(0b11111100);
    assert_eq!(bitfield.get_01(), 0b00);

    bitfield.set(0b11111101);
    assert_eq!(bitfield.get_01(), 0b01);

    bitfield.set(0b11111110);
    assert_eq!(bitfield.get_01(), 0b10);

    bitfield.set(0b11110011);
    assert_eq!(bitfield.get_23(), 0b00);

    bitfield.set(0b11110111);
    assert_eq!(bitfield.get_23(), 0b01);

    bitfield.set(0b11111011);
    assert_eq!(bitfield.get_23(), 0b10);

    bitfield.set(0b11001111);
    assert_eq!(bitfield.get_45(), 0b00);

    bitfield.set(0b11011111);
    assert_eq!(bitfield.get_45(), 0b01);

    bitfield.set(0b11101111);
    assert_eq!(bitfield.get_45(), 0b10);

    bitfield.set(0b00111111);
    assert_eq!(bitfield.get_67(), 0b00);

    bitfield.set(0b01111111);
    assert_eq!(bitfield.get_67(), 0b01);

    bitfield.set(0b10111111);
    assert_eq!(bitfield.get_67(), 0b10);
}
