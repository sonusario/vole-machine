//#![allow(warnings)]

// Imports for sleeping
use std::io::Write;
use std::thread;
use std::time::{Duration, Instant};

// Aliases
type Address = u8;
type HeatLevel = u8;

// Sleep for 0.5 seconds
const USER_ITERATED: bool = true;
const SLEEP_DURATION: Duration = Duration::from_millis(500);
const MAX_HEAT: HeatLevel = 5;

fn main() {
    let mut cpu = Cpu::new();
    cpu.load_program(sample_program("B"));
    cpu.run();
}

struct Cpu {
    register: [u8; 16],
    heated_register: [HeatLevel; 16],
    memory: [u8; 256],
    heated_memory: [HeatLevel; 256],
    pc: Address, // Program Counter
    program_name: String,
    cycles: u128,
}

impl Cpu {
    fn new() -> Cpu {
        Cpu {
            register: [0; 16],
            heated_register: [0; 16],
            memory: [0; 256],
            heated_memory: [0; 256],
            pc: 0,
            program_name: String::new(),
            cycles: 0,
        }
    }

    fn run(&mut self) {
        let start_time = Instant::now();

        loop {
            self.print();
            self.cool_down();
            self.cycles += 1;
            
            let mut possible_jump_address: Option<Address> = None;
            match self.memory[self.pc as usize] >> 4 {
                // Match against the upper 4 bits of the current byte
                0x0 => self.no_op(),     // 0x0000 :: No Operation
                0x1 => self.load_from(), // 0x1[RXY] :: Load from m0xXY into rR
                0x2 => self.load(),      // 0x2[RXY] :: Load 0xXY into rR
                0x3 => self.store(),     // 0x3[RXY] :: Store from rR into m0xXY
                0x4 => self.move_op(),   // 0x40[RS] :: Move from rR to rS
                0x5 => self.add_tc(),    // 0x5[RST] :: rS + rT into rR (Two's Complement)
                0x6 => self.add_fl(),    // 0x6[RST] :: rS + rT into rR (Floating Point)
                0x7 => self.or(),        // 0x7[RST] :: rS | rT into R
                0x8 => self.and(),       // 0x8[RST] :: rS & rT into rR
                0x9 => self.xor(),       // 0x9[RST] :: rS ^ rT into rR
                0xA => self.rotate(),    // 0xA[R]0[X] :: rR >> 0xX // Rotate Right X bits
                0xB => possible_jump_address = self.jump(), // 0xB[RXY] :: if rR == r0 then PC = m0xXY
                0xC => break,            // 0xC000 :: Stop the CPU
                _ => panic!("Invalid OpCode"),
            }

            match possible_jump_address {
                Some(address) => self.pc = address,
                None => self.pc = self.pc.wrapping_add(2)
            }
        }

        println!("\nProgram {} completed in {:.2} seconds.",
            self.program_name,
            start_time.elapsed().as_secs_f32()
        );

        self.reset();
    }

    fn reset(&mut self) {
        self.register = [0; 16];
        self.memory = [0; 256];
        self.pc = 0;
        self.program_name = String::new();
    }

    fn no_op(&mut self) {
        // 0x0000 :: No Operation
        // Do nothing
    }

    // Load from memory into register
    fn load_from(&mut self) {
        // 0x1[RXY] :: Load from m0xXY into rR
        let r = (self.memory[self.pc as usize] & 0x0F) as usize; // The lower 4 bits of byte 1 are the register
        let addr = self.memory[self.pc as usize + 1]; // The next byte is the address to load from
        self.register[r] = self.memory[addr as usize]; // Load the value from memory into the register
        
        self.heated_register[r] = MAX_HEAT;
    }

    // Load a value into a register
    fn load(&mut self) {
        // 0x2[RXY] :: Load 0xXY into rR
        let r = (self.memory[self.pc as usize] & 0x0F) as usize; // The lower 4 bits of byte 1 are the register
        let value = self.memory[self.pc as usize + 1]; // The next byte is the value to load
        self.register[r] = value; // Load the value into the register

        self.heated_register[r] = MAX_HEAT;
    }

