use std::collections::HashMap;

use super::Cpu6502;

#[derive(Debug, Clone, Copy)]
pub enum AddrMode {
    IMP, ACC, IMM, 
    ZPG, ZPX, ZPY, 
    REL, ABS, ABX, 
    ABY, IND, INX, INY
}

#[derive(Debug, Clone, Copy)]
pub enum Instr {
    ADC, AND, ASL, BCC, BCS, BEQ, BIT, BMI, BNE, BPL, BRK, BVC, BVS, CLC,
    CLD, CLI, CLV, CMP, CPX, CPY, DEC, DEX, DEY, EOR, INC, INX, INY, JMP, 
    JSR, LDA, LDX, LDY, LSR, NOP, ORA, PHA, PHP, PLA, PLP, ROL, ROR, RTI,
    RTS, SBC, SEC, SED, SEI, STA, STX, STY, TAX, TAY, TSX, TXA, TXS, TYA,

    // Unofficial / Illegal Opcodes
    ALR, ANC, ANE, ARR, DCP, ISC, LAS, LAX, RLA, RRA, SAX, SBX, SHA, 
    SHX, SHY, SLO, SRE, TAS, USBC, JAM,
}

#[allow(dead_code)]
pub struct Opcode {
    pub opcode: u8,
    pub instr: Instr,
    pub addr_mode: AddrMode,
    pub addr_mode_fn: fn(&mut Cpu6502),
    pub instr_fn: fn(&mut Cpu6502) -> u32, 
    pub cycles: u32, 
    pub illegal: bool,
}

impl Opcode {
    pub fn execute_op(&self, cpu: &mut Cpu6502) -> u32 {
        (self.addr_mode_fn)(cpu);
        let extra_cycles = (self.instr_fn)(cpu);
        self.cycles + extra_cycles
    }

