use gb_proc::cpu::{Handler, Interrupt};
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

const PATTERNS_SIZE: usize = 256;

pub type ScreenBuffer = [[GrayShade; SCREEN_X]; SCREEN_Y];
pub type BackgroundBuffer = [[GrayShade; BACKGROUND_X]; BACKGROUND_Y];

type Pattern = [[GrayShade; 8]; 8];
type Pattern16 = [[GrayShade; 8]; 16];

type Patterns = [Pattern; PATTERNS_SIZE];

pub struct VideoController {
    cycles: usize,
    total_cycles: usize,

    video_ram: [u8; 8196],
    oam_ram: [u8; 160],

    screen_buffer: ScreenBuffer,
    background_buffer: BackgroundBuffer,
    window_buffer: BackgroundBuffer,

    bg_patterns: Patterns,
    sprite_patterns: Patterns,

    sprites: Vec<Sprite>,

    should_refresh: bool,

    mapper: VideoMemoryMapper,
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

impl Handler for VideoController {
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
    }
}

fn flip_tile(tile: Pattern, y_flip: bool, x_flip: bool) -> Pattern {
    let mut new_tile = tile;
    if y_flip {
        for i in 0..8 {
            new_tile[i] = tile[7 - i];
        }
    }

    let mut new_tile2 = new_tile;
    if x_flip {
        for i in 0..8 {
            for j in 0..8 {
                new_tile2[i][j] = new_tile[i][7 - j];
            }
        }
    }

    new_tile2
}

enum Sprite {
    C8by8(Sprite8),
    C8by16(Sprite16),
}

impl Sprite {
    pub fn is_visible(&self, scanline: usize) -> bool {
        match self {
            &Sprite::C8by8(ref sp) => {
                scanline + 16 >= sp.y && scanline + 8 < sp.y
            },
            &Sprite::C8by16(ref sp) => {
                scanline + 16 >= sp.y && scanline < sp.y
            }
        }
    }

    pub fn is_horizontally_visible(&self, x: usize) -> bool {
        x + 8 >= self.x() && x < self.x()
    }

    pub fn below_bg(&self) -> bool {
        match self {
            &Sprite::C8by8(ref sp) => {
                sp.below_bg
            },
            &Sprite::C8by16(ref sp) => {
                sp.below_bg
            }
        }
    }

    pub fn get(&self, x: usize, y: usize) -> GrayShade {
        match self {
            &Sprite::C8by8(ref sp) => {
                sp.tile[y][x]
            },
            &Sprite::C8by16(ref sp) => {
                sp.tile[y][x]
            }
        }
    }

    pub fn y(&self) -> usize {
        match self {
            &Sprite::C8by8(ref sp) => {
                sp.y
            },
            &Sprite::C8by16(ref sp) => {
                sp.y
            }
        }
    }

    pub fn x(&self) -> usize {
        match self {
            &Sprite::C8by8(ref sp) => {
                sp.x
            },
            &Sprite::C8by16(ref sp) => {
                sp.x
            }
        }
    }
}

struct Sprite8 {
    below_bg: bool,
    tile: Pattern,
    x: usize,
    y: usize,
}

impl Sprite8 {
    pub fn new(x: u8, y: u8, tile: Pattern, below_bg: bool) -> Sprite8 {
        Sprite8 {
            below_bg: below_bg,
            tile: tile,
            x: x as usize,
            y: y as usize,
        }
    }
}

struct Sprite16 {
    below_bg: bool,
    tile: Pattern16,
    x: usize,
    y: usize,
}

