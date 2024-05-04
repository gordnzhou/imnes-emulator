mod opcode;

use self::opcode::OPCODES_LOOKUP;

const MEM_SIZE: usize = 0x10000;

enum Flag { C, Z, I, D, B, V, N }

impl Flag {
    pub fn mask(&self) -> u8 {
        match self {
            Flag::C => 0b00000001,
            Flag::Z => 0b00000010,
            Flag::I => 0b00000100,
            Flag::D => 0b00001000,
            Flag::B => 0b00010000,
            Flag::V => 0b01000000,
            Flag::N => 0b10000000,
        }
    }
}

const STACK_START: u16 = 0x100;

pub struct Cpu6502 {
    accumulator: u8,
    x_index_reg: u8,
    y_index_reg: u8,
    program_counter: u16,
    stack_pointer: u8,
    p_status: u8,

    cycles: u32,

    operand_addr: u16,
    operand_data: u8,
    page_crossed: bool,

    memory: [u8; MEM_SIZE]
}

impl Cpu6502 {
    pub fn new() -> Self {
        Cpu6502 {
            accumulator: 0,
            x_index_reg: 0,
            y_index_reg: 0,
            program_counter: 0,
            stack_pointer: 0,
            p_status: 0,

            cycles: 0,

            operand_addr: 0,
            operand_data: 0,
            page_crossed: false,
            
            memory: [0; MEM_SIZE]
        }
    }

    pub fn execute(&mut self) {
        loop {
            let opcode = self.advance_pc();

            self.cycles = match OPCODES_LOOKUP.get(&opcode) {
                Some(op) => op.execute_op(self),
                None => panic!("Unsupported Opcode: {}", opcode)
            };
        }
    }

    pub(super) fn add_with_carry(cpu: &mut Cpu6502) -> u32 {
        0
    }

    pub(super) fn and_accumulator(cpu: &mut Cpu6502) -> u32 {
        0
    }

    pub(super) fn arithmetic_shift_left(cpu: &mut Cpu6502) -> u32 {
        0
    }

    pub(super) fn branch_if_carry_clear(cpu: &mut Cpu6502) -> u32 {
        0
    }

    pub(super) fn branch_if_carry_set(cpu: &mut Cpu6502) -> u32 {
        0
    }

    pub(super) fn branch_if_equal(cpu: &mut Cpu6502) -> u32 {
        0
    }

    pub(super) fn bit_test(cpu: &mut Cpu6502) -> u32 {
        0
    }

    pub(super) fn branch_if_minus(cpu: &mut Cpu6502) -> u32 {
        0
    }

    pub(super) fn branch_if_not_equal(cpu: &mut Cpu6502) -> u32 {
        0
    }

    pub(super) fn branch_if_positive(cpu: &mut Cpu6502) -> u32 {
        0
    }

    pub(super) fn branch_if_overflow_clear(cpu: &mut Cpu6502) -> u32 {
        0
    }

    pub(super) fn branch_if_overflow_set(cpu: &mut Cpu6502) -> u32 {
        0
    }

    pub(super) fn clear_carry_flag(cpu: &mut Cpu6502) -> u32 {
        0
    }

    pub(super) fn clear_decimal_mode(cpu: &mut Cpu6502) -> u32 {
        0
    }

    pub(super) fn clear_interrupt_disable(cpu: &mut Cpu6502) -> u32 {
        0
    }

    pub(super) fn clear_overflow_flag(cpu: &mut Cpu6502) -> u32 {
        0
    }

    pub(super) fn compare_accumulator(cpu: &mut Cpu6502) -> u32 {
        0
    }

    pub(super) fn compare_x_reg(cpu: &mut Cpu6502) -> u32 {
        0
    }

    pub(super) fn compare_y_reg(cpu: &mut Cpu6502) -> u32 {
        0
    }

    pub(super) fn decrement_memory(cpu: &mut Cpu6502) -> u32 {
        0
    }

    pub(super) fn decrement_x_reg(cpu: &mut Cpu6502) -> u32 {
        0
    }

