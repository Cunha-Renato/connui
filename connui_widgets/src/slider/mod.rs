use connui::{
    event::EventKind,
    renderer::Renderer,
    tree::{ElementSpecs, Style, WidgetSpecs},
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
        let pct = widget
            .set_value
            .map_or(0.0, |set_value| widget.track_value.set_value(set_value));

        let state = if !widget.active {
            SliderState::Default
        } else {
            SliderState::Inactive
        };

        Self {
            style: widget.style.get(state).as_style(),
            track_style: *widget.track_style.get(state),
            track_rail_style: *widget.track_rail_style.get(state),
            thumb_style: *widget.thumb_style.get(state),
            track_value: widget.track_value,
            step: widget.step,
            pct,
            state: state,
        }
    }

    fn update_from_widget(&mut self, widget: Slider<TV>) -> bool {
        if widget.track_value != self.track_value
            || widget.step != self.step
            || widget
                .set_value
                .is_some_and(|set_value| widget.track_value.set_value(set_value) != self.pct)
        {
            *self = Self::new(widget);

            true
        } else {
            false
        }
    }

    fn calculate_pct(
        &mut self,
        mouse_pos: Position<LPixel<f32>>,
        layout_element: &connui::tree::LayoutElement,
    ) -> bool {
        let mut main_size = self
            .style
            .layout
            .axis
            .main(&(layout_element.rect.width(), layout_element.rect.height()))
            .inner();

        let mut main_position = self
            .style
            .layout
            .axis
            .main(&(layout_element.rect.x(), layout_element.rect.y()))
            .inner();

        let half_main_thumb = self.thumb_style.size.main.as_float().inner() / 2.0;

        main_size -= half_main_thumb;
        main_position += half_main_thumb;

        let main_m_position = self
            .style
            .layout
            .axis
            .main(&mouse_pos)
            .inner()
            .clamp(main_position, main_position + main_size);

        let pct = self.track_value.set_value(
            self.track_value
                .get_value(self.step, (main_m_position - main_position) / main_size),
        );

        let relayout = (self.pct - pct).abs() > f32::EPSILON;

        self.pct = pct;

        relayout
    }
}
impl<TV: TrackValue + 'static, T, R: Renderer> ElementSpecs<T, R> for SliderElement<TV> {
    fn style(&self) -> &connui::tree::Style {
        &self.style
    }

    fn layout(&mut self, context: connui::tree::LayoutContext) {
        todo!()
    }

    fn mouse_event(
        &mut self,
        event: connui::event::MouseEvent,
        mut context: connui::tree::LayoutContext,
    ) -> connui::prelude::Response<T> {
        use connui::event::MouseButton;

        if self.state == SliderState::Inactive {
            return Response::None;
        }

        self.state = match event {
            connui::event::MouseEvent::Enter if self.state == SliderState::Default => {
                SliderState::Hover
            }
            connui::event::MouseEvent::Left if self.state == SliderState::Hover => {
                SliderState::Default
            }
            connui::event::MouseEvent::Move(position) if self.state == SliderState::Active => {
                if self.calculate_pct(position.map(|p| p.as_float()), context.layout_element()) {
                    context.relayout();
                }

                self.state
            }
            connui::event::MouseEvent::Press(MouseButton::LEFT)
                if self.state == SliderState::Hover =>
            {
                SliderState::Active
            }
            connui::event::MouseEvent::Release(MouseButton::LEFT)
                if self.state == SliderState::Active =>
            {
                SliderState::Hover
            }
            _ => self.state,
        };

        todo!()
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

        if let SliderState::Active = self.state {
            EventKind::MOVE | EventKind::BUTTON
        } else {
            EventKind::empty()
        }
    }

    fn render(
        &self,
        context: connui::tree::LayoutContextRef,
        element_children: &[connui::tree::VisualElement<T, R>],
        render_element: connui::tree::RenderElement,
        renderer: &mut R,
    ) {
        todo!()
    }
}
