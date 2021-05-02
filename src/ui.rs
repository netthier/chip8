use crate::roms::ROMS;
use crate::State;

use crate::cpu::Cpu;
use crate::disassembler::generate_disassembly;
use egui::menu::menu;

pub struct MenuState {
    selected: String,
    pub show_debugger: bool,
}

pub struct DebuggerState {
    pub running: bool,
    delay_counter: u32,
}

impl Default for MenuState {
    fn default() -> Self {
        Self {
            selected: "TETRIS".to_string(),
            show_debugger: false,
        }
    }
}

impl Default for DebuggerState {
    fn default() -> Self {
        Self {
            running: true,
            delay_counter: 0,
        }
    }
}

pub fn show_menu(state: &mut State, menu_state: &mut MenuState) {
    egui_macroquad::ui(|egui_ctx| {
        egui::Window::new("Menu")
            .default_width(500.0)
            .show(egui_ctx, |ui| {
                ui.label("Welcome to nett_hier's WASM CHIP-8 emulator");
                ui.separator();
                egui::ComboBox::from_label("Select a game!")
                    .width(128.0)
                    .selected_text(&menu_state.selected)
                    .show_ui(ui, |ui| {
                        for rom in ROMS.iter() {
                            ui.selectable_value(&mut menu_state.selected, rom.to_string(), *rom);
                        }
                    });
                ui.checkbox(&mut menu_state.show_debugger, "Enable Debugger");
                ui.separator();
                if ui.button("Start!").clicked() {
                    *state = State::InGame(menu_state.selected.clone());
                }
                ui.label("Once in game, press Esc to return to the menu.");
            });
    });
}

pub fn show_debugger(debugger_state: &mut DebuggerState, cpu: &mut Cpu) {
    egui_macroquad::ui(|egui_ctx| {
        egui::Window::new("Debugger")
            .default_width(500.0)
            .show(egui_ctx, |ui| {
                egui::CollapsingHeader::new("Disassembly")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.monospace(generate_disassembly(cpu, cpu.pc..cpu.pc + 20));
                    });
                ui.separator();
                egui::CollapsingHeader::new("Registers")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.monospace(get_registers(&cpu));
                    });
                ui.checkbox(&mut debugger_state.running, "Run CPU");
                ui.separator();
                if !debugger_state.running && ui.button("Step").clicked() {
                    cpu.step();
                    debugger_state.delay_counter += 1;
                    if debugger_state.delay_counter == 7 {
                        cpu.dec_regs();
                        debugger_state.delay_counter = 0;
                    }
                }
            });
    });
}

fn get_registers(cpu: &Cpu) -> String {
    let mut label = String::new();
    for (idx, reg) in cpu.regs.iter().enumerate() {
        let mut str = format!("V{:X}: 0x{:X} ", idx, reg);
        while str.len() < 13 {
            str.push(' ');
        }
        if (idx + 1) % 4 == 0 {
            str.push('\n');
        }
        label.push_str(&str);
    }
    label.push_str(format!("PC: 0x{:X}\n", cpu.pc).as_str());
    label.push_str(format!("I: 0x{:X}\n", cpu.reg_i).as_str());
    label.push_str(format!("DT: 0x{:X}\n", cpu.reg_delay).as_str());
    label.push_str(format!("ST: 0x{:X}\n", cpu.reg_sound).as_str());
    label
}
