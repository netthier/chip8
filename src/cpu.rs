use macroquad::rand::gen_range;

pub struct Cpu {
    pub mem: [u8; 0x1000],
    pub stack: Vec<usize>,

    pub regs: [u8; 0x10],
    pub reg_i: usize,
    pub reg_delay: u8,
    pub reg_sound: u8,
    pub pc: usize,

    pub framebuffer: [bool; 32 * 64],

    pub keymap: [bool; 0x10],
    block_release: bool,

    pub st_compat: bool,
    pub sh_compat: bool,
}

pub enum ArgType {
    Nnn,
    Xkk,
    Xyn,
}

enum PcMode {
    Step,
    Skip,
    Jump(usize),
}

const DIGITS: [u8; 0x50] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, 0x20, 0x60, 0x20, 0x20, 0x70, 0xF0, 0x10, 0xF0, 0x80, 0xF0, 0xF0,
    0x10, 0xF0, 0x10, 0xF0, 0x90, 0x90, 0xF0, 0x10, 0x10, 0xF0, 0x80, 0xF0, 0x10, 0xF0, 0xF0, 0x80,
    0xF0, 0x90, 0xF0, 0xF0, 0x10, 0x20, 0x40, 0x40, 0xF0, 0x90, 0xF0, 0x90, 0xF0, 0xF0, 0x90, 0xF0,
    0x10, 0xF0, 0xF0, 0x90, 0xF0, 0x90, 0x90, 0xE0, 0x90, 0xE0, 0x90, 0xE0, 0xF0, 0x80, 0x80, 0x80,
    0xF0, 0xE0, 0x90, 0x90, 0x90, 0xE0, 0xF0, 0x80, 0xF0, 0x80, 0xF0, 0xF0, 0x80, 0xF0, 0x80, 0x80,
];

impl Cpu {
    pub fn new() -> Self {
        Self {
            mem: [0; 0x1000],
            stack: Vec::new(),
            regs: [0; 0x10],
            reg_i: 0,
            reg_delay: 0,
            reg_sound: 0,
            pc: 0x200,

            framebuffer: [false; 64 * 32],

            keymap: [false; 0x10],
            block_release: false,

            st_compat: false,
            sh_compat: false,
        }
    }

    pub fn init_mem(&mut self, rom: &[u8]) {
        for (idx, byte) in DIGITS.iter().enumerate() {
            self.mem[idx] = *byte;
        }

        for (idx, byte) in rom.iter().enumerate() {
            self.mem[0x200 + idx] = *byte;
        }
    }

    pub fn get_framebuffer(&self) -> &[bool; 32 * 64] {
        &self.framebuffer
    }

    pub fn dec_regs(&mut self) {
        self.reg_delay = self.reg_delay.saturating_sub(1);
        self.reg_sound -= self.reg_sound.saturating_sub(1);
    }

    pub fn set_key(&mut self, key: usize, value: bool) {
        self.keymap[key] = value;
    }

