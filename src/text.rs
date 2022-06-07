use sdl2::{
    event::Event, keyboard::Keycode, pixels::Color, rect::Rect, render::Canvas, ttf::Font,
    video::Window,
};
use std::cmp::max;
use std::cmp::min;

use crate::atlas::FontAtlas2;

#[derive(Debug)]
pub struct CursorPosition {
    pub line: usize,
    pub col: usize,
}

pub struct Viewport {
    pub lines: usize,
    pub cols: usize,
    pub cur_line: usize,
    pub cur_col: usize,
}

impl Viewport {
    fn contains(&self, lineno: usize) -> bool {
        return (self.cur_line..(self.cur_line + self.lines)).contains(&lineno)
    }
}

pub struct TextArea {
    pub text: String,
    pub cursor_pos: CursorPosition,
    pub viewport: Viewport, 

}

pub trait Render {
    fn render(&mut self, atlas: &mut FontAtlas2, font: &Font, canvas: &mut Canvas<Window>);
}

// impl Render for TextArea {
//     fn render(&mut self, atlas: &mut FontAtlas2, font: &Font, canvas: &mut Canvas<Window>) {
//         let mut x = 0;
//         let mut y = 10;

//         let fg = Color::RGBA(0, 0, 0, 0);
//         let bg = Color::RGBA(255, 255, 255, 255);

//         let mut next_is_newline = false;

//         let mut line = 1;
//         let mut col = 1;

//         for (idx, c) in self.text.chars().enumerate() {
//             if next_is_newline {
//                 line += 1;
//                 col = 1;
//                 next_is_newline = false;
//             }
//             if c == '\n' {
//                 next_is_newline = true;
//             }

//             let active_colors = if line == self.cursor_pos.line && col == self.cursor_pos.col {
//                 (fg, bg)
//             } else {
//                 (bg, fg)
//             };
//             let to_print = if c == '\n' { ' ' } else { c };

//             let tex = atlas.draw_char(font, to_print, active_colors.0, active_colors.1);
//             let q = tex.query();
//             let w = q.width;
//             let h = q.height;
//             canvas
//                 .copy(tex, None, Some(Rect::new(x as i32, y as i32, w, h)))
//                 .unwrap();
//             if c == '\n' {
//                 y += h;
//                 x = 0;
//                 continue;
//             }
//             x += w;
//             col += 1;
//             // dbg!(&query, w, h, x, y);
//         }
//     }
// }

impl Render for TextArea {
    fn render(&mut self, atlas: &mut FontAtlas2, font: &Font, canvas: &mut Canvas<Window>) {
        let mut x = 0;
        let mut y = 10;

        let fg = Color::RGBA(0, 0, 0, 0);
        let bg = Color::RGBA(255, 255, 255, 255);

        let mut line = 1;
        let mut col;
        let mut h = 0;

        for (lineno, tline) in self.text.split_inclusive('\n').enumerate() {
            if !self.viewport.contains(lineno) {
                continue 
            }

            col = 1;
            for c in tline.chars() {
                let active_colors = if line == self.cursor_pos.line && col == self.cursor_pos.col {
                    (fg, bg)
                } else {
                    (bg, fg)
                };
                let to_print = if c == '\n' { ' ' } else { c };

                let tex = atlas.draw_char(font, to_print, active_colors.0, active_colors.1);
                let q = tex.query();
                let w = q.width;
                h = q.height;
                canvas
                    .copy(tex, None, Some(Rect::new(x as i32, y as i32, w, h)))
                    .unwrap();
                x += w;
                col += 1;
                // dbg!(&query, w, h, x, y);
            }
            y += h;
            x = 0;
            line += 1;
        }
    }
}
impl TextArea {
    pub fn new(text: String) -> Self {
        TextArea {
            text,
            cursor_pos: CursorPosition { line: 1, col: 1 },
            viewport: Viewport { cur_line: 0, cur_col: 0, lines: 50, cols: 80 },
        }
    }

    pub fn consume_event(&mut self, event: &Event) {
        match event {
            sdl2::event::Event::TextInput { text, .. } => self.insert_char(dbg!(text.to_owned())),
            sdl2::event::Event::KeyDown {
                keycode: Some(Keycode::Return),
                ..
            } => self.insert_char('\n'.to_string()),
            sdl2::event::Event::KeyDown {
                keycode: Some(Keycode::Backspace),
                ..
            } => self.delete_char(),
            sdl2::event::Event::KeyDown {
                keycode: Some(Keycode::Right),
                ..
            } => self.next_char(),
            sdl2::event::Event::KeyDown {
                keycode: Some(Keycode::Left),
                ..
            } => self.prev_char(),
            sdl2::event::Event::KeyDown {
                keycode: Some(Keycode::Down),
                ..
            } => self.next_line(),
            sdl2::event::Event::KeyDown {
                keycode: Some(Keycode::Up),
                ..
            } => self.prev_line(),
            _ => (),
        };
    }

    fn next_char(&mut self) {
        self.goto(self.cursor_pos.line, self.cursor_pos.col + 1);
    }

    fn prev_char(&mut self) {
        self.goto(self.cursor_pos.line, self.cursor_pos.col - 1);
    }

    fn goto(&mut self, new_l: usize, new_c: usize) {
        dbg!("{} {}", new_l, new_c);

        let lines = self.text.split_inclusive('\n').collect::<Vec<&str>>();
        self.cursor_pos.line = new_l.clamp(1, lines.len());

        // Do this after so you dont overflow the usize
        let line = lines[self.cursor_pos.line - 1];
        dbg!(new_c, 1, line.len());
        self.cursor_pos.col = new_c.clamp(1, line.len());

        dbg!(&self.cursor_pos);
    }
    fn next_line(&mut self) {
        self.goto(self.cursor_pos.line + 1, self.cursor_pos.col);
    }

    fn prev_line(&mut self) {
        self.goto(self.cursor_pos.line - 1, self.cursor_pos.col);
    }

    fn translate_cp_to_idx(&self, cp: &CursorPosition) -> usize {
        let mut accum = 0;
        let (t_c, t_l) = (cp.col - 1, cp.line - 1);

        for (idx, line) in self.text.split_inclusive('\n').enumerate() {
            if idx == t_l {
                accum += t_c;
                break;
            }
            accum += line.len();
        }
        accum
    }

    fn translate_idx_to_cp(&self, idx: usize) -> CursorPosition {
        let mut accum = 0;
        let mut col = 0;
        let mut line = 0;

        for (i, l) in self.text.split_inclusive('\n').enumerate() {
            if accum + l.len() > idx {
                col = idx - accum;
                line = i;
                return CursorPosition{ col: col + 1, line: line + 1 }
            }
            accum += l.len();
        }
        CursorPosition{ col, line }
    }

    pub fn insert_char(&mut self, ch: String) {
        let idx = self.translate_cp_to_idx(&self.cursor_pos);
        let char_boundary = self.text.char_indices().nth(idx).unwrap();
        let (left, right) = self.text.split_at(char_boundary.0);

        self.text = format!("{}{}{}", left, ch, right);

        if ch == "\n".to_string() {
            self.next_line()
        } else {
            self.next_char();
        }
    }

    pub fn delete_char(&mut self) {
        let idx = self.translate_cp_to_idx(&self.cursor_pos);
        if idx == 0 {
            return 
        }
        let char_boundary = self.text.char_indices().nth(idx-1).unwrap();
        dbg!(self.text.remove(char_boundary.0));
        let cp = self.translate_idx_to_cp(idx - 1);
        self.goto(cp.line, cp.col);
    }
}
