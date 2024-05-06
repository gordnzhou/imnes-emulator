mod opcode;

use std::fs::OpenOptions;
use std::io::prelude::*;

use crate::bus::Bus;

use self::opcode::{AddrMode, OPCODES_LOOKUP};

enum Flag { C, Z, I, D, B, U, V, N }

impl Flag {
    pub fn mask(&self) -> u8 {
        match self {
            Flag::C => 0b00000001,
            Flag::Z => 0b00000010,
            Flag::I => 0b00000100,
            Flag::D => 0b00001000,
            Flag::B => 0b00010000,
            Flag::U => 0b00100000,
            Flag::V => 0b01000000,
            Flag::N => 0b10000000,
        }
    }
}

const STACK_START: u16 = 0x100;

fn log_to_file(message: &str) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open("logs/log.txt")?;

    println!("write: {}", message);
    writeln!(file, "{}", message)
}

pub struct Cpu6502 {
    accumulator: u8,
    x_index_reg: u8,
    y_index_reg: u8,
    program_counter: u16,
    stack_pointer: u8,
    processor_status: u8,

    cycles: u32,

    addr_mode: AddrMode,
    operand_addr: u16,
    operand_data: u8,
    page_crossed: bool,

    bus: Bus,
}

impl Cpu6502 {
    pub fn new(bus: Bus) -> Self {
        Cpu6502 {
            accumulator: 0,
            x_index_reg: 0,
            y_index_reg: 0,
            program_counter: 0xC000,
            stack_pointer: 0xFD,
            processor_status: 0x24,

            cycles: 7,

            addr_mode: AddrMode::IMP,
            operand_addr: 0,
            operand_data: 0,
            page_crossed: false,
            
            bus,
        }
    }

    pub fn execute(&mut self) {
        loop {
            let opcode = self.advance_pc();

            self.cycles += match OPCODES_LOOKUP.get(&opcode) {
                Some(op) => {
                    log_to_file(&format!("{:04X} OPCODE:{:?} IMM:{:02X}     A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X}  CYC:{}", 
                        self.program_counter - 1, op.instr, self.read_byte(self.program_counter), 
                        self.accumulator, self.x_index_reg, self.y_index_reg, self.processor_status, self.stack_pointer,
                        self.cycles)).unwrap();

                    op.execute_op(self)
                },
                None => panic!("Unrecognized/Unsupported Opcode: {}", opcode)
            };
        }
    }

    pub(super) fn add_with_carry(&mut self) -> u32 {
        let op1 = self.accumulator;
        let op2 = self.operand_data;
        self.accumulator = op1.wrapping_add(op2).wrapping_add(self.get_flag(Flag::C) as u8);

        self.set_flag(Flag::C, op1 as u16 + op2 as u16 + self.get_flag(Flag::C) as u16 > 0xFF);
        self.set_flag(Flag::Z, self.accumulator == 0);
        self.set_flag(Flag::V, (op1 ^ op2) & 0x80 == 0 && (op1 ^ self.accumulator) & 0x80 != 0);
        self.set_flag(Flag::N, self.accumulator & 0b10000000 != 0);

        self.page_crossed as u32
    }

    pub(super) fn and_accumulator(&mut self) -> u32 {
        self.accumulator = self.accumulator & self.operand_data;

        self.set_flag(Flag::Z, self.accumulator == 0);
        self.set_flag(Flag::N, self.accumulator & 0b10000000 != 0);

        self.page_crossed as u32
    }

    pub(super) fn arithmetic_shift_left(&mut self) -> u32 {
        let result = self.operand_data.wrapping_shl(1);
        self.write_to_operand(result);

        self.set_flag(Flag::C, self.operand_data & 0b10000000 != 0);
        self.set_flag(Flag::Z, result == 0);
        self.set_flag(Flag::N, result & 0b10000000 != 0);

        0
    }

    pub(super) fn branch_if_carry_clear(&mut self) -> u32 {
        if !self.get_flag(Flag::C) {
            self.program_counter = self.get_branch_pc();

            1 + self.page_crossed as u32
        } else {
            0
        }
    }

