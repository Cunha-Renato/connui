use super::*;
use crate::event::Event;
use crate::event::MouseEvent;
use crate::prelude::*;
use crate::state::*;

#[derive(Default, Debug, Clone, Copy)]
struct ScrollState {
    content_position: Point,
    content_size: Size,
    avail_size: Size,
}

pub struct ScrollDiv<T: 'static> {
    content_area: ScrollContent<T>,
    style: Style,
    content_offset: Point,
    id: WidgetId,
}
impl<T: 'static> Widget<T> for ScrollDiv<T> {
    fn init(&mut self, ctx: &mut StateContext) {
        if let Some(state) = ctx
            .entry(self.id)
            .or_insert_with(|| State::new(ScrollState::default()))
            .get_mut::<ScrollState>()
        {
            
        }
    }

    fn update(&mut self, ctx: &mut StateContext, _: Point, size: Size) {
        if let Some(state) = ctx.get_mut::<ScrollState>(&self.id) {
            state.avail_size = size;
        }
    }

    fn get_position(&self) -> Position {
        self.style.position
    }

    fn get_size(&self) -> Size<SizeOp> {
        self.style.size
    }

    fn get_layout(&self) -> Layout {
        self.style.layout
    }

    fn get_children(&mut self) -> Vec<Element<T>> {
        let mut shell = self.content_area.clone();
        std::mem::swap(&mut shell, &mut self.content_area);

        vec![shell.into()]
    }

    fn on_event(&mut self, event: Event) -> Response<T> {
        if let Event::Mouse {
            event: MouseEvent::Scroll(delta),
            ..
        } = event
        {
            self.content_offset.x += delta.x as f32;
            self.content_offset.y += delta.y as f32;
        }

        Response::default()
    }

    fn render(&self, position: Point, size: Size) -> Vec<crate::renderer::RenderCommand> {
        todo!()
    }
}

struct ScrollContent<T: 'static> {
    div: Div<T>,
    id: WidgetId,
}
impl<T: 'static> Clone for ScrollContent<T> {
    fn clone(&self) -> Self {
        let mut div = Div::default();
        div.style = self.div.style;

        Self {
            div,
            id: self.id.clone(),
        }
    }
}
impl<T: 'static> Widget<T> for ScrollContent<T> {
    fn update(&mut self, ctx: &mut StateContext, position: Point, size: Size) {
        if let Some(state) = ctx.get_mut::<ScrollState>(&self.id) {
            state.content_position = position;
            state.content_size = size;
        }
    }

    fn get_position(&self) -> Position {
        self.div.get_position()
    }

    fn get_size(&self) -> Size<SizeOp> {
        self.div.get_size()
    }

    fn get_layout(&self) -> Layout {
        self.div.get_layout()
    }

    fn get_children(&mut self) -> Vec<Element<T>> {
        self.div.get_children()
    }

    fn render(&self, position: Point, size: Size) -> Vec<crate::renderer::RenderCommand> {
        self.div.render(position, size)
    }
}
impl<T: 'static> From<ScrollContent<T>> for Element<T> {
    #[inline]
    fn from(value: ScrollContent<T>) -> Self {
        Self::new(value)
    }
}
