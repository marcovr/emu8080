use std;
use cpu::CPU;
use alu::ALU;
use disassemble::print_instr;

use DIAG;
use DFIX;

macro_rules! append {
	($x:expr, $y:expr) => (((($x) as u16) << 8) | (($y) as u16))
}

macro_rules! splitl {
	($x:expr) => ((($x) >> 8) as u8)
}

macro_rules! splitr {
	($x:expr) => (($x) as u8)
}

macro_rules! push {
	($cpu:expr, $mem:expr, $h:expr, $l:expr) => ({
		let sp_addr = ($cpu).sp as usize;
		($mem)[sp_addr - 1] = $h;
		($mem)[sp_addr - 2] = $l;
		($cpu).sp = ($cpu).sp.wrapping_sub(2);
	})
}

macro_rules! pop {
	($cpu:expr, $mem:expr, $h:expr, $l:expr) => ({
		let sp_addr = ($cpu).sp as usize;
		$l = ($mem)[sp_addr];
		$h = ($mem)[sp_addr + 1];
		($cpu).sp = ($cpu).sp.wrapping_add(2);
	})
}

macro_rules! call {
	($cpu:expr, $mem:expr, $addr:expr) => ({
		let ret = ($cpu).pc;
		push!(($cpu), ($mem), splitl!(ret), splitr!(ret));
		($cpu).pc = $addr;
	})
}

macro_rules! callx {
	($cpu:expr, $mem:expr, $addr:expr) => ({
		let ret = ($cpu).pc.wrapping_add(2);
		push!(($cpu), ($mem), splitl!(ret), splitr!(ret));
		($cpu).pc = $addr;
	})
}

