use std::env;
use std::fs::File;
use std::io::Read;
use chip8::*;
use sdl2::*;

fn main() {
    let args: Vec<_> = env::args().collect(); // collect CL arguments from the user - args[0] stores the name of the program
    if args.len() != 2 {
        println!("Try: cargo run path/to/game");
    }

    let sdl = sdl2::init().unwrap();
    let video = sdl.video().unwrap();
    let window = video
        .window("Chip-8 Emulator", WIDTH, HEIGHT)
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
                    break 'gameloop';
                },
                _ => ()
            }
        }
        chip8.tick(); // CPU tick for each game loop
    }
}

fn draw_to_screen(emulator: &Emulator, canvas: &mut Canvas<Window>) {
    // reset the canvas
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.set_draw_color(Color::RBG(255, 255, 255)); // set canvas to white and locate black pixels
    
    for (i, pixel) in emulator.get_display().iter().enumerate() {
        if (pixel == 0) {
            // convert from 1d array to 2d 
            let x = (i % WIDTH) as u32;
            let y = (i / WIDTH) as u32;

            // draw the pixel rect
            let rect = Rect::new(x as i32, y as i32, WIDTH, HEIGHT);
            canvas.fill_rect(rect).unwrap();
        }
    }
    canvas.present();
}