impl Sprite16 {
    pub fn new(x: u8, y: u8, upper_tile: Pattern, lower_tile: Pattern, below_bg: bool) -> Sprite16 {
        let mut tile = [[GrayShade::C00; 8]; 16];
        for x in 0..8 {
            for y in 0..16 {
                if y < 8 {
                    tile[y][x] = upper_tile[y][x];
                } else {
                    tile[y][x] = lower_tile[y - 8][x];
                }
            }
        }

        Sprite16 {
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
            cycles: 0,
            total_cycles: 0,

            video_ram: [0; 8196],
            oam_ram: [0; 160],

            screen_buffer: [[GrayShade::C00; SCREEN_X]; SCREEN_Y],
            background_buffer: [[GrayShade::C00; BACKGROUND_X]; BACKGROUND_Y],
            window_buffer: [[GrayShade::C00; BACKGROUND_X]; BACKGROUND_Y],

            bg_patterns: [[[GrayShade::C00; 8]; 8]; PATTERNS_SIZE],
            sprite_patterns: [[[GrayShade::C00; 8]; 8]; PATTERNS_SIZE],

            sprites: vec![],

            should_refresh: false,

            mapper: VideoMemoryMapper::new(),
        }
    }

    fn read_pattern(&self, offset: u16) -> Pattern {
        let mut pattern = [[GrayShade::C00; 8]; 8];

        for j in 0..8 {
            let l = self.video_ram[offset as usize + (j * 2)];
            let h = self.video_ram[offset as usize + (j * 2) + 1];

            pattern[j][7] = GrayShade::from( (0b00000001 & l)       + ((0b00000001 & h) << 1));
            pattern[j][6] = GrayShade::from(((0b00000010 & l) >> 1) + ((0b00000010 & h)     ));
            pattern[j][5] = GrayShade::from(((0b00000100 & l) >> 2) + ((0b00000100 & h) >> 1));
            pattern[j][4] = GrayShade::from(((0b00001000 & l) >> 3) + ((0b00001000 & h) >> 2));
            pattern[j][3] = GrayShade::from(((0b00010000 & l) >> 4) + ((0b00010000 & h) >> 3));
            pattern[j][2] = GrayShade::from(((0b00100000 & l) >> 5) + ((0b00100000 & h) >> 4));
            pattern[j][1] = GrayShade::from(((0b01000000 & l) >> 6) + ((0b01000000 & h) >> 5));
            pattern[j][0] = GrayShade::from(((0b10000000 & l) >> 7) + ((0b10000000 & h) >> 6));
        }

        pattern
    }

    fn read_patterns(&self, offset: u16, inverse: bool) -> Patterns {
        let mut patterns = [[[GrayShade::C00; 8]; 8]; PATTERNS_SIZE];

        for i in 0..PATTERNS_SIZE {
            let index = if !inverse { i } else {
                if i < 0x80 {
                    i + 0x80
                } else {
                    i - 0x80
                }
            };

            patterns[index] = self.read_pattern(offset + (i as u16) * 16);
        }

        patterns
    }

    fn get_bg_color(&self, data: GrayShade) -> GrayShade {
        match data {
            GrayShade::C00 => self.mapper.bg_color_00(),
            GrayShade::C01 => self.mapper.bg_color_01(),
            GrayShade::C10 => self.mapper.bg_color_10(),
            GrayShade::C11 => self.mapper.bg_color_11(),
            // The background can never be transparent
            GrayShade::Transparent => panic!(),
        }
    }

    fn read_background(&self, offset: u16, patterns: &Patterns) -> BackgroundBuffer {
        let mut background = [[GrayShade::C00; BACKGROUND_X]; BACKGROUND_Y];

        for i in 0..32 {
            for j in 0..32 {
                let v = self.video_ram[offset as usize + i * 32 + j];

                for k in 0..8 {
                    for h in 0..8 {
                        background[i * 8 + k][j * 8 + h] = self.get_bg_color(patterns[v as usize][k][h]);
                    }
                }
            }
        }

        background
    }

    fn print_sprites(&mut self, scanline: usize) {
        let mut i = 0;

        let mut visible_sprites = vec![];
        while i < 40 && i < self.sprites.len() && visible_sprites.len() < 10 {
            let ref sprite = self.sprites[i];
            i += 1;

            if !sprite.is_visible(scanline) {
                continue;
            }

            visible_sprites.push(sprite);
        }

        visible_sprites.sort_by_key(|sp| sp.x());

        for x in 0..SCREEN_X {
            for sprite in &visible_sprites {
                if !sprite.is_horizontally_visible(x) {
                    continue;
                }

                let color = sprite.get(x + 8 - sprite.x(), scanline + 16 - sprite.y());

                if color != GrayShade::Transparent {
                    if !sprite.below_bg()
                        || (self.screen_buffer[scanline][x] == GrayShade::C00) {
                            if x < SCREEN_X && scanline < SCREEN_Y {
                                self.screen_buffer[scanline][x] = color;
                            }
                        }
                    break;
                }
            }
        }
    }