    pub fn step(&mut self) {
        let nibbles = self.get_instr_addr(self.pc);

        match nibbles {
            [0x0, 0x0, 0xE, 0x0] => self.clear(),
            [0x0, 0x0, 0xE, 0xE] => self.ret(),
            [0x0, 0x0, 0xF, 0xA] => self.set_st_compat(),
            [0x1, _, _, _] => self.jump_nnn(),
            [0x2, _, _, _] => self.call_nnn(),
            [0x3, _, _, _] => self.skip_eq_xkk(),
            [0x4, _, _, _] => self.skip_neq_xkk(),
            [0x5, _, _, 0x0] => self.skip_eq_xy(),
            [0x6, _, _, _] => self.load_x_kk(),
            [0x7, _, _, _] => self.add_xkk(),
            [0x8, _, _, 0x0] => self.load_xy(),
            [0x8, _, _, 0x1] => self.or_xy(),
            [0x8, _, _, 0x2] => self.and_xy(),
            [0x8, _, _, 0x3] => self.xor_xy(),
            [0x8, _, _, 0x4] => self.add_xy(),
            [0x8, _, _, 0x5] => self.sub_xy(),
            [0x8, _, _, 0x6] => self.shr_xy(),
            [0x8, _, _, 0x7] => self.subn_xy(),
            [0x8, _, _, 0xE] => self.shl_xy(),
            [0x9, _, _, 0x0] => self.skip_neq_xy(),
            [0xA, _, _, _] => self.load_i_nnn(),
            [0xB, _, _, _] => self.jump_nnn_offset(),
            [0xC, _, _, _] => self.rand_x_kk(),
            [0xD, _, _, _] => self.draw_xyn(),
            [0xE, _, 0x9, 0xE] => self.skip_key_x(),
            [0xE, _, 0xA, 0x1] => self.skip_nkey_x(),
            [0xF, _, 0x0, 0x7] => self.load_x_dt(),
            [0xF, _, 0x0, 0xA] => self.block_key_x(),
            [0xF, _, 0x1, 0x5] => self.load_dt_x(),
            [0xF, _, 0x1, 0x8] => self.load_st_x(),
            [0xF, _, 0x1, 0xE] => self.add_i_x(),
            [0xF, _, 0x2, 0x9] => self.load_i_digit_x(),
            [0xF, _, 0x3, 0x3] => self.store_bcd_x(),
            [0xF, _, 0x5, 0x5] => self.store_vx(),
            [0xF, _, 0x6, 0x5] => self.restore_vx(),
            _ => {
                if nibbles[0] != 0x0 {
                    self.unimpl_panic();
                }
            }
        }
    }

    fn clear(&mut self) {
        self.framebuffer = [false; 64 * 32];
        self.set_pc(PcMode::Step);
    }

    fn ret(&mut self) {
        let addr = self.stack.pop().unwrap();
        self.set_pc(PcMode::Jump(addr));
        self.set_pc(PcMode::Step);
    }

    fn set_st_compat(&mut self) {
        self.st_compat = true;
        self.set_pc(PcMode::Step);
    }

    fn jump_nnn(&mut self) {
        let args = self.get_args(ArgType::Nnn);
        self.set_pc(PcMode::Jump(args[0]));
    }

    fn call_nnn(&mut self) {
        let args = self.get_args(ArgType::Nnn);
        self.stack.push(self.pc);
        self.set_pc(PcMode::Jump(args[0]));
    }

    fn skip_eq_xkk(&mut self) {
        let args = self.get_args(ArgType::Xkk);

        if self.regs[args[0]] == args[1] as u8 {
            self.set_pc(PcMode::Skip);
        } else {
            self.set_pc(PcMode::Step);
        }
    }

    fn skip_neq_xkk(&mut self) {
        let args = self.get_args(ArgType::Xkk);
        if self.regs[args[0]] != args[1] as u8 {
            self.set_pc(PcMode::Skip);
        } else {
            self.set_pc(PcMode::Step);
        }
    }

    fn skip_eq_xy(&mut self) {
        let args = self.get_args(ArgType::Xyn);
        if self.regs[args[0]] == self.regs[args[1]] {
            self.set_pc(PcMode::Skip);
        } else {
            self.set_pc(PcMode::Step);
        }
    }

    fn load_x_kk(&mut self) {
        let args = self.get_args(ArgType::Xkk);
        self.regs[args[0]] = args[1] as u8;
        self.set_pc(PcMode::Step);
    }

    fn add_xkk(&mut self) {
        let args = self.get_args(ArgType::Xkk);
        self.regs[args[0]] = self.regs[args[0]].wrapping_add(args[1] as u8);
        self.set_pc(PcMode::Step);
    }

    fn load_xy(&mut self) {
        let args = self.get_args(ArgType::Xyn);
        self.regs[args[0]] = self.regs[args[1]];
        self.set_pc(PcMode::Step);
    }

    fn or_xy(&mut self) {
        let args = self.get_args(ArgType::Xyn);
        self.regs[args[0]] |= self.regs[args[1]];
        self.set_pc(PcMode::Step);
    }

    fn and_xy(&mut self) {
        let args = self.get_args(ArgType::Xyn);
        self.regs[args[0]] &= self.regs[args[1]];
        self.set_pc(PcMode::Step);
    }