    pub fn new(opcode: u8, addr_mode: AddrMode, instr: Instr, cycles: u32, illegal: bool) -> Self {
        let addr_mode_fn = match addr_mode {
            AddrMode::IMP => Cpu6502::imp_addressing,
            AddrMode::ACC => Cpu6502::acc_addressing,
            AddrMode::IMM => Cpu6502::imm_addressing,
            AddrMode::ZPG => Cpu6502::zpg_addressing,
            AddrMode::ZPX => Cpu6502::zpx_addressing,
            AddrMode::ZPY => Cpu6502::zpy_addressing,
            AddrMode::REL => Cpu6502::rel_addressing,
            AddrMode::ABS => Cpu6502::abs_addressing,
            AddrMode::ABX => Cpu6502::abx_addressing,
            AddrMode::ABY => Cpu6502::aby_addressing,
            AddrMode::IND => Cpu6502::ind_addressing,
            AddrMode::INX => Cpu6502::inx_addressing,
            AddrMode::INY => Cpu6502::iny_addressing,
        };

        let instr_fn = match instr {
            Instr::ADC => Cpu6502::add_with_carry,
            Instr::AND => Cpu6502::and_accumulator,
            Instr::ASL => Cpu6502::arithmetic_shift_left,
            Instr::BCC => Cpu6502::branch_if_carry_clear,
            Instr::BCS => Cpu6502::branch_if_carry_set,
            Instr::BEQ => Cpu6502::branch_if_equal,
            Instr::BIT => Cpu6502::bit_test,
            Instr::BMI => Cpu6502::branch_if_minus,
            Instr::BNE => Cpu6502::branch_if_not_equal,
            Instr::BPL => Cpu6502::branch_if_positive,
            Instr::BRK => Cpu6502::force_interrupt,
            Instr::BVC => Cpu6502::branch_if_overflow_clear,
            Instr::BVS => Cpu6502::branch_if_overflow_set,
            Instr::CLC => Cpu6502::clear_carry_flag,
            Instr::CLD => Cpu6502::clear_decimal_mode,
            Instr::CLI => Cpu6502::clear_interrupt_disable,
            Instr::CLV => Cpu6502::clear_overflow_flag,
            Instr::CMP => Cpu6502::compare_accumulator,
            Instr::CPX => Cpu6502::compare_x_reg,
            Instr::CPY => Cpu6502::compare_y_reg,
            Instr::DEC => Cpu6502::decrement_memory,
            Instr::DEX => Cpu6502::decrement_x_reg,
            Instr::DEY => Cpu6502::decrement_y_reg,
            Instr::EOR => Cpu6502::exclusive_or_accumulator,
            Instr::INC => Cpu6502::increment_memory,
            Instr::INX => Cpu6502::increment_x_reg,
            Instr::INY => Cpu6502::increment_y_reg,
            Instr::JMP => Cpu6502::jump,
            Instr::JSR => Cpu6502::jump_to_subroutine,
            Instr::LDA => Cpu6502::load_accumulator,
            Instr::LDX => Cpu6502::load_x_reg,
            Instr::LDY => Cpu6502::load_y_reg,
            Instr::LSR => Cpu6502::logical_shift_right,
            Instr::NOP => Cpu6502::no_operation,
            Instr::ORA => Cpu6502::or_accumulator,
            Instr::PHA => Cpu6502::push_accumulator,
            Instr::PHP => Cpu6502::push_processor_status,
            Instr::PLA => Cpu6502::pull_accumulator,
            Instr::PLP => Cpu6502::pull_processor_status,
            Instr::ROL => Cpu6502::rotate_left,
            Instr::ROR => Cpu6502::rotate_right,
            Instr::RTI => Cpu6502::return_from_interrupt,
            Instr::RTS => Cpu6502::return_from_subroutine,
            Instr::SBC => Cpu6502::subtract_with_carry,
            Instr::SEC => Cpu6502::set_carry_flag,
            Instr::SED => Cpu6502::set_decimal_mode,
            Instr::SEI => Cpu6502::set_interrupt_disable,
            Instr::STA => Cpu6502::store_accumulator,
            Instr::STX => Cpu6502::store_x_reg,
            Instr::STY => Cpu6502::store_y_reg,
            Instr::TAX => Cpu6502::transfer_accumulator_to_x,
            Instr::TAY => Cpu6502::transfer_accumulator_to_y,
            Instr::TSX => Cpu6502::transfer_stack_pointer_to_x,
            Instr::TXA => Cpu6502::transfer_x_to_accumulator,
            Instr::TXS => Cpu6502::transfer_x_to_stack_pointer,
            Instr::TYA => Cpu6502::transfer_y_to_accumulator,

            Instr::ALR => Cpu6502::alr,
            Instr::ANC => Cpu6502::anc,
            Instr::ANE => Cpu6502::ane,
            Instr::ARR => Cpu6502::arr,
            Instr::DCP => Cpu6502::dcp,
            Instr::ISC => Cpu6502::isc,
            Instr::LAS => Cpu6502::las,
            Instr::LAX => Cpu6502::lax,
            Instr::RLA => Cpu6502::rla,
            Instr::RRA => Cpu6502::rra,
            Instr::SAX => Cpu6502::sax, 
            Instr::SBX => Cpu6502::sbx,
            Instr::SHA => Cpu6502::sha,
            Instr::SHX => Cpu6502::shx,
            Instr::SHY => Cpu6502::shy,
            Instr::SLO => Cpu6502::slo,
            Instr::SRE => Cpu6502::sre,
            Instr::TAS => Cpu6502::tas,
            Instr::USBC => Cpu6502::usbc,
            Instr::JAM => Cpu6502::jam,
        };

        Opcode { 
            opcode,
            addr_mode,
            instr,
            addr_mode_fn,
            instr_fn,
            cycles,
            illegal,
        }
    }
}

