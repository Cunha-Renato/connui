use connui::{
    event::EventKind,
    has_positioning,
    renderer::Renderer,
    tree::{ElementSpecs, LayoutElement, Style, Widget, WidgetSpecs},
    types::*,
};

use crate::{
    Callback, WidgetStateFlags,
    style::{StyleRules, StyleSelector},
};

use style::*;
use track_value::TrackValue;

pub mod style;
pub mod track_value;

pub type OnChangeFn<V, T> = Box<dyn Fn(V) -> Option<T>>;

pub struct Slider<TV: TrackValue, T> {
    style: StyleSelector<'static, SliderStyle>,
    track_style: StyleSelector<'static, SliderTrackStyle>,
    track_rail_style: StyleSelector<'static, SliderTrackStyle>,
    thumb_style: StyleSelector<'static, SliderThumbStyle>,
    on_change: Option<OnChangeFn<TV::Value, T>>,
    track_value: TV,
    set_value: Option<TV::Value>,
    step: Option<TV::Value>,
    active: bool,
}
impl<TV: TrackValue, T> Slider<TV, T> {
    pub const fn new(track_value: TV) -> Self {
        use crate::style::default;

        Self {
            style: default::slider::STYLE_SELECTOR,
            track_style: default::slider::TRACK_STYLE_SELECTOR,
            track_rail_style: default::slider::TRACK_RAIL_STYLE_SELECTOR,
            thumb_style: default::slider::THUMB_STYLE_SELECTOR,
            track_value,
            set_value: None,
            step: None,
            on_change: None,
            active: true,
        }
    }

    pub fn style(
        mut self,
        base: Option<SliderStyle>,
        rules: StyleRules<'static, Callback<SliderStyle>>,
    ) -> Self {
        self.style = StyleSelector::new(base.unwrap_or_else(|| self.style.base()), rules);

        self
    }

    pub fn track_style(
        mut self,
        base: Option<SliderTrackStyle>,
        rules: StyleRules<'static, Callback<SliderTrackStyle>>,
    ) -> Self {
        self.track_style =
            StyleSelector::new(base.unwrap_or_else(|| self.track_style.base()), rules);

        self
    }

    pub fn track_rail_style(
        mut self,
        base: Option<SliderTrackStyle>,
        rules: StyleRules<'static, Callback<SliderTrackStyle>>,
    ) -> Self {
        self.track_rail_style =
            StyleSelector::new(base.unwrap_or_else(|| self.track_rail_style.base()), rules);

        self
    }

    pub fn thumb_style(
        mut self,
        base: Option<SliderThumbStyle>,
        rules: StyleRules<'static, Callback<SliderThumbStyle>>,
    ) -> Self {
        self.thumb_style =
            StyleSelector::new(base.unwrap_or_else(|| self.thumb_style.base()), rules);

        self
    }

    pub fn on_change<F>(mut self, on_change: F) -> Self
    where
        F: Fn(TV::Value) -> Option<T> + 'static,
    {
        self.on_change = Some(Box::new(on_change));
        self
    }

    pub const fn value(mut self, value: TV::Value) -> Self {
        self.set_value = Some(value);
        self
    }

    pub const fn step(mut self, step: TV::Value) -> Self {
        self.step = Some(step);
        self
    }

    pub const fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }
}
impl<TV: TrackValue + 'static, T: 'static, R: Renderer> WidgetSpecs<T, R> for Slider<TV, T> {
    fn key(&self) -> connui::tree::Key {
        connui::tree::Key::of::<SliderElement<TV, T>>()
    }

    fn mount(self: Box<Self>) -> connui::tree::Element<T, R> {
        connui::tree::Element::new([], SliderElement::new(*self))
    }

    fn update(self: Box<Self>, mut updater: connui::tree::Updater) {
        let mut relayout = false;

        updater.update(*self, |widget, element: &mut SliderElement<TV, T>| {
            relayout = element.update_from_widget(widget);
        });

        if relayout {
            updater.relayout();
        }
    }
}
impl<TV: TrackValue + 'static, T: 'static, R: Renderer> From<Slider<TV, T>> for Widget<T, R> {
    fn from(value: Slider<TV, T>) -> Self {
        Self::new(value)
    }
}

pub struct SliderElement<TV: TrackValue, T> {
    style: Style,
    track_style: SliderTrackStyle,
    track_rail_style: SliderTrackStyle,
    thumb_style: SliderThumbStyle,
    on_change: Option<OnChangeFn<TV::Value, T>>,
    track_value: TV,
    step: Option<TV::Value>,
    pct: f32,
    state: WidgetStateFlags,
}
impl<TV: TrackValue, T> SliderElement<TV, T> {
    fn new(widget: Slider<TV, T>) -> Self {
        let pct = widget.set_value.map_or(0.0, |set_value| {
            Self::pct_from_value(&widget.track_value, set_value, widget.step)
        });

        let state = if widget.active {
            WidgetStateFlags::DEFAULT
        } else {
            WidgetStateFlags::INACTIVE
        };

        Self {
            style: widget.style.get(state).as_style(),
            track_style: widget.track_style.get(state),
            track_rail_style: widget.track_rail_style.get(state),
            thumb_style: widget.thumb_style.get(state),
            track_value: widget.track_value,
            on_change: widget.on_change,
            step: widget.step,
            pct,
            state,
        }
    }

    fn update_from_widget(&mut self, widget: Slider<TV, T>) -> bool {
        let mut relayout = false;

        // Widget just got inactive / active.
        if widget.active && self.state.contains(WidgetStateFlags::INACTIVE) {
            self.state.set_default();

            relayout = true;
        } else if !widget.active && !self.state.contains(WidgetStateFlags::INACTIVE) {
            self.state.set_inactive();

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
        self.on_change = widget.on_change;

        relayout
    }

    fn on_change(
        &mut self,
        context: &mut connui::tree::LayoutContext,
        position: Option<LPosition<i32>>,
    ) -> Option<T> {
        if let Some(position) = position
            && self.calculate_pct(position.map(|p| p.as_float()), context.layout_element())
        {
            context.relayout();

            self.on_change
                .as_ref()
                .map(|oc| oc(self.track_value.value(self.step, self.pct)))
                .flatten()
        } else {
            None
        }
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
impl<TV: TrackValue + 'static, T: 'static, R: Renderer> ElementSpecs<T, R>
    for SliderElement<TV, T>
{
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

        if self.state.contains(WidgetStateFlags::INACTIVE) {
            return Response::None;
        }

        match event {
            connui::event::MouseEvent::Enter => self.state.set_hover(true),
            connui::event::MouseEvent::Left => self.state.set_hover(false),
            connui::event::MouseEvent::Move(position)
                if self.state.contains(WidgetStateFlags::ACTIVE) =>
            {
                if let Some(message) = self.on_change(&mut context, Some(position)) {
                    return Response::Consumed(message);
                }
            }
            connui::event::MouseEvent::Press {
                position,
                button: MouseButton::LEFT,
            } => {
                self.state.set_active();

                if let Some(message) = self.on_change(&mut context, position) {
                    return Response::Consumed(message);
                }
            }
            connui::event::MouseEvent::Release {
                button: MouseButton::LEFT,
                ..
            } => self.state.set_default(),
            _ => {
                return Response::None;
            }
        };

        println!("{:?}", self.state);

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

        if self.state.contains(WidgetStateFlags::ACTIVE) {
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