    fn xor_xy(&mut self) {
        let args = self.get_args(ArgType::Xyn);
        self.regs[args[0]] ^= self.regs[args[1]];
        self.set_pc(PcMode::Step);
    }

    fn add_xy(&mut self) {
        let args = self.get_args(ArgType::Xyn);
        let (res, wrap) = self.regs[args[0]].overflowing_add(self.regs[args[1]]);
        self.regs[args[0]] = res;
        if wrap {
            self.regs[0xF] = 1;
        } else {
            self.regs[0xF] = 0;
        }
        self.set_pc(PcMode::Step);
    }

    fn sub_xy(&mut self) {
        let args = self.get_args(ArgType::Xyn);
        let (res, wrap) = self.regs[args[0]].overflowing_sub(self.regs[args[1]]);
        self.regs[args[0]] = res;
        if wrap {
            self.regs[0xF] = 0;
        } else {
            self.regs[0xF] = 1;
        }
        self.set_pc(PcMode::Step);
    }

    fn shr_xy(&mut self) {
        let args = self.get_args(ArgType::Xyn);

        let (res, wrap) = if self.sh_compat {
            self.regs[args[0]]
        } else {
            self.regs[args[1]]
        }
        .overflowing_shr(1);

        self.regs[args[0]] = res & !0x80;
        if wrap {
            self.regs[0xF] = 1;
        }
        self.set_pc(PcMode::Step);
    }

    fn subn_xy(&mut self) {
        let args = self.get_args(ArgType::Xyn);
        let (res, wrap) = self.regs[args[1]].overflowing_sub(self.regs[args[0]]);
        self.regs[args[0]] = res;
        if wrap {
            self.regs[0xF] = 1;
        }
        self.set_pc(PcMode::Step);
    }

    fn shl_xy(&mut self) {
        let args = self.get_args(ArgType::Xyn);

        let (res, wrap) = if self.sh_compat {
            self.regs[args[0]]
        } else {
            self.regs[args[1]]
        }
        .overflowing_shl(1);

        self.regs[args[0]] = res & !0x1;
        if wrap {
            self.regs[0xF] = 1;
        }
        self.set_pc(PcMode::Step);
    }

    fn skip_neq_xy(&mut self) {
        let args = self.get_args(ArgType::Xyn);
        if self.regs[args[0]] != self.regs[args[1]] {
            self.set_pc(PcMode::Skip);
        } else {
            self.set_pc(PcMode::Step);
        }
    }

    fn load_i_nnn(&mut self) {
        let args = self.get_args(ArgType::Nnn);
        self.reg_i = args[0];
        self.set_pc(PcMode::Step);
    }

    fn jump_nnn_offset(&mut self) {
        let args = self.get_args(ArgType::Nnn);
        self.set_pc(PcMode::Jump(args[0] + self.regs[0] as usize));
    }

    fn rand_x_kk(&mut self) {
        let args = self.get_args(ArgType::Xkk);
        self.regs[args[0]] = gen_range(0, 255) & args[1] as u8;
        self.set_pc(PcMode::Step);
    }

    fn draw_xyn(&mut self) {
        let args = self.get_args(ArgType::Xyn);

        self.regs[0xF] = 0;

        for i in 0..args[2] {
            let y = (self.regs[args[1]] + i as u8) as usize % 32;
            for j in 0..8 {
                let x = (self.regs[args[0]] + j as u8) as usize % 64;
                let pixel = (self.mem[self.reg_i + i as usize] & (0x80 >> j)) == (0x80 >> j);

                let idx = y * 64 + x;

                if self.framebuffer[idx] && pixel {
                    self.regs[0xF] = 1;
                }

                self.framebuffer[idx] ^= pixel;
            }
        }

        self.set_pc(PcMode::Step);
    }

    fn skip_key_x(&mut self) {
        let args = self.get_args(ArgType::Xyn);
        if self.keymap[self.regs[args[0]] as usize] {
            self.set_pc(PcMode::Skip);
        } else {
            self.set_pc(PcMode::Step);
        }
    }

    fn skip_nkey_x(&mut self) {
        let args = self.get_args(ArgType::Xyn);
        if !self.keymap[self.regs[args[0]] as usize] {
            self.set_pc(PcMode::Skip);
        } else {
            self.set_pc(PcMode::Step);
        }
    }

