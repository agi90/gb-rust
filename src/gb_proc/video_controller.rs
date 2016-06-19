use gb_proc::cpu::{Handler, Interrupt};

/* Represents a shade of gray */
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GrayShade {
    C00 = 1,
    C01 = 2,
    C10 = 3,
    C11 = 4,
    Transparent = 5,
}

impl GrayShade {
    pub fn from_u8(v: u8) -> GrayShade {
        match v {
            0b00 => GrayShade::C00,
            0b01 => GrayShade::C01,
            0b10 => GrayShade::C10,
            0b11 => GrayShade::C11,
            _    => panic!(),
        }
    }

    pub fn to_u8(&self) -> u8 {
        match self {
            &GrayShade::C00 => 0b00,
            &GrayShade::C01 => 0b01,
            &GrayShade::C10 => 0b10,
            &GrayShade::C11 => 0b11,
            _               => panic!(),
        }
    }
}

pub type ScreenBuffer = [[GrayShade; 160]; 144];

pub struct VideoController {
    scroll_bg_x: u8,
    scroll_bg_y: u8,
    window_x: u8,
    window_y: u8,
    // Indicates the vertical line to which
    // the present data is transferred to the LCD
    // Driver
    lcd_y_coordinate: u8,

    bg_color_00: GrayShade,
    bg_color_01: GrayShade,
    bg_color_10: GrayShade,
    bg_color_11: GrayShade,

    obp0_palette_01: GrayShade,
    obp0_palette_10: GrayShade,
    obp0_palette_11: GrayShade,

    obp1_palette_01: GrayShade,
    obp1_palette_10: GrayShade,
    obp1_palette_11: GrayShade,

    lcd_controller: LCDController,

    mode: LCDMode,

    cycles: usize,
    total_cycles: usize,

    video_ram: [u8; 8196],
    oam_ram: [u8; 160],

    lyc_coincidence: u8,
    lyc_ly_coincidence_interrupt: bool,
    oam_interrupt: bool,
    v_blank_interrupt: bool,
    h_blank_interrupt: bool,

    screen_buffer: ScreenBuffer,

