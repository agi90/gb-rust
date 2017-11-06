use hardware::cpu;
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

const SCANLINE_CYCLES: usize = 456;

// Includes invisible lines
const VERTICAL_LINES: usize = 154;

const SCREEN_CYCLES: usize = SCANLINE_CYCLES * VERTICAL_LINES;

pub type ScreenBuffer = [[GrayShade; SCREEN_X]; SCREEN_Y];

pub struct Ppu {
    cycles: usize,
    video_ram: [u8; 8196],
    oam_ram: [u8; 160],
    screen_buffer: ScreenBuffer,
    should_refresh: bool,
    mapper: VideoMemoryMapper,
    mode: LCDMode,
    background: [u8; SCREEN_X],
}

u8_enum!{
    LCDMode {
        HBlank = 0,
        VBlank = 1,
        SearchingOAM = 2,
        LCDTransfer = 3,
    }
}

impl cpu::Handler for Ppu {
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
            0xFF41 => self.write_stat(v),
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

impl Ppu {
    pub fn new() -> Ppu {
        Ppu {
            cycles: 0,
            video_ram: [0; 8196],
            oam_ram: [0; 160],
            screen_buffer: [[GrayShade::C00; SCREEN_X]; SCREEN_Y],
            should_refresh: false,
            mapper: VideoMemoryMapper::new(),
            mode: LCDMode::HBlank,
            background: [0; SCREEN_X],
        }
    }

    fn set_mode(&mut self, mode: LCDMode) {
        self.mode = mode;
        self.mapper.set_mode(mode);
    }

    fn write_stat(&mut self, v: u8) {
        // Only bit 3456 are writable
        self.mapper.stat.set_3456((v >> 3) & 0b1111);
    }