    pub(super) fn branch_if_carry_set(&mut self) -> u32 {
        if self.get_flag(Flag::C) {
            self.program_counter = self.get_branch_pc();

            1 + self.page_crossed as u32
        } else {
            0
        }
    }

    pub(super) fn branch_if_equal(&mut self) -> u32 {
        if self.get_flag(Flag::Z) {
            self.program_counter = self.get_branch_pc();

            1 + self.page_crossed as u32
        } else {
            0
        }
    }

    pub(super) fn bit_test(&mut self) -> u32 {
        self.set_flag(Flag::Z, self.accumulator & self.operand_data == 0);
        self.set_flag(Flag::V, self.operand_data & 0b01000000 != 0);
        self.set_flag(Flag::N, self.operand_data & 0b10000000 != 0);

        0
    }

    pub(super) fn branch_if_minus(&mut self) -> u32 {
        if self.get_flag(Flag::N) {
            self.program_counter = self.get_branch_pc();

            1 + self.page_crossed as u32
        } else {
            0
        }
    }

    pub(super) fn branch_if_not_equal(&mut self) -> u32 {
        if !self.get_flag(Flag::Z) {
            self.program_counter = self.get_branch_pc();

            1 + self.page_crossed as u32
        } else {
            0
        }
    }

    pub(super) fn branch_if_positive(&mut self) -> u32 {
        if !self.get_flag(Flag::N) {
            self.program_counter = self.get_branch_pc();

            1 + self.page_crossed as u32
        } else {
            0
        }
    }

    pub(super) fn branch_if_overflow_clear(&mut self) -> u32 {
        if !self.get_flag(Flag::V) {
            self.program_counter = self.get_branch_pc();

            1 + self.page_crossed as u32
        } else {
            0
        }
    }

    pub(super) fn branch_if_overflow_set(&mut self) -> u32 {
        if self.get_flag(Flag::V) {
            self.program_counter = self.get_branch_pc();

            1 + self.page_crossed as u32
        } else {
            0
        }
    }

    pub(super) fn clear_carry_flag(&mut self) -> u32 {
        self.set_flag(Flag::C, false);

        0
    }

    pub(super) fn clear_decimal_mode(&mut self) -> u32 {
        self.set_flag(Flag::D, false);
        
        0
    }

    pub(super) fn clear_interrupt_disable(&mut self) -> u32 {
        self.set_flag(Flag::I, false);
        
        0
    }

    pub(super) fn clear_overflow_flag(&mut self) -> u32 {
        self.set_flag(Flag::V, false);
        
        0
    }

    pub(super) fn compare_accumulator(&mut self) -> u32 {
        self.set_flag(Flag::C, self.accumulator >= self.operand_data);
        self.set_flag(Flag::Z, self.accumulator == self.operand_data);
        self.set_flag(Flag::N, self.accumulator.wrapping_sub(self.operand_data) & 0b10000000 != 0);
        
        self.page_crossed as u32
    }

    pub(super) fn compare_x_reg(&mut self) -> u32 {
        self.set_flag(Flag::C, self.x_index_reg >= self.operand_data);
        self.set_flag(Flag::Z, self.x_index_reg == self.operand_data);
        self.set_flag(Flag::N, self.x_index_reg.wrapping_sub(self.operand_data) & 0b10000000 != 0);

        0
    }

    pub(super) fn compare_y_reg(&mut self) -> u32 {
        self.set_flag(Flag::C, self.y_index_reg >= self.operand_data);
        self.set_flag(Flag::Z, self.y_index_reg == self.operand_data);
        self.set_flag(Flag::N, self.y_index_reg.wrapping_sub(self.operand_data) & 0b10000000 != 0);

        0
    }

    pub(super) fn decrement_memory(&mut self) -> u32 {
        self.write_to_operand(self.operand_data.wrapping_sub(1));
        let result = self.read_byte(self.operand_addr);

        self.set_flag(Flag::Z, result == 0);
        self.set_flag(Flag::N, result & 0b10000000 != 0);

        0
    }

