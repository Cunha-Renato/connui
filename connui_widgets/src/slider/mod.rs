use connui::{
    event::EventKind,
    renderer::Renderer,
    tree::{ElementSpecs, LayoutElement, Style, Widget, WidgetSpecs},
    types::*,
};

use crate::defaults;

use style::*;
use track_value::TrackValue;

pub mod style;
pub mod track_value;

pub struct Slider<TV: TrackValue> {
    style: SliderStates<SliderStyle>,
    track_style: SliderStates<SliderTrackStyle>,
    track_rail_style: SliderStates<SliderTrackStyle>,
    thumb_style: SliderStates<SliderThumbStyle>,
    track_value: TV,
    set_value: Option<TV::Value>,
    step: Option<TV::Value>,
    active: bool,
}
impl<TV: TrackValue> Slider<TV> {
    pub const fn new(track_value: TV) -> Self {
        Self {
            style: defaults::slider::STYLE,
            track_style: defaults::slider::TRACK_STYLE,
            track_rail_style: defaults::slider::TRACK_RAIL_STYLE,
            thumb_style: defaults::slider::THUMB_STYLE,
            track_value,
            set_value: None,
            step: None,
            active: true,
        }
    }

    pub const fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }
}
impl<TV: TrackValue + 'static, T, R: Renderer> WidgetSpecs<T, R> for Slider<TV> {
    fn key(&self) -> connui::tree::Key {
        connui::tree::Key::of::<SliderElement<TV>>()
    }

    fn mount(self: Box<Self>) -> connui::tree::Element<T, R> {
        connui::tree::Element::new([], SliderElement::new(*self))
    }

    fn update(self: Box<Self>, mut updater: connui::tree::Updater) {
        let mut relayout = false;

        updater.update(*self, |widget, element: &mut SliderElement<TV>| {
            relayout = element.update_from_widget(widget);
        });

        if relayout {
            updater.relayout();
        }
    }
}
impl<TV: TrackValue + 'static, T, R: Renderer> From<Slider<TV>> for Widget<T, R> {
    fn from(value: Slider<TV>) -> Self {
        Self::new(value)
    }
}

pub struct SliderElement<TV: TrackValue> {
    style: Style,
    track_style: SliderTrackStyle,
    track_rail_style: SliderTrackStyle,
    thumb_style: SliderThumbStyle,
    track_value: TV,
    step: Option<TV::Value>,
    pct: f32,
    state: SliderState,
}
impl<TV: TrackValue> SliderElement<TV> {
    fn new(widget: Slider<TV>) -> Self {
        let pct = widget.set_value.map_or(0.0, |set_value| {
            Self::pct_from_value(&widget.track_value, set_value, widget.step)
        });

        let inner_state = if widget.active {
            SliderInnerState::Default
        } else {
            SliderInnerState::Inactive
        };

        let state = SliderState {
            inner: inner_state,
            hover: false,
        };

        Self {
            style: widget.style.get(state).as_style(),
            track_style: widget.track_style.get(state),
            track_rail_style: widget.track_rail_style.get(state),
            thumb_style: widget.thumb_style.get(state),
            track_value: widget.track_value,
            step: widget.step,
            pct,
            state: state,
        }
    }

    fn update_from_widget(&mut self, widget: Slider<TV>) -> bool {
        let mut relayout = false;

        // Widget just got inactive / active.
        if widget.active && self.state.inner == SliderInnerState::Inactive {
            self.state.inner = SliderInnerState::Default;
            relayout = true;
        } else if !widget.active && self.state.inner != SliderInnerState::Inactive {
            self.state.inner = SliderInnerState::Inactive;
            relayout = true;
        }

        // Styles.
        let track_style = widget.track_style.get(self.state);
        let track_rail_style = widget.track_rail_style.get(self.state);
        let thumb_style = widget.thumb_style.get(self.state);

        relayout |= self.track_style.should_relayout(&track_style)
            || self.track_rail_style.should_relayout(&track_rail_style)
            || self.thumb_style.should_relayout(&thumb_style);

        self.style = widget.style.get(self.state).as_style();
        self.track_style = track_style;
        self.track_rail_style = track_rail_style;
        self.thumb_style = thumb_style;

        // TrackValue
        let widget_pct = widget.set_value.map_or(self.pct, |set_value| {
            Self::pct_from_value(&widget.track_value, set_value, widget.step)
        });

        relayout |= !Self::eq_pct(self.pct, widget_pct);

        self.track_value = widget.track_value;
        self.step = widget.step;
        self.pct = widget_pct;

        relayout
    }

    fn calculate_pct(
        &mut self,
        mouse_pos: Position<LPixel<f32>>,
        layout_element: &connui::tree::LayoutElement,
    ) -> bool {
        let axis = self.style.layout.axis;
        let half_thumb = self.thumb_style.size.main.as_float().inner() / 2.0;

        let rect_pos = axis
            .main(&(layout_element.rect.x(), layout_element.rect.y()))
            .inner();
        let rect_size = axis
            .main(&(layout_element.rect.width(), layout_element.rect.height()))
            .inner();

        let mouse = axis.main(&mouse_pos).inner();
        let mouse_pct = (rect_pos + half_thumb..=rect_pos + rect_size - half_thumb)
            .pct(mouse)
            .clamp(0.0, 1.0);
        let pct = Self::pct_clamped(&self.track_value, self.step, mouse_pct);

        let changed = !Self::eq_pct(self.pct, pct);

        self.pct = pct;

        changed
    }

