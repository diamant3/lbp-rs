struct Lbp {
    mem: Vec<u8>,
    pc: usize,
}

impl Lbp {
    pub fn new(_ctx: &mut ggez::Context) -> Self {
        Self {
            mem: vec![0; 0x1000000],
            pc: 0,
        }

        // lbp.load(rom_path);

        // lbp
    }

    pub fn cycle(&mut self) {
        self.pc = (self.mem[2].wrapping_shl(16)) as usize
            | (self.mem[3].wrapping_shl(8)) as usize
            | self.mem[4] as usize;

        for _ in 0..0xffff {
            let a = (self.mem[self.pc].wrapping_shl(16)) as usize
                | (self.mem[self.pc + 1].wrapping_shl(8)) as usize
                | self.mem[self.pc + 2] as usize;
            let b = (self.mem[self.pc + 3].wrapping_shl(16)) as usize
                | (self.mem[self.pc + 4].wrapping_shl(8)) as usize
                | self.mem[self.pc + 5] as usize;
            let c = (self.mem[self.pc + 6].wrapping_shl(16)) as usize
                | (self.mem[self.pc + 7].wrapping_shl(8)) as usize
                | self.mem[self.pc + 8] as usize;

            self.mem[b] = self.mem[a];
            self.pc = c;
        }
    }

    pub fn load(&mut self, rom: Vec<u8>) {
        self.mem[0..rom.len()].copy_from_slice(&rom);
    }
}

impl ggez::event::EventHandler for Lbp {
    fn update(&mut self, _ctx: &mut ggez::Context) -> ggez::GameResult {
        self.cycle();

        Ok(())
    }

    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult {
        let canvas = ggez::graphics::Canvas::from_frame(ctx, ggez::graphics::Color::WHITE);
        canvas.finish(ctx)
    }
}

fn main() {
    let (mut ctx, el) = ggez::ContextBuilder::new("lbp-rs", "Diamond Rivero")
        .window_setup(ggez::conf::WindowSetup::default().title("lbp-rs"))
        .window_mode(
            ggez::conf::WindowMode::default()
                .dimensions(256.0, 256.0)
                .resizable(false),
        )
        .build()
        .unwrap();

    let mut lbp = Lbp::new(&mut ctx);

    let rom_path: Vec<String> = std::env::args().collect::<Vec<String>>();
    let rom = std::path::Path::new(rom_path.get(1).unwrap());
    lbp.load(ctx.fs.read(rom).unwrap());

    let _ = ggez::event::run(ctx, el, lbp);
}