    pub(super) fn decrement_x_reg(&mut self) -> u32 {
        self.x_index_reg = self.x_index_reg.wrapping_sub(1);

        self.set_flag(Flag::Z, self.x_index_reg == 0);
        self.set_flag(Flag::N, self.x_index_reg & 0b10000000 != 0);

        0
    }

    pub(super) fn decrement_y_reg(&mut self) -> u32 {
        self.y_index_reg = self.y_index_reg.wrapping_sub(1);

        self.set_flag(Flag::Z, self.y_index_reg == 0);
        self.set_flag(Flag::N, self.y_index_reg & 0b10000000 != 0);

        0
    }

    pub(super) fn exclusive_or_accumulator(&mut self) -> u32 {
        self.accumulator = self.accumulator ^ self.operand_data;

        self.set_flag(Flag::Z, self.accumulator == 0);
        self.set_flag(Flag::N, self.accumulator & 0b10000000 != 0);

        self.page_crossed as u32
    }  

    // TODO: implement interrupts
    pub(super) fn force_interrupt(&mut self) -> u32 {
        self.push_word_to_stack(self.program_counter);
        self.push_byte_to_stack(self.processor_status | Flag::B.mask() | Flag::U.mask());

        // self.program_counter = 0xFFFE;

        self.set_flag(Flag::B, true);

        0
    }

    pub(super) fn increment_memory(&mut self) -> u32 {
        self.write_to_operand(self.operand_data.wrapping_add(1));
        let result = self.read_byte(self.operand_addr);

        self.set_flag(Flag::Z, result == 0);
        self.set_flag(Flag::N, result & 0b10000000 != 0);

        0
    }

    pub(super) fn increment_x_reg(&mut self) -> u32 {
        self.x_index_reg = self.x_index_reg.wrapping_add(1);

        self.set_flag(Flag::Z, self.x_index_reg == 0);
        self.set_flag(Flag::N, self.x_index_reg & 0b10000000 != 0);

        0
    }

    pub(super) fn increment_y_reg(&mut self) -> u32 {
        self.y_index_reg = self.y_index_reg.wrapping_add(1);

        self.set_flag(Flag::Z, self.y_index_reg == 0);
        self.set_flag(Flag::N, self.y_index_reg & 0b10000000 != 0);

        0
    }

    pub(super) fn jump(&mut self) -> u32 {
        self.program_counter = self.operand_addr;

        0
    }

    pub(super) fn jump_to_subroutine(&mut self) -> u32 {
        self.push_word_to_stack(self.program_counter.wrapping_sub(1));
        self.program_counter = self.operand_addr;

        0
    }

    pub(super) fn load_accumulator(&mut self) -> u32 {
        self.accumulator = self.operand_data;

        self.set_flag(Flag::Z, self.accumulator == 0);
        self.set_flag(Flag::N, self.accumulator & 0b10000000 != 0);

        self.page_crossed as u32
    }

    pub(super) fn load_x_reg(&mut self) -> u32 {
        self.x_index_reg = self.operand_data;

        self.set_flag(Flag::Z, self.x_index_reg == 0);
        self.set_flag(Flag::N, self.x_index_reg & 0b10000000 != 0);

        self.page_crossed as u32
    }

    pub(super) fn load_y_reg(&mut self) -> u32 {
        self.y_index_reg = self.operand_data;

        self.set_flag(Flag::Z, self.y_index_reg == 0);
        self.set_flag(Flag::N, self.y_index_reg & 0b10000000 != 0);

        self.page_crossed as u32
    }

    pub(super) fn logical_shift_right(&mut self) -> u32 {
        let result = self.operand_data.wrapping_shr(1);
        self.write_to_operand(result);

        self.set_flag(Flag::C, self.operand_data & 0b00000001 != 0);
        self.set_flag(Flag::Z, result == 0);
        self.set_flag(Flag::N, false);

        0
    }

    pub(super) fn no_operation(&mut self) -> u32 {
        // do nothing

        0
    }