    pub(super) fn decrement_y_reg(cpu: &mut Cpu6502) -> u32 {
        0
    }

    pub(super) fn exclusive_or_accumulator(cpu: &mut Cpu6502) -> u32 {
        0
    }

    pub(super) fn force_interrupt(cpu: &mut Cpu6502) -> u32 {
        0
    }

    pub(super) fn increment_memory(cpu: &mut Cpu6502) -> u32 {
        0
    }

    pub(super) fn increment_x_reg(cpu: &mut Cpu6502) -> u32 {
        0
    }

    pub(super) fn increment_y_reg(cpu: &mut Cpu6502) -> u32 {
        0
    }

    pub(super) fn jump(cpu: &mut Cpu6502) -> u32 {
        0
    }

    pub(super) fn jump_to_subroutine(cpu: &mut Cpu6502) -> u32 {
        0
    }

    pub(super) fn logical_shift_right(cpu: &mut Cpu6502) -> u32 {
        0
    }

    pub(super) fn no_operation(cpu: &mut Cpu6502) -> u32 {
        0
    }

    pub(super) fn or_accumulator(cpu: &mut Cpu6502) -> u32 {
        0
    }

    pub(super) fn push_accumulator(cpu: &mut Cpu6502) -> u32 {
        0
    }

    pub(super) fn push_processor_status(cpu: &mut Cpu6502) -> u32 {
        0
    }

    pub(super) fn pull_accumulator(cpu: &mut Cpu6502) -> u32 {
        0
    }

    pub(super) fn pull_processor_status(cpu: &mut Cpu6502) -> u32 {
        0
    }

    pub(super) fn return_from_interrupt(cpu: &mut Cpu6502) -> u32 {
        0
    }

    pub(super) fn return_from_subroutine(cpu: &mut Cpu6502) -> u32 {
        0
    }

    pub(super) fn rotate_left(cpu: &mut Cpu6502) -> u32 {
        0
    }

    pub(super) fn rotate_right(cpu: &mut Cpu6502) -> u32 {
        0
    }

    pub(super) fn set_carry_flag(cpu: &mut Cpu6502) -> u32 {
        0
    }

    pub(super) fn set_decimal_mode(cpu: &mut Cpu6502) -> u32 {
        0
    }

    pub(super) fn set_interrupt_disable(cpu: &mut Cpu6502) -> u32 {
        0
    }

    pub(super) fn store_accumulator(cpu: &mut Cpu6502) -> u32 {
        0
    }

    pub(super) fn store_x_reg(cpu: &mut Cpu6502) -> u32 {
        0
    }

    pub(super) fn store_y_reg(cpu: &mut Cpu6502) -> u32 {
        0
    }

    pub(super) fn subtract_with_carry(cpu: &mut Cpu6502) -> u32 {
        0
    }

    pub(super) fn transfer_accumulator_to_x(cpu: &mut Cpu6502) -> u32 {
        0
    }

    pub(super) fn transfer_accumulator_to_y(cpu: &mut Cpu6502) -> u32 {
        0
    }

    pub(super) fn transfer_stack_pointer_to_x(cpu: &mut Cpu6502) -> u32 {
        0
    }

    pub(super) fn transfer_x_to_accumulator(cpu: &mut Cpu6502) -> u32 {
        0
    }

    pub(super) fn transfer_x_to_stack_pointer(cpu: &mut Cpu6502) -> u32 {
        0
    }

    pub(super) fn transfer_y_to_accumulator(cpu: &mut Cpu6502) -> u32 {
        0
    }
    
    pub(super) fn load_accumulator(cpu: &mut Cpu6502) -> u32 {
        cpu.accumulator = cpu.operand_data;

        cpu.set_flag(Flag::Z, cpu.accumulator == 0);
        cpu.set_flag(Flag::N, cpu.accumulator & 0b10000000 != 0);

        cpu.page_crossed as u32
    }

