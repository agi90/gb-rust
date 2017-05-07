use hardware::cpu::{Handler, Interrupt};
use bitfield::Bitfield;

/* Represents a shade of gray */
u8_enum!{
    GrayShade {
        C00 = 0,
        C01 = 1,
        C10 = 2,
        C11 = 3,
        Transparent = 4,
    }
}

pub const SCREEN_X: usize = 160;
pub const SCREEN_Y: usize = 144;

const BACKGROUND_X: usize = 256;
const BACKGROUND_Y: usize = 256;

pub type ScreenBuffer = [[GrayShade; SCREEN_X]; SCREEN_Y];

pub struct Ppu {
    cycles: usize,
    total_cycles: usize,
    video_ram: [u8; 8196],
    oam_ram: [u8; 160],
    screen_buffer: ScreenBuffer,
    should_refresh: bool,
    mapper: VideoMemoryMapper,
    mode: LCDMode,
    scanline_reader: ScanlineReader,
}

u8_enum!{
    LCDMode {
        HBlank = 0,
        VBlank = 1,
        SearchingOAM = 2,
        LCDTransfer = 3,
    }
}

impl LCDMode {
    pub fn duration(&self) -> usize {
        match self {
            &LCDMode::HBlank => 200,
            &LCDMode::VBlank => 456,
            &LCDMode::SearchingOAM => 84,
            &LCDMode::LCDTransfer=> 172,
        }
    }
}

impl Handler for Ppu {
    fn read(&self, address: u16) -> u8 {
        match address {
            0x8000 ... 0x9FFF => self.read_ram(address),
            0xFE00 ... 0xFE9F => self.oam_ram[address as usize - 0xFE00],
            _ => self.mapper.read(address),
        }
    }

    fn write(&mut self, address: u16, v: u8) {
        match address {
            0x8000 ... 0x9FFF => self.write_ram(address, v),
            0xFE00 ... 0xFE9F => self.oam_ram[address as usize - 0xFE00] = v,
            _ => self.mapper.write(address, v),
        }

        self.write_callback(address);
    }
}

#[derive(Debug, Clone, Copy)]
struct SpriteFlags {
    below_bg: bool,
    y_flip: bool,
    x_flip: bool,
    palette: SpritePalette,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScanlineStep {
    LatchingOffset,
    ReadingPixel(usize),
}

#[derive(Debug, Clone, Copy)]
enum BackgroundSubStep {
    BackgroundTile,
    Tile0,
    Tile1,
}

struct ScanlineReader {
    offset: usize,
    scanline_step: ScanlineStep,
    sub_step: BackgroundSubStep,
    cycles: i64,
    current_tile: u8,
    tile0_value: u8,
    tile1_value: u8,
    scanline: usize,
}

impl ScanlineReader {
    pub fn for_scanline(scanline: usize) -> ScanlineReader {
        ScanlineReader {
            offset: 0,
            scanline_step: ScanlineStep::LatchingOffset,
            sub_step: BackgroundSubStep::BackgroundTile,
            cycles: 0,
            current_tile: 0,
            tile0_value: 0,
            tile1_value: 0,
            scanline: scanline,
        }
    }

    pub fn add_cycles(&mut self, cycles: usize,
                      mapper: &VideoMemoryMapper, video_ram: &[u8],
                      screen_buffer: &mut ScreenBuffer) {
        self.cycles += cycles as i64;
        while self.cycles >= 2 && self.current_x() < SCREEN_X {
            self.cycles -= 2;

            match self.sub_step {
                BackgroundSubStep::BackgroundTile => {
                    self.background_substep(mapper, video_ram);
                    self.sub_step = BackgroundSubStep::Tile0;
                },
                BackgroundSubStep::Tile0 => {
                    self.tile0_substep(mapper, video_ram);
                    self.sub_step = BackgroundSubStep::Tile1;
                },
                BackgroundSubStep::Tile1 => {
                    self.tile1_substep(mapper, video_ram);
                    self.sub_step = BackgroundSubStep::BackgroundTile;
                    self.write_pixel(mapper, screen_buffer);
                    self.next_step();
                },
            }
        }
    }