pub fn execute(cpu: &mut CPU, alu: &mut ALU, mut mem: &mut Vec<u8>, opcode: u8, arg1: u8, arg2: u8) -> u8 {
	let args = append!(arg2, arg1);
	cpu.pc = cpu.pc.wrapping_add(1);

	let inc = match opcode {
		0x00 => 0,		// NOP
		0x01 => {cpu.c = arg1; cpu.b = arg2; 2},
		0x02 => {mem[cpu.get_bc() as usize] = cpu.a; 0},
		0x03 => {let res = cpu.get_bc().wrapping_add(1); cpu.set_bc(res); 0},
		0x04 => {cpu.b = alu.add(cpu.b, 1); cpu.cc.set_zsp(cpu.b, &alu); 0},
		0x05 => {cpu.b = alu.sub(cpu.b, 1); cpu.cc.set_zsp(cpu.b, &alu); 0},
		0x06 => {cpu.b = arg1; 1},
		0x07 => {cpu.a = cpu.a.rotate_left(1); cpu.cc.cy = cpu.a & 1; 0},
		0x08 => 0,		// NOP
		0x09 => {
			let res = alu.addx(cpu.get_hl(), cpu.get_bc());
			cpu.set_hl(res);
			cpu.cc.cy = alu.cy as u8;
			0
		},
		0x0A => {cpu.a = mem[cpu.get_bc() as usize]; 0},
		0x0B => {let res = cpu.get_bc().wrapping_sub(1); cpu.set_bc(res); 0},
		0x0C => {cpu.c = alu.add(cpu.c, 1); cpu.cc.set_zsp(cpu.c, &alu); 0},
		0x0D => {cpu.c = alu.sub(cpu.c, 1); cpu.cc.set_zsp(cpu.c, &alu); 0},
		0x0E => {cpu.c = arg1; 1},
		0x0F => {cpu.a = cpu.a.rotate_right(1); cpu.cc.cy = (cpu.a & 0x80) >> 7; 0},
		0x10 => 0,		// NOP
		0x11 => {cpu.e = arg1; cpu.d = arg2; 2},
		0x12 => {mem[cpu.get_de() as usize] = cpu.a; 0},
		0x13 => {let res = cpu.get_de().wrapping_add(1); cpu.set_de(res); 0},
		0x14 => {cpu.d = alu.add(cpu.d, 1); cpu.cc.set_zsp(cpu.d, &alu); 0},
		0x15 => {cpu.d = alu.sub(cpu.d, 1); cpu.cc.set_zsp(cpu.d, &alu); 0},
		0x16 => {cpu.d = arg1; 1},
		0x17 => {
			let x = cpu.a;
			cpu.a = cpu.cc.cy | (x << 1);
			cpu.cc.cy = (x & 0x80) >> 7;
			0
		},
		0x18 => 0,		// NOP
		0x19 => {
			let res = alu.addx(cpu.get_hl(), cpu.get_de());
			cpu.set_hl(res);
			cpu.cc.cy = alu.cy as u8;
			0
		},
		0x1A => {cpu.a = mem[cpu.get_de() as usize]; 0},
		0x1B => {let res = cpu.get_de().wrapping_sub(1); cpu.set_de(res); 0},
		0x1C => {cpu.e = alu.add(cpu.e, 1); cpu.cc.set_zsp(cpu.e, &alu); 0},
		0x1D => {cpu.e = alu.sub(cpu.e, 1); cpu.cc.set_zsp(cpu.e, &alu); 0},
		0x1E => {cpu.e = arg1; 1},
		0x1F => {
			let x = cpu.a;
			cpu.a = (cpu.cc.cy << 7) | (x >> 1);
			cpu.cc.cy = x & 1;
			0
		},
		0x20 => 0,		// NOP
		0x21 => {cpu.l = arg1; cpu.h = arg2; 2},
		0x22 => {
			mem[args as usize] = cpu.l;
			mem[args as usize + 1] = cpu.h;
			2
		},
		0x23 => {let res = cpu.get_hl().wrapping_add(1); cpu.set_hl(res); 0},
		0x24 => {cpu.h = alu.add(cpu.h, 1); cpu.cc.set_zsp(cpu.h, &alu); 0},
		0x25 => {cpu.h = alu.sub(cpu.h, 1); cpu.cc.set_zsp(cpu.h, &alu); 0},
		0x26 => {cpu.h = arg1; 1},
		0x27 => {
			if (cpu.a & 0x0F) > 9 || cpu.cc.ac == 1 {
				cpu.a = alu.add(cpu.a, 6);
			}
			cpu.cc.ac = alu.ac as u8;
			if (cpu.a & 0xF0) > 0x90 || cpu.cc.cy == 1 {
				cpu.a = alu.add(cpu.a, 96);
			}
			cpu.cc.set_arith(cpu.a, &alu);
			0
		},
		0x28 => 0,		// NOP
		0x29 => {
			let res = alu.addx(cpu.get_hl(), cpu.get_hl());
			cpu.set_hl(res);
			cpu.cc.cy = alu.cy as u8;
			0
		},
		0x2A => {
			cpu.l = mem[args as usize];
			cpu.h = mem[args as usize + 1];
			2
		},
		0x2B => {let res = cpu.get_hl().wrapping_sub(1); cpu.set_hl(res); 0},
		0x2C => {cpu.l = alu.add(cpu.l, 1); cpu.cc.set_zsp(cpu.l, &alu); 0},
		0x2D => {cpu.l = alu.sub(cpu.l, 1); cpu.cc.set_zsp(cpu.l, &alu); 0},
		0x2E => {cpu.l = arg1; 1},
		0x2F => {cpu.a = !cpu.a; 0},
		0x30 => 0,		// NOP
		0x31 => {cpu.sp = args; 2},
		0x32 => {mem[args as usize] = cpu.a; 2},
		0x33 => {cpu.sp = cpu.sp.wrapping_add(1); 0},
		0x34 => {
			let val = alu.add(cpu.from_hl(&mut mem), 1);
			cpu.cc.set_zsp(val, &alu);
			cpu.to_hl(&mut mem, val);
			0
		},
		0x35 => {
			let val = alu.sub(cpu.from_hl(&mut mem), 1);
			cpu.cc.set_zsp(val, &alu);
			cpu.to_hl(&mut mem, val);
			0
		},
		0x36 => {cpu.to_hl(&mut mem, arg1); 1},
		0x37 => {cpu.cc.cy = 1; 0},
		0x38 => 0,		// NOP
		0x39 => {
			let res = alu.addx(cpu.get_hl(), cpu.sp);
			cpu.set_hl(res);
			cpu.cc.cy = alu.cy as u8;
			0
		},
		0x3A => {cpu.a = mem[args as usize]; 2},
		0x3B => {cpu.sp = cpu.sp.wrapping_sub(1); 0},
		0x3C => {cpu.a = alu.add(cpu.a, 1); cpu.cc.set_zsp(cpu.a, &alu); 0},
		0x3D => {cpu.a = alu.sub(cpu.a, 1); cpu.cc.set_zsp(cpu.a, &alu); 0},
		0x3E => {cpu.a = arg1; 1},
		0x3F => {cpu.cc.cy = 0; 0},
		0x40 => {cpu.b = cpu.b; 0},
		0x41 => {cpu.b = cpu.c; 0},
		0x42 => {cpu.b = cpu.d; 0},
		0x43 => {cpu.b = cpu.e; 0},
		0x44 => {cpu.b = cpu.h; 0},
		0x45 => {cpu.b = cpu.l; 0},
		0x46 => {cpu.b = cpu.from_hl(&mem); 0},
		0x47 => {cpu.b = cpu.a; 0},
		0x48 => {cpu.c = cpu.b; 0},
		0x49 => {cpu.c = cpu.c; 0},
		0x4A => {cpu.c = cpu.d; 0},
		0x4B => {cpu.c = cpu.e; 0},
		0x4C => {cpu.c = cpu.h; 0},
		0x4D => {cpu.c = cpu.l; 0},
		0x4E => {cpu.c = cpu.from_hl(&mem); 0},
		0x4F => {cpu.c = cpu.a; 0},
		0x50 => {cpu.d = cpu.b; 0},
		0x51 => {cpu.d = cpu.c; 0},
		0x52 => {cpu.d = cpu.d; 0},
		0x53 => {cpu.d = cpu.e; 0},
		0x54 => {cpu.d = cpu.h; 0},
		0x55 => {cpu.d = cpu.l; 0},
		0x56 => {cpu.d = cpu.from_hl(&mem); 0},
		0x57 => {cpu.d = cpu.a; 0},
		0x58 => {cpu.e = cpu.b; 0},
		0x59 => {cpu.e = cpu.c; 0},
		0x5A => {cpu.e = cpu.d; 0},
		0x5B => {cpu.e = cpu.e; 0},
		0x5C => {cpu.e = cpu.h; 0},
		0x5D => {cpu.e = cpu.l; 0},
		0x5E => {cpu.e = cpu.from_hl(&mem); 0},
		0x5F => {cpu.e = cpu.a; 0},
		0x60 => {cpu.h = cpu.b; 0},
		0x61 => {cpu.h = cpu.c; 0},
		0x62 => {cpu.h = cpu.d; 0},
		0x63 => {cpu.h = cpu.e; 0},
		0x64 => {cpu.h = cpu.h; 0},
		0x65 => {cpu.h = cpu.l; 0},
		0x66 => {cpu.h = cpu.from_hl(&mem); 0},
		0x67 => {cpu.h = cpu.a; 0},
		0x68 => {cpu.l = cpu.b; 0},
		0x69 => {cpu.l = cpu.c; 0},
		0x6A => {cpu.l = cpu.d; 0},
		0x6B => {cpu.l = cpu.e; 0},
		0x6C => {cpu.l = cpu.h; 0},
		0x6D => {cpu.l = cpu.l; 0},
		0x6E => {cpu.l = cpu.from_hl(&mem); 0},
		0x6F => {cpu.l = cpu.a; 0},
		0x70 => {cpu.to_hl(&mut mem, cpu.b); 0},
		0x71 => {cpu.to_hl(&mut mem, cpu.c); 0},
		0x72 => {cpu.to_hl(&mut mem, cpu.d); 0},
		0x73 => {cpu.to_hl(&mut mem, cpu.e); 0},
		0x74 => {cpu.to_hl(&mut mem, cpu.h); 0},
		0x75 => {cpu.to_hl(&mut mem, cpu.l); 0},
		0x76 => {return 0;}, // HLT
		0x77 => {cpu.to_hl(&mut mem, cpu.a); 0},
		0x78 => {cpu.a = cpu.b; 0},
		0x79 => {cpu.a = cpu.c; 0},
		0x7A => {cpu.a = cpu.d; 0},
		0x7B => {cpu.a = cpu.e; 0},
		0x7C => {cpu.a = cpu.h; 0},
		0x7D => {cpu.a = cpu.l; 0},
		0x7E => {cpu.a = cpu.from_hl(&mem); 0},
		0x7F => {cpu.a = cpu.a; 0},
		0x80 => {
			cpu.a = alu.add(cpu.a, cpu.b);
			cpu.cc.set_arith(cpu.a, &alu);
			0
		},
		0x81 => {
			cpu.a = alu.add(cpu.a, cpu.c);
			cpu.cc.set_arith(cpu.a, &alu);
			0
		},
		0x82 => {
			cpu.a = alu.add(cpu.a, cpu.d);
			cpu.cc.set_arith(cpu.a, &alu);
			0
		},
		0x83 => {
			cpu.a = alu.add(cpu.a, cpu.e);
			cpu.cc.set_arith(cpu.a, &alu);
			0
		},
		0x84 => {
			cpu.a = alu.add(cpu.a, cpu.h);
			cpu.cc.set_arith(cpu.a, &alu);
			0
		},
		0x85 => {
			cpu.a = alu.add(cpu.a, cpu.l);
			cpu.cc.set_arith(cpu.a, &alu);
			0
		},
		0x86 => {
			cpu.a = alu.add(cpu.a, cpu.from_hl(&mem));
			cpu.cc.set_arith(cpu.a, &alu);
			0
		},
		0x87 => {
			cpu.a = alu.add(cpu.a, cpu.a);
			cpu.cc.set_arith(cpu.a, &alu);
			0
		},
		0x88 => {
			cpu.a = alu.add3(cpu.a, cpu.b, cpu.cc.cy);
			cpu.cc.set_arith(cpu.a, &alu);
			0
		},
		0x89 => {
			cpu.a = alu.add3(cpu.a, cpu.c, cpu.cc.cy);
			cpu.cc.set_arith(cpu.a, &alu);
			0
		},
		0x8A => {
			cpu.a = alu.add3(cpu.a, cpu.d, cpu.cc.cy);
			cpu.cc.set_arith(cpu.a, &alu);
			0
		},
		0x8B => {
			cpu.a = alu.add3(cpu.a, cpu.e, cpu.cc.cy);
			cpu.cc.set_arith(cpu.a, &alu);
			0
		},
		0x8C => {
			cpu.a = alu.add3(cpu.a, cpu.h, cpu.cc.cy);
			cpu.cc.set_arith(cpu.a, &alu);
			0
		},
		0x8D => {
			cpu.a = alu.add3(cpu.a, cpu.l, cpu.cc.cy);
			cpu.cc.set_arith(cpu.a, &alu);
			0
		},
		0x8E => {
			cpu.a = alu.add3(cpu.a, cpu.from_hl(&mem), cpu.cc.cy);
			cpu.cc.set_arith(cpu.a, &alu);
			0
		},
		0x8F => {
			cpu.a = alu.add3(cpu.a, cpu.a, cpu.cc.cy);
			cpu.cc.set_arith(cpu.a, &alu);
			0
		},
		0x90 => {
			cpu.a = alu.sub(cpu.a, cpu.b);
			cpu.cc.set_arith(cpu.a, &alu);
			0
		},
		0x91 => {
			cpu.a = alu.sub(cpu.a, cpu.c);
			cpu.cc.set_arith(cpu.a, &alu);
			0
		},
		0x92 => {
			cpu.a = alu.sub(cpu.a, cpu.d);
			cpu.cc.set_arith(cpu.a, &alu);
			0
		},
		0x93 => {
			cpu.a = alu.sub(cpu.a, cpu.e);
			cpu.cc.set_arith(cpu.a, &alu);
			0
		},
		0x94 => {
			cpu.a = alu.sub(cpu.a, cpu.h);
			cpu.cc.set_arith(cpu.a, &alu);
			0
		},
		0x95 => {
			cpu.a = alu.sub(cpu.a, cpu.l);
			cpu.cc.set_arith(cpu.a, &alu);
			0
		},
		0x96 => {
			cpu.a = alu.sub(cpu.a, cpu.from_hl(&mem));
			cpu.cc.set_arith(cpu.a, &alu);
			0
		},
		0x97 => {
			cpu.a = alu.sub(cpu.a, cpu.a);
			cpu.cc.set_arith(cpu.a, &alu);
			0
		},
		0x98 => {
			cpu.a = alu.sub3(cpu.a, cpu.b, cpu.cc.cy);
			cpu.cc.set_arith(cpu.a, &alu);
			0
		},
		0x99 => {
			cpu.a = alu.sub3(cpu.a, cpu.c, cpu.cc.cy);
			cpu.cc.set_arith(cpu.a, &alu);
			0
		},
		0x9A => {
			cpu.a = alu.sub3(cpu.a, cpu.d, cpu.cc.cy);
			cpu.cc.set_arith(cpu.a, &alu);
			0
		},
		0x9B => {
			cpu.a = alu.sub3(cpu.a, cpu.e, cpu.cc.cy);
			cpu.cc.set_arith(cpu.a, &alu);
			0
		},
		0x9C => {
			cpu.a = alu.sub3(cpu.a, cpu.h, cpu.cc.cy);
			cpu.cc.set_arith(cpu.a, &alu);
			0
		},
		0x9D => {
			cpu.a = alu.sub3(cpu.a, cpu.l, cpu.cc.cy);
			cpu.cc.set_arith(cpu.a, &alu);
			0
		},
		0x9E => {
			cpu.a = alu.sub3(cpu.a, cpu.from_hl(&mem), cpu.cc.cy);
			cpu.cc.set_arith(cpu.a, &alu);
			0
		},
		0x9F => {
			cpu.a = alu.sub3(cpu.a, cpu.a, cpu.cc.cy);
			cpu.cc.set_arith(cpu.a, &alu);
			0
		},
		0xA0 => {cpu.a &= cpu.b; cpu.cc.set_logic(cpu.a); 0},
		0xA1 => {cpu.a &= cpu.c; cpu.cc.set_logic(cpu.a); 0},
		0xA2 => {cpu.a &= cpu.d; cpu.cc.set_logic(cpu.a); 0},
		0xA3 => {cpu.a &= cpu.e; cpu.cc.set_logic(cpu.a); 0},
		0xA4 => {cpu.a &= cpu.h; cpu.cc.set_logic(cpu.a); 0},
		0xA5 => {cpu.a &= cpu.l; cpu.cc.set_logic(cpu.a); 0},
		0xA6 => {cpu.a &= cpu.from_hl(&mem); cpu.cc.set_logic(cpu.a); 0},
		0xA7 => {cpu.a &= cpu.a; cpu.cc.set_logic(cpu.a); 0},
		0xA8 => {cpu.a ^= cpu.b; cpu.cc.set_logic(cpu.a); 0},
		0xA9 => {cpu.a ^= cpu.c; cpu.cc.set_logic(cpu.a); 0},
		0xAA => {cpu.a ^= cpu.d; cpu.cc.set_logic(cpu.a); 0},
		0xAB => {cpu.a ^= cpu.e; cpu.cc.set_logic(cpu.a); 0},
		0xAC => {cpu.a ^= cpu.h; cpu.cc.set_logic(cpu.a); 0},
		0xAD => {cpu.a ^= cpu.l; cpu.cc.set_logic(cpu.a); 0},
		0xAE => {cpu.a ^= cpu.from_hl(&mem); cpu.cc.set_logic(cpu.a); 0},
		0xAF => {cpu.a ^= cpu.a; cpu.cc.set_logic(cpu.a); 0},
		0xB0 => {cpu.a |= cpu.b; cpu.cc.set_logic(cpu.a); 0},
		0xB1 => {cpu.a |= cpu.c; cpu.cc.set_logic(cpu.a); 0},
		0xB2 => {cpu.a |= cpu.d; cpu.cc.set_logic(cpu.a); 0},
		0xB3 => {cpu.a |= cpu.e; cpu.cc.set_logic(cpu.a); 0},
		0xB4 => {cpu.a |= cpu.h; cpu.cc.set_logic(cpu.a); 0},
		0xB5 => {cpu.a |= cpu.l; cpu.cc.set_logic(cpu.a); 0},
		0xB6 => {cpu.a |= cpu.from_hl(&mem); cpu.cc.set_logic(cpu.a); 0},
		0xB7 => {cpu.a |= cpu.a; cpu.cc.set_logic(cpu.a); 0},
		0xB8 => {
			let res = alu.sub(cpu.a, cpu.b);
			cpu.cc.set_arith(res, &alu);
			0
		},
		0xB9 => {
			let res = alu.sub(cpu.a, cpu.c);
			cpu.cc.set_arith(res, &alu);
			0
		},
		0xBA => {
			let res = alu.sub(cpu.a, cpu.d);
			cpu.cc.set_arith(res, &alu);
			0
		},
		0xBB => {
			let res = alu.sub(cpu.a, cpu.e);
			cpu.cc.set_arith(res, &alu);
			0
		},
		0xBC => {
			let res = alu.sub(cpu.a, cpu.h);
			cpu.cc.set_arith(res, &alu);
			0
		},
		0xBD => {
			let res = alu.sub(cpu.a, cpu.l);
			cpu.cc.set_arith(res, &alu);
			0
		},
		0xBE => {
			let res = alu.sub(cpu.a, cpu.from_hl(&mem));
			cpu.cc.set_arith(res, &alu);
			0
		},
		0xBF => {
			let res = alu.sub(cpu.a, cpu.a);
			cpu.cc.set_arith(res, &alu);
			0
		},
		0xC0 => {
			if cpu.cc.z == 0 {
				let sp_addr = cpu.sp as usize;
				cpu.pc = append!(mem[sp_addr + 1], mem[sp_addr]);
				cpu.sp = cpu.sp.wrapping_add(2);
			}
			0
		},
		0xC1 => {pop!(cpu, mem, cpu.b, cpu.c); 0},
		0xC2 => {
			if cpu.cc.z == 0 {
				cpu.pc = args;
				0
			}
			else {2}
		},
		0xC3 => {cpu.pc = args; 0},
		0xC4 => {
			if cpu.cc.z == 0 {callx!(cpu, mem, args); 0}
			else {2}
		},
		0xC5 => {push!(cpu, mem, cpu.b, cpu.c); 0},
		0xC6 => {
			cpu.a = alu.add(cpu.a, arg1);
			cpu.cc.set_arith(cpu.a, &alu);
			1
		},
		0xC7 => {call!(cpu, mem, 0); 0},
		0xC8 => {
			if cpu.cc.z == 1 {
				let sp_addr = cpu.sp as usize;
				cpu.pc = append!(mem[sp_addr + 1], mem[sp_addr]);
				cpu.sp = cpu.sp.wrapping_add(2);
			}
			0
		},
		0xC9 => {
			let sp_addr = cpu.sp as usize;
			cpu.pc = append!(mem[sp_addr + 1], mem[sp_addr]);
			cpu.sp = cpu.sp.wrapping_add(2);
			0
		},
		0xCA => {
			if cpu.cc.z == 1 {cpu.pc = args; 0}
			else {2}
		},

		0xCC => {
			if cpu.cc.z == 1 {callx!(cpu, mem, args); 0}
			else {2}
		},
		0xCD => {
			if DIAG && args == 5 {
				// BDOS syscall -> print
				if cpu.c == 9 {
					let mut addr = cpu.get_de() as usize;
					if DFIX {
						addr += 1;
					}
					loop {
						let d = mem[addr] as u32;
						let c = std::char::from_u32(d).expect("invalid char");
						if c == '$' {
							break;
						}
						print!("{}", c);
						addr += 1;
					}
				}
				else if cpu.c == 2 {print!("{:04X}", cpu.get_hl());}
				2
			}
			else {
				callx!(cpu, mem, args);
				0
			}
		},
		0xCE => {
			cpu.a = alu.add3(cpu.a, arg1, cpu.cc.cy);
			cpu.cc.set_arith(cpu.a, &alu);
			1
		},
		0xCF => {call!(cpu, mem, 8); 0},
		0xD0 => {
			if cpu.cc.cy == 0 {
				let sp_addr = cpu.sp as usize;
				cpu.pc = append!(mem[sp_addr + 1], mem[sp_addr]);
				cpu.sp = cpu.sp.wrapping_add(2);
			}
			0
		},
		0xD1 => {pop!(cpu, mem, cpu.d, cpu.e); 0},
		0xD2 => {
			if cpu.cc.cy == 0 {cpu.pc = args; 0}
			else {2}
		},
		0xD3 => 1, //{println!("=> OUT: {:#04X}", arg1); 1},
		0xD4 => {
			if cpu.cc.cy == 0 {callx!(cpu, mem, args); 0}
			else {2}
		},
		0xD5 => {push!(cpu, mem, cpu.d, cpu.e); 0},
		0xD6 => {
			cpu.a = alu.sub(cpu.a, arg1);
			cpu.cc.set_arith(cpu.a, &alu);
			1
		},
		0xD7 => {call!(cpu, mem, 0x10); 0},
		0xD8 => {
			if cpu.cc.cy == 1 {
				let sp_addr = cpu.sp as usize;
				cpu.pc = append!(mem[sp_addr + 1], mem[sp_addr]);
				cpu.sp = cpu.sp.wrapping_add(2);
			}
			0
		},

		0xDA => {
			if cpu.cc.cy == 1 {
				cpu.pc = args;
				0
			}
			else {2}
		},
		0xDB => 1, //{println!("=> IN: {:#04X}", arg1); 1},
		0xDC => {
			if cpu.cc.cy == 1 {callx!(cpu, mem, args); 0}
			else {2}
		},

		0xDE => {
			cpu.a = alu.sub3(cpu.a, arg1, cpu.cc.cy);
			cpu.cc.set_arith(cpu.a, &alu);
			1
		},
		0xDF => {call!(cpu, mem, 0x18); 0},
		0xE0 => {
			if cpu.cc.p == 0 {
				let sp_addr = cpu.sp as usize;
				cpu.pc = append!(mem[sp_addr + 1], mem[sp_addr]);
				cpu.sp = cpu.sp.wrapping_add(2);
			}
			0
		},
		0xE1 => {pop!(cpu, mem, cpu.h, cpu.l); 0},
		0xE2 => {
			if cpu.cc.p == 0 {cpu.pc = args; 0}
			else {2}
		},
		0xE3 => {
			let sp_addr = cpu.sp as usize;
			std::mem::swap(&mut cpu.l, &mut mem[sp_addr]);
			std::mem::swap(&mut cpu.h, &mut mem[sp_addr + 1]);
			0
		},
		0xE4 => {
			if cpu.cc.p == 0 {callx!(cpu, mem, args); 0}
			else {2}
		},
		0xE5 => {push!(cpu, mem, cpu.h, cpu.l); 0},
		0xE6 => {cpu.a &= arg1; cpu.cc.set_logic(cpu.a); 1},
		0xE7 => {call!(cpu, mem, 0x20); 0},
		0xE8 => {
			if cpu.cc.p == 1 {
				let sp_addr = cpu.sp as usize;
				cpu.pc = append!(mem[sp_addr + 1], mem[sp_addr]);
				cpu.sp = cpu.sp.wrapping_add(2);
			}
			0
		},
		0xE9 => {cpu.pc = cpu.get_hl(); 0},
		0xEA => {
			if cpu.cc.p == 1 {cpu.pc = args; 0}
			else {2}
		},
		0xEB => {
			std::mem::swap(&mut cpu.d, &mut cpu.h);
			std::mem::swap(&mut cpu.e, &mut cpu.l);
			0
		},
		0xEC => {
			if cpu.cc.p == 1 {callx!(cpu, mem, args); 0}
			else {2}
		},

		0xEE => {
			cpu.a ^= arg1;
			cpu.cc.set_zsp(cpu.a, &alu);
			cpu.cc.cy = 0;
			1
		},
		0xEF => {call!(cpu, mem, 0x28); 0},
		0xF0 => {
			if cpu.cc.s == 0 {
				let sp_addr = cpu.sp as usize;
				cpu.pc = append!(mem[sp_addr + 1], mem[sp_addr]);
				cpu.sp = cpu.sp.wrapping_add(2);
			}
			0
		},
		0xF1 => {
			let sp_addr = cpu.sp as usize;
			cpu.cc.set_psw(mem[sp_addr]);
			cpu.a = mem[sp_addr + 1];
			cpu.sp = cpu.sp.wrapping_add(2);
			0
		},
		0xF2 => {
			if cpu.cc.s == 0 {
				cpu.pc = args;
				0
			}
			else {2}
		},
		0xF3 => {cpu.int_enable = 0; 0},
		0xF4 => {
			if cpu.cc.s == 0 {callx!(cpu, mem, args); 0}
			else {2}
		},
		0xF5 => {push!(cpu, mem, cpu.a, cpu.cc.get_psw()); 0},
		0xF6 => {
			cpu.a |= arg1;
			cpu.cc.set_zsp(cpu.a, &alu);
			cpu.cc.cy = 0;
			1
		},
		0xF7 => {call!(cpu, mem, 0x30); 0},
		0xF8 => {
			if cpu.cc.s == 1 {
				let sp_addr = cpu.sp as usize;
				cpu.pc = append!(mem[sp_addr + 1], mem[sp_addr]);
				cpu.sp = cpu.sp.wrapping_add(2);
			}
			0
		},
		0xF9 => {cpu.sp = cpu.get_hl(); 0},
		0xFA => {
			if cpu.cc.s != 0 {
				cpu.pc = args;
				0
			}
			else {2}
		},
		0xFB => {cpu.int_enable = 1; 0},
		0xFC => {
			if cpu.cc.s == 1 {callx!(cpu, mem, args); 0}
			else {2}
		},

		0xFE => {
			let x = alu.sub(cpu.a, arg1);
			cpu.cc.set_arith(x, &alu);
			1
		},
		0xFF => {call!(cpu, mem, 0x38); 0},

		_ => {
			println!("\n==== LAST STATE ====");
			print_instr(opcode, arg1, arg2, cpu.pc);
			print!(" \t");
			cpu.print_state();
			println!("\nopcode not implemented: {:02X}", opcode);
			return 0;
		}
	};

	cpu.pc = cpu.pc.wrapping_add(inc);
	CYCLES8080[opcode as usize]
}