    fn translate_tile(&self, tile: Pattern, palette: SpritePalette) -> Pattern {
        let mut translated = [[GrayShade::C00; 8]; 8];

        for i in 0..8 {
            for j in 0..8 {
                let color = tile[i][j];
                translated[i][j] = match &palette {
                    &SpritePalette::C0 => {
                        match color {
                            GrayShade::C00 => GrayShade::Transparent,
                            GrayShade::C01 => self.mapper.obp0_palette_01(),
                            GrayShade::C10 => self.mapper.obp0_palette_10(),
                            GrayShade::C11 => self.mapper.obp0_palette_11(),
                            GrayShade::Transparent => panic!(),
                        }
                    },
                    &SpritePalette::C1 => {
                        match color {
                            GrayShade::C00 => GrayShade::Transparent,
                            GrayShade::C01 => self.mapper.obp1_palette_01(),
                            GrayShade::C10 => self.mapper.obp1_palette_10(),
                            GrayShade::C11 => self.mapper.obp1_palette_11(),
                            GrayShade::Transparent => panic!(),
                        }
                    },
                };
            }
        }

        translated
    }

    fn read_sprites(&mut self) {
        let mut sprites = vec![];

        // OAM ram contains information for 40 sprites. For each sprite:
        for i in 0..40 {
            // - byte 1 is the Y position
            let y = self.oam_ram[i * 4];

            // - byte 2 is the X position
            let x = self.oam_ram[i * 4 + 1];

            // - byte 3 contains the tile number (or the tile numbers for 16x8 sprites)
            let tile_index = self.oam_ram[i * 4 + 2];

            // - byte 4 contains some flags about the sprite
            let flags = self.oam_ram[i * 4 + 3];

            let below_bg =     flags & (0b10000000) > 0;
            let y_flip   =     flags & (0b01000000) > 0;
            let x_flip   =     flags & (0b00100000) > 0;
            let palette  =  if flags & (0b00010000) > 0 { SpritePalette::C1 } else { SpritePalette::C0 };

            match self.mapper.sprite_size() {
                SpriteSize::C8by8 => {
                    let tile = self.sprite_patterns[tile_index as usize];
                    let translated_tile = flip_tile(self.translate_tile(tile, palette), y_flip, x_flip);
                    sprites.push(Sprite::C8by8(Sprite8::new(x, y, translated_tile, below_bg)));
                },
                SpriteSize::C8by16 => {
                    let tile1 = self.sprite_patterns[(tile_index & 0xFE) as usize];
                    let tile2 = self.sprite_patterns[(tile_index | 0x01) as usize];
                    let translated_tile1 = flip_tile(self.translate_tile(tile1, palette), y_flip, x_flip);
                    let translated_tile2 = flip_tile(self.translate_tile(tile2, palette), y_flip, x_flip);
                    sprites.push(Sprite::C8by16(Sprite16::new(x, y, translated_tile1, translated_tile2, below_bg)));
                }
            }
        }

        self.sprites = sprites;
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
        self.sprite_patterns = self.read_patterns(0x0000, false);
        self.bg_patterns = self.read_patterns(0x0800, true);

        {
            let patterns = if self.mapper.bg_tile_data() == BgTileData::C8000 {
                &self.sprite_patterns
            } else {
                &self.bg_patterns
            };

            self.background_buffer = if self.mapper.bg_tile_map() == TileMap::C9800 {
                self.read_background(0x1800, patterns)
            } else {
                self.read_background(0x1C00, patterns)
            };

            self.window_buffer = if self.mapper.window_tile_map() == TileMap::C9800 {
                self.read_background(0x1800, patterns)
            } else {
                self.read_background(0x1C00, patterns)
            };
        }

        self.read_sprites();
    }