    fn current_x(&self) -> usize {
        match self.scanline_step {
            ScanlineStep::LatchingOffset => 0,
            ScanlineStep::ReadingPixel(x) => x,
        }
    }

    fn background_color_from_raw(&self, mapper: &VideoMemoryMapper,
                                 raw: u8) -> GrayShade {
        match raw {
            0b00 => mapper.bg_color_00(),
            0b01 => mapper.bg_color_01(),
            0b10 => mapper.bg_color_10(),
            0b11 => mapper.bg_color_11(),
            _ => panic!(),
        }
    }

    fn write_pixel(&self, mapper: &VideoMemoryMapper,
                   screen_buffer: &mut ScreenBuffer) {
        if self.scanline_step == ScanlineStep::LatchingOffset {
            // During this step we just discard the value
            return;
        }

        let mut x = self.current_x();
        let bg_x = (BACKGROUND_X + x - mapper.scroll_bg_x as usize)
            % BACKGROUND_X;

        let l = self.tile0_value;
        let h = self.tile1_value;

        for i in bg_x .. bg_x + 8 {
            let pixel = match i % 8 {
                7 =>  (0b00000001 & l)       + ((0b00000001 & h) << 1),
                6 => ((0b00000010 & l) >> 1) + ((0b00000010 & h)     ),
                5 => ((0b00000100 & l) >> 2) + ((0b00000100 & h) >> 1),
                4 => ((0b00001000 & l) >> 3) + ((0b00001000 & h) >> 2),
                3 => ((0b00010000 & l) >> 4) + ((0b00010000 & h) >> 3),
                2 => ((0b00100000 & l) >> 5) + ((0b00100000 & h) >> 4),
                1 => ((0b01000000 & l) >> 6) + ((0b01000000 & h) >> 5),
                0 => ((0b10000000 & l) >> 7) + ((0b10000000 & h) >> 6),
                _ => unreachable!(),
            };

            let color = self.background_color_from_raw(mapper, pixel);
            screen_buffer[self.scanline][x] = color;
            x = (x + 1) % SCREEN_X;
        }
    }

    fn tile_address(&self, tile_value: u8,
                    mapper: &VideoMemoryMapper) -> usize {
        match mapper.bg_tile_data() {
            BgTileData::C8000 => {
                tile_value as usize * 16 + self.scanline % 8 * 2
            },
            BgTileData::C8800 => {
                let tile = if tile_value < 0x80 {
                    tile_value + 0x80
                } else {
                    tile_value - 0x80
                };

                0x0800 + tile as usize * 16 + self.scanline % 8 * 2
            },
        }
    }

    fn tile0_substep(&mut self, mapper: &VideoMemoryMapper,
                     video_ram: &[u8]) {
        self.tile0_value = video_ram[
            self.tile_address(self.current_tile, mapper)];
    }

    fn tile1_substep(&mut self, mapper: &VideoMemoryMapper,
                     video_ram: &[u8]) {
        self.tile1_value = video_ram[
            self.tile_address(self.current_tile, mapper) + 1];
    }

    fn background_substep(&mut self, mapper: &VideoMemoryMapper,
                          video_ram: &[u8]) {
        let x;

        match self.scanline_step {
            ScanlineStep::LatchingOffset => {
                self.latch_offset(mapper);
                x = 0;
            },
            ScanlineStep::ReadingPixel(pixel) => {
                x = pixel;
            },
        }

        let bg_x = (BACKGROUND_X + x - mapper.scroll_bg_x as usize)
            % BACKGROUND_X;
        self.current_tile = video_ram[self.offset + bg_x / 8];
    }

    fn latch_offset(&mut self, mapper: &VideoMemoryMapper) {
        let tile_offset = if mapper.bg_tile_map() == TileMap::C9800 {
            0x1800
        } else {
            0x1C00
        };

        let bg_scanline = (BACKGROUND_Y
            + self.scanline - mapper.scroll_bg_y as usize) % BACKGROUND_Y;

        self.offset = tile_offset + (bg_scanline / 8 as usize) * 32;
    }