    // Store a value from a register into memory
    fn store(&mut self) {
        // 0x3[RXY] :: Store from rR into m0xXY
        let r = (self.memory[self.pc as usize] & 0x0F) as usize; // The lower 4 bits of byte 1 are the register
        let addr = self.memory[self.pc as usize + 1]; // The next byte is the address to store into
        self.memory[addr as usize] = self.register[r]; // Store the value from the register into memory
        
        self.heated_memory[addr as usize] = MAX_HEAT;
    }

    // Move a value from one register to another
    fn move_op(&mut self) {
        // 0x40[RS] :: Move from rR to rS
        let r1 = (self.memory[self.pc as usize + 1] >> 4) as usize; // The upper 4 bits of byte 2 are the first register
        let r2 = (self.memory[self.pc as usize + 1] & 0x0F) as usize; // The lower 4 bits of byte 2 are the second register
        self.register[r2] = self.register[r1]; // Move the value from r1 to r2
        
        self.heated_register[r1] = MAX_HEAT;
    }

    // Add two values together using two's complement
    fn add_tc(&mut self) {
        // 0x5[RST] :: rS + rT into rR (Two's Complement)
        let r1 = (self.memory[self.pc as usize] & 0x0F) as usize; // The lower 4 bits of byte 1 are the first register
        let r2 = (self.memory[self.pc as usize + 1] >> 4) as usize; // The upper 4 bits of byte 2 are the second register
        let r3 = (self.memory[self.pc as usize + 1] & 0x0F) as usize; // The lower 4 bits of byte 2 are the third register
        let sum = (self.register[r2] as i8).wrapping_add(self.register[r3] as i8); // Add the two values together
        self.register[r1] = sum as u8; // Store the result in the first register
        
        self.heated_register[r1] = MAX_HEAT;
    }

    // Add two values together using floating point
    fn add_fl(&mut self) {
        // 0x6[RST] :: rS + rT into rR Add (Floating Point)
        let r1 = (self.memory[self.pc as usize] & 0x0F) as usize; // The lower 4 bits of byte 1 are the first register
        let r2 = (self.memory[self.pc as usize + 1] >> 4) as usize; // The upper 4 bits of byte 2 are the second register
        let r3 = (self.memory[self.pc as usize + 1] & 0x0F) as usize; // The lower 4 bits of byte 2 are the third register
        let sum = self.register[r2] as f32 + self.register[r3] as f32; // Add the two values together
        self.register[r1] = sum as u8; // Store the result in the first register
        
        self.heated_register[r1] = MAX_HEAT;
    }

    // Bitwise OR two values together
    fn or(&mut self) {
        // 0x7[RST] :: rS | rT into R
        let r1 = (self.memory[self.pc as usize] & 0x0F) as usize; // The lower 4 bits of byte 1 are the first register
        let r2 = (self.memory[self.pc as usize + 1] >> 4) as usize; // The upper 4 bits of byte 2 are the second register
        let r3 = (self.memory[self.pc as usize + 1] & 0x0F) as usize; // The lower 4 bits of byte 2 are the third register
        self.register[r1] = self.register[r2] | self.register[r3]; // OR the two values together and store the result in the first register
        
        self.heated_register[r1] = MAX_HEAT;
    }

    // Bitwise AND two values together
    fn and(&mut self) {
        // 0x8[RST] :: rS & rT into rR
        let r1 = (self.memory[self.pc as usize] & 0x0F) as usize; // The lower 4 bits of byte 1 are the first register
        let r2 = (self.memory[self.pc as usize + 1] >> 4) as usize; // The upper 4 bits of byte 2 are the second register
        let r3 = (self.memory[self.pc as usize + 1] & 0x0F) as usize; // The lower 4 bits of byte 2 are the third register
        self.register[r1] = self.register[r2] & self.register[r3]; // AND the two values together and store the result in the first register
        
        self.heated_register[r1] = MAX_HEAT;
    }