    pub(super) fn or_accumulator(&mut self) -> u32 {
        self.accumulator = self.accumulator | self.operand_data;

        self.set_flag(Flag::Z, self.accumulator == 0);
        self.set_flag(Flag::N, self.accumulator & 0b10000000 != 0);

        self.page_crossed as u32
    }

    pub(super) fn push_accumulator(&mut self) -> u32 {
        self.push_byte_to_stack(self.accumulator);

        0
    }

    pub(super) fn push_processor_status(&mut self) -> u32 {
        self.push_byte_to_stack(self.processor_status | Flag::B.mask() | Flag::U.mask());

        0
    }

    pub(super) fn pull_accumulator(&mut self) -> u32 {
        self.accumulator = self.pop_byte_from_stack();

        self.set_flag(Flag::Z, self.accumulator == 0);
        self.set_flag(Flag::N, self.accumulator & 0b10000000 != 0);

        0
    }

    pub(super) fn pull_processor_status(&mut self) -> u32 {
        self.processor_status = self.pop_byte_from_stack();
        self.processor_status &= !Flag::B.mask();
        self.processor_status |= Flag::U.mask();

        0
    }

    pub(super) fn return_from_interrupt(&mut self) -> u32 {
        self.processor_status = self.pop_byte_from_stack();
        self.processor_status &= !Flag::B.mask();
        self.processor_status |= Flag::U.mask();
        self.program_counter = self.pop_word_from_stack();

        0
    }

    pub(super) fn return_from_subroutine(&mut self) -> u32 {
        self.program_counter = self.pop_word_from_stack().wrapping_add(1);

        0
    }

    pub(super) fn rotate_left(&mut self) -> u32 {
        let result = self.operand_data.wrapping_shl(1) | (self.get_flag(Flag::C) as u8);
        self.write_to_operand(result);

        self.set_flag(Flag::C, self.operand_data & 0b10000000 != 0);
        self.set_flag(Flag::Z, result == 0);
        self.set_flag(Flag::N, result & 0b10000000 != 0);

        0
    }

    pub(super) fn rotate_right(&mut self) -> u32 {
        let result = self.operand_data.wrapping_shr(1) | ((self.get_flag(Flag::C) as u8) << 7);
        self.write_to_operand(result);

        self.set_flag(Flag::C, self.operand_data & 0b00000001 != 0);
        self.set_flag(Flag::Z, result == 0);
        self.set_flag(Flag::N, result & 0b10000000 != 0);

        0
    }

    pub(super) fn set_carry_flag(&mut self) -> u32 {
        self.set_flag(Flag::C, true);
        
        0
    }

    pub(super) fn set_decimal_mode(&mut self) -> u32 {
        self.set_flag(Flag::D, true);
        
        0
    }

    pub(super) fn set_interrupt_disable(&mut self) -> u32 {
        self.set_flag(Flag::I, true);
        
        0
    }

    pub(super) fn store_accumulator(&mut self) -> u32 {
        self.write_to_operand(self.accumulator);

        0
    }

    pub(super) fn store_x_reg(&mut self) -> u32 {
        self.write_to_operand(self.x_index_reg);

        0
    }

    pub(super) fn store_y_reg(&mut self) -> u32 {
        self.write_to_operand(self.y_index_reg);

        0
    }

    pub(super) fn subtract_with_carry(&mut self) -> u32 {
        // how SBC is done in the original hardware
        self.operand_data = !self.operand_data;
        self.add_with_carry()

        // let op1 = self.accumulator;
        // let op2 = self.operand_data.wrapping_add(1 - (self.get_flag(Flag::C) as u8));
        // self.accumulator = op1.wrapping_sub(op2);

        // self.set_flag(Flag::C, op1 > op2);
        // self.set_flag(Flag::Z, self.accumulator == 0);
        // self.set_flag(Flag::V, (op1 ^ op2) & 0x80 != 0 && (op1 ^ self.accumulator) & 0x80 != 0);
        // self.set_flag(Flag::N, self.accumulator & 0b10000000 != 0);

        // self.page_crossed as u32
    }