    fn next_step(&mut self) {
        match self.scanline_step {
            ScanlineStep::LatchingOffset => {
                self.scanline_step = ScanlineStep::ReadingPixel(0);
            },
            ScanlineStep::ReadingPixel(x) => {
                self.scanline_step = ScanlineStep::ReadingPixel(x + 8);
            }
        }
    }
}

impl Ppu {
    pub fn new() -> Ppu {
        Ppu {
            cycles: 0,
            total_cycles: 0,
            video_ram: [0; 8196],
            oam_ram: [0; 160],
            screen_buffer: [[GrayShade::C00; SCREEN_X]; SCREEN_Y],
            should_refresh: false,
            mapper: VideoMemoryMapper::new(),
            mode: LCDMode::HBlank,
            scanline_reader: ScanlineReader::for_scanline(0),
        }
    }

    fn set_mode(&mut self, mode: LCDMode) {
        self.mode = mode;
        self.mapper.set_mode(mode);

        if mode == LCDMode::LCDTransfer {
            self.scanline_reader = ScanlineReader::for_scanline(
                self.mapper.lcd_y_coordinate as usize);
        }
    }

    fn write_callback(&mut self, address: u16) {
        match address {
            0xFF41 => {
                self.mode = self.mapper.mode();
            },
            _ => {},
        }
    }

    fn raw_background_at(&self, tile_map: TileMap, x: usize, y: usize) -> u8 {
        let offset = if tile_map == TileMap::C9800 {
            0x1800
        } else {
            0x1C00
        };

        let raw_tile = self.video_ram[offset + (y / 8 as usize) * 32 + x / 8] as usize;

        let tile;

        let pattern_offset = if self.mapper.bg_tile_data() == BgTileData::C8000 {
            tile = raw_tile;

            0x0000
        } else {
            tile = if raw_tile < 0x80 {
                raw_tile + 0x80
            } else {
                raw_tile - 0x80
            };

            0x0800
        };

        self.pattern_value(pattern_offset + tile * 16, x % 8, y % 8)
    }

    fn background_color_from_raw(&self, raw: u8) -> GrayShade {
        match raw {
            0b00 => self.mapper.bg_color_00(),
            0b01 => self.mapper.bg_color_01(),
            0b10 => self.mapper.bg_color_10(),
            0b11 => self.mapper.bg_color_11(),
            _ => panic!(),
        }
    }

    fn background_value_at(&self, tile_map: TileMap, x: usize, y: usize) -> GrayShade {
        let raw = self.raw_background_at(tile_map, x, y);
        self.background_color_from_raw(raw)
    }

    fn sprite_value_at(&self, id: usize, x: usize, y: usize) -> GrayShade {
        let flags = self.sprite_flags(id);

        let x = if flags.x_flip { 7 - x } else { x };

        let y = if !flags.y_flip { y } else {
            match self.mapper.sprite_size() {
                SpriteSize::C8by8  =>  7 - y,
                SpriteSize::C8by16 => 15 - y,
            }
        };

        let tile_index = match self.mapper.sprite_size() {
            SpriteSize::C8by8  => self.sprite_tile_index(id),
            SpriteSize::C8by16 => self.sprite_tile_index(id) & 0xFE,
        };

        let color = self.pattern_value(tile_index * 16, x, y);

        match &flags.palette {
            &SpritePalette::C0 => {
                match color {
                    0b00 => GrayShade::Transparent,
                    0b01 => self.mapper.obp0_palette_01(),
                    0b10 => self.mapper.obp0_palette_10(),
                    0b11 => self.mapper.obp0_palette_11(),
                    _ => unreachable!(),
                }
            },
            &SpritePalette::C1 => {
                match color {
                    0b00 => GrayShade::Transparent,
                    0b01 => self.mapper.obp1_palette_01(),
                    0b10 => self.mapper.obp1_palette_10(),
                    0b11 => self.mapper.obp1_palette_11(),
                    _ => unreachable!(),
                }
            }
        }
    }

