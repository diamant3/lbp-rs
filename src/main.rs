use macroquad::prelude::*;

const SCREEN_WIDTH: usize = 256;
const SCREEN_HEIGHT: usize = 256;
const MEMORY_SIZE: usize = 16 * 1024 * 1024 + 8;

struct Lbp {
    mem: Vec<u8>,
    pc: usize,
    audiobuffer: Vec<u8>,
}

impl Lbp {
    pub fn new() -> Self {
        Self {
            mem: vec![0u8; MEMORY_SIZE],
            pc: 0,
            audiobuffer: vec![0; 256],
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

    pub fn poll_keys(&mut self) {
        let keys = [
            macroquad::input::KeyCode::X,
            macroquad::input::KeyCode::Key1,
            macroquad::input::KeyCode::Key2,
            macroquad::input::KeyCode::Key3,
            macroquad::input::KeyCode::Q,
            macroquad::input::KeyCode::W,
            macroquad::input::KeyCode::E,
            macroquad::input::KeyCode::A,
            macroquad::input::KeyCode::S,
            macroquad::input::KeyCode::D,
            macroquad::input::KeyCode::Z,
            macroquad::input::KeyCode::C,
            macroquad::input::KeyCode::Key4,
            macroquad::input::KeyCode::R,
            macroquad::input::KeyCode::F,
            macroquad::input::KeyCode::V,
        ];

        let mut key_value: u16 = 0;
        for (key_id, key_code) in keys.iter().enumerate() {
            if macroquad::input::is_key_down(*key_code) {
                key_value |= 1 << key_id;
            }
        }

        self.mem[0] = key_value.to_be_bytes()[0];
        self.mem[1] = key_value.to_be_bytes()[1];
    }

    // thanks to Google AI mode :)
    pub fn process_audio(&mut self) -> Vec<u8> {
        let audio_address = (self.mem[6] as usize) << 8 | self.mem[7] as usize;

        for byte in 0..256 {
            let sample_byte = self.mem[audio_address + byte];
            self.audiobuffer.push(sample_byte);
        }

        let audio_data_len = self.audiobuffer.len() as u32;
        let file_size = 36 + audio_data_len;
        let byte_rate = 15360u32;
        let block_alignment = 1u16;
        let bits_per_sample = 8u16;

        let mut wav: Vec<u8> = std::vec::Vec::with_capacity(44 + self.audiobuffer.len());

        wav.extend_from_slice(b"RIFF");
        wav.extend_from_slice(&file_size.to_le_bytes());
        wav.extend_from_slice(b"WAVE");

        wav.extend_from_slice(b"fmt ");
        wav.extend_from_slice(&16u32.to_le_bytes());
        wav.extend_from_slice(&1u16.to_le_bytes());
        wav.extend_from_slice(&1u16.to_le_bytes());
        wav.extend_from_slice(&byte_rate.to_le_bytes());
        wav.extend_from_slice(&byte_rate.to_le_bytes());
        wav.extend_from_slice(&block_alignment.to_le_bytes());
        wav.extend_from_slice(&bits_per_sample.to_le_bytes());

        wav.extend_from_slice(b"data");
        wav.extend_from_slice(&audio_data_len.to_le_bytes());
        wav.extend_from_slice(&self.audiobuffer);

        wav
    }
}

fn window_config() -> macroquad::window::Conf {
    macroquad::window::Conf {
        window_title: "lbp-rs".to_owned(),
        window_width: SCREEN_WIDTH as i32,
        window_height: SCREEN_HEIGHT as i32,
        icon: None,
        ..Default::default()
    }
}

#[macroquad::main(window_config)]
async fn main() {
    let mut lbp = Lbp::new();

    let args: Vec<String> = std::env::args().collect::<Vec<String>>();
    if let Some(rom_path) = args.get(1) {
        let rom = macroquad::file::load_file(rom_path).await.unwrap();
        lbp.load(rom);
    }

    let mut image = macroquad::texture::Image::gen_image_color(
        SCREEN_WIDTH as u16,
        SCREEN_HEIGHT as u16,
        macroquad::color::BLACK,
    );
    let texture = macroquad::texture::Texture2D::from_image(&image);

    texture.set_filter(macroquad::texture::FilterMode::Nearest);
    loop {
        lbp.cycle();
        let audio_data = lbp.process_audio();
        if let Ok(sound) = macroquad::audio::load_sound_from_bytes(&audio_data).await {
            macroquad::audio::play_sound(
                &sound,
                macroquad::audio::PlaySoundParams {
                    looped: false,
                    volume: 100.0,
                },
            );
        }

        lbp.poll_keys();
        if macroquad::input::is_key_down(macroquad::input::KeyCode::Escape) {
            break;
        }

        let bank = lbp.mem[5] as usize;
        for y in 0..SCREEN_HEIGHT {
            for x in 0..SCREEN_WIDTH {
                let offset = (bank << 16) | (y << 8) | x;
                let color_index = lbp.mem[offset];
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
    if (216..255).contains(&color_index) {
        return macroquad::color::BLACK;
    }

    let r = color_index.saturating_div(36);
    let g = color_index.saturating_div(6).rem_euclid(6);
    let b = color_index.rem_euclid(6);

    let r = r.wrapping_mul(0x33);
    let b = b.wrapping_mul(0x33);
    let g = g.wrapping_mul(0x33);

    macroquad::color_u8!(r, g, b, 255)
}
