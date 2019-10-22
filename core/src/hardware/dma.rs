use hardware::cpu;
use hardware::cpu::MapperHolder;

pub struct DmaController {
    running: bool,
    base: u16,
    cycles: usize,
    pub oam_ram: [u8; 160],
}

impl cpu::Handler for DmaController {
    fn read(&self, address: u16) -> u8 {
        match address {
            0xFE00 ..= 0xFE9F => self.oam_ram[address as usize - 0xFE00],
            // TODO: not sure what should happen here, so let's just crash
            0xFF46            => panic!("Trying to read from DMA."),
            _ => unreachable!(),
        }
    }

    fn write(&mut self, address: u16, v: u8) {
        match address {
            0xFE00 ..= 0xFE9F => self.oam_ram[address as usize - 0xFE00] = v,
            0xFF46 => {
                if v >= 0xE0 {
                    // It's not really clear to me what happens when we try to
                    // DMA to a high address so let's just crash
                    unimplemented!();
                }

                self.base = (v as u16) << 8;
                // TODO: what if it's already running?
                self.running = true;
                self.cycles = 0;
            },
            _ => unreachable!(),
        }
    }
}

impl DmaController {
    pub fn new() -> DmaController {
        DmaController {
            running: false,
            base: 0,
            cycles: 0,
            oam_ram: [0; 160],
        }
    }

    pub fn cpu_step(&mut self, mapper_holder: &dyn MapperHolder) {
        if !self.running {
            return;
        }

        match self.cycles {
            0 => {
                // There's a 1 cycle wait after enabling DMA
            },
            1 ..= 160 => {
                let dma_step = self.cycles as u16 - 1;
                let from = self.base + dma_step;
                let v = mapper_holder.get_handler_read(from).read(from);
                self.oam_ram[dma_step as usize] = v;
            },
            161 => {
                // DMA is done
                self.running = false;
            },
            _ => unreachable!(),
        }

        self.cycles += 1;
    }
}
