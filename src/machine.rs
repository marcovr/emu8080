use cpu::CPU;
use alu::ALU;
use operation::*;
use disassemble::*;
use iocontroller::*;

use std::io::*;
use std::vec::Vec;
use std::fs::File;
use std::time::SystemTime;
use piston_window::Key;

use DFIX;

const NANOS_PER_SEC: u64 = 1_000_000_000;
const CPU_HZ: u64 = 2_000_000;
const SCREEN_HZ: u64 = 60;
const NANOS_PER_CYCLE: u64 = NANOS_PER_SEC / CPU_HZ;
const INTERRUPT_CYCLES: i64 = (CPU_HZ / SCREEN_HZ / 2) as i64;

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
	
	int_type: u8,
	halt: bool,
	steps: u64,
	time: SystemTime,
	cycles_to_int: i64
}

impl Machine {
	pub fn new() -> Machine {
		let mut mem: Vec<u8> = Vec::new();
		mem.resize(0x10000, 0);
		if DFIX {
			// prevent "reboot"
			mem[0] = 0x76;
		}

		Machine {
			cpu: CPU::new(),
			alu: ALU::new(),
			mem,
			io: IOController::new(),

			int_type: 1,
			halt: false,
			steps: 0,
			time: SystemTime::now(),
			cycles_to_int: INTERRUPT_CYCLES
		}
	}

	fn emulate_op(&mut self, prnt: bool) -> u8 {
		let mut cpu = &mut self.cpu;
		let mut alu = &mut self.alu;
		let mut mem = &mut self.mem;
		let io = &mut self.io;
		let pc_addr = cpu.pc as usize;
		let opcode = mem[pc_addr];
		let arg1 = mem[pc_addr + 1];
		let arg2 = mem[pc_addr + 2];
		
		if prnt {
			print!("{} \t", self.steps);
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
			generate_interrupt(&mut self.cpu, &mut self.mem, self.int_type);
			self.int_type = (!self.int_type) & 3;
		}
	}

	pub fn dump_mem(&self, offset: usize, len: usize) {
		for i in 0..len {
			let n = offset + i;
			print_instr(self.mem[n], self.mem[n + 1], self.mem[n + 2], n as u16);
			println!("");
		}
	}

	pub fn framebuffer(&self) -> &[u8] {
		&self.mem[0x2400..0x4000]
	}

	pub fn load_rom(&mut self, filename: &str, offset: usize) {
		let buffer = read_file(&filename).expect(&format!("Can't read ROM file: {}", filename));

		let l = buffer.len() + offset;
		self.mem[offset..l].copy_from_slice(&buffer);
		self.cpu.pc = offset as u16;
		println!("=> {} bytes loaded to {:#03X}", buffer.len(), offset);
	}

	pub fn run(&mut self) {
		match self.time.elapsed() {
			Ok(elapsed) => {
				let nanos_elapsed = (elapsed.as_secs() * NANOS_PER_SEC) + (elapsed.subsec_nanos() as u64);
				let mut cycles_needed = (nanos_elapsed / NANOS_PER_CYCLE) as i64;

				while !self.halt && cycles_needed > 0 {
					let cycles = self.emulate_op(false) as i64;
					self.steps += 1;

					if cycles == 0 {
						self.halt = true;
						println!("\n=> CPU halted\n");
					}
					cycles_needed -= cycles;
					self.cycles_to_int -= cycles;

					if self.cycles_to_int <= 0 {
						self.interrupt();
						self.cycles_to_int += INTERRUPT_CYCLES;
					}
				}
			}
			Err(e) => {
				println!("Time error: {:?}", e);
			}
		}
		self.time = SystemTime::now();
	}

	pub fn key_pressed(&mut self, key: Key) {
		self.io.set_key(key, 1);
	}

	pub fn key_released(&mut self, key: Key) {
		self.io.set_key(key, 0);
	}
}
