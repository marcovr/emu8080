#[cfg(feature = "audio")]
extern crate ears;
#[cfg(feature = "audio")]
use std::vec::Vec;
#[cfg(feature = "audio")]
use self::ears::{AudioController, Sound};

use std::fmt;
use piston_window::Key;


#[derive(Debug)]
pub struct IOController {
	input: u8,

	audio: Audio,

	shift0: u8,
	shift1: u8,
	shift_offset: u8
}

impl IOController {
	pub fn new() -> IOController {
		IOController {
			input: 4, audio: Audio::new(), shift0: 0, shift1: 0, shift_offset: 0
		}
	}

	pub fn write(&mut self, port: u8, value: u8) {
		match port {
			2 => {
				self.shift_offset = value & 0x7;
			},
			3 => {
				self.audio.play(3, value);
			},
			4 => {
				self.shift0 = self.shift1;
				self.shift1 = value;
			},
			5 => {
				self.audio.play(5, value);
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

	pub fn set_key(&mut self, key: Key, value: u8) {
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
		if value == 1 {
			//self.input ^= 1 << shift;
			self.input |= 1 << shift;
		}
		else {
			self.input &= !(1 << shift);
		}
	}
}

#[cfg(feature = "audio")]
struct Audio {
	sounds: Vec<Sound>
}

#[cfg(feature = "audio")]
impl Audio {
	fn new() -> Self {
		let mut sounds: Vec<Sound> = Vec::new();
		for i in 0..9 {
			let file = &format!("../sound/{}.wav", i);
			let error = &format!("{}.wav sound not found", i);
			sounds.push(Sound::new(file).expect(error));
		}
		sounds[0].set_looping(true); // loop UFO sound

		Audio {
			sounds
		}
	}

	fn play(&mut self, port: u8, value: u8) {
		if port == 3 {
			for i in 0..4 {
				if (value >> i) & 1 == 1 {
					if !self.sounds[i].is_playing() {
						self.sounds[i].play();
					}
				}
			}
			if value & 1 == 0 {
				if self.sounds[0].is_playing() {
					self.sounds[0].stop(); // stop UFO sound if needed
				}
			}
		}
		else {
			for i in 4..9 {
				if (value >> (i - 4)) & 1 == 1 {
					if !self.sounds[i].is_playing() {
						self.sounds[i].play();
					}
				}
			}
		}
	}
}


// Dummy Version if audio not enabled

#[cfg(not(feature = "audio"))]
struct Audio {}

#[cfg(not(feature = "audio"))]
impl Audio {
	fn new() -> Self {
		Audio {}
	}

	fn play(&mut self, _port: u8, _value: u8) {}
}

impl fmt::Debug for Audio {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "Audio")
	}
}
