use std::env;
use std::fs::File;
use std::io::Read;
use chip8::*;
use sdl2::*;
use sdl2::event::Event;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::keyboard::Keycode;

fn main() {
    let args: Vec<_> = env::args().collect(); // collect CL arguments from the user - args[0] stores the name of the program
    if args.len() != 2 {
        println!("Try: cargo run path/to/game");
    }

    let sdl = sdl2::init().unwrap();
    let video = sdl.video().unwrap();
    let window = video
        .window("Chip-8 Emulator", W_WIDTH, W_HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    canvas.clear();
    canvas.present();

    // init the emulator object and prepare to put gamefile in RAM
    let mut chip8 = Emulator::new();
    let mut rom = File::open(&args[1]).expect("Unable to locate file! Please check path and try again"); // if fail, expect
    let mut buffer = Vec::new();
    rom.read_to_end(&mut buffer).unwrap(); // need reference to be mutable
    chip8.load(&buffer);

    // sdl uses an event pump to pull events for every loop - react to keypress etc.
    let mut event_pump = sdl.event_pump().unwrap(); 
    'gameloop: loop { // game loop! break if we encounter quit keypress
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit{..} => {
                    break 'gameloop;
                },
                Event::KeyDown{keycode: Some(key), ..} => {
                    if let Some(k) = key2btn(key) {
                        chip8.keypress(k, true);
                    }
                },
                Event::KeyUp{keycode: Some(key), ..} => {
                    if let Some(k) = key2btn(key) {
                        chip8.keypress(k, false);
                    }
                },
                _ => ()
            }
        }
        
        for frame in 0..10 {
            chip8.tick(); // CPU tick for each game loop
        }
        chip8.tick_timers(); 
        draw_to_screen(&chip8, &mut canvas);
    }
}

fn draw_to_screen(emulator: &Emulator, canvas: &mut Canvas<Window>) {
    // reset the canvas
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.set_draw_color(Color::RGB(255, 255, 255)); // set canvas to white and locate black pixels
    
    for (i, pixel) in emulator.get_display().iter().enumerate() {
        if *pixel {
            // convert from 1d array to 2d 
            let x = (i % (WIDTH as usize)) as u32;
            let y = (i / (WIDTH as usize)) as u32;

            // draw the pixel rect
            let rect = Rect::new((x * SCALER) as i32, (y * SCALER) as i32, SCALER, SCALER);
            canvas.fill_rect(rect).unwrap();
        }
    }
    canvas.present();
}

fn key2btn(key: Keycode) -> Option<usize> {
    // convert keypress to u8 for backend processing
    match key {
        Keycode::Num1 => Some(0x1),
        Keycode::Num2 => Some(0x2),
        Keycode::Num3 => Some(0x3),
        Keycode::Num4 => Some(0xC),
        Keycode::Q => Some(0x4),
        Keycode::W => Some(0x5),
        Keycode::E => Some(0x6),
        Keycode::R => Some(0xD),
        Keycode::A => Some(0x7),
        Keycode::S => Some(0x8),
        Keycode::D => Some(0x9),
        Keycode::F => Some(0xE),
        Keycode::Z => Some(0xA),
        Keycode::X => Some(0x0),
        Keycode::C => Some(0xB),
        Keycode::V => Some(0xF),
        _ => None,
    }
}