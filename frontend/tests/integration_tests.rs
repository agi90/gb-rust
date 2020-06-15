use std::env;
use std::process::Command;
use std::str;

pub fn bin_dir() -> String {
    let mut path = env::current_exe().unwrap();
    path.pop();
    if path.ends_with("deps") {
        path.pop();
    }
    path.join("gb-rust-frontend").to_str().unwrap().to_string()
}

pub fn gekkio_test_rom(name: &str, timeout: usize) {
    let test_rom = "tests/gekkio/".to_owned() + name + ".gb";
    let output = Command::new(bin_dir())
        .args(&[&test_rom, "--headless", "--timeout", &timeout.to_string()])
        .output()
        .unwrap();

    // Gekkio's tests output this magin number on success,
    // a failure will be 42, 42, 42, 42, 42
    assert_eq!(
        str::from_utf8(&output.stdout[..]).unwrap_or(""),
        "\u{3}\u{5}\u{8}\r\u{15}\""
    );
}

pub fn blargg_test_rom_with_address(name: &str, expected: &str, address: u16, timeout: usize) {
    let test_rom = "tests/blargg/".to_owned() + name + ".gb";
    let output = Command::new(bin_dir())
        .args(&[
            &test_rom,
            "--headless",
            "--timeout",
            &timeout.to_string(),
            "--result",
            &format!("{:04X}", address),
        ])
        .output()
        .unwrap();

    assert_eq!(str::from_utf8(&output.stdout[..]).unwrap(), expected);
}

pub fn blargg_test_rom(name: &str, expected: &str, timeout: usize) {
    let test_rom = "tests/blargg/".to_owned() + name + ".gb";
    let output = Command::new(bin_dir())
        .args(&[&test_rom, "--headless", "--timeout", &timeout.to_string()])
        .output()
        .unwrap();

    assert_eq!(str::from_utf8(&output.stdout[..]).unwrap(), expected);
}

#[test]
pub fn blargg_instr_timing() {
    blargg_test_rom("instr_timing", "instr_timing\n\n\nPassed\n", 1);
}

#[test]
pub fn blargg_mem_timing_2() {
    blargg_test_rom_with_address(
        "mem_timing",
        "mem_timing\n\n01:ok  02:ok  03:ok  \n\nPassed\n",
        0xA004,
        2,
    );
}

#[test]
pub fn blargg_cpu_instrs() {
    blargg_test_rom(
        "cpu_instrs",
        "cpu_instrs\n\n01:ok  02:ok  03:ok  04:ok  05:ok  06:ok  07:ok  08:ok  \
09:ok  10:ok  11:ok  \n\nPassed all tests\n",
        30,
    );
}

#[test]
pub fn blargg_halt_bug() {
    // TODO:
    blargg_test_rom_with_address(
        "halt_bug",
        "halt bug\n\nIE IF IF DE\n01 10 F1 0C04 \n01 00 E1 0C04 \n01 \
01 E1 0C04 \n11 00 E1 0C04 \n11 10 F1 0C04 \n11 11 F1 0C04 \n\
E1 00 E1 0C04 \nE1 E0 E1 0C04 \nE1 E1 E1 0C04 \n2A6CE34B \n\
Failed\n",
        0xA004,
        2,
    );
}

#[test]
pub fn blargg_interrupt_time() {
    // TODO:
    blargg_test_rom_with_address(
        "interrupt_time",
        "interrupt time\n\n00 00 00 \n00 08 0D \n00 00 00 \n00 08 0D \n\
7F8F4AAF \nFailed\n",
        0xA004,
        1,
    );
}

#[test]
pub fn blargg_dmg_sound() {
    // TODO: Fix test 9, 10, 12 failures
    blargg_test_rom_with_address(
        "dmg_sound",
        "dmg_sound\n\n01:ok  02:ok  03:ok  04:ok  05:ok  06:ok  07:ok  08:ok  \
09:01  10:01  11:ok  12:01  \n\nRun failed tests\nindividually for\nmore \
details.\n\nFailed #9\n",
        0xA004,
        30,
    );
}

#[test]
pub fn gekkio_acceptance_ei_sequence() {
    gekkio_test_rom("acceptance/ei_sequence", 1);
}

#[test]
pub fn gekkio_acceptance_ei_timing() {
    gekkio_test_rom("acceptance/ei_timing", 1);
}

#[test]
pub fn gekkio_acceptance_div_timing() {
    gekkio_test_rom("acceptance/div_timing", 1);
}

#[test]
pub fn gekkio_acceptance_interrupts_ie_push() {
    gekkio_test_rom("acceptance/interrupts/ie_push", 1);
}

#[test]
pub fn gekkio_acceptance_intr_timing() {
    gekkio_test_rom("acceptance/intr_timing", 1);
}

#[test]
pub fn gekkio_acceptance_di_timing_gs() {
    gekkio_test_rom("acceptance/di_timing-GS", 1);
}
