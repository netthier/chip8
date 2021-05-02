use crate::cpu::Cpu;
use crate::ui::show_menu;
use macroquad::prelude::*;

mod cpu;
mod ui;

#[macroquad::main("CHIP-8 EMU")]
async fn main() {
    let material = load_material(
        include_str!("CRT_shader.vert"),
        include_str!("CRT_shader.frag"),
        MaterialParams::default(),
    )
    .unwrap();

    let mut buffer = Image {
        width: 64,
        height: 32,
        bytes: vec![0; 4 * 64 * 32],
    };

    let texture = Texture2D::from_image(&buffer);
    texture.set_filter(FilterMode::Nearest);

    let target = render_target(68, 36);
    target.texture.set_filter(FilterMode::Nearest);

    let mut cpu = Cpu::new();

    cpu.init_mem(include_bytes!("../roms/BRIX"));

    loop {
        for _ in 0..8 {
            cpu.step();
        }

        cpu.dec_regs();
        process_input(&mut cpu);

        fb_to_img(&mut buffer, &cpu.get_framebuffer());
        texture.update(&buffer);

        set_camera(&Camera2D {
            render_target: Some(target),
            ..Camera2D::from_display_rect(Rect::new(0.0, 0.0, 68.0, 36.0))
        });

        draw_texture(texture, 2.0, 2.0, WHITE);

        set_default_camera();

        gl_use_material(material);

        draw_texture_ex(
            target.texture,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(get_dims().0, get_dims().1)),
                flip_y: true,
                ..Default::default()
            },
        );

        gl_use_default_material();

        next_frame().await
    }
}

fn get_dims() -> (f32, f32) {
    if screen_width() / 2.0 > screen_height() {
        (screen_height() * 2.0, screen_height())
    } else {
        (screen_width(), screen_width() / 2.0)
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
                    Color::from_rgba(0, 0, 0, 64)
                },
            )
        }
    }
}
