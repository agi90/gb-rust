use bitfield::Bitfield;

#[test]
pub fn test_bitfield_set() {
    let mut bitfield = Bitfield::new(0b11111111);
    bitfield.set_0(0);
    assert_eq!(bitfield.get(), 0b11111110);

    bitfield.set(0b11111111);
    bitfield.set_1(0);
    assert_eq!(bitfield.get(), 0b11111101);

    bitfield.set(0b11111111);
    bitfield.set_2(0);
    assert_eq!(bitfield.get(), 0b11111011);

    bitfield.set(0b11111111);
    bitfield.set_3(0);
    assert_eq!(bitfield.get(), 0b11110111);

    bitfield.set(0b11111111);
    bitfield.set_4(0);
    assert_eq!(bitfield.get(), 0b11101111);

    bitfield.set(0b11111111);
    bitfield.set_5(0);
    assert_eq!(bitfield.get(), 0b11011111);

    bitfield.set(0b11111111);
    bitfield.set_6(0);
    assert_eq!(bitfield.get(), 0b10111111);

    bitfield.set(0b11111111);
    bitfield.set_7(0);
    assert_eq!(bitfield.get(), 0b01111111);

    bitfield.set(0b00000000);
    bitfield.set_0(1);
    assert_eq!(bitfield.get(), 0b00000001);

    bitfield.set(0b00000000);
    bitfield.set_1(1);
    assert_eq!(bitfield.get(), 0b00000010);

    bitfield.set(0b00000000);
    bitfield.set_2(1);
    assert_eq!(bitfield.get(), 0b00000100);

    bitfield.set(0b00000000);
    bitfield.set_3(1);
    assert_eq!(bitfield.get(), 0b00001000);

    bitfield.set(0b00000000);
    bitfield.set_4(1);
    assert_eq!(bitfield.get(), 0b00010000);

    bitfield.set(0b00000000);
    bitfield.set_5(1);
    assert_eq!(bitfield.get(), 0b00100000);

    bitfield.set(0b00000000);
    bitfield.set_6(1);
    assert_eq!(bitfield.get(), 0b01000000);

    bitfield.set(0b00000000);
    bitfield.set_7(1);
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
    assert_eq!(bitfield.get_0(), 1);
    assert_eq!(bitfield.get_1(), 1);
    assert_eq!(bitfield.get_2(), 1);
    assert_eq!(bitfield.get_3(), 1);
    assert_eq!(bitfield.get_4(), 1);
    assert_eq!(bitfield.get_5(), 1);
    assert_eq!(bitfield.get_6(), 1);
    assert_eq!(bitfield.get_7(), 1);

    bitfield.set(0b11110000);
    assert_eq!(bitfield.get(), 0b11110000);

    bitfield.set(0b11111110);
    assert_eq!(bitfield.get_0(), 0);
    assert_eq!(bitfield.get_1(), 1);
    assert_eq!(bitfield.get_2(), 1);
    assert_eq!(bitfield.get_3(), 1);
    assert_eq!(bitfield.get_4(), 1);
    assert_eq!(bitfield.get_5(), 1);
    assert_eq!(bitfield.get_6(), 1);
    assert_eq!(bitfield.get_7(), 1);

    bitfield.set(0b11111101);
    assert_eq!(bitfield.get_1(), 0);

    bitfield.set(0b11111011);
    assert_eq!(bitfield.get_2(), 0);

    bitfield.set(0b11110111);
    assert_eq!(bitfield.get_3(), 0);

    bitfield.set(0b11101111);
    assert_eq!(bitfield.get_4(), 0);

    bitfield.set(0b11011111);
    assert_eq!(bitfield.get_5(), 0);

    bitfield.set(0b10111111);
    assert_eq!(bitfield.get_6(), 0);

    bitfield.set(0b01111111);
    assert_eq!(bitfield.get_7(), 0);

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
