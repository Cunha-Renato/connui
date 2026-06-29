use super::*;
use crate::event::Event;
use crate::event::MouseEvent;
use crate::prelude::*;
use crate::renderer::RenderCommand;
use crate::state::*;

#[derive(Default, Debug, Clone, Copy)]
struct ScrollState {
    // Parent
    position: Point,
    size: Size,

    // Content Area
    content_position: Point,
    content_size: Size,
}

pub struct ScrollDiv<T: 'static> {
    content_area: ScrollContent<T>,
    style: Style,
    content_offset: Point,
    id: WidgetId,
    sense: f32,
}
impl<T: 'static> ScrollDiv<T> {
    #[inline]
    pub fn new(id: impl Into<WidgetId>) -> Self {
        let id = id.into();

        Self {
            content_area: ScrollContent::new(id),
            style: Style::default(),
            content_offset: Point::default(),
            sense: 5.0,
            id,
        }
    }

    #[inline]
    pub fn children(mut self, children: impl Into<Vec<Element<T>>>) -> Self {
        self.content_area.children = children.into();
        self
    }
}
impl<T: 'static> Widget<T> for ScrollDiv<T> {
    fn init(&mut self, ctx: &mut StateContext) {
        if let Some(state) = ctx
            .entry(self.id)
            .or_insert_with(|| State::new(ScrollState::default()))
            .get_mut::<ScrollState>()
        {
            self.content_area.layout = self.style.layout;
            self.content_area.position = Position::Pinned {
                position: Point {
                    x: state.content_position.x as i16,
                    y: state.content_position.y as i16,
                },
                parent_relative: true,
                overlay: false,
            };
        }
    }

    fn update(&mut self, ctx: &mut StateContext, position: Point, size: Size) {
        if let Some(state) = ctx.get_mut::<ScrollState>(&self.id) {
            state.content_position.y += self.content_offset.y;
            state.position = position;
            state.size = size;
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
            self.content_offset.x += delta.x as f32 * self.sense;
            self.content_offset.y += delta.y as f32 * self.sense;
        }

        Response::default()
    }

    fn render(&self, position: Point, size: Size) -> Vec<RenderCommand> {
        vec![RenderCommand::DrawRect {
            x: position.x,
            y: position.y,
            width: size.width,
            height: size.height,
            color: self.style.color,
        }]
    }
}
impl<T: 'static> From<ScrollDiv<T>> for Element<T> {
    #[inline]
    fn from(value: ScrollDiv<T>) -> Self {
        Self::new(value)
    }
}

impl_has_color!(ScrollDiv<T> { style.color });
impl_has_layout!(ScrollDiv<T> { style.layout });
impl_has_position!(ScrollDiv<T> { style.position });
impl_has_margin!(<u16> ScrollDiv<T> { style.margin });
impl_has_padding!(<u16> ScrollDiv<T> { style.padding });
impl_has_size!(<crate::types::SizeOp> ScrollDiv<T> { style.size });

struct ScrollContent<T: 'static> {
    children: Vec<Element<T>>,
    position: Position,
    layout: Layout,
    id: WidgetId,
}
impl<T: 'static> ScrollContent<T> {
    #[inline]
    fn new(id: WidgetId) -> Self {
        Self {
            children: vec![],
            position: Position::default(),
            layout: Layout::default(),
            id,
        }
    }
}
impl<T: 'static> Clone for ScrollContent<T> {
    fn clone(&self) -> Self {
        Self {
            children: vec![],
            position: self.position,
            layout: self.layout,
            id: self.id,
        }
    }
}
impl<T: 'static> Widget<T> for ScrollContent<T> {
    fn update(&mut self, ctx: &mut StateContext, position: Point, size: Size) {
        // if let Some(state) = ctx.get_mut::<ScrollState>(&self.id) {
        //     state.content_position = position;
        //     state.content_size = size;
        // }
    }

    #[inline]
    fn get_position(&self) -> Position {
        self.position
    }

    #[inline]
    fn get_size(&self) -> Size<SizeOp> {
        Size::default()
    }

    #[inline]
    fn get_layout(&self) -> Layout {
        self.layout
    }

    #[inline]
    fn get_children(&mut self) -> Vec<Element<T>> {
        std::mem::take(&mut self.children)
    }

    #[inline]
    fn render(&self, _: Point, _: Size) -> Vec<RenderCommand> {
        vec![]
    }
}
impl<T: 'static> From<ScrollContent<T>> for Element<T> {
    #[inline]
    fn from(value: ScrollContent<T>) -> Self {
        Self::new(value)
    }
}
