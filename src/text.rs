use std::fmt::Debug;

use sdl2::{
    event::Event,
    keyboard::Keycode,
    pixels::{Color, PixelFormatEnum},
    rect::Rect,
    render::{Canvas, Texture},
    ttf::Font,
    video::Window,
};

use crate::{
    atlas::FontAtlas2,
    panels::{EventConsumer, Focusable, Panel, Render},
};

#[derive(Debug)]
pub struct CursorPosition {
    pub line: usize,
    pub col: usize,
}

pub struct Viewport {
    pub cols: usize,
    pub cur_col: usize,
    pub cur_line: usize,
    pub lines: usize,
}

impl Viewport {
    fn contains(&self, lineno: usize) -> bool {
        return (self.cur_line..(self.cur_line + self.lines)).contains(&lineno);
    }
}

pub struct TextArea {
    pub cursor_pos: CursorPosition,
    pub text: String,
    pub viewport: Viewport,
    pub focused: bool,
    pub filepath: String,
}

impl Focusable for TextArea {
    fn is_focused(&self) -> bool {
        self.focused
    }
    fn focus(&mut self) {
        self.focused = true;
    }
    fn unfocus(&mut self) {
        self.focused = false;
    }
}

impl EventConsumer for TextArea {
    fn consume_event(&mut self, event: &Event) {
        if !self.is_focused() {
            return;
        }
        match event {
            sdl2::event::Event::TextInput { text, .. } => self.insert_char(dbg!(text.to_owned())),
            sdl2::event::Event::KeyDown {
                keycode: Some(Keycode::Return),
                ..
            } => self.insert_char('\n'.to_string()),
            sdl2::event::Event::KeyDown {
                keycode: Some(Keycode::S),
                keymod: sdl2::keyboard::Mod::LCTRLMOD,
                ..
            } => self.save(),
            sdl2::event::Event::KeyDown {
                keycode: Some(Keycode::Backspace),
                ..
            } => self.delete_char(),
            sdl2::event::Event::KeyDown {
                keycode: Some(Keycode::Left),
                keymod: sdl2::keyboard::Mod::LCTRLMOD,
                ..
            } => self.home(),
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
}
impl Panel for TextArea {}

impl Render for TextArea {
    fn id(&self) -> String {
        format!("{}", self.filepath)
    }

    fn render(
        &mut self,
        atlas: &mut FontAtlas2,
        font: &Font,
        canvas: &mut Canvas<Window>,
        rect: Rect,
    ) {
        // dbg!(&self.filepath, &self.text);
        let tc = canvas.texture_creator();
        let mut x = 0;
        let mut y = 10;

        canvas.set_draw_color(Color::RGBA(50, 48, 47, 255));
        canvas.clear();

        let fg = Color::RGBA(253, 244, 193, 255);

        let mut h = 0;

        for (lineno, tline) in self.text.split_inclusive('\n').enumerate() {
            if !self.viewport.contains(lineno) {
                continue;
            }
            let mut col = 0;

            for c in tline.chars() {
                let to_print = match c {
                    '\n' => ' ',
                    _ => c,
                };
                let is_cursor = lineno == self.cursor_pos.line && col == self.cursor_pos.col;
                let tex = atlas.draw_char(
                    font,
                    to_print,
                    if is_cursor {
                        Color::RGBA(0, 0, 0, 0)
                    } else {
                        fg
                    },
                );
                let q = tex.query();
                let mut cursor: Texture;

                let tex_final = if is_cursor {
                    cursor = tc
                        .create_texture_target(PixelFormatEnum::RGBA8888, q.width, q.height)
                        .unwrap();
                    canvas
                        .with_texture_canvas(&mut cursor, |c| {
                            c.set_draw_color(Color::RGBA(255, 255, 255, 255));
                            c.clear();
                            c.copy(tex, None, None).unwrap();
                        })
                        .unwrap();
                    &cursor
                } else {
                    tex
                };

                let q = tex_final.query();
                let w = q.width;
                h = q.height;
                canvas
                    .copy(&tex_final, None, Some(Rect::new(x as i32, y as i32, w, h)))
                    .unwrap();
                x += w;
                col += 1;
            }
            y += h;
            x = 0;
        }

        let info = format!("{}:{}", self.cursor_pos.line + 1, self.cursor_pos.col + 1);
        let info = atlas.draw_string(info, canvas, font, fg);
        canvas
            .copy(
                &info,
                None,
                Some(Rect::new(
                    0,
                    (rect.height() - (info.query().height + 1)) as i32,
                    info.query().width,
                    info.query().height,
                )),
            )
            .unwrap();
    }
}

impl TextArea {
    pub fn new(path: String) -> Self {
        let text = std::fs::read_to_string(&path).unwrap_or_else(|_| String::from(" "));
        TextArea {
            text,
            filepath: path,
            cursor_pos: CursorPosition { line: 0, col: 0 },
            viewport: Viewport {
                cur_line: 0,
                cur_col: 0,
                lines: 50,
                cols: 80,
            },
            focused: false,
        }
    }

    fn translate_cp_to_idx(&self, cp: &CursorPosition) -> usize {
        let mut accum = 0;

        for (idx, line) in self.text.split_inclusive('\n').enumerate() {
            if idx == cp.line {
                accum += cp.col;
                break;
            }
            accum += line.chars().count();
        }
        accum
    }

    fn translate_idx_to_cp(&self, idx: usize) -> CursorPosition {
        let mut accum = 0;
        let mut col = 0;
        let mut line = 0;

        for (i, l) in self.text.split_inclusive('\n').enumerate() {
            if accum + l.char_indices().count() > idx {
                col = idx - accum;
                line = i;
                return CursorPosition { col, line };
            }
            accum += l.chars().count();
        }
        CursorPosition { col, line }
    }

    fn next_char(&mut self) {
        self.goto(self.cursor_pos.line, self.cursor_pos.col + 1);
    }

    fn prev_char(&mut self) {
        // we need to do this because of overflow
        let f_c = if self.cursor_pos.col == 0 {
            0
        } else {
            self.cursor_pos.col - 1
        };
        self.goto(self.cursor_pos.line, f_c);
    }

    fn goto(&mut self, new_l: usize, new_c: usize) {
        dbg!("{} {}", new_l, new_c);

        let lines = self.text.split_inclusive('\n').collect::<Vec<&str>>();
        self.cursor_pos.line = new_l.clamp(0, lines.len() - 1);

        // Do this after so you dont overflow the usize
        let line = lines[self.cursor_pos.line];
        dbg!(new_c, 1, line.len());
        self.cursor_pos.col = new_c.clamp(0, line.char_indices().count() - 1);

        dbg!(&self.cursor_pos);
        let vp_v_reach = self.viewport.cur_line + self.viewport.lines;

        if self.cursor_pos.line >= vp_v_reach - 3 && vp_v_reach < self.text.lines().count() {
            self.viewport.cur_line += 10;
        }

        if self.cursor_pos.line <= self.viewport.cur_line + 3 && self.viewport.cur_line > 0 {
            self.viewport.cur_line -= std::cmp::min(self.viewport.cur_line, 10);
        }
    }
    fn next_line(&mut self) {
        self.goto(self.cursor_pos.line + 1, self.cursor_pos.col);
    }

    fn prev_line(&mut self) {
        // we need to do this because of overflow
        let f_l = if self.cursor_pos.line == 0 {
            0
        } else {
            self.cursor_pos.line - 1
        };
        self.goto(f_l, self.cursor_pos.col);
    }

    fn home(&mut self) {
        self.goto(self.cursor_pos.line, 0);
    }

    pub fn insert_char(&mut self, ch: String) {
        let idx = self.translate_cp_to_idx(&self.cursor_pos);
        let char_boundary = self.text.char_indices().nth(idx).unwrap();
        let (left, right) = self.text.split_at(char_boundary.0);

        self.text = format!("{}{}{}", left, ch, right);

        if ch == "\n".to_string() {
            self.next_line();
            self.home();
        } else {
            self.next_char();
        }
    }

    pub fn delete_char(&mut self) {
        let idx = self.translate_cp_to_idx(&self.cursor_pos);
        if idx == 0 {
            return;
        }
        let char_boundary = self.text.char_indices().nth(idx - 1).unwrap();
        dbg!(self.text.remove(char_boundary.0));
        let cp = self.translate_idx_to_cp(idx - 1);
        self.goto(cp.line, cp.col);
    }

    pub fn save(&mut self) {
        std::fs::write(self.filepath.clone(), self.text.clone()).unwrap();
    }
}