    pub(super) fn transfer_accumulator_to_x(&mut self) -> u32 {
        self.x_index_reg = self.accumulator;

        self.set_flag(Flag::Z, self.x_index_reg == 0);
        self.set_flag(Flag::N, self.x_index_reg & 0b10000000 != 0);

        0
    }

    pub(super) fn transfer_accumulator_to_y(&mut self) -> u32 {
        self.y_index_reg = self.accumulator;

        self.set_flag(Flag::Z, self.y_index_reg == 0);
        self.set_flag(Flag::N, self.y_index_reg & 0b10000000 != 0);

        0
    }

    pub(super) fn transfer_stack_pointer_to_x(&mut self) -> u32 {
        self.x_index_reg = self.stack_pointer;

        self.set_flag(Flag::Z, self.x_index_reg == 0);
        self.set_flag(Flag::N, self.x_index_reg & 0b10000000 != 0);

        0
    }

    pub(super) fn transfer_x_to_accumulator(&mut self) -> u32 {
        self.accumulator = self.x_index_reg;

        self.set_flag(Flag::Z, self.accumulator == 0);
        self.set_flag(Flag::N, self.accumulator & 0b10000000 != 0);

        0
    }

    pub(super) fn transfer_x_to_stack_pointer(&mut self) -> u32 {
        self.stack_pointer = self.x_index_reg;

        0
    }

    pub(super) fn transfer_y_to_accumulator(&mut self) -> u32 {
        self.accumulator = self.y_index_reg;

        self.set_flag(Flag::Z, self.accumulator == 0);
        self.set_flag(Flag::N, self.accumulator & 0b10000000 != 0);

        0
    }

    // TODO: implement illegal instructions
    pub(super) fn alr(&mut self) -> u32 {
        0
    }

    pub(super) fn anc(&mut self) -> u32 {
        0
    }

    pub(super) fn ane(&mut self) -> u32 {
        0
    }

    pub(super) fn arr(&mut self) -> u32 {
        0
    }

    pub(super) fn dcp(&mut self) -> u32 {
        0
    }

    pub(super) fn isc(&mut self) -> u32 {
        0
    }

    pub(super) fn las(&mut self) -> u32 {
        0
    }

    pub(super) fn lax(&mut self) -> u32 {
        0
    }

    pub(super) fn lxa(&mut self) -> u32 {
        0
    }

    pub(super) fn rla(&mut self) -> u32 {
        0
    }

    pub(super) fn rra(&mut self) -> u32 {
        0
    }

    pub(super) fn sax(&mut self) -> u32 {
        0
    }

    pub(super) fn sbx(&mut self) -> u32 {
        0
    }

    pub(super) fn sha(&mut self) -> u32 {
        0
    }

    pub(super) fn shx(&mut self) -> u32 {
        0
    }

    pub(super) fn shy(&mut self) -> u32 {
        0
    }

    pub(super) fn slo(&mut self) -> u32 {
        0
    }

    pub(super) fn sre(&mut self) -> u32 {
        0
    }

    pub(super) fn tas(&mut self) -> u32 {
        0
    }

    pub(super) fn usbc(&mut self) -> u32 {
        0
    }

    pub(super) fn jam(&mut self) -> u32 {
        0
    }

    #[inline]
    pub(super) fn imp_addressing(&mut self) { 
        self.addr_mode = AddrMode::IMP;

        self.set_operand_data(0);
    }

    #[inline]
    pub(super) fn acc_addressing(&mut self) { 
        self.addr_mode = AddrMode::ACC;

        self.set_operand_data(self.accumulator);
    }

    #[inline]
    pub(super) fn imm_addressing(&mut self) { 
        self.addr_mode = AddrMode::IMM;
        let operand_data = self.advance_pc();
        
        self.set_operand_data(operand_data);
    }

    #[inline]
    pub(super) fn zpg_addressing(&mut self) {
        self.addr_mode = AddrMode::ZPG;
        let operand_addr = self.advance_pc() as u16;

        self.set_operand_addr(operand_addr);
    }

