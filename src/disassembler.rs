use crate::cpu::{ArgType, Cpu};
use std::ops::Range;

pub fn generate_disassembly(cpu: &mut Cpu, range: Range<usize>) -> String {
    let mut disassembly = String::new();
    for pc in range.collect::<Vec<usize>>().chunks(2) {
        let pc = pc[0];
        let nibbles = cpu.get_instr_addr(pc);
        let nnn = cpu.get_args_addr(ArgType::Nnn, pc);
        let xkk = cpu.get_args_addr(ArgType::Xkk, pc);
        let xyn = cpu.get_args_addr(ArgType::Xyn, pc);
        disassembly.push_str(format!("0x{:X}: ", pc).as_str());
        disassembly.push_str(&match nibbles {
            [0x0, 0x0, 0xE, 0x0] => "cls".to_string(),
            [0x0, 0x0, 0xE, 0xE] => "ret".to_string(),
            [0x0, 0x0, 0xF, 0xA] => "compat".to_string(),
            [0x1, _, _, _] => format!("JP 0x{:X}", nnn[0]),
            [0x2, _, _, _] => format!("CALL 0x{:X}", nnn[0]),
            [0x3, _, _, _] => format!("SE V{:X}, {:X}", xkk[0], xkk[1]),
            [0x4, _, _, _] => format!("SNE V{:X}, 0x{:X}", xkk[0], xkk[1]),
            [0x5, _, _, 0x0] => format!("SE V{:X}, V{:X}", xyn[0], xyn[1]),
            [0x6, _, _, _] => format!("LD V{:X}, 0x{:X}", xkk[0], xkk[1]),
            [0x7, _, _, _] => format!("ADD V{:X}, 0x{:X}", xkk[0], xkk[1]),
            [0x8, _, _, 0x0] => format!("LD V{:X}, V{:X}", xyn[0], xyn[1]),
            [0x8, _, _, 0x1] => format!("OR V{:X}, V{:X}", xyn[0], xyn[1]),
            [0x8, _, _, 0x2] => format!("AND V{:X}, V{:X}", xyn[0], xyn[1]),
            [0x8, _, _, 0x3] => format!("XOR V{:X}, V{:X}", xyn[0], xyn[1]),
            [0x8, _, _, 0x4] => format!("ADD V{:X}, V{:X}", xyn[0], xyn[1]),
            [0x8, _, _, 0x5] => format!("SUB V{:X}, V{:X}", xyn[0], xyn[1]),
            [0x8, _, _, 0x6] => format!("SHR V{:X}", xyn[0]),
            [0x8, _, _, 0x7] => format!("SUBN V{:X}, V{:X}", xyn[0], xyn[1]),
            [0x8, _, _, 0xE] => format!("SHL V{:X}", xyn[0]),
            [0x9, _, _, 0x0] => format!("SNE V{:X}, V{:X}", xyn[0], xyn[1]),
            [0xA, _, _, _] => format!("LD I, 0x{:X}", nnn[0]),
            [0xB, _, _, _] => format!("JP 0x{:X} + V0", nnn[0]),
            [0xC, _, _, _] => format!("RND V{:X}, 0x{:X}", xkk[0], xkk[1]),
            [0xD, _, _, _] => format!("DRW V{:X}, V{:X}, 0x{:X}", xyn[0], xyn[1], xyn[2]),
            [0xE, _, 0x9, 0xE] => format!("SKP V{:X}", xyn[0]),
            [0xE, _, 0xA, 0x1] => format!("SKNP V{:X}", xyn[0]),
            [0xF, _, 0x0, 0x7] => format!("LD V{:X}, DT", xyn[0]),
            [0xF, _, 0x0, 0xA] => format!("LD V{:X}, K", xyn[0]),
            [0xF, _, 0x1, 0x5] => format!("LD DT, V{:X}", xyn[0]),
            [0xF, _, 0x1, 0x8] => format!("LD ST, V{:X}", xyn[0]),
            [0xF, _, 0x1, 0xE] => format!("ADD I, V{:X}", xyn[0]),
            [0xF, _, 0x2, 0x9] => format!("LD F, V{:X}", xyn[0]),
            [0xF, _, 0x3, 0x3] => format!("LD B, V{:X}", xyn[0]),
            [0xF, _, 0x5, 0x5] => format!("LD [I], V{:X}", xyn[0]),
            [0xF, _, 0x6, 0x5] => format!("LD V{:X}, [I]", xyn[0]),
            _ => format!(
                "0x{:X}{:X}{:X}{:X}",
                nibbles[0], nibbles[1], nibbles[2], nibbles[3]
            ),
        });
        disassembly.push('\n');
    }

    disassembly
}
