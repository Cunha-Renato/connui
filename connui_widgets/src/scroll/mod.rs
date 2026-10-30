use bitflags::bitflags;
use connui::{
    event::EventKind,
    has_children,
    prelude::*,
    renderer::Renderer,
    tree::{ElementSpecs, LayoutElement, Style, Widget, WidgetSpecs},
};
use style::ScrollStyle;

mod style;

// fn create_viewport<T, R: Renderer>(div: Div<T, R>, style: &ScrollStyle) -> Div<T, R> {
//     let style = &style.style;

//     div.style_base(
//         crate::DivStyle::default()
//             .pinned((0, 0), PinnedFlags::RELATIVE_TO_PARENT)
//             .layout(style.layout)
//             .padding(style.padding.clone())
//             .hor_align(style.hor_align)
//             .ver_align(style.ver_align)
//             .color(Color::MAGENTA),
//     )
// }

pub struct Scroll<T, R: Renderer> {
    style: ScrollStyle,
    children: Vec<Widget<T, R>>,
}
impl<T, R: Renderer> Default for Scroll<T, R> {
    fn default() -> Self {
        Self {
            style: Default::default(),
            children: Default::default(),
        }
    }
}
impl<T: 'static, R: Renderer + 'static> WidgetSpecs<T, R> for Scroll<T, R> {
    fn key(&self) -> connui::tree::Key {
        connui::tree::Key::of::<ScrollElement>()
    }

    fn mount(self: Box<Self>) -> connui::tree::Element<T, R> {
        connui::tree::Element::new(self.children, ScrollElement::new(self.style))
    }

    fn update(self: Box<Self>, mut updater: connui::tree::Updater) {
        updater.update(*self, |widget, element: &mut ScrollElement| {
            element.update(widget.style)
        });
    }

    fn children(&mut self) -> Vec<Widget<T, R>> {
        std::mem::take(&mut self.children)
    }
}
impl<T: 'static, R: Renderer + 'static> From<Scroll<T, R>> for Widget<T, R> {
    fn from(value: Scroll<T, R>) -> Self {
        Self::new(value)
    }
}

has_children!({T, R: Renderer} Scroll {T, R} => {T, R} with {children});

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct Overflow: u8 {
        const HORIZONTAL = 0b1;
        const VERTICAL = 0b10;
    }
}
impl Overflow {
    fn horizontal(&mut self, cond: bool) {
        if cond {
            self.insert(Self::HORIZONTAL);
        } else {
            self.remove(Self::HORIZONTAL);
        }
    }

    fn vertical(&mut self, cond: bool) {
        if cond {
            self.insert(Self::VERTICAL);
        } else {
            self.remove(Self::VERTICAL);
        }
    }
}

#[derive(Default)]
struct ViewPort {
    offset: LPosition<i32>,
    size: LSize<u16>,
}

pub struct ScrollElement {
    style: ScrollStyle,
    viewport: ViewPort,
    overflow: Overflow,
}
impl ScrollElement {
    fn new(style: ScrollStyle) -> Self {
        Self {
            style,
            viewport: ViewPort::default(),
            overflow: Overflow::empty(),
        }
    }

    fn update(&mut self, style: ScrollStyle) {
        self.style = style;
    }

    fn check_overflow<'a>(&mut self, layout_element: &'a connui::tree::LayoutElementRef) {
        let mut hor_overflow = false;
        let mut ver_overflow = false;

        let layout_data = layout_element.data();

        layout_element.children(|children| {
            for child in children {
                let child_data = child.data();

                if child_data.rect.x() + child_data.rect.width()
                    > layout_data.rect.x() + layout_data.rect.width()
                {
                    hor_overflow = true;
                }

                if child_data.rect.y() + child_data.rect.height()
                    > layout_data.rect.y() + layout_data.rect.height()
                {
                    ver_overflow = true;
                }
            }
        });

        self.overflow.horizontal(hor_overflow);
        self.overflow.vertical(ver_overflow);
    }
}
impl<T: 'static, R: Renderer + 'static> ElementSpecs<T, R> for ScrollElement {
    fn style(&self) -> &Style {
        &self.style.style
    }

    fn layout(&mut self, layout_element: &connui::tree::LayoutElementRef) {
        self.check_overflow(layout_element);


        let viewport = LayoutElement::new_ref(
            Style::default()
                .layout(self.style.style.layout)
                .padding(self.style.style.padding.clone())
                .positioning(Positioning::Pinned {
                    position: self.viewport.offset,
                    flags: PinnedFlags::RELATIVE_TO_PARENT,
                }),
        );

        let dummy = {
            let data = layout_element.data();
            LayoutElement::new_ref(
                Style::default()
                    .width(data.rect.width().as_unsigned())
                    .height(data.rect.height().as_unsigned()),
            )
        };

        layout_element.swap_children(&viewport);
        layout_element.set_children(vec![viewport, dummy]);
        layout_element.mark_dirty();
    }

    fn mouse_event(
        &mut self,
        mouse_event: connui::event::MouseEvent,
        layout_element: &connui::tree::LayoutElementRef,
    ) -> Response<T> {
        match mouse_event {
            connui::event::MouseEvent::Scroll(delta) => {
                self.viewport.offset.x += delta.x;
                self.viewport.offset.y += delta.y;

                layout_element.mark_dirty();
            }
            _ => {}
        };

        Response::None
    }

    fn window_event(
        &mut self,
        _: connui::event::WindowEvent,
        _: &connui::tree::LayoutElementRef,
    ) -> Option<T> {
        None
    }

    fn input_capture(&self) -> EventKind {
        EventKind::empty()
    }

    fn render(
        &self,
        _: &connui::tree::LayoutElementRef,
        element_children: &[connui::tree::VisualElement<T, R>],
        render_element: connui::tree::RenderElement,
        renderer: &mut R,
    ) {
        renderer.draw_quad(&render_element.rect, Color::GREEN, None);
        if !element_children.is_empty() {
            renderer.push_scissor(&render_element.scissor);
            for child in element_children {
                child.render(renderer);
            }

            renderer.pop_scissor();
        }
    }
}
