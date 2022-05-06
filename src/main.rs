use libqoi;
use image;
use libqoi::Part;
use std::io::Write;

// convert a color into a number from 0..64
pub fn color_hash(r: u8, g: u8, b: u8, a: u8) -> usize {
    (r as usize * 3 + g as usize * 5 + b as usize * 7 + a as usize * 11) % 64
}

#[inline]
pub fn add_hash_and_last(
    r: u8,
    g: u8,
    b: u8,
    a: u8,
    array: &mut [(u8, u8, u8, u8); 64],
    last: &mut (u8, u8, u8, u8),
) {
    let hash = color_hash(r, g, b, a);
    array[hash] = (r, g, b, a);
    *last = (r, g, b, a);
}

fn read_qoi_to_pixels(mut qoi: &[u8], mut skip: u32) -> Vec<u8> {
    let mut pxlbuffer = Vec::new();
    let mut last_pxl:(u8,u8,u8,u8) = (0,0,0,255);
    let mut colorhashes = [(0u8, 0u8, 0u8, 0u8); 64];
    loop {
        let (rest,part) = match Part::decode(qoi) {
            Some(data) => data,
            None => break
        };
        qoi = rest;
        if skip > 0 {
            skip -= 1;
            continue;
        }
        match part {
            Part::RGBA(r,g,b,a) => {
                pxlbuffer.push(r);
                pxlbuffer.push(g);
                pxlbuffer.push(b);
                pxlbuffer.push(a);
                add_hash_and_last(r, g, b, a, &mut colorhashes, &mut last_pxl);
            }
            Part::RGB(r,g,b) => {
                pxlbuffer.push(r);
                pxlbuffer.push(g);
                pxlbuffer.push(b);
                pxlbuffer.push(last_pxl.3);
                add_hash_and_last(r, g, b, last_pxl.3, &mut colorhashes, &mut last_pxl);
            }
            Part::Run(runlen) => {
                for _ in 0..runlen {
                    pxlbuffer.push(last_pxl.0);
                    pxlbuffer.push(last_pxl.1);
                    pxlbuffer.push(last_pxl.2);
                    pxlbuffer.push(last_pxl.3);
                }
            }
            Part::LumaDiff(drdg, dg, dbdg) => {
                let dr = drdg + dg;
                let db = dbdg + dg;
                let r = (last_pxl.0 as i8).wrapping_add(dr) as u8;
                let g = (last_pxl.1 as i8).wrapping_add(dg) as u8;
                let b = (last_pxl.2 as i8).wrapping_add(db) as u8;
                pxlbuffer.push(r);
                pxlbuffer.push(g);
                pxlbuffer.push(b);
                pxlbuffer.push(last_pxl.3);
                add_hash_and_last(r, g, b, last_pxl.3, &mut colorhashes, &mut last_pxl);
            }
            Part::SmallDiff(dr, dg, db) => {
                let r = (last_pxl.0 as i8).wrapping_add(dr) as u8;
                let g = (last_pxl.1 as i8).wrapping_add(dg) as u8;
                let b = (last_pxl.2 as i8).wrapping_add(db) as u8;
                pxlbuffer.push(r);
                pxlbuffer.push(g);
                pxlbuffer.push(b);
                pxlbuffer.push(last_pxl.3);
                add_hash_and_last(r, g, b, last_pxl.3, &mut colorhashes, &mut last_pxl);
            }
            Part::Idx(index) => {
                let pxl = colorhashes[index as usize];
                pxlbuffer.push(pxl.0);
                pxlbuffer.push(pxl.1);
                pxlbuffer.push(pxl.2);
                pxlbuffer.push(pxl.3);
                // Adding this to hash is redundent.
                add_hash_and_last(pxl.0, pxl.1, pxl.2, pxl.3, &mut colorhashes, &mut last_pxl);
            }
        }
//         println!("{:?}", part)
    }
    pxlbuffer
}

extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use std::time::Duration;

pub fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let file = std::fs::read("cat.qoi").unwrap();
    
    let mut skip = 10000;
    
    let mut img = read_qoi_to_pixels(&file, skip);
    println!("Loaded {} pixels", img.len());
    let title = format!("QOI recovery {} pixels", img.len());
    
    let window = video_subsystem
        .window(&title, 800, 600)
        .position_centered()
        .resizable()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump()?;

    let mut width = 767;
    
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'running,
                Event::KeyDown { keycode: Some(Keycode::A), .. } => width -= 1,
                Event::KeyDown { keycode: Some(Keycode::D), .. } => width += 1,
                Event::KeyDown { keycode: Some(Keycode::W), .. } => {
                    skip -= 100;
                    img = read_qoi_to_pixels(&file, skip);
                },
                Event::KeyDown { keycode: Some(Keycode::S), .. } => {
                    skip += 100;
                    img = read_qoi_to_pixels(&file, skip);
                },
                Event::KeyDown { keycode: Some(Keycode::Z), .. } => {
                    skip -= 5;
                    img = read_qoi_to_pixels(&file, skip);
                },
                 Event::KeyDown { keycode: Some(Keycode::X), .. } => {
                    skip += 5;
                    img = read_qoi_to_pixels(&file, skip);
                },
                Event::KeyDown { keycode: Some(Keycode::C), .. } => {
                    skip -= 1;
                    img = read_qoi_to_pixels(&file, skip);
                },
                 Event::KeyDown { keycode: Some(Keycode::V), .. } => {
                    skip += 1;
                    img = read_qoi_to_pixels(&file, skip);
                },
                Event::MouseMotion { .. } => {}
                e => {
                    println!("{:?}", e);
                }
            }
        }
        println!("skip {}", skip);

        canvas.clear();
        let hight = img.len() / width / 4;
        let mut imgsurf = sdl2::surface::Surface::new(width as u32, hight as u32, sdl2::pixels::PixelFormatEnum::RGBA8888).unwrap();
        imgsurf.with_lock_mut(|data| for i in 0..(width * hight) {
            // Alpha
            data[i*4+0] = 255;
            // blue
            data[i*4+1] = img[i * 4 + 2];
            // green
            data[i*4+2] = img[i * 4 + 1];
            // red
            data[i*4+3] = img[i * 4 + 0];
        });
        let texture_creator = canvas.texture_creator();
        let imgtex = imgsurf.as_texture(&texture_creator).unwrap();
        canvas.copy(&imgtex,None,sdl2::rect::Rect::new(0,0, width as u32, hight as u32)).unwrap();
        
        
//         canvas.clear();
        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));
        // The rest of the game loop goes here...
    }
    
    let hight = img.len() / width / 4;
    let encoded = libqoi::encode_qoi(&img, hight, width, 4, 0).unwrap();
    std::fs::File::create("out.qoi").unwrap().write_all(&encoded);
    

    Ok(())
}
