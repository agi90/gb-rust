extern {
}

extern crate gb;

use std::ptr;
use std::mem;
use std::ffi::{
    CString,
};
use std::os::raw::{
    c_char,
    c_void,
};
use std::slice;

use gb::{
    Emulator,
    Interrupt,
    Hardware,
    Key,
};

fn store_frame(screen: &gb::ScreenBuffer, data: &mut [u8]) {
    for i in 0..gb::SCREEN_X {
        for j in 0..gb::SCREEN_Y {
            data[i * gb::SCREEN_Y + j] = screen[j][i] as u8;
        }
    }
}

static mut EMULATOR: Option<Emulator> = None;
static mut SCREEN: *mut u8 = 0 as *mut u8;
static mut SOUND: *mut u8 = 0 as *mut u8;
static mut GAMEPAD: &mut [u8] = &mut [];

#[derive(Debug)]
struct GamepadStatus {
    a: bool,
    b: bool,
    start: bool,
    select: bool,
    up: bool,
    down: bool,
    left: bool,
    right: bool,
}

#[no_mangle]
pub extern "C" fn alloc(size: usize) -> *mut c_void {
    let mut buf = Vec::with_capacity(size);
    let ptr = buf.as_mut_ptr();
    mem::forget(buf);
    return ptr as *mut c_void;
}

#[no_mangle]
pub extern "C" fn dealloc(ptr: *mut c_void, cap: usize) {
    unsafe  {
        let _buf = Vec::from_raw_parts(ptr, 0, cap);
    }
}

#[no_mangle]
pub unsafe extern "C" fn main_loop() {
	if EMULATOR.is_none() || GAMEPAD.len() == 0 || SOUND == ptr::null_mut()
        || SCREEN == ptr::null_mut() {
		return;
	}

    let sound = slice::from_raw_parts_mut(SOUND as *mut i16, 1470);
    let screen = slice::from_raw_parts_mut(SCREEN as *mut u8,
                                           gb::SCREEN_X * gb::SCREEN_Y * 3);
	main_loop_internal(EMULATOR.as_mut().unwrap(),
        screen, sound, &GamepadStatus::from_raw(GAMEPAD));
}

impl GamepadStatus {
    fn from_raw(data: &[u8]) -> GamepadStatus {
        GamepadStatus {
            a: data[0] == 1,
            b: data[1] == 1,
            start: data[2] == 1,
            select: data[3] == 1,
            up: data[4] == 1,
            down: data[5] == 1,
            left: data[6] == 1,
            right: data[7] == 1,
        }
    }
}

fn update_button(emulator: &mut Emulator, button: Key, pressed: bool) {
    if pressed {
        emulator.cpu.key_down(button);
    } else {
        emulator.cpu.key_up(button);
    }
}

fn main_loop_internal(emulator: &mut Emulator, screen: &mut [u8],
                      sound: &mut [i16], gamepad: &GamepadStatus) {
    update_button(emulator, Key::A, gamepad.a);
    update_button(emulator, Key::B, gamepad.b);
    update_button(emulator, Key::Select, gamepad.select);
    update_button(emulator, Key::Start, gamepad.start);
    update_button(emulator, Key::Up, gamepad.up);
    update_button(emulator, Key::Down, gamepad.down);
    update_button(emulator, Key::Left, gamepad.left);
    update_button(emulator, Key::Right, gamepad.right);
    emulator.cpu.interrupt(Interrupt::Joypad);

	loop {
		emulator.cpu.next_instruction();
		if emulator.cpu.handler_holder.should_refresh() {
			break;
		}
	}

    store_frame(emulator.cpu.handler_holder.get_screen_buffer(),
                screen);

    emulator.generate_sound_into(sound);
}

#[no_mangle]
pub fn init(data: *mut u8, data_size: isize, screen_data: *mut u8,
                   sound_data: *mut u8, gamepad_data: *mut u8) -> *mut c_char {
    unsafe {
        let bytes = slice::from_raw_parts(data, data_size as usize);
		EMULATOR = Some(Emulator::from_data(&bytes, 44100.00).unwrap());
        SCREEN = screen_data;
        SOUND = sound_data;
        GAMEPAD = slice::from_raw_parts_mut(gamepad_data, 8);
	}

    CString::new("OK")
        .unwrap()
        .into_raw()
}

fn main() {
}

