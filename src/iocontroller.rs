use piston_window::Key;

#[derive(Debug)]
pub struct IOController {
	input: u8,

	out3: u8,
	out5: u8,

	shift0: u8,
	shift1: u8,
	shift_offset: u8
}

impl IOController {
	pub fn new() -> IOController {
		IOController {
			input: 4, out3: 0, out5: 0, shift0: 0, shift1: 0, shift_offset: 0
		}
	}

	pub fn write(&mut self, port: u8, value: u8) {
		match port {
			2 => {
				self.shift_offset = value & 0x7;
			},
			3 => {
				self.out3 = value;
			},
			4 => {
				self.shift0 = self.shift1;
				self.shift1 = value;
			},
			5 => {
				self.out5 = value;
			},
			_ => ()
		};
	}

	pub fn read(&self, port: u8) -> u8 {
		match port {
			0 => 0xF,
			1 => self.input,
			2 => 0,
			3 => {
				let v = ((self.shift1 as u16) << 8) | self.shift0 as u16;
				(v >> (8 - self.shift_offset)) as u8
			},
			_ => 0
		}
	}

	pub fn set_key(&mut self, key: Key, val: u8) {
		let shift = match key {
			Key::C => 0,
			Key::Return => 2,
			Key::D1 => 2,
			Key::Space => 4,
			Key::Period => 4,
			Key::Left => 5,
			Key::Z => 5,
			Key::Right => 6,
			Key::X => 6,
			_ => 7
		};
		if val == 1 {
			self.input ^= 1 << shift;
		}
		else {
			self.input &= !(1 << shift);
		}
	}
}
