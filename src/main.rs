#![allow(while_true)]
#![allow(dead_code)]
//#![allow(unused_mut)]

mod cpu;
mod alu;
mod operation;
mod disassemble;
mod iocontroller;

use cpu::CPU;
use alu::ALU;
use operation::*;
use disassemble::*;
use iocontroller::*;

use std::io::*;
use std::vec::Vec;
use std::fs::File;

fn read_file(filename: &str) -> Result<Vec<u8>> {
	let mut file = File::open(&filename)?;
	let mut buffer = vec![];
	file.read_to_end(&mut buffer)?;
	Ok(buffer)
}

#[derive(Debug)]
struct Machine {
	cpu: CPU,
	alu: ALU,
	mem: Vec<u8>,
	io: IOController
}

impl Machine {
	fn new() -> Machine {
		let mut mem: Vec<u8> = Vec::new();
		mem.resize(0x10000, 0);

		Machine {
			cpu: CPU::new(),
			alu: ALU::new(),
			mem,
			io: IOController::new()
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

		/*if !DIAG && steps == 42039 {
			assert_eq!(opcode, 0xFB);
		}*/

		let cycles = match opcode {
			0xD3 => {io.write(arg1, cpu.a); cpu.pc = cpu.pc.wrapping_add(1); 3}, // OUT
			0xDB => {cpu.a = io.read(arg1); cpu.pc = cpu.pc.wrapping_add(1); 3}, // IN
			_ => execute(&mut cpu, &mut alu, &mut mem, opcode, arg1, arg2)
		};

		if prnt {
			print!(" \t|\t");
			cpu.print_state();
		}

		cycles
	}

	fn interrupt(&mut self, arg: u8) {
		if self.cpu.int_enable == 1 {
			generate_interrupt(&mut self.cpu, &mut self.mem, arg);
		}
	}

	fn load_rom(&mut self, filename: &str, offset: usize) {
		let buffer = read_file(&filename).expect("Can't read file");

		let l = buffer.len() + offset;
		self.mem[offset..l].copy_from_slice(&buffer);
		self.cpu.pc = offset as u16;
		println!("=> {} bytes loaded to {:#03X}", buffer.len(), offset);
	}
}

const DIAG: bool = false;
const DFIX: bool = false;

fn main() -> Result<()> {
	let mut m = Machine::new();

	//m.load_rom("../rom/cpudiag.bin", 0x100);
	//m.load_rom("../rom/test.bin", 0x100);
	m.load_rom("../rom/invaders.bin", 0);

	if DFIX {
		// prevent "reboot"
		m.mem[0] = 0x76;
	}

	let mut halt = false;
	let mut n :u64 = 0;
	let mut prnt;
	let mut time = std::time::SystemTime::now();
	let mut int_type = 1;

	while !halt && n <= 60005 {
		prnt = n >= 59999;
		let cycles = m.emulate_op(prnt, n);
		halt = cycles == 0; // Halt if no cycles used
		n += 1;
		//std::thread::sleep(std::time::Duration::from_millis(1));

		match time.elapsed() {
			Ok(elapsed) => {
				if elapsed > std::time::Duration::from_millis(1) { //1000/60) { //from_micros(1000000/60) {
					println!("ir {}", int_type);
					m.interrupt(int_type);
					int_type = (!int_type) & 3;
					time = std::time::SystemTime::now();
				}
			}
			Err(e) => {
				println!("Time error: {:?}", e);
			}
		}
	}

	println!("\n=> CPU halted\n");

	Ok(())
}
