use macroquad::prelude::*;

const SCREEN_WIDTH: usize = 256;
const SCREEN_HEIGHT: usize = 256;
const MEMORY_SIZE: usize = 16 * 1024 * 1024 + 8;

struct Lbp {
    mem: Vec<u8>,
    pc: usize,
}

impl Lbp {
    pub fn new() -> Self {
        Self {
            mem: vec![0u8; MEMORY_SIZE],
            pc: 0,
        }
    }

    pub fn cycle(&mut self) {
        self.pc = read_24bit(&self.mem, 2);

        for _ in 0..0x10000 {
            let a = read_24bit(&self.mem, self.pc);
            let b = read_24bit(&self.mem, self.pc + 3);

            self.mem[b] = self.mem[a];

            self.pc = read_24bit(&self.mem, self.pc + 6);
        }
    }

    pub fn load(&mut self, rom: Vec<u8>) {
        let rom_len = rom.len().min(self.mem.len());
        self.mem[0..rom_len].copy_from_slice(&rom[0..rom_len]);
    }
}

#[macroquad::main("lbp-rs")]
async fn main() {
    let mut lbp = Lbp::new();

    let args: Vec<String> = std::env::args().collect::<Vec<String>>();
    if let Some(rom_path) = args.get(1) {
        let rom = macroquad::file::load_file(rom_path).await.unwrap();
        lbp.load(rom);
    }

    macroquad::window::request_new_screen_size(SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32);
    let mut image = macroquad::texture::Image::gen_image_color(
        SCREEN_WIDTH as u16,
        SCREEN_HEIGHT as u16,
        macroquad::color::WHITE,
    );
    let texture = macroquad::texture::Texture2D::from_image(&image);

    loop {
        lbp.cycle();

        let bank = lbp.mem[5] as usize;
        for y in 0..SCREEN_HEIGHT {
            for x in 0..SCREEN_WIDTH {
                let offset = (bank << 16) | (y << 8) | x;
                let color_index = lbp.mem[offset];
                println!("color index: {}", color_index);
                let color_value = get_palette(color_index);

                image.set_pixel(x as u32, y as u32, color_value);
            }
        }

        texture.update(&image);
        macroquad::texture::draw_texture_ex(
            &texture,
            0.0,
            0.0,
            macroquad::color::WHITE,
            macroquad::texture::DrawTextureParams {
                dest_size: Some(macroquad::math::vec2(
                    macroquad::window::screen_width(),
                    macroquad::window::screen_height(),
                )),
                ..Default::default()
            },
        );
        macroquad::window::next_frame().await
    }
}

fn read_24bit(mem: &[u8], offset: usize) -> usize {
    let first_byte = mem[offset] as usize;
    let second_byte = mem[offset + 1] as usize;
    let third_byte = mem[offset + 2] as usize;

    (first_byte << 16) | (second_byte << 8) | third_byte
}

fn get_palette(color_index: u8) -> macroquad::color::Color {
    if color_index > 216 {
        return macroquad::color::WHITE;
    }

    let r = color_index / 36;
    let g = color_index / 6 % 6;
    let b = color_index % 6;

    macroquad::color_u8!(r, g, b, 255)
}
