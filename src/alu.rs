#[derive(Debug)]
pub struct ALU {
	pub cy: bool,
	pub ac: bool,
}

impl ALU {
	pub fn new() -> ALU {
		ALU {
			cy: false,
			ac: false,
		}
	}

	pub fn add(&mut self, x: u8, y: u8) -> u8 {
		let res = (x as i32) + (y as i32);
		let ac = ((x & 0x0F) as i32) + ((y & 0x0F) as i32);
		self.check_ac(ac);
		self.check_carry(res);
		res as u8
	}

	pub fn add3(&mut self, x: u8, y: u8, z: u8) -> u8 {
		let res = (x as i32) + (y as i32) + (z as i32);
		let ac = ((x & 0x0F) as i32) + ((y & 0x0F) as i32) + ((z & 0x0F) as i32);
		self.check_ac(ac);
		self.check_carry(res);
		res as u8
	}

	pub fn addx(&mut self, x: u16, y: u16) -> u16 {
		let res = (x as i32) + (y as i32);
		self.check_carry16(res);
		res as u16
	}

	pub fn sub(&mut self, x: u8, y: u8) -> u8 {
		let res = (x as i32) - (y as i32);
		let ac = ((x & 0x0F) as i32) - ((y & 0x0F) as i32);
		self.check_ac(ac);
		self.check_carry(res);
		res as u8
	}

	pub fn sub3(&mut self, x: u8, y: u8, z: u8) -> u8 {
		let res = (x as i32) - (y as i32) - (z as i32);
		let ac = ((x & 0x0F) as i32) - ((y & 0x0F) as i32) - ((z & 0x0F) as i32);
		self.check_ac(ac);
		self.check_carry(res);
		res as u8
	}

	pub fn subx(&mut self, x: u16, y: u16) -> u16 {
		let res = (x as i32) - (y as i32);
		self.check_carry16(res);
		res as u16
	}


	fn check_carry(&mut self, res: i32) {
		let min = u8::min_value() as i32;
		let max = u8::max_value() as i32;
		self.cy = res < min || res > max;
	}

	fn check_carry16(&mut self, res: i32) {
		let min = u16::min_value() as i32;
		let max = u16::max_value() as i32;
		self.cy = res < min || res > max;
	}

	fn check_ac(&mut self, val: i32) {
		self.ac = val > 0xF;
	}
}