    #[inline]
    pub(super) fn zpx_addressing(&mut self) {
        self.addr_mode = AddrMode::ZPX;
        let operand_addr = self.advance_pc().wrapping_add(self.x_index_reg) as u16;

        self.set_operand_addr(operand_addr);
    }

    #[inline]
    pub(super) fn zpy_addressing(&mut self) {
        self.addr_mode = AddrMode::ZPY;
        let operand_addr = self.advance_pc().wrapping_add(self.y_index_reg) as u16;

        self.set_operand_addr(operand_addr);
    }

    #[inline]
    pub(super) fn rel_addressing(&mut self) {
        self.addr_mode = AddrMode::REL;
        let offset =  (self.advance_pc() as i8) as i32;

        self.set_operand_addr((self.program_counter as i32 + offset) as u16);
        self.page_crossed = (self.program_counter ^ self.operand_addr) & 0xFF00 != 0;
    }

    #[inline]
    pub(super) fn abs_addressing(&mut self) {
        self.addr_mode = AddrMode::ABS;
        let abs_address = self.fetch_abs_address();

        self.set_operand_addr(abs_address);
    }

    #[inline]
    pub(super) fn abx_addressing(&mut self) {
        self.addr_mode = AddrMode::ABX;
        let addr = self.fetch_abs_address();
        
        self.set_operand_addr(addr.wrapping_add(self.x_index_reg as u16));
        self.page_crossed = ((self.operand_addr ^ addr) & 0xFF00) != 0;
    }

    #[inline]
    pub(super) fn aby_addressing(&mut self) {
        self.addr_mode = AddrMode::ABY;
        let addr = self.fetch_abs_address();

        self.set_operand_addr(addr.wrapping_add(self.y_index_reg as u16));
        self.page_crossed = ((self.operand_addr ^ addr) & 0xFF00) != 0;
    }

    #[inline]
    pub(super) fn ind_addressing(&mut self) { 
        let ptr = self.fetch_abs_address();

        let lo = self.read_byte(ptr) as u16;

        let hi = if ptr & 0xFF == 0xFF {
            self.read_byte(ptr & 0xFF00) // Simulate Hardware Bug
        } else { 
            self.read_byte(ptr.wrapping_add(1))
        } as u16;

        self.set_operand_addr((hi << 8) | lo);
    }

    #[inline]
    pub(super) fn inx_addressing(&mut self) {
        let ptr = self.advance_pc().wrapping_add(self.x_index_reg);

        let lo = self.read_byte(ptr as u16) as u16;
        let hi = self.read_byte(ptr.wrapping_add(1) as u16) as u16;
        
        self.set_operand_addr((hi << 8) | lo);
    }

    #[inline]
    pub(super) fn iny_addressing(&mut self) {
        let ptr = self.advance_pc();

        let lo = self.read_byte(ptr as u16) as u16;
        let hi = self.read_byte(ptr.wrapping_add(1) as u16) as u16;

        let addr = (hi << 8) | lo;

        self.set_operand_addr(addr.wrapping_add(self.y_index_reg as u16));
        self.page_crossed = ((self.operand_addr ^ addr) & 0xFF00) != 0;
    }

    #[inline]
    fn write_to_operand(&mut self, byte: u8) {
        match self.addr_mode {
            AddrMode::ACC => self.accumulator = byte,
            _ => self.write_byte(self.operand_addr, byte)
        }
    }

    #[inline]
    fn get_branch_pc(&self) -> u16 {
        assert!(matches!(self.addr_mode, AddrMode::REL));
        self.operand_addr
    }

    #[inline]
    fn set_operand_addr(&mut self, operand_addr: u16) {
        self.operand_addr = operand_addr;
        self.set_operand_data(self.read_byte(self.operand_addr));
    }

    #[inline]
    fn set_operand_data(&mut self, operand_data: u8) {
        self.operand_data = operand_data;
        self.page_crossed = false;
    }

    #[inline]
    fn fetch_abs_address(&mut self) -> u16 {
        let lo = self.advance_pc() as u16;
        let hi = self.advance_pc() as u16;
        (hi << 8) | lo
    }