    pub(super) fn load_x_reg(cpu: &mut Cpu6502) -> u32 {
        cpu.x_index_reg = cpu.operand_data;

        cpu.set_flag(Flag::Z, cpu.x_index_reg == 0);
        cpu.set_flag(Flag::N, cpu.x_index_reg & 0b10000000 != 0);

        cpu.page_crossed as u32
    }

    pub(super) fn load_y_reg(cpu: &mut Cpu6502) -> u32 {
        cpu.y_index_reg = cpu.operand_data;

        cpu.set_flag(Flag::Z, cpu.y_index_reg == 0);
        cpu.set_flag(Flag::N, cpu.y_index_reg & 0b10000000 != 0);

        cpu.page_crossed as u32
    }

    pub(super) fn imp_addressing(cpu: &mut Cpu6502) { 
        cpu.operand_data = 0;
        cpu.page_crossed = false;
    }

    pub(super) fn acc_addressing(cpu: &mut Cpu6502) { 
        cpu.operand_data = cpu.accumulator;
        cpu.page_crossed = false; 
    }

    pub(super) fn imm_addressing(cpu: &mut Cpu6502) { 
        cpu.operand_data = cpu.advance_pc();
        cpu.page_crossed = false; 
    }

    pub(super) fn zpg_addressing(cpu: &mut Cpu6502) {
        cpu.operand_addr = cpu.advance_pc() as u16;
        cpu.operand_data = cpu.read_byte(cpu.operand_addr);
        cpu.page_crossed = false; 
    }

    pub(super) fn zpx_addressing(cpu: &mut Cpu6502) {
        cpu.operand_addr = cpu.advance_pc().wrapping_add(cpu.x_index_reg) as u16;
        cpu.operand_data = cpu.read_byte(cpu.operand_addr);
        cpu.page_crossed = false; 
    }

    pub(super) fn zpy_addressing(cpu: &mut Cpu6502) {
        cpu.operand_addr = cpu.advance_pc().wrapping_add(cpu.y_index_reg) as u16;
        cpu.operand_data = cpu.read_byte(cpu.operand_addr);
        cpu.page_crossed = false; 
    }

    pub(super) fn rel_addressing(cpu: &mut Cpu6502) {
        let offset =  (cpu.advance_pc() as i8) as i32;
        
        cpu.operand_addr = (cpu.program_counter as i32 + offset) as u16;
        cpu.operand_data = cpu.read_byte(cpu.operand_addr);
        cpu.page_crossed = (cpu.program_counter ^ cpu.operand_addr) & 0xFF00 != 0;
    }

    pub(super) fn abs_addressing(cpu: &mut Cpu6502) {
        cpu.operand_addr = Cpu6502::fetch_abs_address(cpu);
        cpu.operand_data = cpu.read_byte(cpu.operand_addr);
        cpu.page_crossed = false;
    }

    pub(super) fn abx_addressing(cpu: &mut Cpu6502) {
        let addr = Cpu6502::fetch_abs_address(cpu);

        cpu.operand_addr = addr.wrapping_add(cpu.x_index_reg as u16);
        cpu.operand_data = cpu.read_byte(cpu.operand_addr);
        cpu.page_crossed = ((cpu.operand_addr ^ addr) & 0xFF00) != 0;
    }

    pub(super) fn aby_addressing(cpu: &mut Cpu6502) {
        let addr = Cpu6502::fetch_abs_address(cpu);

        cpu.operand_addr = addr.wrapping_add(cpu.y_index_reg as u16);
        cpu.operand_data = cpu.read_byte(cpu.operand_addr);
        cpu.page_crossed = ((cpu.operand_addr ^ addr) & 0xFF00) != 0;
    }

    pub(super) fn ind_addressing(cpu: &mut Cpu6502) { 
        let ptr = Cpu6502::fetch_abs_address(cpu);

        let lo = cpu.read_byte(ptr) as u16;

        let hi = if ptr & 0xFF == 0xFF {
            cpu.read_byte(ptr & 0xFF00) // Simulate Hardware Bug
        } else { 
            cpu.read_byte(ptr.wrapping_add(1))
        } as u16;

        cpu.operand_addr = (hi << 8) | lo;
        cpu.operand_data = cpu.read_byte(cpu.operand_addr);
        cpu.page_crossed = false;
    }

