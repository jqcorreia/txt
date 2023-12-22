use std::fmt::Debug;

use sdl2::{event::Event, rect::Rect, render::Canvas, ttf::Font, video::Window};

use crate::atlas::FontAtlas2;

pub trait Panel: Render + EventConsumer + Focusable {}

impl Debug for dyn Panel {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.id())
    }
}

pub trait Render {
    fn id(&self) -> String;
    fn render(
        &mut self,
        atlas: &mut FontAtlas2,
        font: &Font,
        canvas: &mut Canvas<Window>,
        rect: Rect,
    );
}

pub trait EventConsumer {
    fn consume_event(&mut self, event: &Event);
}

pub trait Focusable {
    fn is_focused(&self) -> bool;
    fn focus(&mut self);
    fn unfocus(&mut self);
}