    #[inline]
    fn push_word_to_stack(&mut self, word: u16) {
        self.push_byte_to_stack(((word & 0xFF00) >> 8) as u8);
        self.push_byte_to_stack(word as u8);
    }

    #[inline]
    fn pop_word_from_stack(&mut self) -> u16 {
        let lo = self.pop_byte_from_stack() as u16;
        let hi = self.pop_byte_from_stack() as u16;
        (hi << 8) | lo
    }

    #[inline]
    fn push_byte_to_stack(&mut self, byte: u8) {
        self.write_byte(STACK_START | self.stack_pointer as u16, byte);
        self.stack_pointer = self.stack_pointer.wrapping_sub(1);
    }

    #[inline]
    fn pop_byte_from_stack(&mut self) -> u8 {
        self.stack_pointer = self.stack_pointer.wrapping_add(1);
        self.read_byte(STACK_START | self.stack_pointer as u16)
    }

    #[inline]
    fn set_flag(&mut self, flag: Flag, val: bool) {
        let mask = flag.mask();
        if val {
            self.processor_status |= mask;
        } else {
            self.processor_status &= !mask;
        }
    }

    #[inline]
    fn get_flag(&self, flag: Flag) -> bool {
        (self.processor_status & flag.mask()) != 0
    }

    #[inline]
    fn advance_pc(&mut self) -> u8 {
        let ret = self.read_byte(self.program_counter);
        self.program_counter += 1;

        if self.program_counter == 0x2010 {
            println!("ERROR CODE: {:02x}, ERROR BYTE LOCATION: {:02x}", self.read_byte(2), self.read_byte(3));
        }

        ret
    }

    #[inline]
    fn read_byte(&self, addr: u16) -> u8 {
        self.bus.read_byte(addr)
    }

    #[inline]
    fn write_byte(&mut self, addr: u16, byte: u8) {
        self.bus.write_byte(addr, byte);
    }
}

#[cfg(test)]
mod tests {
    use crate::{bus::Bus, cpu::Flag};

    use super::{opcode::OPCODES_LOOKUP, Cpu6502};

    #[test]
    pub fn test_lda() {
        let mut bus = Bus::new();
        bus.load_memory(&vec![0xA9, 0x11, 0xA5, 0xFE, 0xB5, 0xFC, 0xAD, 0x34, 0x12, 0xBD, 0x34, 0x12, 0xB9, 0x34, 0x12]);
        
        let mut cpu = Cpu6502::new(bus);
        cpu.program_counter = 0x00;
        cpu.x_index_reg = 2;
        cpu.y_index_reg = 3;
        cpu.write_byte(0xFE, 0x22);
        cpu.write_byte(0x1234, 0x33);
        cpu.write_byte(0x1236, 0x44);
        cpu.write_byte(0x1237, 0x55);

        let mut opcode = cpu.advance_pc();
        match OPCODES_LOOKUP.get(&opcode) {
            Some(op) => op.execute_op(&mut cpu),
            None => panic!("Unsupported Opcode: {}", opcode)
        };
        assert!(cpu.program_counter == 0x02);
        assert_eq!(cpu.accumulator, 0x11, "FAILED: imm");

        opcode = cpu.advance_pc();
        match OPCODES_LOOKUP.get(&opcode) {
            Some(op) => op.execute_op(&mut cpu),
            None => panic!("Unsupported Opcode: {}", opcode)
        };
        assert!(cpu.program_counter == 0x04);
        assert!(cpu.accumulator == 0x22, "FAILED: zpg");

        opcode = cpu.advance_pc();
        match OPCODES_LOOKUP.get(&opcode) {
            Some(op) => op.execute_op(&mut cpu),
            None => panic!("Unsupported Opcode: {}", opcode)
        };
        assert_eq!(cpu.accumulator, 0x22, "FAILED: zpx");

        opcode = cpu.advance_pc();
        match OPCODES_LOOKUP.get(&opcode) {
            Some(op) => op.execute_op(&mut cpu),
            None => panic!("Unsupported Opcode: {}", opcode)
        };
        assert_eq!(cpu.accumulator, 0x33, "FAILED: abs");

        opcode = cpu.advance_pc();
        match OPCODES_LOOKUP.get(&opcode) {
            Some(op) => op.execute_op(&mut cpu),
            None => panic!("Unsupported Opcode: {}", opcode)
        };
        assert_eq!(cpu.accumulator, 0x44, "FAILED: abx");

        opcode = cpu.advance_pc();
        match OPCODES_LOOKUP.get(&opcode) {
            Some(op) => op.execute_op(&mut cpu),
            None => panic!("Unsupported Opcode: {}", opcode)
        };
        assert_eq!(cpu.accumulator, 0x55, "FAILED: aby");
    }

