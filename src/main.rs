extern crate sdl2;

pub mod atlas;
pub mod text;

use atlas::{FontAtlas, FontAtlas2, TextureInfo};
use sdl2::{
    keyboard::Keycode, pixels::Color, rect::Rect, render::Canvas, ttf::Font, video::Window,
};
use text::TextArea;
use text::Render;
use std::{collections::HashMap, fs, time::Instant};

fn draw_fps(canvas: &mut Canvas<Window>, font: &Font, fps: u32) {
    let x = canvas.viewport().width() - 200;
    let y = canvas.viewport().width() - 100;

    let tc = canvas.texture_creator();
    let texture = tc
        .create_texture_from_surface(
            font.render(&fps.to_string())
                .blended(Color::RGBA(255, 255, 255, 255))
                .unwrap(),
        )
        .unwrap();
    let query = texture.query();
    let w = query.width;
    let h = query.height;
    canvas
        .copy(&texture, None, Some(Rect::new(x as i32, y as i32, w, h)))
        .unwrap();
}

fn main() {
    let sdl = sdl2::init().unwrap();
    let video = sdl.video().unwrap();
    let ttf = sdl2::ttf::init().unwrap();
    let mut window = video.window("txt", 1024, 768).resizable().build().unwrap();
    window.show();

    let font = ttf
        .load_font("/usr/share/fonts/droid/DroidSansMono.ttf", 14)
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();

    let mut event_pump = sdl.event_pump().unwrap();
    let mut running = true;

    let mut n: u64 = 0;
    let text = fs::read_to_string("/home/jqcorreia/jira.py").unwrap();

    let tc2 = canvas.texture_creator();
    let mut cur_time = Instant::now();
    let mut atlas2 = FontAtlas2::new(&ttf, &tc2);
    let mut text_area = TextArea::new(text);

    while running {
        n = n + 1;
        for event in event_pump.poll_iter() {
            text_area.consume_event(&event);
            match event {
                sdl2::event::Event::KeyDown {
                    keycode: Some(Keycode::Q),
                    keymod: sdl2::keyboard::Mod::LCTRLMOD, 
                    ..
                } => running = false,
                sdl2::event::Event::Quit { .. } => running = false,
                _ => (),
            }
        }
        let fps = (1_000_000_000 / (&cur_time.elapsed().as_nanos())) as u32;
        cur_time = Instant::now();

        canvas.set_draw_color(Color::RGBA(0, 0, 0, 0));
        canvas.clear();
        text_area.render(&mut atlas2, &font, &mut canvas);
        // canvas.copy(s, None, Some(Rect::new(100, 100, 100, 100)));
        // draw_text(text.to_string(), &mut canvas, &font, &cursor_pos, &mut atlas2);
        draw_fps(&mut canvas, &font, fps);
        canvas.present();
    }
}
