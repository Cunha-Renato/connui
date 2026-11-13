use crate::{
    Callback,
    style::{StyleRules, StyleSelector},
};
use connui::{
    renderer::Renderer,
    tree::{
        ElementSpecs, LayoutElement, LayoutElementRef, LayoutElementWeak, Style, Widget,
        WidgetSpecs, WidgetStateFlags,
    },
    types::*,
};
use style::*;
use track_value::TrackValue;

pub mod style;
pub mod track_value;

pub struct Slider<TV: TrackValue, T> {
    style: StyleSelector<'static, SliderStyle>,
    track_style: StyleSelector<'static, SliderTrackStyle>,
    track_rail_style: StyleSelector<'static, SliderTrackStyle>,
    thumb_style: StyleSelector<'static, SliderThumbStyle>,
    on_change: Option<Callback<TV::Value, Option<T>>>,
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

    pub fn style(mut self, style: StyleSelector<'static, SliderStyle>) -> Self {
        self.style = style;
        self
    }

    pub fn style_base(mut self, base: SliderStyle) -> Self {
        self.style.base = base;
        self
    }

    pub fn style_rules(mut self, rules: StyleRules<'static, SliderStyle>) -> Self {
        self.style.rules = rules;
        self
    }

    pub fn track_style(mut self, style: StyleSelector<'static, SliderTrackStyle>) -> Self {
        self.track_style = style;
        self
    }

    pub fn track_base(mut self, base: SliderTrackStyle) -> Self {
        self.track_style.base = base;
        self
    }

    pub fn track_rules(mut self, rules: StyleRules<'static, SliderTrackStyle>) -> Self {
        self.track_style.rules = rules;

        self
    }

    pub fn track_rail_style(mut self, style: StyleSelector<'static, SliderTrackStyle>) -> Self {
        self.track_rail_style = style;
        self
    }

    pub fn track_rail_base(mut self, base: SliderTrackStyle) -> Self {
        self.track_rail_style.base = base;
        self
    }

    pub fn track_rail_rules(mut self, rules: StyleRules<'static, SliderTrackStyle>) -> Self {
        self.track_rail_style.rules = rules;

        self
    }

    pub fn thumb_style(mut self, style: StyleSelector<'static, SliderThumbStyle>) -> Self {
        self.thumb_style = style;
        self
    }

    pub fn thumb_base(mut self, base: SliderThumbStyle) -> Self {
        self.thumb_style.base = base;
        self
    }

    pub fn thumb_rules(mut self, rules: StyleRules<'static, SliderThumbStyle>) -> Self {
        self.thumb_style.rules = rules;

        self
    }

    pub fn on_change<F>(mut self, on_change: F) -> Self
    where
        F: Fn(TV::Value) -> Option<T> + 'static + Sync,
    {
        self.on_change = Some(on_change.into());
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
            updater.mark_dirty();
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
    on_change: Option<Callback<TV::Value, Option<T>>>,
    track: LayoutElementWeak,
    track_rail: LayoutElementWeak,
    thumb: LayoutElementWeak,
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
            WidgetStateFlags::DISABLED
        };

