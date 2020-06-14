macro_rules! u8_enum {
    {
        $name: ident {
            $($field_name: ident = $value: expr),+,
        }
    } => {
        #[derive(PartialEq, Eq, Clone, Copy, Debug)]
        pub enum $name {
            $(
                $field_name = $value,
            )+
        }

        impl Into<u8> for $name {
            fn into(self) -> u8 {
                self as u8
            }
        }

        impl From<u8> for $name {
            fn from(v: u8) -> $name {
                match v {
                    $(
                        $value => $name::$field_name
                    ),+,
                    _ => panic!("Enum value not recognized: ${:02X}", v),
                }
            }
        }
    }
}

macro_rules! memory_mapper {
    {
        name: $name: ident,
        fields: [
            $($hex_f:expr, $mask_f:expr, $field_name_f: ident, $default_f: expr);*;
        ],
        bitfields: {
            getters: [
                $($hex:expr, $mask: expr, $field_name: ident, $default: expr, [
                    $($bitfield_getter: ident,
                    $method_getter: ident,
                    $method_type: ident);*
                ]);*
            ],
            getter_setters: [
                $($hex_s:expr, $mask_s: expr, $field_name_s: ident, $default_s: expr, [
                    $($bitfield_getter_s: ident,
                    $bitfield_setter_s: ident,
                    $method_getter_s: ident,
                    $method_setter_s: ident,
                    $method_type_s: ident);*
                ]);*
            ],
        },
    } => {
        struct $name {
            $($field_name: Bitfield),+,
            $($field_name_f: u8),+
            $(,$field_name_s: Bitfield)*
        }

        #[allow(dead_code)]
        impl $name {
            pub fn new() -> $name {
                $name {
                    $($field_name: Bitfield::new($default)),+,
                    $($field_name_f: $default_f),+,
                    $($field_name_s: Bitfield::new($default_s)),*
                }
            }

            $($(
                pub fn $method_getter(&self) -> $method_type {
                    self.$field_name.$bitfield_getter().into()
                }
            )*)*

            $($(
                pub fn $method_getter_s(&self) -> $method_type_s {
                    self.$field_name_s.$bitfield_getter_s().into()
                }

                pub fn $method_setter_s(&mut self, v: $method_type_s) {
                    self.$field_name_s.$bitfield_setter_s(v.into());
                }
            )*)*
        }

        impl cpu::Handler for $name {
            fn read(&self, address: u16) -> u8 {
                match address {
                    $($hex => self.$field_name.get() | $mask),+,
                    $($hex_f => self.$field_name_f | $mask_f),+
                    $(,$hex_s => self.$field_name_s.get() | $mask_s)*,
                    _ => panic!("Could not handle read at ${:04X}", address),
                }
            }

            fn write(&mut self, address: u16, v: u8) {
                match address {
                    $($hex_f => self.$field_name_f = v),+,
                    $($hex => {
                        self.$field_name.set(v);
                    }),+
                    $(,$hex_s => self.$field_name_s.set(v))*,
                    _ => panic!("Could not handle write at ${:04X} v=${:02X}", address, v),
                }
            }
        }
    }
}

macro_rules! memory_handler {
    {
        parent: $parent: ident,
        mapper: $mapper: ident,
        callback: $callback: ident,
    } => {
        impl cpu::Handler for $parent {
            fn read(&self, address: u16) -> u8 {
                self.$mapper.read(address)
            }

            fn write(&mut self, address: u16, v: u8) {
                self.$mapper.write(address, v);
                self.$callback(address);
            }
        }
    }
}

#[allow(non_snake_case)]
pub mod cpu;
#[allow(non_snake_case)]
pub mod opcodes;

pub mod apu;
pub mod dma;
#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
pub mod memory_controller;
pub mod ppu;
pub mod timer_controller;

pub mod cartridge;

#[allow(non_snake_case)]
pub mod handler_holder;