    #[inline]
    fn pattern_value(&self, offset: usize, x: usize, y: usize) -> u8 {
        let l = self.video_ram[offset + (y * 2)];
        let h = self.video_ram[offset + (y * 2) + 1];

        match x {
            7 =>  (0b00000001 & l)       + ((0b00000001 & h) << 1),
            6 => ((0b00000010 & l) >> 1) + ((0b00000010 & h)     ),
            5 => ((0b00000100 & l) >> 2) + ((0b00000100 & h) >> 1),
            4 => ((0b00001000 & l) >> 3) + ((0b00001000 & h) >> 2),
            3 => ((0b00010000 & l) >> 4) + ((0b00010000 & h) >> 3),
            2 => ((0b00100000 & l) >> 5) + ((0b00100000 & h) >> 4),
            1 => ((0b01000000 & l) >> 6) + ((0b01000000 & h) >> 5),
            0 => ((0b10000000 & l) >> 7) + ((0b10000000 & h) >> 6),
            _ => unreachable!(),
        }
    }

    #[inline]
    fn sprite_y(&self, id: usize) -> usize {
        self.oam_ram[id * 4] as usize
    }

    #[inline]
    fn sprite_x(&self, id: usize) -> usize {
        self.oam_ram[id * 4 + 1] as usize
    }

    #[inline]
    fn sprite_tile_index(&self, id: usize) -> usize {
        self.oam_ram[id * 4 + 2] as usize
    }

    #[inline]
    fn sprite_flags(&self, id: usize) -> SpriteFlags {
        let v = self.oam_ram[id * 4 + 3];

        SpriteFlags {
            below_bg:    v & (0b10000000) > 0,
            y_flip:      v & (0b01000000) > 0,
            x_flip:      v & (0b00100000) > 0,
            palette:  if v & (0b00010000) > 0 { SpritePalette::C1 } else { SpritePalette::C0 },
        }
    }

    fn is_sprite_visible(&self, id: usize, scanline: usize) -> bool {
        let y = self.sprite_y(id);
        match self.mapper.sprite_size() {
            SpriteSize::C8by8 =>  scanline + 16 >= y && scanline + 8 < y,
            SpriteSize::C8by16 => scanline + 16 >= y && scanline < y,
        }
    }

    fn is_sprite_horizontally_visible(&self, id: usize, h: usize) -> bool {
        let x = self.sprite_x(id);
        h + 8 >= x && h < x
    }

    fn print_sprites(&mut self, scanline: usize, background: &[u8]) {
        // Normally visible_sprites would be an array
        // but this code path is very hot so we need to
        // be cautios about performance.
        let mut visible_sprites = [0; 10];
        let mut visible_sprites_len = 0;

        for i in 0..40 {
            if visible_sprites_len >= 10 {
                break;
            }

            if !self.is_sprite_visible(i, scanline) {
                continue;
            }

            visible_sprites[visible_sprites_len] = i;
            visible_sprites_len += 1;
        }

        &visible_sprites[0..visible_sprites_len]
            .sort_by_key(|&id| self.sprite_x(id));

        for x in 0..SCREEN_X {
            for i in 0..visible_sprites_len {
                let id = visible_sprites[i];
                if !self.is_sprite_horizontally_visible(id, x) {
                    continue;
                }

                let flags = self.sprite_flags(id);

                let color = self.sprite_value_at(
                        id,
                        x + 8 - self.sprite_x(id),
                        scanline + 16 - self.sprite_y(id));

                if color != GrayShade::Transparent {
                    if !flags.below_bg || background[x] == 0 {
                            if x < SCREEN_X && scanline < SCREEN_Y {
                                self.screen_buffer[scanline][x] = color;
                            }
                        }
                    break;
                }
            }
        }
    }

    pub fn get_screen(&self) -> &ScreenBuffer {
        &self.screen_buffer
    }

    pub fn should_refresh(&mut self) -> bool {
        let result = self.should_refresh;
        self.should_refresh = false;
        result
    }

