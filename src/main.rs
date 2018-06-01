#![allow(unused_mut)]
#![allow(dead_code)]

extern crate image;
extern crate piston_window;

mod machine;
mod cpu;
mod alu;
mod operation;
mod disassemble;
mod iocontroller;

use piston_window::*;
use image::RgbaImage;
use machine::*;

const DIAG: bool = false;
const DFIX: bool = false;

const WIDTH: u32 = 224;
const HEIGHT: u32 = 256;
const SCALE: u32 = 2;

fn main() {
	let mut img_buffer = RgbaImage::new(WIDTH, HEIGHT);
	
	let mut window: PistonWindow = WindowSettings::new("SpaceInvaders", [SCALE * WIDTH, SCALE * HEIGHT])
		.exit_on_esc(true)
		.opengl(OpenGL::V3_2)
		.vsync(true)
		.build()
		.unwrap();

	let ts = TextureSettings::new().filter(piston_window::Filter::Nearest);
	let mut texture = Texture::from_image(&mut window.factory, &img_buffer, &ts).unwrap();


	let mut m = Machine::new();

	//m.load_rom("../rom/cpudiag.bin", 0x100);
	//m.load_rom("../rom/test.bin", 0x100);
	m.load_rom("../rom/invaders.bin", 0);

	while let Some(e) = window.next() {
		if let Some(_) = e.render_args() {
			update(&mut m, &mut img_buffer);
			texture.update(&mut window.encoder, &img_buffer).unwrap();
			window.draw_2d(&e, |_, g| {
				clear([1.0; 4], g);
				image(&texture, [[2./WIDTH as f64, 0., -1.], [0., 2./HEIGHT as f64, -1.]], g);
			});
		}

		if let Some(_) = e.update_args() {
			m.run();
		}

		if let Some(Button::Keyboard(key)) = e.press_args() {
			m.key_pressed(key);
		}

		if let Some(Button::Keyboard(key)) = e.release_args() {
			m.key_released(key);
		}
	}
}

fn update(machine: &mut Machine, img_buffer: &mut RgbaImage) {
	for (n, byte) in machine.framebuffer().iter().enumerate() {
		let i = n as u32 * 8;

		let x = i / HEIGHT;
		for shift in 0..8 {
			let y = (i % HEIGHT) + shift as u32;

			let color = if (byte >> shift) as u32 & 1 == 0 {
				// off => always black
				[0, 0, 0, 255]
			} else if y <= 63 && (y >= 15 || y <= 15 && x >= 20 && x <= 120) {
				// green in bottom area
				[0, 255, 0, 255]
			} else if y >= 200 && y <= 220 {
				// red in UFO-area
				[255, 0, 0, 255]
			} else {
				// white otherwise
				[255; 4]
			};

			img_buffer.put_pixel(x, y, image::Rgba(color));
		}
	}
}
