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
        "halt bug\n\nIE IF IF DE\n01 10 11 0C04 \n01 00 01 0C04 \n01 \
01 01 0C04 \n11 00 01 0C04 \n11 10 11 0C04 \n11 11 11 0C04 \n\
E1 00 01 0C04 \nE1 E0 01 0C04 \nE1 E1 01 0C04 \n1783F602 \n\
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
        "interrupt time\n\n00 00 00 \n00 08 00 \n00 00 00 \n00 08 00 \
\n60E957D2 \nFailed\n",
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