        Self {
            style: widget.style.get(state).as_style(),
            track_style: widget.track_style.get(state),
            track_rail_style: widget.track_rail_style.get(state),
            thumb_style: widget.thumb_style.get(state),
            track_value: widget.track_value,
            on_change: widget.on_change,
            track: LayoutElementWeak::new(),
            track_rail: LayoutElementWeak::new(),
            thumb: LayoutElementWeak::new(),
            step: widget.step,
            pct,
            state,
        }
    }

    fn update_from_widget(&mut self, widget: Slider<TV, T>) -> bool {
        let mut relayout = false;

        // Widget just got inactive / active.
        if widget.active && self.state.contains(WidgetStateFlags::DISABLED) {
            self.state.set_default();

            relayout = true;
        } else if !widget.active && !self.state.contains(WidgetStateFlags::DISABLED) {
            self.state.set_disabled();

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
        let mut new_pct = widget.set_value.map_or(self.pct, |set_value| {
            Self::pct_from_value(&widget.track_value, set_value, widget.step)
        });

        // Maybe this is not that good to have.
        new_pct = widget
            .track_value
            .pct(widget.track_value.value(widget.step, new_pct));

        relayout |= !Self::eq_pct(self.pct, new_pct);

        self.on_change = widget.on_change;
        self.track_value = widget.track_value;
        self.step = widget.step;
        self.pct = new_pct;

        relayout
    }

    fn on_change(
        &mut self,
        layout_element: &connui::tree::LayoutElementRef,
        position: LPosition<i32>,
    ) -> Option<T> {
        if self.calculate_pct(position.map(|p| p.as_float()), layout_element) {
            self.update_layout(layout_element);

            self.on_change
                .as_ref()
                .and_then(|oc| oc.call(self.track_value.value(self.step, self.pct)))
        } else {
            None
        }
    }

    fn calculate_pct(
        &mut self,
        mouse_pos: Position<LPixel<f32>>,
        layout_element: &connui::tree::LayoutElementRef,
    ) -> bool {
        let layout_borrow = layout_element.borrow();

        let axis = self.style.layout.axis;
        let half_thumb = self.thumb_style.size.main.as_float().inner() / 2.0;

        let rect_pos = axis
            .main(&(layout_borrow.rect.x(), layout_borrow.rect.y()))
            .inner();
        let rect_size = axis
            .main(&(layout_borrow.rect.width(), layout_borrow.rect.height()))
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

    fn get_sizes(&self, layout_element: &connui::tree::LayoutElementData) -> (u16, u16) {
        let axis = self.style.layout.axis;

        let half_thumb = self.thumb_style.size.main.as_float().inner() / 2.0;
        let rect_main_size = (axis
            .main(&(layout_element.rect.width(), layout_element.rect.height()))
            .inner())
        .max(0.0);

        (
            (half_thumb..=(rect_main_size - half_thumb).abs())
                .value(None, self.pct)
                .round() as u16,
            rect_main_size.round() as u16,
        )
    }

    fn update_layout(&mut self, layout_element: &connui::tree::LayoutElementRef) {
        let mut layout_borrow_mut = layout_element.borrow_mut();

        let axis = self.style.layout.axis;
        let (track_size, avail_size) = self.get_sizes(&layout_borrow_mut);

        if let Some(track) = self.track.upgrade() {
            let mut borrow_mut = track.borrow_mut();

            *axis.main_mut(&mut borrow_mut.rect.size) = (track_size as f32).into();
        }

        if let Some(track_rail) = self.track_rail.upgrade() {
            let mut borrow_mut = track_rail.borrow_mut();

            let size = (avail_size as f32 - track_size as f32).max(0.0).round();
            *axis.main_mut(&mut borrow_mut.rect.size) = size.into();
        }

        layout_borrow_mut.mark_dirty_position();
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

    fn pre_layout(&mut self, layout_element: &connui::tree::LayoutElementRef) {
        let mut layout_borrow_mut = layout_element.borrow_mut();

        let (track_size, _) = self.get_sizes(&layout_borrow_mut);
        let axis = self.style.layout.axis;

        let track_style = self.track_style.as_style(track_size.into(), axis);
        let track_rail_style = self.track_rail_style.as_style(Sizing::fill_default(), axis);
        let thumb_style = self.thumb_style.as_style(axis);

        let track = LayoutElement::new_ref(track_style);
        let track_rail = LayoutElement::new_ref(track_rail_style);
        let thumb = LayoutElement::new_ref(thumb_style);

        self.track = LayoutElementRef::downgrade(&track);
        self.track_rail = LayoutElementRef::downgrade(&track_rail);
        self.thumb = LayoutElementRef::downgrade(&thumb);

        layout_borrow_mut.set_children(vec![track, thumb, track_rail]);
    }

    fn post_layout(&mut self, layout_element: &LayoutElementRef) {
        self.update_layout(layout_element);
    }

    fn mouse_event(
        &mut self,
        event: connui::event::internal::MouseEvent,
        layout_element: &connui::tree::LayoutElementRef,
    ) -> connui::prelude::Response<T> {
        use connui::event::{internal::*, *};

        if self.state.contains(WidgetStateFlags::DISABLED) {
            return Response::empty();
        }

        match event.event {
            MouseEventKind::Enter => self.state.set_hover(true),
            MouseEventKind::Left => self.state.set_hover(false),
            MouseEventKind::Move if self.state.contains(WidgetStateFlags::ACTIVE) => {
                if let Some(position) = event.position
                    && let Some(message) = self.on_change(layout_element, position)
                {
                    return message.into();
                }
            }
            MouseEventKind::Button { button, state } => {
                if button.contains(MouseButton::LEFT)
                    && let ButtonState::Pressed(true) = state
                {
                    self.state.set_active();

                    if let Some(position) = event.position
                        && let Some(message) = self.on_change(layout_element, position)
                    {
                        return message.into();
                    }
                } else {
                    self.state.set_default();
                }
            }
            _ => return Response::empty(),
        };

        Response::empty()
    }

    fn window_event(
        &mut self,
        _: connui::event::WindowEvent,
        _: &connui::tree::LayoutElementRef,
    ) -> Option<T> {
        None
    }

    fn input_capture(&self) -> connui::event::internal::EventKind {
        use connui::event::internal::EventKind;

        if self.state.contains(WidgetStateFlags::ACTIVE) {
            EventKind::MOVE | EventKind::BUTTON
        } else {
            EventKind::empty()
        }
    }

    // We don't have any element_children, since all of the children are only LayoutElements.
    fn render(
        &self,
        _: &connui::tree::LayoutElementRef,
        _: &[connui::tree::VisualElement<T, R>],
        render_element: connui::tree::RenderElement,
        renderer: &mut R,
    ) {
        renderer.push_scissor(&render_element.scissor);

        if let Some(track) = self.track.upgrade() {
            let render_element = track.render_element(renderer);
            renderer.draw_quad(&render_element.rect, self.track_style.color, None);
        }

        if let Some(track_rail) = self.track_rail.upgrade() {
            let render_element = track_rail.render_element(renderer);
            renderer.draw_quad(&render_element.rect, self.track_rail_style.color, None);
        }

        if let Some(thumb) = self.thumb.upgrade() {
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

            let mut thumb_render_element = thumb.render_element(renderer);

            thumb_render_element.rect.size = thumb_size.into();
            thumb_render_element.rect.position.x -= thumb_position_offset.0;
            thumb_render_element.rect.position.y -= thumb_position_offset.1;

            renderer.draw_quad(&thumb_render_element.rect, self.thumb_style.color, None);
        }

        renderer.pop_scissor();
    }
}