    fn eq_pct(a: f32, b: f32) -> bool {
        (a - b).abs() <= f32::EPSILON
    }

    fn pct_clamped(tv: &TV, step: Option<TV::Value>, pct: f32) -> f32 {
        tv.pct(tv.value(step, pct))
    }

    fn pct_from_value(tv: &TV, value: TV::Value, step: Option<TV::Value>) -> f32 {
        tv.pct(tv.value(step, tv.pct(value)))
    }
}
impl<TV: TrackValue + 'static, T, R: Renderer> ElementSpecs<T, R> for SliderElement<TV> {
    fn style(&self) -> &connui::tree::Style {
        &self.style
    }

    fn layout(&mut self, mut context: connui::tree::LayoutContext) {
        let children = {
            let layout_element = context.layout_element();
            let axis = self.style.layout.axis;

            let half_thumb = self.thumb_style.size.main.as_float().inner() / 2.0;
            let rect_main_size = (axis
                .main(&(layout_element.rect.width(), layout_element.rect.height()))
                .inner())
            .max(0.0);
            let thumb_main_pos = (half_thumb..=rect_main_size - half_thumb)
                .value(None, self.pct)
                .round() as u16;

            let track_style = self
                .track_style
                .as_style(LPixel::new(thumb_main_pos).into(), axis);
            let track_rail_style = self.track_rail_style.as_style(Sizing::fill_default(), axis);
            let thumb_style = self.thumb_style.as_style(axis);

            [
                LayoutElement::new(track_style),
                LayoutElement::new(thumb_style),
                LayoutElement::new(track_rail_style),
            ]
        };

        context.set_children(children);
        context.relayout();
    }

    fn mouse_event(
        &mut self,
        event: connui::event::MouseEvent,
        mut context: connui::tree::LayoutContext,
    ) -> connui::prelude::Response<T> {
        use connui::event::MouseButton;

        if self.state.inner == SliderInnerState::Inactive {
            return Response::None;
        }

        match event {
            connui::event::MouseEvent::Enter => self.state.hover = true,
            connui::event::MouseEvent::Left => self.state.hover = false,
            connui::event::MouseEvent::Move(position)
                if self.state.inner == SliderInnerState::Active =>
            {
                if self.calculate_pct(position.map(|p| p.as_float()), context.layout_element()) {
                    context.relayout();
                }
            }
            connui::event::MouseEvent::Press {
                position,
                button: MouseButton::LEFT,
            } => {
                if let Some(position) = position
                    && self.calculate_pct(position.map(|p| p.as_float()), context.layout_element())
                {
                    context.relayout();
                }

                self.state.inner = SliderInnerState::Active;
            }
            connui::event::MouseEvent::Release {
                button: MouseButton::LEFT,
                ..
            } => self.state.inner = SliderInnerState::Default,
            _ => {
                return Response::None;
            }
        };

        Response::ConsumedEmpty
    }

    fn window_event(
        &mut self,
        _: connui::event::WindowEvent,
        _: connui::tree::LayoutContext,
    ) -> Option<T> {
        None
    }

    fn input_capture(&self) -> connui::event::EventKind {
        use connui::event::EventKind;

        if let SliderInnerState::Active = self.state.inner {
            EventKind::MOVE | EventKind::BUTTON
        } else {
            EventKind::empty()
        }
    }

    // We don't have any element_children, since all of the children are only LayoutElements.
    fn render(
        &self,
        context: connui::tree::LayoutContextRef,
        _: &[connui::tree::VisualElement<T, R>],
        render_element: connui::tree::RenderElement,
        renderer: &mut R,
    ) {
        renderer.push_scissor(&render_element.scissor);
        renderer.draw_quad(&render_element.rect, Color::TRANSPARENT, None);
        for (idx, child) in context.children().enumerate() {
            // Skip thumb because it needs do be drawn latter.
            if idx == 1 {
                continue;
            }

            let color = match idx {
                0 => self.track_style.color,
                2 => self.track_rail_style.color,
                _ => unreachable!(),
            };

            let child_render_element = child.render_element(renderer);
            renderer.draw_quad(&child_render_element.rect, color, None);
        }

        // Drawing the thumb.
        if let Some(thumb_element) = context.get_child(1) {
            let axis = self.style.layout.axis;
            let thumb_size_float = self
                .thumb_style
                .size
                .map(|s| s.as_float().to_physical(renderer.scale_factor()));
            let thumb_size = axis.horizontal_vertical(&thumb_size_float);
            let thumb_position_offset = axis.horizontal_vertical(&RelativeValue::new(
                thumb_size_float.main / PPixel::new(2.0),
                PPixel::new(0.0),
            ));

            let mut thumb_render_element = thumb_element.render_element(renderer);

            thumb_render_element.rect.size = thumb_size.into();
            thumb_render_element.rect.position.x -= thumb_position_offset.0;
            thumb_render_element.rect.position.y -= thumb_position_offset.1;

            renderer.draw_quad(&thumb_render_element.rect, self.thumb_style.color, None);
        }

        renderer.pop_scissor();
    }
}