    #[test]
    pub fn test_stack() {
        let mut cpu = Cpu6502::new(Bus::new());

        cpu.push_byte_to_stack(0x88);
        assert_eq!(cpu.pop_byte_from_stack(), 0x88);

        cpu.push_word_to_stack(0x1122);
        assert_eq!(cpu.pop_word_from_stack(), 0x1122);

        cpu.push_word_to_stack(0x3344);
        cpu.push_word_to_stack(0x5566);

        assert_eq!(cpu.pop_word_from_stack(), 0x5566);

        cpu.push_word_to_stack(0x8899);

        assert_eq!(cpu.pop_word_from_stack(), 0x8899);

        cpu.push_word_to_stack(0x1010);
        cpu.push_word_to_stack(0x6969);

        assert_eq!(cpu.pop_word_from_stack(), 0x6969);
        assert_eq!(cpu.pop_word_from_stack(), 0x1010);
        assert_eq!(cpu.pop_word_from_stack(), 0x3344);
    }

    #[test]
    pub fn test_adc() {
        do_adc(1, 1, 2, false, false);
        do_adc(0x7F, 0x7F, 0xFE, true, false);
        do_adc(50, 25, 75, false, false);
        do_adc(128, 128, 0, true, true);
        do_adc(0b01111111, 0b00000010, 0b10000001, true, false);
        do_adc(255, 1, 0, false, true);
    }

    #[test]
    pub fn test_sbc() {
        do_sbc(3, 1, 2, false, true);
        do_sbc(100, 50, 50, false, true);
        do_sbc(128, 1, 127, true, true);
        do_sbc(0, 1, 255, false, false);
    }


    pub fn do_adc(operand1: u8, operand2: u8, result: u8, overflow: bool, carry: bool) {
        let mut bus = Bus::new();
        bus.load_memory(&vec![0x69, operand2]);

        let mut cpu = Cpu6502::new(bus);
        cpu.program_counter = 0x00;
        cpu.accumulator = operand1;

        let opcode = cpu.advance_pc();
        match OPCODES_LOOKUP.get(&opcode) {
            Some(op) => op.execute_op(&mut cpu),
            None => panic!("Unsupported Opcode: {}", opcode)
        };

        assert_eq!(cpu.accumulator, result, "Incorrect Result");
        assert_eq!(cpu.get_flag(Flag::C), carry, "Incorrect Carry Result");
        assert_eq!(cpu.get_flag(Flag::V), overflow, "Incorrect Overflow Result");
    }

    pub fn do_sbc(operand1: u8, operand2: u8, result: u8, overflow: bool, carry: bool) {
        let mut bus = Bus::new();
        bus.load_memory(&vec![0xE9, operand2]);

        let mut cpu = Cpu6502::new(bus);
        cpu.program_counter = 0x00;
        cpu.set_flag(Flag::C, true);
        cpu.accumulator = operand1;

        let opcode = cpu.advance_pc();
        match OPCODES_LOOKUP.get(&opcode) {
            Some(op) => op.execute_op(&mut cpu),
            None => panic!("Unsupported Opcode: {}", opcode)
        };

        assert_eq!(cpu.accumulator, result, "Incorrect Result");
        assert_eq!(cpu.get_flag(Flag::C), carry, "Incorrect Carry Result");
        assert_eq!(cpu.get_flag(Flag::V), overflow, "Incorrect Overflow Result");
    }
}