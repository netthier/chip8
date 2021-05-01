use crate::cpu::Cpu;
use macroquad::prelude::*;

mod cpu;

#[macroquad::main("CHIP-8 EMU")]
async fn main() {
    let mut buffer = Image {
        width: 64,
        height: 32,
        bytes: vec![0; 4 * 64 * 32],
    };

    let texture = Texture2D::from_image(&buffer);
    texture.set_filter(FilterMode::Nearest);

    let mut cpu = Cpu::new();

    cpu.init_mem(include_bytes!("../roms/TETRIS"));

    loop {
        for _ in 0..8 {
            cpu.step();
        }

        cpu.dec_regs();
        process_input(&mut cpu);

        fb_to_img(&mut buffer, &cpu.get_framebuffer());
        texture.update(&buffer);

        draw_texture_ex(
            texture,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(screen_width(), screen_width() / 2.0)),
                ..Default::default()
            },
        );

        next_frame().await
    }
}

fn process_input(cpu: &mut Cpu) {
    let key_codes = [
        KeyCode::X,
        KeyCode::Key1,
        KeyCode::Key2,
        KeyCode::Key3,
        KeyCode::Q,
        KeyCode::W,
        KeyCode::E,
        KeyCode::A,
        KeyCode::S,
        KeyCode::D,
        KeyCode::Z,
        KeyCode::C,
        KeyCode::Key4,
        KeyCode::R,
        KeyCode::F,
        KeyCode::V,
    ];

    for (idx, code) in key_codes.iter().enumerate() {
        cpu.set_key(idx, is_key_down(*code));

        // Special case for QWERTZ keyboards
        if *code == KeyCode::Z && is_key_down(KeyCode::Y) {
            cpu.set_key(idx, true);
        }
    }
}

fn fb_to_img(img: &mut Image, fb: &[bool; 32 * 64]) {
    for y in 0..32 {
        for x in 0..64 {
            img.set_pixel(
                x,
                y,
                if fb[y as usize * 64 + x as usize] {
                    WHITE
                } else {
                    Color::from_rgba(0, 0, 0, 0)
                },
            )
        }
    }
}
