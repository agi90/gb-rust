use bitfield::Bitfield;
use hardware::cpu;

/* Represents a shade of gray */
u8_enum! {
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
    screen_buffer: ScreenBuffer,
    should_refresh: bool,
    mapper: VideoMemoryMapper,
    mode: LCDMode,
    visible_sprites: [usize; 10],
    visible_sprites_len: usize,
    background: [u8; SCREEN_X],
    pixel_fifo: PixelPipeline,
    x: usize,
}

u8_enum! {
    LCDMode {
        HBlank = 0,
        VBlank = 1,
        SearchingOAM = 2,
        LCDTransfer = 3,
    }
}

#[inline]
fn pattern_value(video_ram: &[u8], offset: usize, x: usize, y: usize) -> u8 {
    let l = video_ram[offset + (y * 2)];
    let h = video_ram[offset + (y * 2) + 1];

    match x {
        7 => (0b00000001 & l) + ((0b00000001 & h) << 1),
        6 => ((0b00000010 & l) >> 1) + (0b00000010 & h),
        5 => ((0b00000100 & l) >> 2) + ((0b00000100 & h) >> 1),
        4 => ((0b00001000 & l) >> 3) + ((0b00001000 & h) >> 2),
        3 => ((0b00010000 & l) >> 4) + ((0b00010000 & h) >> 3),
        2 => ((0b00100000 & l) >> 5) + ((0b00100000 & h) >> 4),
        1 => ((0b01000000 & l) >> 6) + ((0b01000000 & h) >> 5),
        0 => ((0b10000000 & l) >> 7) + ((0b10000000 & h) >> 6),
        _ => unreachable!(),
    }
}

impl cpu::Handler for Ppu {
    fn read(&self, address: u16) -> u8 {
        match address {
            0x8000..=0x9FFF => self.read_ram(address),
            _ => self.mapper.read(address),
        }
    }

    fn write(&mut self, address: u16, v: u8) {
        match address {
            0x8000..=0x9FFF => self.write_ram(address, v),
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
            screen_buffer: [[GrayShade::C00; SCREEN_X]; SCREEN_Y],
            should_refresh: false,
            mapper: VideoMemoryMapper::new(),
            mode: LCDMode::HBlank,
            visible_sprites: [0; 10],
            visible_sprites_len: 0,
            background: [0; SCREEN_X],
            pixel_fifo: PixelPipeline::new(),
            x: 0,
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
            }
            _ => {}
        }
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

    pub fn get_screen(&self) -> &ScreenBuffer {
        &self.screen_buffer
    }

    pub fn should_refresh(&mut self) -> bool {
        let result = self.should_refresh;
        self.should_refresh = false;
        result
    }

    /// Checks if the current x is the start of the window section
    fn check_window_x(&mut self) -> bool {
        if self.mapper.window_on() == 1
            && self.mapper.bg_window_on() == 1
            && self.x + 7 == self.mapper.window_x as usize
            && self.scanline() >= self.mapper.window_y
        {
            let y = self.scanline() as usize - self.mapper.window_y as usize;
            self.pixel_fifo
                .reset(0, BackgroundFetcher::new(self.window_offset(), 0, y));
            return true;
        }

        false
    }

    fn scanline_offset(&self, scroll_y: i16) -> usize {
        let offset = ((self.scanline() as i16 + scroll_y) / 8) as usize % (BACKGROUND_Y / 8);
        offset * 32
    }

    fn window_offset(&self) -> usize {
        let offset = if self.mapper.window_tile_map() == TileMap::C9800 {
            0x1800
        } else {
            0x1C00
        };

        offset + self.scanline_offset(-(self.mapper.window_y as i16))
    }

