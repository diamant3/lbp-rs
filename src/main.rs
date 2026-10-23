struct Lbp {
    mem: Vec<u8>,
    pc: usize,
}

impl Lbp {
    pub fn new() -> Self {
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
        let mut canvas = ggez::graphics::Canvas::from_frame(ctx, ggez::graphics::Color::BLACK);

        let zz = self.mem[5] << 16;
        let mut colors = [0; 216];
        
        for yy in 0..255 {
            let _yy = yy << 8;

            for xx in 0..255 {
                let color = self.mem[(zz | _yy | xx) as usize];
                colors[xx as usize] = color;
            }

            let mut rect = ggez::graphics::Rect::new(0.0, 0.0, 0.0, 0.0);
            let mut mb = ggez::graphics::MeshBuilder::new();
            for xx in 0..255 {
                let mut color = colors[xx];
                if color <= 216 {
                    let mut b = color % 6 * 51;
                    color /= 6;

                    let mut g = color % 6 * 51;
                    color /= 6;

                    let mut r = color % 6 * 51;

                    r /= 255;
                    g /= 255;
                    b /= 255;

                    let _ = mb.rectangle(ggez::graphics::DrawMode::fill(), rect, ggez::graphics::Color::from_rgb(r, g, b));
                } else if color >= 217 && color < 255 {
                    let _ = mb.rectangle(ggez::graphics::DrawMode::fill(), rect, ggez::graphics::Color::BLACK);
                } else {
                    let _ = mb.rectangle(ggez::graphics::DrawMode::fill(), rect, ggez::graphics::Color::WHITE);
                }

                rect = ggez::graphics::Rect::new(xx as f32, yy as f32, 1.0, 1.0);

                let mesh = ggez::graphics::Mesh::from_data(ctx, mb.build());
                canvas.draw(&mesh, ggez::graphics::DrawParam::default());
            }
        }
        canvas.finish(ctx)
    }
}

fn main() {
    let (ctx, el) = ggez::ContextBuilder::new("lbp-rs", "Diamond Rivero")
        .window_setup(ggez::conf::WindowSetup::default().title("lbp-rs"))
        .window_mode(
            ggez::conf::WindowMode::default()
                .dimensions(256.0, 256.0)
                .resizable(false),
        )
        .build()
        .unwrap();

    let mut lbp = Lbp::new();

    let rom_path: Vec<String> = std::env::args().collect::<Vec<String>>();
    let rom = std::path::Path::new(rom_path.get(1).unwrap());
    lbp.load(ctx.fs.read(rom).unwrap());

    let _ = ggez::event::run(ctx, el, lbp);
}