    fn write_callback(&mut self, address: u16) {
        match address {
            0xFF40 => {
                if self.mapper.lcd_on() == 0 {
                    self.cycles = 0;
                    self.set_mode(LCDMode::HBlank);
                    self.mapper.lcd_y_coordinate = 0;
                }
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

    fn print_sprites(&mut self, scanline: usize) {
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
                    if !flags.below_bg || self.background[x] == 0 {
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

    fn write_pixel(&mut self, i: usize, j: usize) {
        assert!(i < SCREEN_Y);
        assert!(j < SCREEN_X);

        // Step 0: Blank screen
        self.write_raw_pixel(j as usize, i as usize, GrayShade::C00);
        self.background[j] = 0;

        // Step 1: paint background
        if self.mapper.bg_window_on() == 1 {
            let y = (i + (self.mapper.scroll_bg_y as usize)) % BACKGROUND_Y;
            let x = (j + (self.mapper.scroll_bg_x as usize)) % BACKGROUND_X;
            self.background[j] = self.raw_background_at(self.mapper.bg_tile_map(), x, y);

            let color = self.background_color_from_raw(self.background[j]);
            self.write_raw_pixel(j, i, color);
        }

        // Step 2: paint the window
        if self.mapper.window_on() == 1
                && self.mapper.bg_window_on() == 1 {
            if i >= self.mapper.window_y as usize
                    && j + 7 >= self.mapper.window_x as usize
                    && j < BACKGROUND_X - 7 + self.mapper.window_x as usize
                    && i < BACKGROUND_Y + self.mapper.window_y as usize {
                let x = j - ((self.mapper.window_x as usize) - 7);
                let y = i - (self.mapper.window_y as usize);

                let pixel = self.background_value_at(self.mapper.window_tile_map(), x, y);
                self.write_raw_pixel(j as usize, i as usize, pixel);
            }
        }
    }

    fn write_raw_pixel(&mut self, x: usize, y: usize, color: GrayShade) {
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

    pub fn cpu_step(&mut self) {
        if self.mapper.lcd_on() == 0 {
            return;
        }

        self.cycles = (self.cycles + cpu::CYCLES_PER_STEP) % SCREEN_CYCLES;
    }

    fn switch_to(&mut self, mode: LCDMode) -> Option<cpu::Interrupt> {
        let enabled = match mode {
            LCDMode::SearchingOAM => self.mapper.oam_interrupt() == 1,
            LCDMode::HBlank       => self.mapper.h_blank_interrupt() == 1,
            LCDMode::VBlank       => self.mapper.v_blank_interrupt() == 1,
            LCDMode::LCDTransfer  => false,
        };

        self.set_mode(mode);

        if enabled {
            Some(cpu::Interrupt::Stat)
        } else {
            None
        }
    }

    fn scanline(&self) -> u8 {
        self.mapper.lcd_y_coordinate
    }

    fn update_scanline(&mut self) {
        assert!(self.scanline() == 153 ||
            self.scanline() + 1 == (self.cycles / SCANLINE_CYCLES) as u8);

        // Beginning of the scanline, we need to update LY
        self.mapper.lcd_y_coordinate = (self.cycles / SCANLINE_CYCLES) as u8;

        // In this cycle, ly_coincidence is 0 no matter what
        self.mapper.set_ly_coincidence(0);
    }

    fn update_ly_lyc_coincidence(&mut self) -> Option<cpu::Interrupt> {
        if self.scanline() == self.mapper.lyc_coincidence {
            self.mapper.set_ly_coincidence(1);
            if self.mapper.lyc_ly_coincidence_interrupt() == 1 {
                return Some(cpu::Interrupt::Stat);
            }
        }

        None
    }

    fn check_interrupts_vblank(&mut self) -> Option<cpu::Interrupt> {
        match self.cycles % SCANLINE_CYCLES {
            0 => self.update_scanline(),
            4 => {
                let mut interrupt = self.update_ly_lyc_coincidence();

                if self.scanline() == 0 {
                    // End of VBlank
                    interrupt = interrupt.or(self.switch_to(LCDMode::SearchingOAM));
                }

                return interrupt;
            },
            8 ... 452 => {
                // VBlank running, nothing to do
            },
            _ => unreachable!(),
        }

        None
    }

    pub fn check_interrupts(&mut self) -> Option<cpu::Interrupt> {
        if self.mapper.lcd_on() == 0 {
            return None;
        }

        let mut interrupt = None;

        if self.cycles % 4 != 0 {
            // We only care about M-cycles
            return None;
        }

        if self.mode == LCDMode::VBlank {
            return self.check_interrupts_vblank();
        }

        // If we're not in VBlank we must have a coordinate inside the screen
        assert!(self.scanline() as usize <= SCREEN_Y);

        let scanline_cycle = self.cycles % SCANLINE_CYCLES;

        match scanline_cycle {
            0 => self.update_scanline(),
            4 => {
                interrupt = interrupt.or(self.update_ly_lyc_coincidence());

                if self.scanline() == SCREEN_Y as u8 {
                    // Let's notify the front-end that we're ready to refresh the screen
                    self.should_refresh = true;

                    // VBlank interrupt takes precedence over STAT
                    let _ = self.switch_to(LCDMode::VBlank);
                    return Some(cpu::Interrupt::VBlank);
                }

                interrupt = interrupt.or(self.switch_to(LCDMode::SearchingOAM));
            },
            8 ... 80 | 88 | 252 | 260 ... 452 => {
                // The ppu is running one of the modes, do nothing
            },
            84 => {
                interrupt = interrupt.or(self.switch_to(LCDMode::LCDTransfer));
            },
            92 ... 248 => {
                for i in 0..4 {
                    // This is only an approximation of what actually happens.
                    // More information is available at:
                    // http://blog.kevtris.org/blogfiles/Nitty%20Gritty%20Gameboy%20VRAM%20Timing.txt
                    let scanline = self.scanline() as usize;
                    self.write_pixel(scanline, scanline_cycle - 92 + i);
                }
            },
            256 => {
                interrupt = interrupt.or(self.switch_to(LCDMode::HBlank));
                let scanline = self.scanline() as usize;
                self.print_sprites(scanline);
            },
            _ => unreachable!(),
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
