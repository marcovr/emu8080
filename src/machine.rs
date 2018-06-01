#![allow(dead_code)]

mod cpu;
mod alu;
mod operation;
mod disassemble;
mod iocontroller;

//use std;
use cpu::CPU;
use alu::ALU;
use operation::*;
use disassemble::*;
use iocontroller::*;

use std::io::*;
use std::vec::Vec;
use std::fs::File;
use std::time::{SystemTime, Duration};
use std::thread::sleep;

fn read_file(filename: &str) -> Result<Vec<u8>> {
    let mut file = File::open(&filename)?;
    let mut buffer = vec![];
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}

#[derive(Debug)]
pub struct Machine {
    cpu: CPU,
    alu: ALU,
    mem: Vec<u8>,
    io: IOController,
    int_type: u8
}

impl Machine {
    fn new() -> Machine {
        let mut mem: Vec<u8> = Vec::new();
        mem.resize(0x10000, 0);

        Machine {
            cpu: CPU::new(),
            alu: ALU::new(),
            mem,
            io: IOController::new(),
            int_type: 1
        }
    }

    fn emulate_op(&mut self, prnt: bool, steps: u64) -> u8 {
        let mut cpu = &mut self.cpu;
        let mut alu = &mut self.alu;
        let mut mem = &mut self.mem;
        let io = &mut self.io;
        let pc_addr = cpu.pc as usize;
        let opcode = mem[pc_addr];
        let arg1 = mem[pc_addr + 1];
        let arg2 = mem[pc_addr + 2];
        
        if prnt {
            print!("{} \t", steps);
            print_instr(opcode, arg1, arg2, cpu.pc);
        }

        let cycles = match opcode {
            0xD3 => {io.write(arg1, cpu.a); cpu.pc = cpu.pc.wrapping_add(2); 3}, // OUT
            0xDB => {cpu.a = io.read(arg1); cpu.pc = cpu.pc.wrapping_add(2); 3}, // IN
            _ => execute(&mut cpu, &mut alu, &mut mem, opcode, arg1, arg2)
        };

        if prnt {
            print!(" \t|\t");
            cpu.print_state();
        }

        cycles
    }

    fn interrupt(&mut self) {
        if self.cpu.int_enable == 1 {
            //println!("RST {}", self.int_type);
            generate_interrupt(&mut self.cpu, &mut self.mem, self.int_type);
            self.int_type = (!self.int_type) & 3;
        }
    }

    fn dump_mem(&self, offset: usize) {
        for i in 0..100 {
            let n = offset + i;
            print_instr(self.mem[n], self.mem[n + 1], self.mem[n + 2], n as u16);
            println!("");
        }
    }

    pub fn framebuffer(&self) -> &[u8] {
        &self.mem[0x2400..0x4000]
    }

    pub fn load_rom(&mut self, filename: &str, offset: usize) {
        let buffer = read_file(&filename).expect("Can't read file");

        let l = buffer.len() + offset;
        self.mem[offset..l].copy_from_slice(&buffer);
        self.cpu.pc = offset as u16;
        println!("=> {} bytes loaded to {:#03X}", buffer.len(), offset);
    }
}

const DIAG: bool = false;
const DFIX: bool = false;
const FULL_SPEED: bool = true;

const NANOS_PER_SEC: u64 = 1_000_000_000;
const CPU_HZ: u64 = 2_000_000;
const SCREEN_HZ: u64 = 60;
const NANOS_PER_CYCLE: u64 = NANOS_PER_SEC / CPU_HZ;
const INTERRUPT_CYCLES: i64 = (CPU_HZ / SCREEN_HZ / 2) as i64;

pub fn main() -> Result<()> {
    let mut m = Machine::new();

    //m.load_rom("../rom/cpudiag.bin", 0x100);
    //m.load_rom("../rom/test.bin", 0x100);
    m.load_rom("../rom/invaders.bin", 0);

    if DFIX {
        // prevent "reboot"
        m.mem[0] = 0x76;
    }

    let mut halt = false;
    let mut steps :u64 = 0;
    let mut prnt = false;
    let mut time = SystemTime::now();
    let mut cycles_to_int = INTERRUPT_CYCLES;

    let limit = 10000000;

    while !halt && steps <= limit {
        sleep(Duration::from_millis(1));
        match time.elapsed() {
            Ok(elapsed) => {
                let nanos_elapsed = (elapsed.as_secs() * NANOS_PER_SEC) + (elapsed.subsec_nanos() as u64);
                let mut cycles_needed = (nanos_elapsed / NANOS_PER_CYCLE) as i64;

                while !halt && cycles_needed > 0 && steps <= limit {
                    //prnt = steps >= 50000;

                    let cycles = m.emulate_op(prnt, steps) as i64;
                    steps += 1;

                    halt = cycles == 0; // Halt if no cycles used
                    cycles_needed -= cycles;
                    cycles_to_int -= cycles;

                    if cycles_to_int <= 0 {
                        m.interrupt();
                        cycles_to_int += INTERRUPT_CYCLES;
                        //prnt = steps >= 50000;
                    }

                    /*if m.cpu.pc == 0x87 {
                        println!("RET");
                        prnt = false;
                    }*/

                    if FULL_SPEED {
                        cycles_needed = 1000;
                    }
                }
            }
            Err(e) => {
                println!("Time error: {:?}", e);
            }
        }
        time = SystemTime::now();
    }

    println!("\n=> CPU halted\n");

    Ok(())
}
