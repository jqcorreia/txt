use sdl2::{
    pixels::{Color, PixelFormatEnum},
    rect::Rect,
    render::{Canvas, Texture, TextureCreator},
    ttf::{Font, Sdl2TtfContext},
    video::{Window, WindowContext},
};
use std::collections::HashMap;

pub struct FontAtlas<'a> {
    pub texture: Texture<'a>,
    pub atlas: HashMap<char, Rect>,
}

impl<'a> FontAtlas<'a> {
    pub fn new(
        tc: &'a mut TextureCreator<WindowContext>,
        canvas: &mut Canvas<Window>,
        font: &'a Font,
    ) -> FontAtlas<'a> {
        let mut atlas = HashMap::new();
        let mut x: i32 = 0;
        let y: i32 = 0;
        let mut tex_list = Vec::new();

        // Generate all the textures and create all the rects
        for code in 1..255 as u8 {
            let font_render = font
                .render_char(code as char)
                .blended(Color::RGBA(255, 255, 255, 255));

            match font_render {
                Ok(_) => (),
                Err(_) => continue,
            }

            let texture = tc
                .create_texture_from_surface(font_render.unwrap())
                .unwrap();
            let query = texture.query();
            tex_list.push(texture);

            let rect = Rect::new(x, y, query.width, query.height);
            x += query.width as i32;
            atlas.insert(code as char, rect);
        }

        // Generate the final "atlas" texture
        let mut final_texture = tc
            .create_texture_target(
                tc.default_pixel_format(),
                atlas.values().into_iter().map(|v| v.width()).sum(),
                atlas
                    .values()
                    .into_iter()
                    .map(|v| v.height())
                    .max()
                    .unwrap(),
            )
            .unwrap();

        canvas
            .with_texture_canvas(&mut final_texture, |tcanvas| {
                let mut x: i32 = 0;
                for t in tex_list {
                    let y: i32 = 0;
                    let q = t.query();
                    let w = q.width;
                    let h = q.height;

                    tcanvas.copy(&t, None, Some(Rect::new(x, y, w, h))).unwrap();
                    x += w as i32;
                }
            })
            .unwrap();

        FontAtlas {
            texture: final_texture,
            atlas,
        }
    }
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct TextureInfo {
    pub font_name: String,
    pub font_size: i32,
    pub fg: Color,
    pub ch: char,
}

pub struct FontAtlas2<'fa> {
    pub atlas: HashMap<TextureInfo, Texture<'fa>>,
    tc: &'fa TextureCreator<WindowContext>,
}

impl<'fa> FontAtlas2<'fa> {
    pub fn new(tc: &'fa TextureCreator<WindowContext>) -> Self {
        FontAtlas2 {
            atlas: HashMap::new(),
            tc,
        }
    }
    pub fn generate_new_texture(&mut self, font: &Font, te: TextureInfo) -> &Texture {
        let surf = font.render_char(te.ch as char).blended(te.fg).unwrap();

        let tex: Texture<'fa> = self.tc.create_texture_from_surface(surf).unwrap();

        self.atlas.insert(te.clone(), tex);
        self.atlas.get(&te).unwrap()
    }

    pub fn draw_char(&mut self, font: &Font, ch: char, fg: Color) -> &Texture {
        let font_name = font.face_family_name().unwrap();
        let te = TextureInfo {
            font_name,
            fg,
            ch,
            font_size: font.height(),
        };

        let mut new = false;
        if let None = self.atlas.get(&te) {
            new = true
        }
        if new {
            self.generate_new_texture(font, te)
        } else {
            self.atlas.get(&te).unwrap()
        }
    }

    pub fn draw_string(
        &mut self,
        s: String,
        canvas: &mut Canvas<Window>,
        font: &Font,
        fg: Color,
    ) -> Texture {
        let mut x = 0;
        // let mut y = 0;

        let mut tw = 0;
        let mut th = 0;

        //FIXME this is stupid has we need to traverse the string twice FIXME
        for c in s.chars() {
            let ch = c as char;
            let t = self.draw_char(font, ch, fg);
            tw += t.query().width;
            th = t.query().height;
        }

        let mut final_tex = self
            .tc
            .create_texture_target(PixelFormatEnum::RGBA8888, tw, th)
            .unwrap();

        canvas
            .with_texture_canvas(&mut final_tex, |texture_canvas| {
                for c in s.chars() {
                    let ch = c as char;
                    let t = self.draw_char(font, ch, fg);

                    texture_canvas
                        .copy(&t, None, Rect::new(x, 0, t.query().width, t.query().height))
                        .unwrap();
                    x += t.query().width as i32;
                }
            })
            .unwrap();

        final_tex
    }
}
