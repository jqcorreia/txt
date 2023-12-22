extern crate sdl2;

pub mod atlas;
pub mod layout;
pub mod panels;
pub mod text;

use atlas::FontAtlas2;
use layout::{Container, ContainerType, Layout};
use panels::Panel;
use sdl2::pixels::PixelFormatEnum;

use sdl2::rect::Point;
use sdl2::{
    keyboard::Keycode, pixels::Color, rect::Rect, render::Canvas, ttf::Font, video::Window,
};
use std::collections::HashMap;
use std::time::Instant;
use text::TextArea;

fn draw_fps(canvas: &mut Canvas<Window>, font: &Font, fps: u32) {
    let x = canvas.viewport().width() - 200;
    let y = canvas.viewport().height() - 100;

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
    // canvas.set_draw_color(Color::RGBA(100, 100, 100, 100));
    // canvas.fill_rect(Rect::new(100, 100, 100, 100));
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

    let mut font_size = 14;
    let mut font = ttf
        .load_font("/usr/share/fonts/droid/DroidSansMono.ttf", font_size)
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    let mut event_pump = sdl.event_pump().unwrap();
    let mut running = true;
    let mut draw_debug_info = true;

    let mut n: u64 = 0;

    let tc2 = canvas.texture_creator();
    let mut cur_time = Instant::now();
    let mut atlas2 = FontAtlas2::new(&tc2);
    let text_area = TextArea::new(String::from("/home/jqcorreia/code/tests/txt/SANDBOX"));
    let t2 = TextArea::new(String::from("up"));
    let t3 = TextArea::new(String::from("down"));

    let mut components: HashMap<String, Box<dyn Panel>> = HashMap::new();
    let l = Layout {
        gap: 5,
        root: Container {
            size: 100,
            size_type: layout::SizeTypeEnum::Percent,
            container_type: ContainerType::HSplit,
            nodes: Some(Vec::from([
                Container {
                    size: 100,
                    size_type: layout::SizeTypeEnum::Percent,
                    container_type: ContainerType::Leaf,
                    nodes: None,
                    key: Some(String::from("t1")),
                },
                Container {
                    size: 300,
                    size_type: layout::SizeTypeEnum::Fixed,
                    container_type: ContainerType::VSplit,
                    nodes: Some(Vec::from([
                        Container {
                            size: 50,
                            size_type: layout::SizeTypeEnum::Percent,
                            container_type: ContainerType::Leaf,
                            nodes: None,
                            key: Some(String::from("t2")),
                        },
                        Container {
                            size: 50,
                            size_type: layout::SizeTypeEnum::Percent,
                            container_type: ContainerType::Leaf,
                            nodes: None,
                            key: Some(String::from("t3")),
                        },
                    ])),
                    key: None,
                },
            ])),
            key: None,
        },
    };

    // let l = Layout {
    //      gap: 5,
    //      root: Container {
    //          size: 100,
    //          size_type: layout::SizeTypeEnum::Percent,
    //          container_type: ContainerType::Leaf,
    //          nodes: None,
    //          key: Some(String::from("t1")),de
    //      },
    //  };

    components.insert(String::from("t1"), Box::new(text_area));
    components.insert(String::from("t2"), Box::new(t2));
    components.insert(String::from("t3"), Box::new(t3));
    components.get_mut("t1").unwrap().focus();

    while running {
        let mut lay = l.generate(
            canvas.window().size().0 as usize,
            canvas.window().size().1 as usize,
        );

        n = n + 1;
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::KeyDown {
                    keycode: Some(Keycode::F1),
                    ..
                } => draw_debug_info = !draw_debug_info,
                sdl2::event::Event::KeyDown {
                    keycode: Some(Keycode::F2),
                    ..
                } => {
                    font_size += 1;
                    font = ttf
                        .load_font("/usr/share/fonts/droid/DroidSansMono.ttf", font_size)
                        .unwrap()
                }
                sdl2::event::Event::KeyDown {
                    keycode: Some(Keycode::F3),
                    ..
                } => {
                    font_size -= 1;
                    font = ttf
                        .load_font("/usr/share/fonts/droid/DroidSansMono.ttf", font_size)
                        .unwrap()
                }
                sdl2::event::Event::KeyDown {
                    keycode: Some(Keycode::Q),
                    keymod: sdl2::keyboard::Mod::LCTRLMOD,
                    ..
                } => running = false,
                sdl2::event::Event::Quit { .. } => running = false,
                sdl2::event::Event::MouseButtonDown { x, y, .. } => {
                    for (rect, key) in lay.iter() {
                        let comp = components.get_mut(key).unwrap();
                        dbg!(&rect, &comp);
                        if rect.contains_point(Point::new(x, y)) {
                            dbg!("should focus");
                            comp.focus();
                        }
                        if !rect.contains_point(Point::new(x, y)) {
                            comp.unfocus();
                        }
                    }

                    println!("{} {}", x, y)
                }
                _ => (),
            }
            for (_, key) in lay.iter() {
                let comp = components.get_mut(key).unwrap();
                comp.consume_event(&event);
            }
        }
        let fps = (1_000_000_000 / (&cur_time.elapsed().as_nanos())) as u32;
        cur_time = Instant::now();

        canvas.set_draw_color(Color::RGBA(0, 0, 0, 0));
        canvas.clear();

        for (rect, key) in lay.iter_mut() {
            let comp = components.get_mut(key).unwrap();
            let mut tex = tc2
                .create_texture_target(PixelFormatEnum::RGBA8888, rect.width(), rect.height())
                .unwrap();

            canvas
                .with_texture_canvas(&mut tex, |c| {
                    comp.render(&mut atlas2, &font, c, *rect);
                    let border_color = if comp.is_focused() {
                        Color::RGBA(0, 255, 0, 255)
                    } else {
                        Color::RGBA(100, 100, 100, 255)
                    };
                    c.set_draw_color(border_color);
                    c.draw_rect(Rect::new(0, 0, rect.width(), rect.height()))
                        .unwrap();
                })
                .unwrap();

            canvas.copy(&tex, None, *rect).unwrap();
        }

        // Draw the FPS counter directly into the window canvas
        if draw_debug_info {
            draw_fps(&mut canvas, &font, fps);
        }
        canvas.present();
    }
}