    // Bitwise XOR two values together
    fn xor(&mut self) {
        // 0x9[RST] :: rS ^ rT into rR
        let r1 = (self.memory[self.pc as usize] & 0x0F) as usize; // The lower 4 bits of byte 1 are the first register
        let r2 = (self.memory[self.pc as usize + 1] >> 4) as usize; // The upper 4 bits of byte 2 are the second register
        let r3 = (self.memory[self.pc as usize + 1] & 0x0F) as usize; // The lower 4 bits of byte 2 are the third register
        self.register[r1] = self.register[r2] ^ self.register[r3]; // XOR the two values together and store the result in the first register
        
        self.heated_register[r1] = MAX_HEAT;
    }

    // Rotate a value right by a number of bits
    fn rotate(&mut self) {
        // 0xA[R]0[X] :: rR >> 0xX // Rotate Right X bits
        let r = (self.memory[self.pc as usize] & 0x0F) as usize; // The lower 4 bits of byte 1 are the register
        let bits = (self.memory[self.pc as usize + 1] & 0x0F) as u32; // The lower 4 bits of byte 2 is the number of bits to rotate by
        self.register[r] = self.register[r].rotate_right(bits); // Rotate the value in the register right by the number of bits
        
        self.heated_register[r] = MAX_HEAT;
    }

    // Jump to an address if a condition is met
    fn jump(&mut self) -> Option<Address> {
        // 0xB[RXY] :: if rR == r0 then PC = m0xXY
        let r = (self.memory[self.pc as usize] & 0x0F) as usize; // The lower 4 bits of byte 1 are the register
        let address = self.memory[self.pc as usize + 1]; // The next byte is the address to jump to
        if self.register[r] == self.register[0] {
            // Check if the value in the register is equal to the value in r0
            // Return the address to Jump to if the condition is met
            
            self.cool_down();
            return Some(address);
        }

        None
    }

    fn cool_down(&mut self) {
        for i in 0..16 {
            self.heated_register[i] = self.heated_register[i].saturating_sub(1);
        }

        for i in 0..256 {
            self.heated_memory[i] = self.heated_memory[i].saturating_sub(1);
        }
    }

    fn print_registers(&self) {
        println!();
        for i in 0..16 {
            print!(" r{i:02X}");
        }

        println!();
        for (i, &register) in self.register.iter().enumerate() {
            print!("  {}{register:02X}{}",
                Terminal::get_fg_color(Foreground::heat_from(self.heated_register[i])),
                Terminal::get_reset_all()
            );
        }

        println!();
    }

    fn print_memory(&self) {
        // Print the memory, 16 bytes per line with a '*' prefix indicating the program counter

        print!("\n{:<4}", " ");
        for i in 0..16 {
            print!(" m{i:02X}");
        }

        for (i, &byte) in self.memory.iter().enumerate() {
            let pc_marker = if i == self.pc as usize {
                format!(" *{}", //" 👉"
                    Terminal::get_fg_color(Foreground::Magenta),
                )
            } else {
                format!("  {}",
                    Terminal::get_fg_color(Foreground::heat_from(self.heated_memory[i]))
                )
            };
            let first_address = i - i % 16;

            if i % 16 == 0 {
                print!("\n m{first_address:02X}");
            }
            print!("{pc_marker}{byte:02X}{}",
                Terminal::get_reset_all()
            );
        }

        println!();
    }

    fn print(&self) {
        Terminal::clear();
        println!("\nProgram Cycles: {0:#02X}::{0}", self.cycles);
        println!("Program Counter: m{:#02X}", self.pc);
        self.print_registers();
        self.print_memory();
        if USER_ITERATED {
            print!("Press Enter to continue...");
            std::io::stdout().flush().unwrap();
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
        } else {
            thread::sleep(SLEEP_DURATION);
        }  
    }
    
    fn load_program(&mut self, program: Program) {
        match program {
            Program {name, code, start_address} if name != String::new() => {
                // Set the program name
                self.program_name = name;

                // Load the program into memory 
                for (i, &byte) in code.iter().enumerate() {
                    self.memory[start_address as usize + i] = byte;
                }

                // Set the address for the program counter to start at
                self.pc = start_address;
            },
            _ => unreachable!()
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy)]
enum Foreground {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
}

impl Foreground {
    fn heat_from(value: u8) -> Foreground {
        // Max heat is 5, min heat is 0
        // Excluding black because of visibility
        // Excluding magenta because its reserved for the program counter
        // 6 should be the warmest color, 0 should be the coldest (white)
        match value {
            0 => Foreground::White,
            1 => Foreground::Blue,
            2 => Foreground::Cyan,
            3 => Foreground::Green,
            4 => Foreground::Yellow,
            5 => Foreground::Red,
            _ => unreachable!("Invalid heat value")
        }
    }
}

struct Terminal;

#[allow(dead_code)]
impl Terminal { 
    fn clear() {
        print!("\x1Bc");
    }
    
