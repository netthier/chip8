use crate::roms::ROMS;
use crate::State;

use egui::menu::menu;

pub struct MenuState {
    selected: String,
}

impl Default for MenuState {
    fn default() -> Self {
        Self {
            selected: "TETRIS".to_string(),
        }
    }
}

pub fn show_menu(state: &mut State, menu_state: &mut MenuState) {
    egui_macroquad::ui(|egui_ctx| {
        egui::Window::new("Menu")
            .default_width(500.0)
            .show(egui_ctx, |ui| {
                ui.label("Welcome to nett_hier's WASM CHIP-8 emulator");
                egui::ComboBox::from_label("Select a game!")
                    .width(128.0)
                    .selected_text(&menu_state.selected)
                    .show_ui(ui, |ui| {
                        for rom in ROMS.iter() {
                            ui.selectable_value(&mut menu_state.selected, rom.to_string(), *rom);
                        }
                    });
                if ui.button("Start!").clicked() {
                    *state = State::InGame(menu_state.selected.clone());
                }
                ui.label("Once in game, press Esc to return to the menu.");
            });
    });
}
