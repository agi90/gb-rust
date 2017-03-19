# gb-rust
Yet another DMG (Game Boy) emulator.

Written mostly for fun, it's not intended for general use.

### Run

```
$ cargo build --release
$ cargo run --release -- rom.gb
```

### Command Line options
* `-d --debug` Will start the debugger immediately
* `-m --magnification` Allows changing the magnification of the emulated screen.

### Features
* MBC0, MBC1, MBC3 support
* Works on Linux and Windows (didn't test OSX)
* No sound

### Commands
* `F1` breaks execution and enters the debugger
* `F2` toggle between normal speed and unlimited frame rate
* Arrow keys control up/left/right/down
* `A` controls gameboy button `A`
* `S` controls gameboy button `B`
* `D` controls gameboy button `Start`
* `F` controls gameboy button `Select`
* `h` will print a list of commands for the debugger while in debugging mode

This project is released under the MIT license.

### Tests

[Blargg](http://gbdev.gg8.se/files/roms/blargg-gb-tests/) tests:

![cpu_instr_passed](https://cloud.githubusercontent.com/assets/4297388/22866804/c91cf416-f130-11e6-9304-984390a9e1f3.png)
![instr_timing_passed](https://cloud.githubusercontent.com/assets/4297388/22866802/c91a427a-f130-11e6-9271-3be30e5823d8.png)
![mem_timing_passed](https://cloud.githubusercontent.com/assets/4297388/22866801/c918fdf2-f130-11e6-9ba5-aa466feaaa57.png)
![mem_timing2_passed](https://cloud.githubusercontent.com/assets/4297388/22866866/b374d68c-f131-11e6-8112-afd2614648a6.png)
![screenshot_priority_passed](https://cloud.githubusercontent.com/assets/4297388/22866803/c91ae00e-f130-11e6-97dd-e6c199545481.png)
![halt_bug_failed](https://cloud.githubusercontent.com/assets/4297388/22866864/affbd7a8-f131-11e6-86eb-aad2c9fe3cf4.png)
![interrupt_time_failed](https://cloud.githubusercontent.com/assets/4297388/22866865/b1b52cde-f131-11e6-9319-f5c8fa701d25.png)
![dmg_sound_failed_3](https://cloud.githubusercontent.com/assets/4297388/24085964/59e889e4-0cc3-11e7-8466-f9cf05506006.png)

#### Todo

- oam_bug (crashes)

### Screenshots

#### Pokemon Red/Blue

![pokemon-title](https://cloud.githubusercontent.com/assets/4297388/22866903/33867e84-f132-11e6-87ce-e0106849af65.png)
![pokemon-in-game](https://cloud.githubusercontent.com/assets/4297388/22866902/33843340-f132-11e6-963c-1e2558e3ef4d.png)
![pokemon-menu](https://cloud.githubusercontent.com/assets/4297388/22866901/3381729a-f132-11e6-942d-36514e62b36f.png)

#### Tetris
[video](https://www.youtube.com/watch?v=gjSM0qQbSxE)

![tetris-1](https://cloud.githubusercontent.com/assets/4297388/23596578/c1ba7ca0-01e0-11e7-9aae-caf4faac3649.png)
![tetris-2](https://cloud.githubusercontent.com/assets/4297388/23596579/c1bafaae-01e0-11e7-8ed1-030f89d437a1.png)
![tetris-3](https://cloud.githubusercontent.com/assets/4297388/23596601/fdee1f9c-01e0-11e7-8427-8254614028be.png)