pub fn generate_interrupt(cpu: &mut CPU, mem: &mut Vec<u8>, interrupt_num: u8) {
	//This is identical to an "RST interrupt_num" instruction.    
	call!(cpu, mem, 8 * interrupt_num as u16);
	cpu.int_enable = 0;
}

static CYCLES8080: [u8; 0x100] = [
	4, 10, 7, 5, 5, 5, 7, 4, 4, 10, 7, 5, 5, 5, 7, 4, //0x00..0x0f
	4, 10, 7, 5, 5, 5, 7, 4, 4, 10, 7, 5, 5, 5, 7, 4, //0x10..0x1f
	4, 10, 16, 5, 5, 5, 7, 4, 4, 10, 16, 5, 5, 5, 7, 4, //etc
	4, 10, 13, 5, 10, 10, 10, 4, 4, 10, 13, 5, 5, 5, 7, 4,
	
	5, 5, 5, 5, 5, 5, 7, 5, 5, 5, 5, 5, 5, 5, 7, 5, //0x40..0x4f
	5, 5, 5, 5, 5, 5, 7, 5, 5, 5, 5, 5, 5, 5, 7, 5,
	5, 5, 5, 5, 5, 5, 7, 5, 5, 5, 5, 5, 5, 5, 7, 5,
	7, 7, 7, 7, 7, 7, 7, 7, 5, 5, 5, 5, 5, 5, 7, 5,
	
	4, 4, 4, 4, 4, 4, 7, 4, 4, 4, 4, 4, 4, 4, 7, 4, //0x80..8x4f
	4, 4, 4, 4, 4, 4, 7, 4, 4, 4, 4, 4, 4, 4, 7, 4,
	4, 4, 4, 4, 4, 4, 7, 4, 4, 4, 4, 4, 4, 4, 7, 4,
	4, 4, 4, 4, 4, 4, 7, 4, 4, 4, 4, 4, 4, 4, 7, 4,
	
	11, 10, 10, 10, 17, 11, 7, 11, 11, 10, 10, 10, 10, 17, 7, 11, //0xc0..0xcf
	11, 10, 10, 10, 17, 11, 7, 11, 11, 10, 10, 10, 10, 17, 7, 11, 
	11, 10, 10, 18, 17, 11, 7, 11, 11, 5, 10, 5, 17, 17, 7, 11, 
	11, 10, 10, 4, 17, 11, 7, 11, 11, 5, 10, 4, 17, 17, 7, 11
];
