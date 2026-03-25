use std::sync::mpsc::Receiver;

use sdl2::{event::Event, keyboard::Keycode, pixels::PixelFormatEnum};

const RESULTING_WIDTH:u32 = 1920;
const RESULTING_HEIGHT:u32 = 1080;


pub fn start_window(rx:Receiver<(Vec<u8>, Vec<u8>, Vec<u8>)>){
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("rust-sdl2 demo", RESULTING_WIDTH, RESULTING_HEIGHT)
        .position(0, 400)
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
    .create_texture_streaming(PixelFormatEnum::IYUV, RESULTING_WIDTH, RESULTING_HEIGHT)
    .unwrap();

    'running: loop {
        canvas.clear();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} => {break 'running}  //Kann in switch cases verwendet werden um keine code dupplication zu verursachen
                Event::KeyDown {keycode: Some(key), ..} => {
                    match key {
                        
                        Keycode::ESCAPE => {break 'running}
                        
                        _ => {}
                    }
                }
        
                _ => {}
            }
        }
        // The rest of the game loop goes here...
    
    if let Ok(frame) = rx.try_recv() {
        //println!("Frame empfangen: {} bytes", frame.0.len());
        texture.update_yuv(None, &frame.0, RESULTING_WIDTH as usize, &frame.1, 960 as usize, &frame.2, 960 as usize).unwrap();
    }

        let _ = canvas.copy(&texture, None, None);
        
        canvas.present();
       // ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

    }
