//! Tests for [`egui::style::Interaction::drag_to_scroll`].
//!
//! Each test drags a tall [`ScrollArea`] upwards with the mouse and checks how
//! far it scrolled. Headless tests have no touch screen, so
//! [`DragScroll::OnTouch`] behaves as it does for a desktop mouse.

use egui::{
    Pos2, ScrollArea, Sense, Vec2,
    scroll_area::{DragScroll, ScrollSource},
};
use egui_kittest::Harness;

struct State {
    /// The style default under test.
    style_drag: DragScroll,

    /// An explicit [`ScrollArea::scroll_source`], if any.
    explicit: Option<ScrollSource>,

    /// The scroll offset after the last frame.
    offset: Vec2,
}

fn build(style_drag: DragScroll, explicit: Option<ScrollSource>) -> Harness<'static, State> {
    let state = State {
        style_drag,
        explicit,
        offset: Vec2::ZERO,
    };
    let mut harness = Harness::builder()
        .with_size(Vec2::new(300.0, 300.0))
        .with_max_steps(40) // Kinetic scrolling repaints after release.
        .build_ui_state(
            |ui, state: &mut State| {
                ui.style_mut().interaction.drag_to_scroll = state.style_drag;
                let mut area = ScrollArea::vertical().animated(false);
                if let Some(source) = state.explicit {
                    area = area.scroll_source(source);
                }
                let output = area.show(ui, |ui| {
                    // Passive content far taller than the viewport, so only
                    // the scroll area itself can take the drag.
                    ui.allocate_response(Vec2::new(200.0, 2000.0), Sense::hover());
                });
                state.offset = output.state.offset;
            },
            state,
        );
    // A scroll area only knows its content is too large from the previous frame.
    harness.run_steps(2);
    harness
}

/// Drag the pointer 100pt upwards over several frames, then release.
fn drag_up(harness: &mut Harness<'_, State>) {
    let from = Pos2::new(100.0, 200.0);
    let to = Pos2::new(100.0, 100.0);
    harness.drag_at(from);
    harness.run_steps(2);
    harness.hover_at(Pos2::new(100.0, 150.0));
    harness.run_steps(2);
    harness.hover_at(to);
    harness.run_steps(2);
    harness.drop_at(to);
    harness.run_steps(2);
}

#[test]
fn on_touch_style_ignores_mouse_drag() {
    let mut harness = build(DragScroll::OnTouch, None);
    drag_up(&mut harness);
    assert_eq!(
        harness.state().offset.y,
        0.0,
        "the default OnTouch style should not scroll on a mouse drag"
    );
}

#[test]
fn always_style_scrolls_on_mouse_drag() {
    let mut harness = build(DragScroll::Always, None);
    drag_up(&mut harness);
    assert!(
        harness.state().offset.y >= 100.0,
        "an Always style should scroll on a mouse drag (offset = {:?})",
        harness.state().offset
    );
}

#[test]
fn explicit_scroll_source_overrides_style() {
    let never = ScrollSource {
        drag: DragScroll::Never,
        ..Default::default()
    };
    let mut harness = build(DragScroll::Always, Some(never));
    drag_up(&mut harness);
    assert_eq!(
        harness.state().offset.y,
        0.0,
        "a scroll area's own ScrollSource should win over the style"
    );
}