    fn erase_line_cr() {
        print!("\x1B[1K\x1B[0G");
        std::io::stdout().flush().unwrap();
    }

    fn _set_fg_color(color: Foreground) {
        print!("\x1B[3{}m", color as u8);
    }

    fn get_fg_color(color: Foreground) -> String {
        format!("\x1B[3{}m", color as u8)
    }

    fn get_rgb_fg_color(r: u8, g: u8, b: u8) -> String {
        format!("\x1B[38;2;{r};{g};{b}m")
    }

    fn get_reset_all() -> String {
        String::from("\x1B[0m")
    }

    fn _reset_all() {
        print!("\x1B[0m");
    }
}

#[derive(Clone)]
struct Program {
    name: String,
    code: Vec<u8>,
    start_address: Address,
}

impl Program {
    fn new(name: String, code: Vec<u8>, start_address: Address) -> Program {
        Program {
            name,
            code,
            start_address,
        }
    }
}


fn sample_program(q: &str) -> Program {
    let program = [
        Program::new(
            String::from("A"),
            vec![
                /*           | m0x30, m0x31 | */ 0x20, 0x03, // Load 0x03 into r0
                /*           | m0x32, m0x33 | */ 0x21, 0x01, // Load 0x01 into r1
                /*           | m0x34, m0x35 | */ 0x22, 0x00, // Load 0x00 into r2
                /*           | m0x36, m0x37 | */ 0x23, 0x10, // Load 0x10 into r3 // 0x10 == 16
                /* 'B, 'C    | m0x38, m0x39 | */ 0x14, 0x00, // Load from m0x00 into r4
                /*   , 'D    | m0x3A, m0x3B | */ 0x34, 0x10, // Store from r4 into m0x10 // m0x10 == memory[16]
                /*           | m0x3C, m0x3D | */ 0x52, 0x21, // r2 + r1 into r2
                /*           | m0x3E, m0x3F | */ 0x53, 0x31, // r3 + r1 into r3
                /* UPDATE C  | m0x40, m0x41 | */ 0x32, 0x39, // Store from r2 into m0x39 // m0x39 == memory[57]
                /* UPDATE D  | m0x42, m0x43 | */ 0x33, 0x3B, // Store from r3 into m0x3B // m0x3B == memory[59]
                /* GOES TO A | m0x44, m0x45 | */ 0xB2, 0x48, // Jump to m0x48 if r2 == r0 // m0x48 == memory[72]
                /* GOES TO B | m0x46, m0x47 | */ 0xB0, 0x38, // Jump to m0x38 if r0 == r0 // m0x38 == memory[56]
                /* 'A        | m0x48, m0x49 | */ 0xC0, 0x00, // Halt
            ],
            0x30, // Load program at m0x30
        ),

        Program::new(
            String::from("B"),
            vec![
                /* m0x00, m0x01 */ 0x20, 0x04, // Load 0x04 into r0
                /* m0x02, m0x03 */ 0x21, 0x01, // Load 0x01 into r1
                /* m0x04, m0x05 */ 0x40, 0x12, // Move from r1 to r2
                /* m0x06, m0x07 */ 0x51, 0x12, // r1 + r2 into r1
                /* m0x08, m0x09 */ 0xB1, 0x0C, // Jump to m0x0C if r1 == r0
                /* m0x0A, m0x0B */ 0xB0, 0x06, // Jump to m0x06 if r0 == r0
                /* m0x0C, m0x0D */ 0xC0, 0x00, // Halt
            ],
            0x00, // Load program at m0x00
        )
    ];

    match q {
        "A" => program[0].clone(),
        "B" => program[1].clone(),
        _ => panic!("Invalid program name"),
    }
}