lazy_static! {
    static ref OPCODES: Vec<Opcode> = vec![
        Opcode::new(0x69, AddrMode::IMM, Instr::ADC, 2, false),
        Opcode::new(0x65, AddrMode::ZPG, Instr::ADC, 3, false),
        Opcode::new(0x75, AddrMode::ZPX, Instr::ADC, 4, false),
        Opcode::new(0x6D, AddrMode::ABS, Instr::ADC, 4, false),
        Opcode::new(0x7D, AddrMode::ABX, Instr::ADC, 4, false),
        Opcode::new(0x79, AddrMode::ABY, Instr::ADC, 4, false),
        Opcode::new(0x61, AddrMode::INX, Instr::ADC, 6, false),
        Opcode::new(0x71, AddrMode::INY, Instr::ADC, 5, false),

        Opcode::new(0x29, AddrMode::IMM, Instr::AND, 2, false),
        Opcode::new(0x25, AddrMode::ZPG, Instr::AND, 3, false),
        Opcode::new(0x35, AddrMode::ZPX, Instr::AND, 4, false),
        Opcode::new(0x2D, AddrMode::ABS, Instr::AND, 4, false),
        Opcode::new(0x3D, AddrMode::ABX, Instr::AND, 4, false),
        Opcode::new(0x39, AddrMode::ABY, Instr::AND, 4, false),
        Opcode::new(0x21, AddrMode::INX, Instr::AND, 6, false),
        Opcode::new(0x31, AddrMode::INY, Instr::AND, 5, false),

        Opcode::new(0x0A, AddrMode::ACC, Instr::ASL, 2, false),
        Opcode::new(0x06, AddrMode::ZPG, Instr::ASL, 5, false),
        Opcode::new(0x16, AddrMode::ZPX, Instr::ASL, 6, false),
        Opcode::new(0x0E, AddrMode::ABS, Instr::ASL, 6, false),
        Opcode::new(0x1E, AddrMode::ABX, Instr::ASL, 7, false),

        Opcode::new(0x90, AddrMode::REL, Instr::BCC, 2, false),

        Opcode::new(0xB0, AddrMode::REL, Instr::BCS, 2, false),

        Opcode::new(0xF0, AddrMode::REL, Instr::BEQ, 2, false),

        Opcode::new(0x24, AddrMode::ZPG, Instr::BIT, 3, false),
        Opcode::new(0x2C, AddrMode::ABS, Instr::BIT, 4, false),

        Opcode::new(0x30, AddrMode::REL, Instr::BMI, 2, false),

        Opcode::new(0xD0, AddrMode::REL, Instr::BNE, 2, false),

        Opcode::new(0x10, AddrMode::REL, Instr::BPL, 2, false),

        Opcode::new(0x00, AddrMode::IMP, Instr::BRK, 7, false),

        Opcode::new(0x50, AddrMode::REL, Instr::BVC, 2, false),

        Opcode::new(0x70, AddrMode::REL, Instr::BVS, 2, false),

        Opcode::new(0x18, AddrMode::IMP, Instr::CLC, 2, false),

        Opcode::new(0xD8, AddrMode::IMP, Instr::CLD, 2, false),

        Opcode::new(0x58, AddrMode::IMP, Instr::CLI, 2, false),

        Opcode::new(0xB8, AddrMode::IMP, Instr::CLV, 2, false),

        Opcode::new(0xC9, AddrMode::IMM, Instr::CMP, 2, false),
        Opcode::new(0xC5, AddrMode::ZPG, Instr::CMP, 3, false),
        Opcode::new(0xD5, AddrMode::ZPX, Instr::CMP, 4, false),
        Opcode::new(0xCD, AddrMode::ABS, Instr::CMP, 4, false),
        Opcode::new(0xDD, AddrMode::ABX, Instr::CMP, 4, false),
        Opcode::new(0xD9, AddrMode::ABY, Instr::CMP, 4, false),
        Opcode::new(0xC1, AddrMode::INX, Instr::CMP, 6, false),
        Opcode::new(0xD1, AddrMode::INY, Instr::CMP, 5, false),

        Opcode::new(0xE0, AddrMode::IMM, Instr::CPX, 2, false),
        Opcode::new(0xE4, AddrMode::ZPG, Instr::CPX, 3, false),
        Opcode::new(0xEC, AddrMode::ABS, Instr::CPX, 4, false),

        Opcode::new(0xC0, AddrMode::IMM, Instr::CPY, 2, false),
        Opcode::new(0xC4, AddrMode::ZPG, Instr::CPY, 3, false),
        Opcode::new(0xCC, AddrMode::ABS, Instr::CPY, 4, false),

        Opcode::new(0xC6, AddrMode::ZPG, Instr::DEC, 5, false),
        Opcode::new(0xD6, AddrMode::ZPX, Instr::DEC, 6, false),
        Opcode::new(0xCE, AddrMode::ABS, Instr::DEC, 6, false),
        Opcode::new(0xDE, AddrMode::ABX, Instr::DEC, 7, false),

        Opcode::new(0xCA, AddrMode::IMP, Instr::DEX, 2, false),

        Opcode::new(0x88, AddrMode::IMP, Instr::DEY, 2, false),

        Opcode::new(0x49, AddrMode::IMM, Instr::EOR, 2, false),
        Opcode::new(0x45, AddrMode::ZPG, Instr::EOR, 3, false),
        Opcode::new(0x55, AddrMode::ZPX, Instr::EOR, 4, false),
        Opcode::new(0x4D, AddrMode::ABS, Instr::EOR, 4, false),
        Opcode::new(0x5D, AddrMode::ABX, Instr::EOR, 4, false),
        Opcode::new(0x59, AddrMode::ABY, Instr::EOR, 4, false),
        Opcode::new(0x41, AddrMode::INX, Instr::EOR, 6, false),
        Opcode::new(0x51, AddrMode::INY, Instr::EOR, 5, false),

        Opcode::new(0xE6, AddrMode::ZPG, Instr::INC, 5, false),
        Opcode::new(0xF6, AddrMode::ZPX, Instr::INC, 6, false),
        Opcode::new(0xEE, AddrMode::ABS, Instr::INC, 6, false),
        Opcode::new(0xFE, AddrMode::ABX, Instr::INC, 7, false),

        Opcode::new(0xE8, AddrMode::IMP, Instr::INX, 2, false),

        Opcode::new(0xC8, AddrMode::IMP, Instr::INY, 2, false),

        Opcode::new(0x4C, AddrMode::ABS, Instr::JMP, 3, false),
        Opcode::new(0x6C, AddrMode::IND, Instr::JMP, 5, false),

        Opcode::new(0x20, AddrMode::ABS, Instr::JSR, 6, false),

        Opcode::new(0xA9, AddrMode::IMM, Instr::LDA, 2, false),
        Opcode::new(0xA5, AddrMode::ZPG, Instr::LDA, 3, false),
        Opcode::new(0xB5, AddrMode::ZPX, Instr::LDA, 4, false),
        Opcode::new(0xAD, AddrMode::ABS, Instr::LDA, 4, false),
        Opcode::new(0xBD, AddrMode::ABX, Instr::LDA, 4, false),
        Opcode::new(0xB9, AddrMode::ABY, Instr::LDA, 4, false),
        Opcode::new(0xA1, AddrMode::INX, Instr::LDA, 6, false),
        Opcode::new(0xB1, AddrMode::INY, Instr::LDA, 5, false),

        Opcode::new(0xA2, AddrMode::IMM, Instr::LDX, 2, false),
        Opcode::new(0xA6, AddrMode::ZPG, Instr::LDX, 3, false),
        Opcode::new(0xB6, AddrMode::ZPY, Instr::LDX, 4, false),
        Opcode::new(0xAE, AddrMode::ABS, Instr::LDX, 4, false),
        Opcode::new(0xBE, AddrMode::ABY, Instr::LDX, 4, false),

        Opcode::new(0xA0, AddrMode::IMM, Instr::LDY, 2, false),
        Opcode::new(0xA4, AddrMode::ZPG, Instr::LDY, 3, false),
        Opcode::new(0xB4, AddrMode::ZPX, Instr::LDY, 4, false),
        Opcode::new(0xAC, AddrMode::ABS, Instr::LDY, 4, false),
        Opcode::new(0xBC, AddrMode::ABX, Instr::LDY, 4, false),

        Opcode::new(0x4A, AddrMode::ACC, Instr::LSR, 2, false),
        Opcode::new(0x46, AddrMode::ZPG, Instr::LSR, 5, false),
        Opcode::new(0x56, AddrMode::ZPX, Instr::LSR, 6, false),
        Opcode::new(0x4E, AddrMode::ABS, Instr::LSR, 6, false),
        Opcode::new(0x5E, AddrMode::ABX, Instr::LSR, 7, false),

        Opcode::new(0xEA, AddrMode::IMP, Instr::NOP, 2, false),

        Opcode::new(0x09, AddrMode::IMM, Instr::ORA, 2, false),
        Opcode::new(0x05, AddrMode::ZPG, Instr::ORA, 3, false),
        Opcode::new(0x15, AddrMode::ZPX, Instr::ORA, 4, false),
        Opcode::new(0x0D, AddrMode::ABS, Instr::ORA, 4, false),
        Opcode::new(0x1D, AddrMode::ABX, Instr::ORA, 4, false),
        Opcode::new(0x19, AddrMode::ABY, Instr::ORA, 4, false),
        Opcode::new(0x01, AddrMode::INX, Instr::ORA, 6, false),
        Opcode::new(0x11, AddrMode::INY, Instr::ORA, 5, false),

        Opcode::new(0x48, AddrMode::IMP, Instr::PHA, 3, false),

        Opcode::new(0x08, AddrMode::IMP, Instr::PHP, 3, false),

        Opcode::new(0x68, AddrMode::IMP, Instr::PLA, 4, false),

        Opcode::new(0x28, AddrMode::IMP, Instr::PLP, 4, false),

        Opcode::new(0x2A, AddrMode::ACC, Instr::ROL, 2, false),
        Opcode::new(0x26, AddrMode::ZPG, Instr::ROL, 5, false),
        Opcode::new(0x36, AddrMode::ZPX, Instr::ROL, 6, false),
        Opcode::new(0x2E, AddrMode::ABS, Instr::ROL, 6, false),
        Opcode::new(0x3E, AddrMode::ABX, Instr::ROL, 7, false),

        Opcode::new(0x6A, AddrMode::ACC, Instr::ROR, 2, false),
        Opcode::new(0x66, AddrMode::ZPG, Instr::ROR, 5, false),
        Opcode::new(0x76, AddrMode::ZPX, Instr::ROR, 6, false),
        Opcode::new(0x6E, AddrMode::ABS, Instr::ROR, 6, false),
        Opcode::new(0x7E, AddrMode::ABX, Instr::ROR, 7, false),

        Opcode::new(0x40, AddrMode::IMP, Instr::RTI, 6, false),

        Opcode::new(0x60, AddrMode::IMP, Instr::RTS, 6, false),

        Opcode::new(0xE9, AddrMode::IMM, Instr::SBC, 2, false),
        Opcode::new(0xE5, AddrMode::ZPG, Instr::SBC, 3, false),
        Opcode::new(0xF5, AddrMode::ZPX, Instr::SBC, 4, false),
        Opcode::new(0xED, AddrMode::ABS, Instr::SBC, 4, false),
        Opcode::new(0xFD, AddrMode::ABX, Instr::SBC, 4, false),
        Opcode::new(0xF9, AddrMode::ABY, Instr::SBC, 4, false),
        Opcode::new(0xE1, AddrMode::INX, Instr::SBC, 6, false),
        Opcode::new(0xF1, AddrMode::INY, Instr::SBC, 5, false),

        Opcode::new(0x38, AddrMode::IMP, Instr::SEC, 2, false),

        Opcode::new(0xF8, AddrMode::IMP, Instr::SED, 2, false),

        Opcode::new(0x78, AddrMode::IMP, Instr::SEI, 2, false),

        Opcode::new(0x85, AddrMode::ZPG, Instr::STA, 3, false),
        Opcode::new(0x95, AddrMode::ZPX, Instr::STA, 4, false),
        Opcode::new(0x8D, AddrMode::ABS, Instr::STA, 4, false),
        Opcode::new(0x9D, AddrMode::ABX, Instr::STA, 5, false),
        Opcode::new(0x99, AddrMode::ABY, Instr::STA, 5, false),
        Opcode::new(0x81, AddrMode::INX, Instr::STA, 6, false),
        Opcode::new(0x91, AddrMode::INY, Instr::STA, 6, false),

        Opcode::new(0x86, AddrMode::ZPG, Instr::STX, 3, false),
        Opcode::new(0x96, AddrMode::ZPY, Instr::STX, 4, false),
        Opcode::new(0x8E, AddrMode::ABS, Instr::STX, 4, false),

        Opcode::new(0x84, AddrMode::ZPG, Instr::STY, 3, false),
        Opcode::new(0x94, AddrMode::ZPX, Instr::STY, 4, false),
        Opcode::new(0x8C, AddrMode::ABS, Instr::STY, 4, false),

        Opcode::new(0xAA, AddrMode::IMP, Instr::TAX, 2, false),

        Opcode::new(0xA8, AddrMode::IMP, Instr::TAY, 2, false),

        Opcode::new(0xBA, AddrMode::IMP, Instr::TSX, 2, false),

        Opcode::new(0x8A, AddrMode::IMP, Instr::TXA, 2, false),

        Opcode::new(0x9A, AddrMode::IMP, Instr::TXS, 2, false),

        Opcode::new(0x98, AddrMode::IMP, Instr::TYA, 2, false),

        
        // Illegal/Unoffical Opcodes
        Opcode::new(0x4B, AddrMode::IMM, Instr::ALR, 2, true),

        Opcode::new(0x0B, AddrMode::IMM, Instr::ANC, 2, true),
        Opcode::new(0x2B, AddrMode::IMM, Instr::ANC, 2, true),

        Opcode::new(0x8B, AddrMode::IMM, Instr::ANE, 2, true),

        Opcode::new(0x6B, AddrMode::IMM, Instr::ARR, 2, true),

        Opcode::new(0xC7, AddrMode::ZPG, Instr::DCP, 5, true),
        Opcode::new(0xD7, AddrMode::ZPX, Instr::DCP, 6, true),
        Opcode::new(0xCF, AddrMode::ABS, Instr::DCP, 6, true),
        Opcode::new(0xDF, AddrMode::ABX, Instr::DCP, 7, true),
        Opcode::new(0xDB, AddrMode::ABY, Instr::DCP, 7, true),
        Opcode::new(0xC3, AddrMode::INX, Instr::DCP, 8, true),
        Opcode::new(0xD3, AddrMode::INY, Instr::DCP, 8, true),

        Opcode::new(0xE7, AddrMode::ZPG, Instr::ISC, 5, true),
        Opcode::new(0xF7, AddrMode::ZPX, Instr::ISC, 6, true),
        Opcode::new(0xEF, AddrMode::ABS, Instr::ISC, 6, true),
        Opcode::new(0xFF, AddrMode::ABX, Instr::ISC, 7, true),
        Opcode::new(0xFB, AddrMode::ABY, Instr::ISC, 7, true),
        Opcode::new(0xE3, AddrMode::INX, Instr::ISC, 8, true),
        Opcode::new(0xF3, AddrMode::INY, Instr::ISC, 8, true),

        Opcode::new(0xBB, AddrMode::IMM, Instr::LAS, 2, true),

        Opcode::new(0xAB, AddrMode::IMM, Instr::LAX, 2, true),
        Opcode::new(0xA7, AddrMode::ZPG, Instr::LAX, 3, true),
        Opcode::new(0xB7, AddrMode::ZPY, Instr::LAX, 4, true),
        Opcode::new(0xAF, AddrMode::ABS, Instr::LAX, 4, true),
        Opcode::new(0xBF, AddrMode::ABY, Instr::LAX, 4, true),
        Opcode::new(0xA3, AddrMode::INX, Instr::LAX, 6, true),
        Opcode::new(0xB3, AddrMode::INY, Instr::LAX, 5, true),

        Opcode::new(0x27, AddrMode::ZPG, Instr::RLA, 5, true),
        Opcode::new(0x37, AddrMode::ZPX, Instr::RLA, 6, true),
        Opcode::new(0x2F, AddrMode::ABS, Instr::RLA, 6, true),
        Opcode::new(0x3F, AddrMode::ABX, Instr::RLA, 7, true),
        Opcode::new(0x3B, AddrMode::ABY, Instr::RLA, 7, true),
        Opcode::new(0x23, AddrMode::INX, Instr::RLA, 8, true),
        Opcode::new(0x33, AddrMode::INY, Instr::RLA, 8, true),

        Opcode::new(0x67, AddrMode::ZPG, Instr::RRA, 5, true),
        Opcode::new(0x77, AddrMode::ZPX, Instr::RRA, 6, true),
        Opcode::new(0x6F, AddrMode::ABS, Instr::RRA, 6, true),
        Opcode::new(0x7F, AddrMode::ABX, Instr::RRA, 7, true),
        Opcode::new(0x7B, AddrMode::ABY, Instr::RRA, 7, true),
        Opcode::new(0x63, AddrMode::INX, Instr::RRA, 8, true),
        Opcode::new(0x73, AddrMode::INY, Instr::RRA, 8, true),

        Opcode::new(0x87, AddrMode::ZPG, Instr::SAX, 3, true),
        Opcode::new(0x97, AddrMode::ZPY, Instr::SAX, 4, true),
        Opcode::new(0x8F, AddrMode::ABS, Instr::SAX, 4, true),
        Opcode::new(0x83, AddrMode::INX, Instr::SAX, 6, true),

        Opcode::new(0xCB, AddrMode::IMM, Instr::SBX, 2, true),

        Opcode::new(0x9F, AddrMode::ABY, Instr::SHA, 5, true),
        Opcode::new(0x93, AddrMode::INY, Instr::SHA, 6, true),
    
        Opcode::new(0x9E, AddrMode::ABY, Instr::SHX, 5, true),

        Opcode::new(0x9C, AddrMode::ABX, Instr::SHY, 5, true),

        Opcode::new(0x07, AddrMode::ZPG, Instr::SLO, 5, true),
        Opcode::new(0x17, AddrMode::ZPX, Instr::SLO, 6, true),
        Opcode::new(0x0F, AddrMode::ABS, Instr::SLO, 6, true),
        Opcode::new(0x1F, AddrMode::ABX, Instr::SLO, 7, true),
        Opcode::new(0x1B, AddrMode::ABY, Instr::SLO, 7, true),
        Opcode::new(0x03, AddrMode::INX, Instr::SLO, 8, true),
        Opcode::new(0x13, AddrMode::INY, Instr::SLO, 8, true),

        Opcode::new(0x47, AddrMode::ZPG, Instr::SRE, 5, true),
        Opcode::new(0x57, AddrMode::ZPX, Instr::SRE, 6, true),
        Opcode::new(0x4F, AddrMode::ABS, Instr::SRE, 6, true),
        Opcode::new(0x5F, AddrMode::ABX, Instr::SRE, 7, true),
        Opcode::new(0x5B, AddrMode::ABY, Instr::SRE, 7, true),
        Opcode::new(0x43, AddrMode::INX, Instr::SRE, 8, true),
        Opcode::new(0x53, AddrMode::INY, Instr::SRE, 8, true),

        Opcode::new(0x9B, AddrMode::ABY, Instr::TAS, 5, true),

        Opcode::new(0xEB, AddrMode::IMM, Instr::USBC, 2, true),

        Opcode::new(0x1A, AddrMode::IMP, Instr::NOP, 2, true),
        Opcode::new(0x3A, AddrMode::IMP, Instr::NOP, 2, true),
        Opcode::new(0x5A, AddrMode::IMP, Instr::NOP, 2, true),
        Opcode::new(0x7A, AddrMode::IMP, Instr::NOP, 2, true),
        Opcode::new(0xDA, AddrMode::IMP, Instr::NOP, 2, true),
        Opcode::new(0xFA, AddrMode::IMP, Instr::NOP, 2, true),
        Opcode::new(0x80, AddrMode::IMM, Instr::NOP, 2, true),
        Opcode::new(0x82, AddrMode::IMM, Instr::NOP, 2, true),
        Opcode::new(0x89, AddrMode::IMM, Instr::NOP, 2, true),
        Opcode::new(0xC2, AddrMode::IMM, Instr::NOP, 2, true),
        Opcode::new(0xE2, AddrMode::IMM, Instr::NOP, 2, true),
        Opcode::new(0x04, AddrMode::ZPG, Instr::NOP, 3, true),
        Opcode::new(0x44, AddrMode::ZPG, Instr::NOP, 3, true),
        Opcode::new(0x64, AddrMode::ZPG, Instr::NOP, 3, true),
        Opcode::new(0x14, AddrMode::ZPX, Instr::NOP, 4, true),
        Opcode::new(0x34, AddrMode::ZPX, Instr::NOP, 4, true),
        Opcode::new(0x54, AddrMode::ZPX, Instr::NOP, 4, true),
        Opcode::new(0x74, AddrMode::ZPX, Instr::NOP, 4, true),
        Opcode::new(0xD4, AddrMode::ZPX, Instr::NOP, 4, true),
        Opcode::new(0xF4, AddrMode::ZPX, Instr::NOP, 4, true),
        Opcode::new(0x0C, AddrMode::ABS, Instr::NOP, 4, true),
        Opcode::new(0x1C, AddrMode::ABX, Instr::NOP, 4, true),
        Opcode::new(0x3C, AddrMode::ABX, Instr::NOP, 4, true),
        Opcode::new(0x5C, AddrMode::ABX, Instr::NOP, 4, true),
        Opcode::new(0x7C, AddrMode::ABX, Instr::NOP, 4, true),
        Opcode::new(0xDC, AddrMode::ABX, Instr::NOP, 4, true),
        Opcode::new(0xFC, AddrMode::ABX, Instr::NOP, 4, true),

        Opcode::new(0x02, AddrMode::IMP, Instr::JAM, 2, true),
        Opcode::new(0x12, AddrMode::IMP, Instr::JAM, 2, true),
        Opcode::new(0x22, AddrMode::IMP, Instr::JAM, 2, true),
        Opcode::new(0x32, AddrMode::IMP, Instr::JAM, 2, true),
        Opcode::new(0x42, AddrMode::IMP, Instr::JAM, 2, true),
        Opcode::new(0x52, AddrMode::IMP, Instr::JAM, 2, true),
        Opcode::new(0x62, AddrMode::IMP, Instr::JAM, 2, true),
        Opcode::new(0x72, AddrMode::IMP, Instr::JAM, 2, true),
        Opcode::new(0x92, AddrMode::IMP, Instr::JAM, 2, true),
        Opcode::new(0xB2, AddrMode::IMP, Instr::JAM, 2, true),
        Opcode::new(0xD2, AddrMode::IMP, Instr::JAM, 2, true),
        Opcode::new(0xF2, AddrMode::IMP, Instr::JAM, 2, true),
    ];

    pub static ref OPCODES_LOOKUP: HashMap<u8, &'static Opcode> = {
        let mut lookup = HashMap::new();
        for op in &*OPCODES {
            lookup.insert(op.opcode, op);
        }
        lookup
    };
}