    fn write_scanline(&mut self, i: usize) {
        let mut background = [0; SCREEN_X];

        // Step 1: paint background
        /* if self.mapper.bg_window_on() == 1 {
            let y = (i + (self.mapper.scroll_bg_y as usize)) % BACKGROUND_Y;
            for j in 0..SCREEN_X {
                let x = (j + (self.mapper.scroll_bg_x as usize)) % BACKGROUND_X;
                background[j] = self.raw_background_at(self.mapper.bg_tile_map(), x, y);
            }

            for j in 0..SCREEN_X {
                let color = self.background_color_from_raw(background[j]);
                self.write_pixel(j as usize, i, color);
            }
        } */

        // Step 2: paint the window
        if self.mapper.window_on() == 1
                && self.mapper.bg_window_on() == 1 {
            for j in 0..SCREEN_X {
                if i >= self.mapper.window_y as usize
                        && j + 7 >= self.mapper.window_x as usize
                        && j < BACKGROUND_X - 7 + self.mapper.window_x as usize
                        && i < BACKGROUND_Y + self.mapper.window_y as usize {
                    let x = j - ((self.mapper.window_x as usize) - 7);
                    let y = i - (self.mapper.window_y as usize);

                    let pixel = self.background_value_at(self.mapper.window_tile_map(), x, y);
                    self.write_pixel(j as usize, i as usize, pixel);
                }
            }
        }

        // Step 3: paint sprites
        self.print_sprites(i, &background);
    }

    fn write_pixel(&mut self, x: usize, y: usize, color: GrayShade) {
        if x >= SCREEN_X || y >= SCREEN_Y {
            return;
        }

        self.screen_buffer[y][x] = color;
    }

    pub fn read_ram(&self, address: u16) -> u8 {
        if self.mode == LCDMode::LCDTransfer {
            0xFF
        } else {
            self.video_ram[(address - 0x8000) as usize]
        }
    }

    pub fn write_ram(&mut self, address: u16, v: u8) {
        if self.mode == LCDMode::LCDTransfer {
            // In theory the gb should not be allowed to write to RAM
            // when in this state TODO: double check. In practice I suspect
            // there are some timing bugs that make this miss some video ram
            // updates. For now we ignore this and just happily write to RAM.
        }

        self.video_ram[(address - 0x8000) as usize] = v;
    }

    pub fn add_cycles(&mut self, cycles: usize) {
        if self.mapper.lcd_on() == 0 {
            self.cycles = 0;
            self.set_mode(LCDMode::SearchingOAM);
            self.mapper.lcd_y_coordinate = 0;

            return;
        }

        self.cycles += cycles;
        self.total_cycles += cycles;

        if self.mode == LCDMode::LCDTransfer {
            self.scanline_reader.add_cycles(cycles,
                &self.mapper, &self.video_ram, &mut self.screen_buffer);
        }
    }

    #[must_use]
    fn switch_to(&mut self, mode: LCDMode) -> Option<Interrupt> {
        let enabled = match mode {
            LCDMode::SearchingOAM => self.mapper.oam_interrupt() == 1,
            LCDMode::HBlank       => self.mapper.h_blank_interrupt() == 1,
            LCDMode::VBlank       => self.mapper.v_blank_interrupt() == 1,
            LCDMode::LCDTransfer  => false,
        };

        self.set_mode(mode);

        if enabled {
            Some(Interrupt::Stat)
        } else {
            None
        }
    }

