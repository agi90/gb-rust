extern "C" {}

extern crate gb;

use std::ffi::CString;
use std::mem;
use std::os::raw::{c_char, c_void};
use std::ptr;
use std::slice;

use gb::{Emulator, Hardware, Interrupt, Key};

const KEYS: [Key; 8] = [
    Key::A,
    Key::B,
    Key::Up,
    Key::Down,
    Key::Left,
    Key::Right,
    Key::Select,
    Key::Start,
];

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
static mut SAVE: &mut [u8] = &mut [];
static mut GAMEPAD: &mut [u8] = &mut [];
static mut PREVIOUS_GAMEPAD: Option<GamepadStatus> = None;

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

impl GamepadStatus {
    fn new() -> GamepadStatus {
        GamepadStatus {
            a: false,
            b: false,
            start: false,
            select: false,
            up: false,
            down: false,
            left: false,
            right: false,
        }
    }

    fn get(&self, key: &Key) -> bool {
        match key {
            Key::A => self.a,
            Key::B => self.b,
            Key::Select => self.select,
            Key::Start => self.start,
            Key::Up => self.up,
            Key::Down => self.down,
            Key::Left => self.left,
            Key::Right => self.right,
        }
    }
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
    unsafe {
        let _buf = Vec::from_raw_parts(ptr, 0, cap);
    }
}

#[no_mangle]
pub unsafe extern "C" fn main_loop() {
    if EMULATOR.is_none()
        || GAMEPAD.len() == 0
        || SOUND == ptr::null_mut()
        || SCREEN == ptr::null_mut()
        || PREVIOUS_GAMEPAD.is_none()
    {
        return;
    }

    let sound = slice::from_raw_parts_mut(SOUND as *mut i16, 1470);
    let screen = slice::from_raw_parts_mut(SCREEN as *mut u8, gb::SCREEN_X * gb::SCREEN_Y * 3);
    main_loop_internal(
        EMULATOR.as_mut().unwrap(),
        screen,
        sound,
        GamepadStatus::from_raw(GAMEPAD),
        &mut PREVIOUS_GAMEPAD,
    );
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

fn update_gamepad(emulator: &mut Emulator, gamepad: &GamepadStatus, previous: &GamepadStatus) {
    for key in KEYS.iter() {
        if gamepad.get(key) != previous.get(key) {
            // State didn't change, nothing to do
            continue;
        }

        if gamepad.get(key) {
            emulator.cpu.key_down(*key);
            emulator.cpu.interrupt(Interrupt::Joypad);
        } else {
            emulator.cpu.key_up(*key);
        }
    }
}

fn main_loop_internal(
    emulator: &mut Emulator,
    screen: &mut [u8],
    sound: &mut [i16],
    gamepad: GamepadStatus,
    previous_gamepad: &mut Option<GamepadStatus>,
) {
    update_gamepad(emulator, &gamepad, previous_gamepad.as_ref().unwrap());

    loop {
        emulator.cpu.next_instruction();
        if emulator.cpu.handler_holder.should_refresh() {
            break;
        }
    }

    previous_gamepad.replace(gamepad);

    store_frame(emulator.cpu.handler_holder.get_screen_buffer(), screen);

    emulator.generate_sound_into(sound);
}

#[no_mangle]
pub unsafe extern "C" fn copy_save() {
    if EMULATOR.is_none() || SAVE.len() == 0 {
        return;
    }

    copy_save_internal(EMULATOR.as_mut().unwrap(), SAVE);
}

pub fn copy_save_internal(emulator: &mut Emulator, save: &mut [u8]) {
    let ram = emulator.cpu.handler_holder.ram();
    save[..ram.len()].copy_from_slice(ram);
}

#[no_mangle]
pub fn init(
    data: *mut u8,
    data_size: isize,
    save_data: *mut u8,
    screen_data: *mut u8,
    sound_data: *mut u8,
    gamepad_data: *mut u8,
) -> *mut c_char {
    unsafe {
        let bytes = slice::from_raw_parts(data, data_size as usize);
        let mut emulator = Emulator::from_data(&bytes, 44100.00).unwrap();
        SAVE = slice::from_raw_parts_mut(save_data, 32768);
        {
            let ram = emulator.cpu.handler_holder.ram();
            // borrow checker quirk...
            let ram_len = ram.len();
            ram.copy_from_slice(&SAVE[..ram_len]);
        }
        EMULATOR = Some(emulator);
        SCREEN = screen_data;
        SOUND = sound_data;
        GAMEPAD = slice::from_raw_parts_mut(gamepad_data, 8);
        PREVIOUS_GAMEPAD = Some(GamepadStatus::new());
    }

    CString::new("OK").unwrap().into_raw()
}

fn main() {}
