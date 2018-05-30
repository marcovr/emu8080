use alu::ALU;

macro_rules! append {
	($x:expr, $y:expr) => (((($x) as u16) << 8) | (($y) as u16))
}

macro_rules! splitl {
	($x:expr) => ((($x) >> 8) as u8)
}

macro_rules! splitr {
	($x:expr) => (($x) as u8)
}

#[derive(Debug)]
pub struct CPU {
	pub a: u8,
	pub b: u8,
	pub c: u8,
	pub d: u8,
	pub e: u8,
	pub h: u8,
	pub l: u8,

	pub sp: u16,
	pub pc: u16,

	pub cc: ConditionCodes,
	pub int_enable: u8
}

impl CPU {
	pub fn new() -> CPU {
		CPU {a: 0, b: 0, c: 0, d: 0, e:0, h: 0, l: 0, sp: 0, pc: 0,
			cc: ConditionCodes::new(), int_enable: 0}
	}

	pub fn print_state(&self) {
		print!("{:02X}{:02X} {:02X}{:02X} {:02X}{:02X} {:02X}{:02X} {:04X} {:04X}", 
			self.a, self.cc.get_psw(), 
			self.b, self.c, 
			self.d, self.e, 
			self.h, self.l,
			self.pc,
			self.sp);
		println!("\t{}{}{}{}{}", if self.cc.z == 1 {'z'} else {'.'},
			if self.cc.s == 1 {'s'} else {'.'},
			if self.cc.p == 1 {'p'} else {'.'},
			if self.cc.cy == 1 {'c'} else {'.'},
			if self.cc.ac == 1 {'a'} else {'.'});
	}

	pub fn get_hl(&self) -> u16 {
		append!(self.h, self.l)
	}

	pub fn set_hl(&mut self, val: u16) {
		self.h = splitl!(val);
		self.l = splitr!(val);
	}

	pub fn get_bc(&self) -> u16 {
		append!(self.b, self.c)
	}

	pub fn set_bc(&mut self, val: u16) {
		self.b = splitl!(val);
		self.c = splitr!(val);
	}

	pub fn get_de(&self) -> u16 {
		append!(self.d, self.e)
	}

	pub fn set_de(&mut self, val: u16) {
		self.d = splitl!(val);
		self.e = splitr!(val);
	}

	pub fn from_hl(&self, mem: &Vec<u8>) -> u8 {
		mem[self.get_hl() as usize]
	}

	pub fn to_hl(&self, mem: &mut Vec<u8>, val: u8) {
		mem[self.get_hl() as usize] = val;
	}
}

fn parity(mut byte: u8) -> u8 {
	let mut result = 1;

	while byte != 0 {
		result ^= 1;
		byte &= byte - 1;
	}

	result
}

#[derive(Debug)]
pub struct ConditionCodes {
	pub z: u8,
	pub s: u8,
	pub p: u8,
	pub cy: u8,
	pub ac: u8
}

impl ConditionCodes {
	pub fn new() -> ConditionCodes {
		ConditionCodes {z: 0, s: 0, p: 0, cy: 0, ac: 0}
	}

	pub fn get_psw(&self) -> u8 {
		self.cy | self.p << 2 | self.ac << 4 | self.z << 6 | self.s << 7 | 2
	}

	pub fn set_psw(&mut self, psw: u8) {
		self.cy = psw & 1;
		self.p = (psw >> 2) & 1;
		self.ac = (psw >> 4) & 1;
		self.z = (psw >> 6) & 1;
		self.s = (psw >> 7) & 1;
	}

	pub fn set_arith(&mut self, val: u8, alu: &ALU) {
		self.z = (val == 0) as u8;
		self.s = (val & 0x80) >> 7;
		self.p = parity(val);
		self.cy = alu.cy as u8;
		self.ac = alu.ac as u8;
	}

	pub fn set_zsp(&mut self, val: u8, alu: &ALU) {
		self.z = (val == 0) as u8;
		self.s = (val & 0x80) >> 7;
		self.p = parity(val);
		self.ac = alu.ac as u8;
	}

	pub fn set_logic(&mut self, val: u8) {
		self.cy = 0; self.ac = 0;
		self.z = (val == 0) as u8;
		self.s = (val & 0x80) >> 7;
		self.p = parity(val);
	}
}