    pub fn check_interrupts(&mut self) -> Option<Interrupt> {
        let done = if self.mode == LCDMode::LCDTransfer {
            self.scanline_reader.current_x() >= SCREEN_X
        } else {
            if self.cycles > self.mode.duration() {
                self.cycles -= self.mode.duration();
                true
            } else {
                false
            }
        };

        if !done {
            return None;
        }

        let mut interrupt = None;

        match self.mode {
            LCDMode::SearchingOAM => {
                interrupt = interrupt.or(self.switch_to(LCDMode::LCDTransfer));
            },
            LCDMode::LCDTransfer => {
                interrupt = interrupt.or(self.switch_to(LCDMode::HBlank));
            },
            LCDMode::HBlank => {
                if self.mapper.lcd_y_coordinate < SCREEN_Y as u8 - 1 {
                    interrupt = interrupt.or(self.switch_to(LCDMode::SearchingOAM));
                } else {
                    let _ = self.switch_to(LCDMode::VBlank);
                    // VBlank takes precedence over Stat
                    interrupt = Some(Interrupt::VBlank);
                }

                let scanline = self.mapper.lcd_y_coordinate as usize;
                self.write_scanline(scanline);
                self.mapper.lcd_y_coordinate += 1;
            },
            LCDMode::VBlank => {
                if self.mapper.lcd_y_coordinate == 153 {
                    interrupt = interrupt.or(self.switch_to(LCDMode::SearchingOAM));
                    self.should_refresh = true;
                }
                self.mapper.lcd_y_coordinate += 1;
            }
        }

        self.mapper.lcd_y_coordinate %= 154;

        if self.mapper.lcd_y_coordinate == self.mapper.lyc_coincidence {
            self.mapper.set_ly_coincidence(1);
            if self.mapper.lyc_ly_coincidence_interrupt() == 1 {
                interrupt = interrupt.or(Some(Interrupt::Stat));
            }
        } else {
            self.mapper.set_ly_coincidence(0);
        }

        interrupt
    }
}

u8_enum!{
    SpritePalette {
        C0 = 0b0,
        C1 = 0b1,
    }
}

u8_enum!{
    TileMap {
        // Area $9800 - $9BFF
        C9800 = 0b0,
        // Area $9C00 - $9FFF
        C9C00 = 0b1,
    }
}

u8_enum!{
    BgTileData {
        // Area $8800 - 97FF
        C8800 = 0b0,
        // Area $8000 - 8FFF
        C8000 = 0b1,
    }
}

u8_enum!{
    SpriteSize {
        // 8 by 8 sprite
        C8by8 = 0b0,
        // 8 by 16 sprite
        C8by16 = 0b1,
    }
}

memory_mapper!{
    name: VideoMemoryMapper,
    fields: [
        0xFF42, 0b00000000, scroll_bg_y, 0;
        0xFF43, 0b00000000, scroll_bg_x, 0;
        0xFF44, 0b00000000, lcd_y_coordinate, 0;
        0xFF45, 0b00000000, lyc_coincidence, 0;
        0xFF4A, 0b00000000, window_y, 0;
        0xFF4B, 0b00000000, window_x, 0;
    ],
    bitfields: {
        getters: [
            0xFF40, 0b00000000, lcd_controller, 0x91, [
                get_0, bg_window_on,       u8;
                get_1, obj_sprite_display, u8;
                get_2, sprite_size,        SpriteSize;
                get_3, bg_tile_map,        TileMap;
                get_4, bg_tile_data,       BgTileData;
                get_5, window_on,          u8;
                get_6, window_tile_map,    TileMap;
                get_7, lcd_on,             u8
            ];
            0xFF47, 0b00000000, bgp, 0xFC, [
                get_01, bg_color_00, GrayShade;
                get_23, bg_color_01, GrayShade;
                get_45, bg_color_10, GrayShade;
                get_67, bg_color_11, GrayShade
            ];
            0xFF48, 0b00000000, obp0, 0xFF, [
                get_23, obp0_palette_01, GrayShade;
                get_45, obp0_palette_10, GrayShade;
                get_67, obp0_palette_11, GrayShade
            ];
            0xFF49, 0b00000000, obp1, 0xFF, [
                get_23, obp1_palette_01, GrayShade;
                get_45, obp1_palette_10, GrayShade;
                get_67, obp1_palette_11, GrayShade
            ]
        ],
        getter_setters: [
            0xFF41, 0b10000000, stat, 0, [
                get_01, set_01, mode,              set_mode,              LCDMode;
                get_2,  set_2,  ly_coincidence,    set_ly_coincidence,    u8;
                get_3,  set_3,  h_blank_interrupt, set_h_blank_interrupt, u8;
                get_4,  set_4,  v_blank_interrupt, set_v_blank_interrupt, u8;
                get_5,  set_5,  oam_interrupt,     set_oam_interrupt,     u8;
                get_6,  set_6,  lyc_ly_coincidence_interrupt, set_lyc_ly_coincidence, u8
            ]
        ],
    },
}