    fn background_offset(&self) -> usize {
        let offset = if self.mapper.bg_tile_map() == TileMap::C9800 {
            0x1800
        } else {
            0x1C00
        };

        offset + self.scanline_offset(self.mapper.scroll_bg_y as i16)
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
            LCDMode::HBlank => self.mapper.h_blank_interrupt() == 1,
            LCDMode::VBlank => self.mapper.v_blank_interrupt() == 1,
            LCDMode::LCDTransfer => false,
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
        assert!(
            self.scanline() == 153 || self.scanline() + 1 == (self.cycles / SCANLINE_CYCLES) as u8
        );

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
            }
            8..=452 => {
                // VBlank running, nothing to do
            }
            _ => unreachable!(),
        }

        None
    }

    pub fn check_interrupts(&mut self, oam_ram: &[u8]) -> Option<cpu::Interrupt> {
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
            }
            8..=76 => {
                // The ppu is running OAM Search, do nothing
            }
            80 => {
                let scanline = self.scanline() as usize;
                let sprite_module = SpriteModule {
                    oam_ram,
                    video_ram: &self.video_ram,
                    mapper: &self.mapper,
                    background: &self.background,
                    screen_buffer: &mut self.screen_buffer,
                };

                let (sprites, len) = sprite_module.visible_sprites(scanline);
                self.visible_sprites = sprites;
                self.visible_sprites_len = len;
            }
            84 => {
                interrupt = interrupt.or(self.switch_to(LCDMode::LCDTransfer));
                self.x = 0;
                if self.mapper.bg_window_on() == 1 {
                    let y = (self.scanline() + self.mapper.scroll_bg_y) as usize;
                    self.pixel_fifo.reset(
                        (self.mapper.scroll_bg_x % 8) as usize,
                        BackgroundFetcher::new(
                            self.background_offset(),
                            self.mapper.scroll_bg_x as usize / 8,
                            y,
                        ),
                    );
                    // FIXME: every CPU cycle is 2 PPU cycles
                    for _ in 0..2 {
                        self.pixel_fifo.step(&self.mapper, &self.video_ram, oam_ram);
                    }
                }
                self.check_window_x();
            }
            88..=452 => {
                if self.mode == LCDMode::HBlank {
                    // nothing to do
                    return None;
                }

                if self.x >= 160 {
                    interrupt = interrupt.or(self.switch_to(LCDMode::HBlank));
                    let scanline = self.scanline() as usize;
                    self.print_sprites(oam_ram, scanline);
                }

                // FIXME: every CPU cycle is 2 PPU cycles
                for _ in 0..2 {
                    self.fetcher_step(oam_ram);
                }
            }
            _ => unreachable!(),
        }

        interrupt
    }

    fn fetcher_step(&mut self, oam_ram: &[u8]) {
        if self.mapper.bg_window_on() != 1 {
            self.x += 2;
            return;
        }

        self.pixel_fifo.step(&self.mapper, &self.video_ram, oam_ram);

        if !self.pixel_fifo.has_pixels() {
            return;
        }

        for _ in 0..2 {
            let raw = self.pixel_fifo.pop();
            if self.x < SCREEN_X {
              self.background[self.x] = raw;
            }
            let color = self.background_color_from_raw(raw);
            self.write_raw_pixel(self.x, self.scanline() as usize, color);
            self.x += 1;
            if self.check_window_x() {
                return;
            }
        }
    }

    fn print_sprites(&mut self, oam_ram: &[u8], scanline: usize) {
        let mut sprite_module = SpriteModule {
            oam_ram,
            video_ram: &self.video_ram,
            mapper: &self.mapper,
            background: &self.background,
            screen_buffer: &mut self.screen_buffer,
        };

        sprite_module.print_sprites(scanline, self.visible_sprites, self.visible_sprites_len);
    }
}

struct SpriteModule<'a> {
    oam_ram: &'a [u8],
    video_ram: &'a [u8],
    background: &'a [u8],
    mapper: &'a VideoMemoryMapper,
    screen_buffer: &'a mut ScreenBuffer,
}

impl<'a> SpriteModule<'a> {
    fn visible_sprites(&self, scanline: usize) -> ([usize; 10], usize) {
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

        &visible_sprites[0..visible_sprites_len].sort_by_key(|&id| self.sprite_x(id));

        (visible_sprites, visible_sprites_len)
    }

