/* Represents a shade of gray */
enum GrayShade {
    C00,
    C01,
    C10,
    C11,
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
}

pub struct VideoController {
    scroll_bg_x: u8,
    scroll_bg_y: u8,
    window_x: u8,
    window_y: u8,
    // Indicates the vertical line to which
    // the present data is transferred to the LCD
    // Driver
    lcd_y_coordinate: u8,

    dot_data_00: GrayShade,
    dot_data_01: GrayShade,
    dot_data_10: GrayShade,
    dot_data_11: GrayShade,

    obp0_palette_00: GrayShade,
    obp0_palette_01: GrayShade,
    obp0_palette_10: GrayShade,
    obp0_palette_11: GrayShade,

    obp1_palette_00: GrayShade,
    obp1_palette_01: GrayShade,
    obp1_palette_10: GrayShade,
    obp1_palette_11: GrayShade,

    lcd_controller: LCDController,
}

impl VideoController {
    pub fn new() -> VideoController {
        VideoController {
            scroll_bg_x: 0,
            scroll_bg_y: 0,
            window_x: 0,
            window_y: 0,
            lcd_y_coordinate: 0,

            dot_data_00: GrayShade::C00,
            dot_data_01: GrayShade::C11,
            dot_data_10: GrayShade::C11,
            dot_data_11: GrayShade::C11,

            obp0_palette_00: GrayShade::C11,
            obp0_palette_01: GrayShade::C11,
            obp0_palette_10: GrayShade::C11,
            obp0_palette_11: GrayShade::C11,

            obp1_palette_00: GrayShade::C11,
            obp1_palette_01: GrayShade::C11,
            obp1_palette_10: GrayShade::C11,
            obp1_palette_11: GrayShade::C11,

            lcd_controller: LCDController::new(),
        }
    }

    fn read_obp1(&self) -> u8 { unimplemented!(); }
    fn read_obp0(&self) -> u8 { unimplemented!(); }
    fn read_bgp(&self) -> u8 { unimplemented!(); }

    fn write_bgp(&mut self, v: u8) {
        self.dot_data_00 = GrayShade::from_u8(0b00000011 & v);
        self.dot_data_01 = GrayShade::from_u8((0b00001100 & v) >> 2);
        self.dot_data_10 = GrayShade::from_u8((0b00110000 & v) >> 4);
        self.dot_data_11 = GrayShade::from_u8((0b11000000 & v) >> 6);
    }

    fn write_obp1(&mut self, v: u8) {
        self.obp1_palette_00 = GrayShade::from_u8(0b00000011 & v);
        self.obp1_palette_01 = GrayShade::from_u8((0b00001100 & v) >> 2);
        self.obp1_palette_10 = GrayShade::from_u8((0b00110000 & v) >> 4);
        self.obp1_palette_11 = GrayShade::from_u8((0b11000000 & v) >> 6);
    }

    fn write_obp0(&mut self, v: u8) {
        self.obp0_palette_00 = GrayShade::from_u8(0b00000011 & v);
        self.obp0_palette_01 = GrayShade::from_u8((0b00001100 & v) >> 2);
        self.obp0_palette_10 = GrayShade::from_u8((0b00110000 & v) >> 4);
        self.obp0_palette_11 = GrayShade::from_u8((0b11000000 & v) >> 6);
    }

    fn stat_read(&self) -> u8 {
        unimplemented!();
    }

    fn stat_write(&mut self, v: u8) {
        // TODO:
    }

    fn copy_memory_to_vram(&mut self, v: u8) {

    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            0xFF40 => self.lcd_controller.read(),
            0xFF41 => self.stat_read(),
            0xFF42 => self.scroll_bg_y,
            0xFF43 => self.scroll_bg_x,
            0xFF44 => 0x91,//self.lcd_y_coordinate,
            0xFF46 => panic!("Cannot read from $FF46"),
            0xFF47 => self.read_bgp(),
            0xFF48 => self.read_obp0(),
            0xFF49 => self.read_obp1(),
            0xFF4A => self.window_y,
            0xFF4B => self.window_x,
            _       => unimplemented!(),
        }
    }

    pub fn write(&mut self, address: u16, v: u8) {
        match address {
            0xFF40 => { self.lcd_controller.write(v) },
            0xFF41 => { self.stat_write(v) },
            0xFF42 => { self.scroll_bg_y = v },
            0xFF43 => { self.scroll_bg_x = v },
            0xFF44 => { self.lcd_y_coordinate = 0 },
            0xFF46 => { self.copy_memory_to_vram(v) },
            0xFF47 => { self.write_bgp(v) },
            0xFF48 => { self.write_obp0(v) },
            0xFF49 => { self.write_obp1(v) },
            0xFF4A => { self.window_y = v },
            0xFF4B => { self.window_x = v },
            _       => unimplemented!(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum TileMap {
    // Area $9800 - $9BFF
    C9800,
    // Area $9C00 - $9FFF
    C9C00,
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
    tile_map_selected: TileMap,
    window_display_on: bool,
    bg_tile_data: BgTileData,
    bg_tile_map: TileMap,
    obj_sprite_size: SpriteSize,
    obj_sprite_display: bool,
    bg_window_on: bool,
}

impl LCDController {
    pub fn new() -> LCDController {
        LCDController {
            lcd_on: true,
            tile_map_selected: TileMap::C9800,
            window_display_on: false,
            bg_tile_data: BgTileData::C8000,
            bg_tile_map: TileMap::C9800,
            obj_sprite_size: SpriteSize::C8by8,
            obj_sprite_display: false,
            bg_window_on: true,
        }
    }

    pub fn read(&self) -> u8 {
        (if self.bg_window_on                           { 0b00000001 } else { 0 }) +
        (if self.obj_sprite_display                     { 0b00000010 } else { 0 }) +
        (if self.obj_sprite_size == SpriteSize::C8by16  { 0b00000100 } else { 0 }) +
        (if self.bg_tile_map == TileMap::C9C00          { 0b00001000 } else { 0 }) +
        (if self.bg_tile_data == BgTileData::C8000      { 0b00010000 } else { 0 }) +
        (if self.window_display_on                      { 0b00100000 } else { 0 }) +
        (if self.tile_map_selected == TileMap::C9C00    { 0b01000000 } else { 0 }) +
        (if self.lcd_on                                 { 0b10000000 } else { 0 })
    }

    pub fn write(&mut self, v: u8) {
       self.bg_window_on      =    (v & 0b00000001) > 0;
       self.obj_sprite_display=    (v & 0b00000010) > 0;
       self.obj_sprite_size   = if (v & 0b00000100) > 0 { SpriteSize::C8by16 } else { SpriteSize::C8by8 };
       self.bg_tile_map       = if (v & 0b00001000) > 0 { TileMap::C9C00 } else { TileMap::C9800 };
       self.bg_tile_data      = if (v & 0b00010000) > 0 { BgTileData::C8000 } else { BgTileData::C8800 };
       self.window_display_on =    (v & 0b00100000) > 0;
       self.tile_map_selected = if (v & 0b01000000) > 0 { TileMap::C9C00 } else { TileMap::C9800 };
       self.lcd_on            =    (v & 0b10000000) > 0;
    }
}
