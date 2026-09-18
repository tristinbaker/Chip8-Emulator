use chip8_core::*;
use minifb::{Key, Window, WindowOptions};
use std::env;
use std::fs::File;
use std::io::Read;

const SCALE: usize = 15;
const TICKS_PER_FRAME: usize = 10;
const NUM_KEYS_LOCAL: usize = 16;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: cargo run path/to/rom");
        return;
    }

    let mut emu = Emu::new();

    let mut rom = File::open(&args[1]).expect("Failed to open ROM");
    let mut buffer = Vec::new();
    rom.read_to_end(&mut buffer).expect("Failed to read ROM");
    emu.load(&buffer);

    let win_width = SCREEN_WIDTH * SCALE;
    let win_height = SCREEN_HEIGHT * SCALE;

    let mut window = Window::new(
        "Chip-8 Emulator",
        win_width,
        win_height,
        WindowOptions::default(),
    )
    .expect("Failed to open window");

    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    let mut pixel_buffer: Vec<u32> = vec![0; win_width * win_height];

    while window.is_open() && !window.is_key_down(Key::Escape) {
        // Handle input — poll each Chip-8 hex key every frame
        for i in 0..NUM_KEYS_LOCAL {
            let pressed = window.is_key_down(hex_to_key(i));
            emu.keypress(i, pressed);
        }

        // Run CPU ticks (Chip-8 runs at ~500-600Hz, 10 ticks * 60fps ≈ 600Hz)
        for _ in 0..TICKS_PER_FRAME {
            emu.tick();
        }

        // Tick timers at 60Hz (once per frame)
        emu.tick_timers();

        // Render the emulator's screen to the pixel buffer
        let screen = emu.get_display();
        for (i, &on) in screen.iter().enumerate() {
            let sx = (i % SCREEN_WIDTH) * SCALE;
            let sy = (i / SCREEN_WIDTH) * SCALE;
            let color = if on { 0x00FFFFFF } else { 0x00000000 };
            for dy in 0..SCALE {
                for dx in 0..SCALE {
                    pixel_buffer[(sy + dy) * win_width + (sx + dx)] = color;
                }
            }
        }

        window
            .update_with_buffer(&pixel_buffer, win_width, win_height)
            .expect("Failed to update window");
    }
}

// Chip-8 hex keypad → QWERTY layout:
//   1 2 3 C         1 2 3 4
//   4 5 6 D    →    Q W E R
//   7 8 9 E         A S D F
//   A 0 B F         Z X C V
fn hex_to_key(hex: usize) -> Key {
    match hex {
        0x0 => Key::X,
        0x1 => Key::Key1,
        0x2 => Key::Key2,
        0x3 => Key::Key3,
        0x4 => Key::Q,
        0x5 => Key::W,
        0x6 => Key::E,
        0x7 => Key::A,
        0x8 => Key::S,
        0x9 => Key::D,
        0xA => Key::Z,
        0xB => Key::C,
        0xC => Key::Key4,
        0xD => Key::R,
        0xE => Key::F,
        0xF => Key::V,
        _ => Key::Unknown,
    }
}