    fn print_sprites(
        &mut self,
        scanline: usize,
        visible_sprites: [usize; 10],
        visible_sprites_len: usize,
    ) {
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
                    scanline + 16 - self.sprite_y(id),
                );

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
            below_bg: v & (0b10000000) > 0,
            y_flip: v & (0b01000000) > 0,
            x_flip: v & (0b00100000) > 0,
            palette: if v & (0b00010000) > 0 {
                SpritePalette::C1
            } else {
                SpritePalette::C0
            },
        }
    }

    fn is_sprite_visible(&self, id: usize, scanline: usize) -> bool {
        let y = self.sprite_y(id);
        match self.mapper.sprite_size() {
            SpriteSize::C8by8 => scanline + 16 >= y && scanline + 8 < y,
            SpriteSize::C8by16 => scanline + 16 >= y && scanline < y,
        }
    }

    fn is_sprite_horizontally_visible(&self, id: usize, h: usize) -> bool {
        let x = self.sprite_x(id);
        h + 8 >= x && h < x
    }

    fn sprite_value_at(&self, id: usize, x: usize, y: usize) -> GrayShade {
        let flags = self.sprite_flags(id);

        let x = if flags.x_flip { 7 - x } else { x };

        let y = if !flags.y_flip {
            y
        } else {
            match self.mapper.sprite_size() {
                SpriteSize::C8by8 => 7 - y,
                SpriteSize::C8by16 => 15 - y,
            }
        };

        let tile_index = match self.mapper.sprite_size() {
            SpriteSize::C8by8 => self.sprite_tile_index(id),
            SpriteSize::C8by16 => self.sprite_tile_index(id) & 0xFE,
        };

        let color = pattern_value(&self.video_ram, tile_index * 16, x, y);

        match &flags.palette {
            &SpritePalette::C0 => match color {
                0b00 => GrayShade::Transparent,
                0b01 => self.mapper.obp0_palette_01(),
                0b10 => self.mapper.obp0_palette_10(),
                0b11 => self.mapper.obp0_palette_11(),
                _ => unreachable!(),
            },
            &SpritePalette::C1 => match color {
                0b00 => GrayShade::Transparent,
                0b01 => self.mapper.obp1_palette_01(),
                0b10 => self.mapper.obp1_palette_10(),
                0b11 => self.mapper.obp1_palette_11(),
                _ => unreachable!(),
            },
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum PipelineStage {
    ReadPattern,
    ReadTile0,
    ReadTile1,
    Wait,
}

const FIFO_SIZE: usize = 16;
// Fifo queue for the Pixel Pipeline
struct Fifo {
    queue: [u8; FIFO_SIZE],
    start: usize,
    end: usize,
    size: usize,
}

impl Fifo {
    fn new() -> Fifo {
        Fifo {
            queue: [0; FIFO_SIZE],
            start: 0,
            end: 0,
            size: 0,
        }
    }

    fn reset(&mut self) {
        self.start = 0;
        self.end = 0;
        self.size = 0;
    }

    fn push(&mut self, v: u8) {
        assert!(self.size < FIFO_SIZE);

        self.queue[self.end] = v;
        self.end = (self.end + 1) % FIFO_SIZE;
        self.size += 1;
    }

    fn pop(&mut self) -> u8 {
        assert!(self.size > 0);

        let v = self.queue[self.start];
        self.start = (self.start + 1) % FIFO_SIZE;
        self.size -= 1;
        v
    }

    fn size(&self) -> usize {
        self.size
    }
}

struct PipelineUnit {
    pattern: u8,
    tile0: u8,
    tile1: u8,
}

trait Fetcher {
    fn tile(
        &self,
        mapper: &VideoMemoryMapper,
        pattern: u8,
        video_ram: &[u8],
        oam_ram: &[u8],
        tile: usize,
    ) -> u8;
    fn pattern(&self, video_ram: &[u8], oam_ram: &[u8]) -> u8;
    fn next_step(&mut self);
}

struct NullFetcher;

impl Fetcher for NullFetcher {
    fn tile(&self, _: &VideoMemoryMapper, _: u8, _: &[u8], _: &[u8], _: usize) -> u8 {
        0
    }
    fn pattern(&self, _: &[u8], _: &[u8]) -> u8 {
        0
    }
    fn next_step(&mut self) {}
}

struct BackgroundFetcher {
    address: usize,
    step: usize,
    y: usize,
}

impl BackgroundFetcher {
    fn new(address: usize, step: usize, y: usize) -> BackgroundFetcher {
        BackgroundFetcher { address, step, y }
    }
}

impl Fetcher for BackgroundFetcher {
    fn tile(
        &self,
        mapper: &VideoMemoryMapper,
        pattern: u8,
        video_ram: &[u8],
        _oam_ram: &[u8],
        tile: usize,
    ) -> u8 {
        let resolved_pattern;
        let offset;

        if mapper.bg_tile_data() == BgTileData::C8000 {
            resolved_pattern = pattern as usize;
            offset = 0x0000;
        } else {
            resolved_pattern = if pattern < 0x80 {
                pattern as usize + 0x80
            } else {
                pattern as usize - 0x80
            };
            offset = 0x0800;
        };

        video_ram[offset + resolved_pattern * 16 + (self.y % 8) * 2 + tile]
    }
    fn pattern(&self, video_ram: &[u8], _oam_ram: &[u8]) -> u8 {
        video_ram[self.address + self.step]
    }
    fn next_step(&mut self) {
        self.step = (self.step + 1) % (BACKGROUND_X / 8);
    }
}

struct PixelPipeline {
    fifo: Fifo,
    stage: PipelineStage,
    drop: usize,
    current: PipelineUnit,
    fetcher: Box<dyn Fetcher>,
}

impl PixelPipeline {
    fn new() -> PixelPipeline {
        PixelPipeline {
            fetcher: Box::new(NullFetcher {}),
            fifo: Fifo::new(),
            stage: PipelineStage::ReadPattern,
            drop: 0,
            current: PipelineUnit {
                pattern: 0,
                tile0: 0,
                tile1: 0,
            },
        }
    }

    fn reset(&mut self, drop: usize, fetcher: impl Fetcher + 'static) {
        self.drop = drop;
        self.fifo.reset();
        self.stage = PipelineStage::ReadPattern;
        self.fetcher = Box::new(fetcher);
    }

    fn push_pixels(&mut self) {
        let l = self.current.tile0;
        let h = self.current.tile1;

        for i in 0..7 {
            self.fifo
                .push(((l >> (7 - i)) & 0b1) + ((h >> (6 - i)) & 0b10));
        }

        // rust doesn't like |h >> -1| so we do it manually here
        self.fifo.push((l & 0b1) + ((h << 1) & 0b10));

        while self.drop > 0 {
            self.fifo.pop();
            self.drop -= 1;
        }
    }

    fn step(&mut self, mapper: &VideoMemoryMapper, video_ram: &[u8], oam_ram: &[u8]) {
        match self.stage {
            PipelineStage::ReadPattern => {
                self.current.pattern = self.fetcher.pattern(video_ram, oam_ram);
                self.stage = PipelineStage::ReadTile0;
            }
            PipelineStage::ReadTile0 => {
                self.current.tile0 =
                    self.fetcher
                        .tile(mapper, self.current.pattern, video_ram, oam_ram, 0);
                self.stage = PipelineStage::ReadTile1;
            }
            PipelineStage::ReadTile1 => {
                self.current.tile1 =
                    self.fetcher
                        .tile(mapper, self.current.pattern, video_ram, oam_ram, 1);
                self.push_pixels();

                self.fetcher.next_step();

                if self.fifo.size() <= 8 {
                    self.stage = PipelineStage::ReadPattern;
                } else {
                    // Queue is full, so we wait a cycle
                    self.stage = PipelineStage::Wait;
                }
            }
            PipelineStage::Wait => {
                self.stage = PipelineStage::ReadPattern;
            }
        }
    }

    fn pop(&mut self) -> u8 {
        self.fifo.pop()
    }

    fn has_pixels(&self) -> bool {
        self.fifo.size() > 8
    }
}

u8_enum! {
    SpritePalette {
        C0 = 0b0,
        C1 = 0b1,
    }
}

u8_enum! {
    TileMap {
        // Area $9800 - $9BFF
        C9800 = 0b0,
        // Area $9C00 - $9FFF
        C9C00 = 0b1,
    }
}

u8_enum! {
    BgTileData {
        // Area $8800 - 97FF
        C8800 = 0b0,
        // Area $8000 - 8FFF
        C8000 = 0b1,
    }
}

u8_enum! {
    SpriteSize {
        // 8 by 8 sprite
        C8by8 = 0b0,
        // 8 by 16 sprite
        C8by16 = 0b1,
    }
}

memory_mapper! {
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn fifo() {
        let fifo_size = FIFO_SIZE as u8;

        let mut fifo = Fifo::new();
        for i in 0..fifo_size {
            fifo.push(i);
        }

        assert_eq!(fifo.size(), FIFO_SIZE);
        assert_eq!(fifo.pop(), 0);

        assert_eq!(fifo.size(), FIFO_SIZE - 1);
        assert_eq!(fifo.pop(), 1);

        assert_eq!(fifo.size(), FIFO_SIZE - 2);
        assert_eq!(fifo.pop(), 2);

        assert_eq!(fifo.size(), FIFO_SIZE - 3);

        fifo.push(99);
        fifo.push(100);
        fifo.push(101);
        assert_eq!(fifo.size(), FIFO_SIZE);

        for _ in 3..FIFO_SIZE {
            fifo.pop();
        }

        assert_eq!(fifo.pop(), 99);
        assert_eq!(fifo.size(), 2);

        assert_eq!(fifo.pop(), 100);
        assert_eq!(fifo.size(), 1);

        assert_eq!(fifo.pop(), 101);
        assert_eq!(fifo.size(), 0);
    }
}