    pub(super) fn inx_addressing(cpu: &mut Cpu6502) {
        let ptr = cpu.advance_pc().wrapping_add(cpu.x_index_reg);

        let lo = cpu.read_byte(ptr as u16) as u16;
        let hi = cpu.read_byte(ptr.wrapping_add(1) as u16) as u16;
        
        cpu.operand_addr = (hi << 8) | lo;
        cpu.operand_data = cpu.read_byte(cpu.operand_addr);
        cpu.page_crossed = false;
    }

    pub(super) fn iny_addressing(cpu: &mut Cpu6502) {
        let ptr = cpu.advance_pc();

        let lo = cpu.read_byte(ptr as u16) as u16;
        let hi = cpu.read_byte(ptr.wrapping_add(1) as u16) as u16;

        let addr = (hi << 8) | lo;

        cpu.operand_addr = addr.wrapping_add(cpu.y_index_reg as u16);
        cpu.operand_data = cpu.read_byte(cpu.operand_addr);
        cpu.page_crossed = ((cpu.operand_addr ^ addr) & 0xFF00) != 0;
    }

    fn fetch_abs_address(cpu: &mut Cpu6502) -> u16 {
        let lo = cpu.advance_pc() as u16;
        let hi = cpu.advance_pc() as u16;
        (hi << 8) | lo
    }

    fn push_word_to_stack(&mut self, word: u16) {
        self.push_byte_to_stack(word as u8);
        self.push_byte_to_stack(((word & 0xFF00) >> 8) as u8);
    }

    fn pop_word_from_stack(&mut self) -> u16 {
        let hi = self.pop_byte_from_stack() as u16;
        let lo = self.pop_byte_from_stack() as u16;
        (hi << 8) | lo
    }

    fn push_byte_to_stack(&mut self, byte: u8) {
        self.write_byte(STACK_START | self.stack_pointer as u16, byte);
        self.stack_pointer = self.stack_pointer.wrapping_sub(1);
    }

    fn pop_byte_from_stack(&mut self) -> u8 {
        self.stack_pointer = self.stack_pointer.wrapping_add(1);
        self.read_byte(STACK_START | self.stack_pointer as u16)
    }

    fn set_flag(&mut self, flag: Flag, val: bool) {
        let mask = flag.mask();
        if val {
            self.p_status |= mask;
        } else {
            self.p_status &= !mask;
        }
    }

    fn get_flag(&self, flag: Flag) -> bool {
        (self.p_status & flag.mask()) != 0
    }

    fn advance_pc(&mut self) -> u8 {
        let ret = self.read_byte(self.program_counter);
        self.program_counter += 1;
        ret
    }

    fn read_byte(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    fn write_byte(&mut self, addr: u16, byte: u8) {
        self.memory[addr as usize] = byte;
    }
}

#[cfg(test)]
mod tests {
    use super::{opcode::OPCODES_LOOKUP, Cpu6502};

    #[test]
    pub fn test_lda() {
        let mut cpu = Cpu6502::new();
        cpu.program_counter = 0x00;
        cpu.x_index_reg = 2;
        cpu.y_index_reg = 3;
        cpu.memory[0xFE] = 0x22;
        cpu.memory[0x1234] = 0x33;
        cpu.memory[0x1236] = 0x44;
        cpu.memory[0x1237] = 0x55;

        let data = vec![0xA9, 0x11, 0xA5, 0xFE, 0xB5, 0xFC, 0xAD, 0x34, 0x12, 0xBD, 0x34, 0x12, 0xB9, 0x34, 0x12];
        cpu.memory[..data.len()].copy_from_slice(&data[..data.len()]);

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
        let mut cpu = Cpu6502::new();

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
}