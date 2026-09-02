use connui::{event::*, renderer::Renderer, tree::*, types::*};

pub struct Slider;
impl<T, R: Renderer> WidgetSpecs<T, R> for Slider {
    fn key(&self) -> Key {
        Key::of::<SliderElement>()
    }

    fn mount(self: Box<Self>) -> Element<T, R> {
        Element::new(vec![], SliderElement::default())
    }

    fn update(self: Box<Self>, updater: Updater) {}
}
impl<T, R: Renderer> From<Slider> for Widget<T, R> {
    #[inline]
    fn from(value: Slider) -> Self {
        Self::new(value)
    }
}

pub struct SliderElement {
    style: Style,
    pct: f32,
    capture: EventKind,
    active: bool,
    relayout: bool,
}
impl SliderElement {
    fn calculate_pct(&mut self, point: LPoint<f32>, mut context: LayoutContext) {
        if self.active {
            let rect = &context.layout_element().rect;
            let cursor_x = point.x.clamp(rect.x(), rect.x() + rect.width());

            self.pct = ((cursor_x - rect.x()) / rect.width()).inner();

            self.relayout = true;
            context.relayout();
        }
    }
}
impl Default for SliderElement {
    fn default() -> Self {
        println!("DEFAULT");
        Self {
            style: Style {
                size: Size::new(200.into(), 50.into()),
                ..Default::default()
            },
            pct: 0.0,
            capture: EventKind::empty(),
            active: false,
            relayout: true,
        }
    }
}
impl<T, R: Renderer> ElementSpecs<T, R> for SliderElement {
    fn style(&self) -> &Style {
        &self.style
    }

    fn layout(&mut self, mut context: LayoutContext) {
        if !self.relayout {
            return;
        }

        let pct = self.pct.clamp(0.0, 1.0);

        let avail_width = context.layout_element().rect.width().inner();
        let left_width = LPixel::new(avail_width * pct).as_unsigned();
        let right_width = LPixel::new(avail_width - left_width.as_float().inner()).as_unsigned();
        let cursor_x = (left_width.as_signed() - LPixel::new(5))
            .clamp(0.into(), (left_width + right_width).as_signed() - 10.into());
        let height = SizeOp::Fill {
            min: 0.into(),
            max: u16::MAX.into(),
            initial: 0.into(),
        };

        let left_style = Style {
            size: Size::new(left_width.into(), height),
            ..Default::default()
        };

        let right_style = Style {
            size: Size::new(right_width.into(), height),
            ..Default::default()
        };

        let cursor_style = Style {
            size: Size::new(10.into(), self.style.size.height),
            position: Position::Pinned {
                position: Point::new(cursor_x, 0.into()),
                flags: PinnedFlags::PARENT_RELATIVE,
            },
            ..Default::default()
        };

        context.set_children([
            LayoutElement::new(left_style),
            LayoutElement::new(right_style),
            LayoutElement::new(cursor_style),
        ]);

        self.relayout = false;
        context.relayout();
    }

    fn mouse_event(&mut self, event: MouseEvent, context: LayoutContext) -> Response<T> {
        println!("{}", self.pct);
        match event {
            MouseEvent::Press(MouseButton::LEFT) => {
                self.active = true;
                self.capture = EventKind::MOVE | EventKind::BUTTON;
            }
            MouseEvent::Release(MouseButton::LEFT) => {
                self.active = false;
                self.capture = EventKind::empty();
            }
            MouseEvent::Move(point) => self.calculate_pct(point.map(|p| p.as_float()), context),

            _ => {}
        }

        Response::ConsumedEmpty
    }

    fn window_event(&mut self, _: WindowEvent, _: LayoutContext) -> Option<T> {
        None
    }

    fn input_capture(&self) -> EventKind {
        self.capture
    }

    fn render(
        &self,
        context: LayoutContextRef,
        _: &[VisualElement<T, R>],
        render_element: RenderElement,
        renderer: &mut R,
    ) {
        renderer.push_scissor(&render_element.scissor);
        let colors = [Color::CYAN, Color::YELLOW, Color::BLACK];
        for idx in 0..3 {
            if let Some(child) = context.get_child(idx) {
                let rel = child.render_element(renderer);
                renderer.draw_quad(&rel.rect, colors[idx], None);
            }
        }
        renderer.pop_scissor();
    }
}