    fn write_scanline(&mut self, i: usize) {
        // Step 0: Blank screen
        for j in 0..SCREEN_X {
            self.write_pixel(j as usize, i as usize, GrayShade::C00);
        }

        // Step 1: paint background
        if self.mapper.bg_window_on() == 1 {
            for j in 0..SCREEN_X {
                let x = (j + (self.mapper.scroll_bg_x as usize)) % BACKGROUND_X;
                let y = (i + (self.mapper.scroll_bg_y as usize)) % BACKGROUND_Y;

                let pixel = self.background_buffer[y][x];
                if pixel != GrayShade::C00 {
                    self.write_pixel(j as usize, i as usize, pixel);
                }
            }
        }

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

                    let pixel = self.window_buffer[y][x];
                    self.write_pixel(j as usize, i as usize, pixel);
                }
            }
        }

        // Step 3: paint sprites
        self.print_sprites(i);
    }

    fn write_pixel(&mut self, x: usize, y: usize, color: GrayShade) {
        if x >= SCREEN_X || y >= SCREEN_Y {
            return;
        }

        self.screen_buffer[y][x] = color;
    }

    pub fn read_ram(&self, address: u16) -> u8 {
        if self.mapper.mode() == LCDMode::LCDTransfer {
            0xFF
        } else {
            self.video_ram[(address - 0x8000) as usize]
        }
    }

    pub fn write_ram(&mut self, address: u16, v: u8) {
        if self.mapper.mode() == LCDMode::LCDTransfer {
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
            self.mapper.set_mode(LCDMode::SearchingOAM);
            self.mapper.lcd_y_coordinate = 0;

            return;
        }

        self.cycles += cycles;
        self.total_cycles += cycles;
    }

    fn switch_to(&mut self, mode: LCDMode, interrupts: &mut Vec<Interrupt>) {
        let enabled = match mode {
            LCDMode::SearchingOAM => self.mapper.oam_interrupt() == 1,
            LCDMode::HBlank       => self.mapper.h_blank_interrupt() == 1,
            LCDMode::VBlank       => self.mapper.v_blank_interrupt() == 1,
            LCDMode::LCDTransfer  => false,
        };

        self.mapper.set_mode(mode);

        if enabled {
            interrupts.push(Interrupt::Stat);
        }
    }

    pub fn check_interrupts(&mut self) -> Vec<Interrupt> {
        let mut interrupts = vec![];

        if self.cycles > self.mapper.mode().duration() {
            self.cycles -= self.mapper.mode().duration();

            match self.mapper.mode() {
                LCDMode::SearchingOAM => {
                    self.switch_to(LCDMode::LCDTransfer, &mut interrupts);
                },
                LCDMode::LCDTransfer => {
                    self.switch_to(LCDMode::HBlank, &mut interrupts);
                },
                LCDMode::HBlank => {
                    if self.mapper.lcd_y_coordinate < SCREEN_Y as u8 - 1 {
                        self.switch_to(LCDMode::SearchingOAM, &mut interrupts);
                    } else {
                        interrupts.push(Interrupt::VBlank);
                        self.switch_to(LCDMode::VBlank, &mut interrupts);
                    }

                    let scanline = self.mapper.lcd_y_coordinate as usize;
                    self.write_scanline(scanline);
                    self.mapper.lcd_y_coordinate += 1;
                },
                LCDMode::VBlank => {
                    if self.mapper.lcd_y_coordinate == 153 {
                        self.switch_to(LCDMode::SearchingOAM, &mut interrupts);
                        self.refresh();
                        self.should_refresh = true;
                    }

                    self.mapper.lcd_y_coordinate += 1;
                }
            }

            self.mapper.lcd_y_coordinate %= 154;

            if self.mapper.lcd_y_coordinate == self.mapper.lyc_coincidence {
                self.mapper.set_ly_coincidence(1);
                if self.mapper.lyc_ly_coincidence_interrupt() == 1 {
                    interrupts.push(Interrupt::Stat);
                }
            } else {
                self.mapper.set_ly_coincidence(0);
            }
        }

        interrupts
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