    should_refresh: bool,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum LCDMode {
    HBlank = 0,
    VBlank = 1,
    SearchingOAM = 2,
    LCDTransfer = 3,
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

impl Handler for VideoController {
    fn read(&self, address: u16) -> u8 {
        match address {
            0x8000 ... 0x9FFF => self.read_ram(address),
            0xFE00 ... 0xFE9F => self.oam_ram[address as usize - 0xFE00],
            0xFF40 => self.lcd_controller.read(),
            0xFF41 => self.stat_read(),
            0xFF42 => self.scroll_bg_y,
            0xFF43 => self.scroll_bg_x,
            0xFF44 => self.lcd_y_coordinate,
            0xFF45 => self.lyc_coincidence,
            0xFF46 => panic!("Cannot read from $FF46"),
            0xFF47 => self.read_bgp(),
            0xFF48 => self.read_obp0(),
            0xFF49 => self.read_obp1(),
            0xFF4A => self.window_y,
            0xFF4B => self.window_x,
            _       => unimplemented!(),
        }
    }

    fn write(&mut self, address: u16, v: u8) {
        match address {
            0x8000 ... 0x9FFF => self.write_ram(address, v),
            0xFE00 ... 0xFE9F => self.oam_ram[address as usize - 0xFE00] = v,
            0xFF40 => { self.lcd_controller.write(v) },
            0xFF41 => { self.stat_write(v) },
            0xFF42 => { self.scroll_bg_y = v },
            0xFF43 => { self.scroll_bg_x = v },
            0xFF45 => { self.lyc_coincidence = v },
            0xFF44 => { self.lcd_y_coordinate = 0 },
            0xFF47 => { self.write_bgp(v) },
            0xFF48 => { self.write_obp0(v) },
            0xFF49 => { self.write_obp1(v) },
            0xFF4A => { self.window_y = v },
            0xFF4B => { self.window_x = v },
            _       => unimplemented!(),
        }
    }
}

fn flip_tile(tile: [[GrayShade; 8]; 8], y_flip: bool, x_flip: bool) -> [[GrayShade; 8]; 8] {
    let mut new_tile = tile;

    if y_flip {
        for i in 0..8 {
            new_tile[i] = tile[7 - i];
        }
    }

    if x_flip {
        for i in 0..8 {
            for j in 0..8 {
                new_tile[i][j] = tile[i][7 - j];
            }
        }
    }

    new_tile
}

struct Sprite {
    below_bg: bool,
    tile: [[GrayShade; 8]; 8],
    x: usize,
    y: usize,
}

impl Sprite {
    pub fn new(x: u8, y: u8, tile: [[GrayShade; 8]; 8], below_bg: bool) -> Sprite {
        Sprite {
            below_bg: below_bg,
            tile: tile,
            x: x as usize,
            y: y as usize,
        }
    }
}

impl VideoController {
    pub fn new() -> VideoController {
        VideoController {
            scroll_bg_x: 0,
            scroll_bg_y: 0,
            window_x: 0,
            window_y: 0,
            lcd_y_coordinate: 0,

            bg_color_00: GrayShade::C00,
            bg_color_01: GrayShade::C11,
            bg_color_10: GrayShade::C11,
            bg_color_11: GrayShade::C11,

            obp0_palette_01: GrayShade::C11,
            obp0_palette_10: GrayShade::C11,
            obp0_palette_11: GrayShade::C11,

            obp1_palette_01: GrayShade::C11,
            obp1_palette_10: GrayShade::C11,
            obp1_palette_11: GrayShade::C11,

            lcd_controller: LCDController::new(),

            mode: LCDMode::SearchingOAM,

            cycles: 0,
            total_cycles: 0,

            video_ram: [0; 8196],
            oam_ram: [0; 160],

            lyc_coincidence: 0,
            lyc_ly_coincidence_interrupt: false,
            oam_interrupt: false,
            v_blank_interrupt: false,
            h_blank_interrupt: false,

            screen_buffer: [[GrayShade::C00; 160]; 144],

            should_refresh: false,
        }
    }

    fn get_color(&self, data: u8) -> GrayShade {
        match data {
            0b00 => self.bg_color_00,
            0b01 => self.bg_color_01,
            0b10 => self.bg_color_10,
            0b11 => self.bg_color_11,
            _ => panic!(),
        }
    }

    fn read_pattern(&self, offset: u16) -> [[GrayShade; 8]; 8] {
        let mut pattern = [[GrayShade::C00; 8]; 8];

        for j in 0..8 {
            let l = self.video_ram[offset as usize + (j * 2)];
            let h = self.video_ram[offset as usize + (j * 2) + 1];

            pattern[j][7] = self.get_color( (0b00000001 & l)       + ((0b00000001 & h) << 1));
            pattern[j][6] = self.get_color(((0b00000010 & l) >> 1) + ((0b00000010 & h)     ));
            pattern[j][5] = self.get_color(((0b00000100 & l) >> 2) + ((0b00000100 & h) >> 1));
            pattern[j][4] = self.get_color(((0b00001000 & l) >> 3) + ((0b00001000 & h) >> 2));
            pattern[j][3] = self.get_color(((0b00010000 & l) >> 4) + ((0b00010000 & h) >> 3));
            pattern[j][2] = self.get_color(((0b00100000 & l) >> 5) + ((0b00100000 & h) >> 4));
            pattern[j][1] = self.get_color(((0b01000000 & l) >> 6) + ((0b01000000 & h) >> 5));
            pattern[j][0] = self.get_color(((0b10000000 & l) >> 7) + ((0b10000000 & h) >> 6));
        }

        pattern
    }

    fn read_patterns(&self, offset: u16, inverse: bool) -> [[[GrayShade; 8]; 8]; 256] {
        let mut patterns = [[[GrayShade::C00; 8]; 8]; 256];

        for i in 0..256 {
            let index = if !inverse { i } else {
                if i < 0x80 {
                    i + 0x80
                } else {
                    i - 0x80
                }
            };

            patterns[index as usize] = self.read_pattern(offset + i * 16);
        }

        patterns
    }

    fn read_background(&self, offset: u16, patterns: [[[GrayShade; 8]; 8]; 256]) -> [[GrayShade; 256]; 256] {
        let mut background = [[GrayShade::C00; 256]; 256];

        for i in 0..32 {
            for j in 0..32 {
                let v = self.video_ram[offset as usize + i * 32 + j];

                for k in 0..8 {
                    for h in 0..8 {
                        background[i * 8 + k][j * 8 + h] = patterns[v as usize][k][h];
                    }
                }
            }
        }

        background
    }

    fn print_sprites(&mut self, sprites: &[Sprite], scanline: usize) {
        let mut i = 0;

        let mut visible_sprites = vec![];
        while i < 40 && visible_sprites.len() < 10 {
            let ref sprite = sprites[i];
            i += 1;

            if scanline + 16 < sprite.y || scanline + 8 >= sprite.y {
                continue;
            }

            visible_sprites.push(sprite);
        }

        for x in 0..160 {
            for sprite in &visible_sprites {
                if x + 8 < sprite.x || x >= sprite.x {
                    continue;
                }

                let color = sprite.tile[scanline + 16 - sprite.y][ x + 8 - sprite.x];

                if color != GrayShade::Transparent {
                    if !sprite.below_bg
                        || (self.screen_buffer[scanline][x] == GrayShade::C00) {
                            self.write_pixel(x, scanline, color);
                        }
                    break;
                }
            }
        }
    }

    fn translate_tile(&self, tile: [[GrayShade; 8]; 8], palette: SpritePalette) -> [[GrayShade; 8]; 8] {
        let mut translated = [[GrayShade::C00; 8]; 8];

        for i in 0..8 {
            for j in 0..8 {
                let color = tile[i][j];
                translated[i][j] = match &palette {
                    &SpritePalette::C0 => {
                        match color {
                            GrayShade::C00 => GrayShade::Transparent,
                            GrayShade::C01 => self.obp0_palette_01,
                            GrayShade::C10 => self.obp0_palette_10,
                            GrayShade::C11 => self.obp0_palette_11,
                            GrayShade::Transparent => panic!(),
                        }
                    },
                    &SpritePalette::C1 => {
                        match color {
                            GrayShade::C00 => GrayShade::Transparent,
                            GrayShade::C01 => self.obp1_palette_01,
                            GrayShade::C10 => self.obp1_palette_10,
                            GrayShade::C11 => self.obp1_palette_11,
                            GrayShade::Transparent => panic!(),
                        }
                    },
                };
            }
        }

        translated
    }

    fn read_sprites(&mut self, patterns: [[[GrayShade; 8]; 8]; 256]) -> Vec<Sprite> {
        let mut sprites = vec![];

        for i in 0..40 {
            let y = self.oam_ram[i * 4];
            let x = self.oam_ram[i * 4 + 1];
            let tile_index = self.oam_ram[i * 4 + 2];
            let flags = self.oam_ram[i * 4 + 3];

            if self.lcd_controller.sprite_size == SpriteSize::C8by8 {
                let tile = patterns[tile_index as usize];
                let below_bg =     flags & (0b10000000) > 0;
                let y_flip   =     flags & (0b01000000) > 0;
                let x_flip   =     flags & (0b00100000) > 0;
                let palette  =  if flags & (0b00010000) > 0 { SpritePalette::C1 } else { SpritePalette::C0 };

                let translated_tile = flip_tile(self.translate_tile(tile, palette), y_flip, x_flip);
                sprites.push(Sprite::new(x, y, translated_tile, below_bg));
            } else {
                unimplemented!();
            }
        }

        // Sprites are ordered by display priority
        sprites.sort_by_key(|s| s.x);
        sprites
    }

    pub fn get_screen(&self) -> &ScreenBuffer {
        &self.screen_buffer
    }

    pub fn should_refresh(&mut self) -> bool {
        let result = self.should_refresh;
        self.should_refresh = false;
        result
    }

    fn refresh(&mut self) {
        let sprite_patterns = self.read_patterns(0x0000, false);
        let patterns = if self.lcd_controller.bg_tile_data == BgTileData::C8000 {
            sprite_patterns
        } else {
            self.read_patterns(0x0800, true)
        };

        let background = if self.lcd_controller.bg_tile_map == TileMap::C9800 {
            self.read_background(0x1800, patterns)
        } else {
            self.read_background(0x1C00, patterns)
        };

        let window = if self.lcd_controller.window_tile_map == TileMap::C9800 {
            self.read_background(0x1800, patterns)
        } else {
            self.read_background(0x1C00, patterns)
        };

        let sprites = self.read_sprites(sprite_patterns);

        for i in 0..144 {
            // Step 0: Blank screen
            for j in 0..160 {
                self.write_pixel(j as usize, i as usize, GrayShade::C00);
            }

            // Step 2: paint background
            if self.lcd_controller.bg_window_on {
                for j in 0..160 {
                    let x = (j + (self.scroll_bg_x as usize)) % 256;
                    let y = (i + (self.scroll_bg_y as usize)) % 256;
                    if background[y][x] != GrayShade::C00 {
                        self.write_pixel(j as usize, i as usize, background[y][x]);
                    }
                }
            }

            // Step 3: paint the window
            if self.lcd_controller.window_on
                    && self.lcd_controller.bg_window_on {
                for j in 0..160 {
                    if i >= self.window_y as usize &&
                            j + 7 >= self.window_x as usize &&
                            j < 249 + self.window_x as usize &&
                            i < 256 + self.window_y as usize {
                        let x = j - ((self.window_x as usize) - 7);
                        let y = i - (self.window_y as usize);
                        self.write_pixel(j as usize, i as usize, window[y][x]);
                    }
                }
            }

            // Step 4: paint sprites
            self.print_sprites(&sprites, i);
        }
    }

    fn write_pixel(&mut self, x: usize, y: usize, color: GrayShade) {
        if x > 159 || y > 143 {
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
            return;
        }

        self.video_ram[(address - 0x8000) as usize] = v;
    }

    pub fn add_cycles(&mut self, cycles: usize) {
        if !self.lcd_controller.lcd_on {
            self.cycles = 0;
            self.mode = LCDMode::SearchingOAM;
            self.lcd_y_coordinate = 0;

            return;
        }

        self.cycles += cycles;
        self.total_cycles += cycles;
    }

    pub fn check_interrupts(&mut self) -> Vec<Interrupt> {
        let mut interrupts = vec![];

        if self.cycles > self.mode.duration() {
            self.cycles -= self.mode.duration();

            match self.mode {
                LCDMode::SearchingOAM => {
                    self.mode = LCDMode::LCDTransfer;
                },
                LCDMode::LCDTransfer => { self.mode = LCDMode::HBlank; },
                LCDMode::HBlank => {
                    if self.lcd_y_coordinate < 143 {
                        self.mode = LCDMode::SearchingOAM;
                    } else {
                        self.mode = LCDMode::VBlank;
                        interrupts.push(Interrupt::VBlank);
                    }

                    self.lcd_y_coordinate += 1;
                },
                LCDMode::VBlank => {
                    if self.lcd_y_coordinate == 153 {
                        self.mode = LCDMode::SearchingOAM;
                        self.refresh();
                        self.should_refresh = true;
                    }

                    self.lcd_y_coordinate += 1;
                }
            }

            self.lcd_y_coordinate %= 154;

            if self.lcd_y_coordinate == self.lyc_coincidence &&
                self.lyc_ly_coincidence_interrupt {
                interrupts.push(Interrupt::Stat);
            }
        }

        interrupts
    }

    fn read_obp1(&self) -> u8 {
            (self.obp1_palette_01.to_u8() << 2) +
            (self.obp1_palette_10.to_u8() << 4) +
            (self.obp1_palette_11.to_u8() << 6)
    }

    fn read_obp0(&self) -> u8 {
        self.bg_color_00.to_u8() +
            (self.bg_color_01.to_u8() << 2) +
            (self.bg_color_10.to_u8() << 4) +
            (self.bg_color_11.to_u8() << 6)
    }

    fn read_bgp(&self) -> u8 { unimplemented!(); }

    fn write_bgp(&mut self, v: u8) {
        self.bg_color_00 = GrayShade::from_u8(0b00000011 & v);
        self.bg_color_01 = GrayShade::from_u8((0b00001100 & v) >> 2);
        self.bg_color_10 = GrayShade::from_u8((0b00110000 & v) >> 4);
        self.bg_color_11 = GrayShade::from_u8((0b11000000 & v) >> 6);
    }

    fn write_obp1(&mut self, v: u8) {
        // The 00 always means transparent for sprites
        self.obp1_palette_01 = GrayShade::from_u8((0b00001100 & v) >> 2);
        self.obp1_palette_10 = GrayShade::from_u8((0b00110000 & v) >> 4);
        self.obp1_palette_11 = GrayShade::from_u8((0b11000000 & v) >> 6);
    }

    fn write_obp0(&mut self, v: u8) {
        // The 00 always means transparent for sprites
        self.obp0_palette_01 = GrayShade::from_u8((0b00001100 & v) >> 2);
        self.obp0_palette_10 = GrayShade::from_u8((0b00110000 & v) >> 4);
        self.obp0_palette_11 = GrayShade::from_u8((0b11000000 & v) >> 6);
    }

    fn stat_read(&self) -> u8 {
        let ly_coincidence = self.lcd_y_coordinate == self.lyc_coincidence;
        self.mode as u8 +
            (if ly_coincidence                    { 0b00000100 } else { 0 }) +
            (if self.h_blank_interrupt            { 0b00001000 } else { 0 }) +
            (if self.v_blank_interrupt            { 0b00010000 } else { 0 }) +
            (if self.oam_interrupt                { 0b00100000 } else { 0 }) +
            (if self.lyc_ly_coincidence_interrupt { 0b01000000 } else { 0 })
    }

    fn stat_write(&mut self, v: u8) {
        self.h_blank_interrupt            = (v & 0b00001000) > 0;
        self.v_blank_interrupt            = (v & 0b00010000) > 0;
        self.oam_interrupt                = (v & 0b00100000) > 0;
        self.lyc_ly_coincidence_interrupt = (v & 0b01000000) > 0;

        if self.h_blank_interrupt ||
            self.v_blank_interrupt ||
            self.oam_interrupt ||
            self.lyc_ly_coincidence_interrupt {
                unimplemented!();
        }
    }
}

#[derive(Clone, Copy)]
enum SpritePalette {
    C0,
    C1,
}

#[derive(Debug, PartialEq, Eq)]
enum TileMap {
    // Area $9800 - $9BFF
    C9800,
    // Area $9C00 - $9FFF
    C9C00
}

#[derive(Debug, PartialEq, Eq)]
enum BgTileData {
    // Area $8800 - 97FF
    C8800,
    // Area $8000 - 8FFF
    C8000,
}

#[derive(Debug, PartialEq, Eq)]
enum SpriteSize {
    // 8 by 8 sprite
    C8by8,
    // 8 by 16 sprite
    C8by16,
}

struct LCDController {
    lcd_on: bool,
    window_tile_map: TileMap,
    window_on: bool,
    bg_tile_data: BgTileData,
    bg_tile_map: TileMap,
    sprite_size: SpriteSize,
    obj_sprite_display: bool,
    bg_window_on: bool,
}

impl LCDController {
    pub fn new() -> LCDController {
        LCDController {
            lcd_on: true,
            window_tile_map: TileMap::C9800,
            window_on: false,
            bg_tile_data: BgTileData::C8000,
            bg_tile_map: TileMap::C9800,
            sprite_size: SpriteSize::C8by8,
            obj_sprite_display: false,
            bg_window_on: true,
        }
    }

    pub fn read(&self) -> u8 {
        (if self.bg_window_on                          { 0b00000001 } else { 0 }) +
        (if self.obj_sprite_display                    { 0b00000010 } else { 0 }) +
        (if self.sprite_size     == SpriteSize::C8by16 { 0b00000100 } else { 0 }) +
        (if self.bg_tile_map     == TileMap::C9C00     { 0b00001000 } else { 0 }) +
        (if self.bg_tile_data    == BgTileData::C8000  { 0b00010000 } else { 0 }) +
        (if self.window_on                             { 0b00100000 } else { 0 }) +
        (if self.window_tile_map == TileMap::C9C00     { 0b01000000 } else { 0 }) +
        (if self.lcd_on                                { 0b10000000 } else { 0 })
    }

    pub fn write(&mut self, v: u8) {
        self.bg_window_on      =    (v & 0b00000001) > 0;
        self.obj_sprite_display=    (v & 0b00000010) > 0;
        self.sprite_size       = if (v & 0b00000100) > 0 { SpriteSize::C8by16 } else { SpriteSize::C8by8 };
        self.bg_tile_map       = if (v & 0b00001000) > 0 { TileMap::C9C00 } else { TileMap::C9800 };
        self.bg_tile_data      = if (v & 0b00010000) > 0 { BgTileData::C8000 } else { BgTileData::C8800 };
        self.window_on         =    (v & 0b00100000) > 0;
        self.window_tile_map   = if (v & 0b01000000) > 0 { TileMap::C9C00 } else { TileMap::C9800 };
        self.lcd_on            =    (v & 0b10000000) > 0;
    }
}