    fn load_x_dt(&mut self) {
        let args = self.get_args(ArgType::Xyn);
        self.regs[args[0]] = self.reg_delay;
        self.set_pc(PcMode::Step);
    }

    fn block_key_x(&mut self) {
        let args = self.get_args(ArgType::Xyn);
        if self.keymap.iter().any(|e| *e) {
            self.block_release = true;
            self.regs[args[0]] = self.keymap.iter().position(|e| *e).unwrap() as u8;
        } else if self.block_release {
            self.block_release = false;
            self.set_pc(PcMode::Step);
        }
    }

    fn load_dt_x(&mut self) {
        let args = self.get_args(ArgType::Xyn);
        self.reg_delay = self.regs[args[0]];
        self.set_pc(PcMode::Step);
    }

    fn load_st_x(&mut self) {
        let args = self.get_args(ArgType::Xyn);
        self.reg_sound = self.regs[args[0]];
        self.set_pc(PcMode::Step);
    }

    fn add_i_x(&mut self) {
        let args = self.get_args(ArgType::Xyn);
        self.reg_i = self.reg_i.wrapping_add(self.regs[args[0]] as usize);
        self.set_pc(PcMode::Step);
    }

    fn load_i_digit_x(&mut self) {
        let args = self.get_args(ArgType::Xyn);
        self.reg_i = self.regs[args[0]] as usize * 5;
        self.set_pc(PcMode::Step);
    }

    fn store_bcd_x(&mut self) {
        let args = self.get_args(ArgType::Xyn);
        let num = self.regs[args[0]];

        self.mem[self.reg_i] = num / 100;
        self.mem[self.reg_i + 1] = (num % 100) / 10;
        self.mem[self.reg_i + 2] = num % 10;

        self.set_pc(PcMode::Step);
    }

    fn store_vx(&mut self) {
        let args = self.get_args(ArgType::Xyn);
        for idx in 0..=args[0] {
            self.mem[self.reg_i] = self.regs[idx];
            self.reg_i += 1;
        }

        if self.st_compat {
            self.reg_i -= args[0] + 1;
        }
        self.set_pc(PcMode::Step);
    }

    fn restore_vx(&mut self) {
        let args = self.get_args(ArgType::Xyn);
        for idx in 0..=args[0] {
            self.regs[idx] = self.mem[self.reg_i];
            self.reg_i += 1;
        }
        if self.st_compat {
            self.reg_i -= args[0] + 1;
        }
        self.set_pc(PcMode::Step);
    }

    fn set_pc(&mut self, pc_mode: PcMode) {
        match pc_mode {
            PcMode::Step => self.pc += 2,
            PcMode::Skip => self.pc += 4,
            PcMode::Jump(n) => self.pc = n,
        }
    }

    pub fn get_instr_addr(&self, addr: usize) -> [u8; 4] {
        let high = self.mem[addr];
        let low = self.mem[addr + 1];

        [
            (high & 0xF0) >> 4,
            high & 0x0F,
            (low & 0xF0) >> 4,
            low & 0x0F,
        ]
    }

    fn get_args(&self, arg_type: ArgType) -> [usize; 3] {
        self.get_args_addr(arg_type, self.pc)
    }

    pub fn get_args_addr(&self, arg_type: ArgType, addr: usize) -> [usize; 3] {
        let mut args = [0, 0, 0];

        let high = self.mem[addr] as usize;
        let low = self.mem[addr + 1] as usize;

        match arg_type {
            ArgType::Nnn => {
                args[0] = ((high & 0x0F) << 8) + low;
            }
            ArgType::Xkk => {
                args[0] = high & 0x0F;
                args[1] = low;
            }
            ArgType::Xyn => {
                args[0] = high & 0x0F;
                args[1] = (low & 0xF0) >> 4;
                args[2] = low & 0x0F;
            }
        }

        args
    }

    fn unimpl_panic(&self) {
        let n = self.get_instr_addr(self.pc);
        println!(
            "Unimplemented instruction: {:X}{:X}{:X}{:X}",
            n[0], n[1], n[2], n[3]
        ); // clean code :)

        panic!();
    }
}
