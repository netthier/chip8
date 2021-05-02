use crate::cpu::Cpu;
use crate::ui::{show_menu, DebuggerState, MenuState};
use macroquad::prelude::*;

mod cpu;
mod disassembler;
mod roms;
mod ui;

#[derive(PartialEq)]
pub enum State {
    Menu,
    InGame(String),
}

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

    let mut state = State::Menu;
    let mut menu_state = MenuState::default();
    let mut debugger_state = DebuggerState::default();

    loop {
        if state == State::Menu {
            ui::show_menu(&mut state, &mut menu_state);
            egui_macroquad::draw();
            if let State::InGame(rom) = &state {
                cpu = Cpu::new();
                cpu.init_mem(&roms::get_bytes(rom));
            }
        } else {
            if is_key_pressed(KeyCode::Escape) {
                state = State::Menu;
            }

            if debugger_state.running {
                // At 60fps, this basically results in a CPU speed of 480Hz
                for _ in 0..8 {
                    cpu.step();
                }
                cpu.dec_regs();
            }

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

            if menu_state.show_debugger {
                ui::show_debugger(&mut debugger_state, &mut cpu);
                egui_macroquad::draw();
            }
        }
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
