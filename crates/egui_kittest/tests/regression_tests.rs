use egui::accesskit::Role;
use egui::{Button, ComboBox, Image, Vec2, Widget};
use egui_kittest::{kittest::Queryable, Harness, SnapshotResults};

#[test]
pub fn focus_should_skip_over_disabled_buttons() {
    let mut harness = Harness::new_ui(|ui| {
        ui.add(Button::new("Button 1"));
        ui.add_enabled(false, Button::new("Button Disabled"));
        ui.add(Button::new("Button 3"));
    });

    harness.press_key(egui::Key::Tab);
    harness.run();

    let button_1 = harness.get_by_label("Button 1");
    assert!(button_1.is_focused());

    harness.press_key(egui::Key::Tab);
    harness.run();

    let button_3 = harness.get_by_label("Button 3");
    assert!(button_3.is_focused());

    harness.press_key(egui::Key::Tab);
    harness.run();

    let button_1 = harness.get_by_label("Button 1");
    assert!(button_1.is_focused());
}

#[test]
fn image_failed() {
    let mut harness = Harness::new_ui(|ui| {
        Image::new("file://invalid/path")
            .alt_text("I have an alt text")
            .max_size(Vec2::new(100.0, 100.0))
            .ui(ui);
    });

    harness.run();
    harness.fit_contents();

    #[cfg(all(feature = "wgpu", feature = "snapshot"))]
    harness.snapshot("image_snapshots");
}

#[test]
fn test_combobox() {
    let items = ["Item 1", "Item 2", "Item 3"];
    let mut harness = Harness::builder()
        .with_size(Vec2::new(300.0, 200.0))
        .build_ui_state(
            |ui, selected| {
                ComboBox::new("combobox", "Select Something").show_index(
                    ui,
                    selected,
                    items.len(),
                    |idx| *items.get(idx).expect("Invalid index"),
                );
            },
            0,
        );

    harness.run();

    let mut results = SnapshotResults::new();

    #[cfg(all(feature = "wgpu", feature = "snapshot"))]
    results.add(harness.try_snapshot("combobox_closed"));

    let combobox = harness.get_by_role_and_label(Role::ComboBox, "Select Something");
    combobox.click();

    harness.run();

    #[cfg(all(feature = "wgpu", feature = "snapshot"))]
    results.add(harness.try_snapshot("combobox_opened"));

    let item_2 = harness.get_by_role_and_label(Role::Button, "Item 2");
    // Node::click doesn't close the popup, so we use simulate_click
    item_2.simulate_click();

    harness.run();

    assert_eq!(harness.state(), &1);

    // Popup should be closed now
    assert!(harness.query_by_label("Item 2").is_none());
}

/// Clicking inside a `TextEdit` that lives in a panned/zoomed [`egui::Scene`]
/// must place the text cursor under the pointer. The pointer is reported in
/// global coordinates while the widget rect is in the scene's local
/// coordinates, so the click->cursor mapping has to undo the layer transform.
/// Regression test for text selection being offset inside a `Scene` (e.g. the
/// notebook canvas).
#[test]
fn text_edit_cursor_respects_scene_transform() {
    use egui::emath::TSTransform;
    use egui::{pos2, vec2, Event, Id, Modifiers, Pos2, Rect, Scene};
    use std::cell::Cell;
    use std::rc::Rc;

    let te_id = Id::new("scene_text_edit");
    // The editor's rect and a click target inside it, in scene-local space. The
    // text spans two lines so an x/y offset lands on a clearly different glyph.
    let editor_rect = Rect::from_min_max(pos2(40.0, 40.0), pos2(360.0, 160.0));
    // Aim at a glyph on the first text row (the editor is taller than the text,
    // so clicking low would just clamp to the end in every case).
    let local_click = pos2(110.0, 52.0);
    let text = "hello there world\nsecond line of text";

    // Inject a primary-button press at `pos` (global coords) and read back the
    // editor's resulting primary cursor char index.
    fn click_and_read(harness: &mut Harness<'_>, pos: Pos2, id: Id) -> usize {
        // Move first and settle so the widget registers as hovered; the cursor
        // is only placed when the press lands on an already-hovered widget.
        harness.input_mut().events.push(Event::PointerMoved(pos));
        harness.run();
        harness.input_mut().events.push(Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: Modifiers::default(),
        });
        harness.run();
        egui::text_edit::TextEditState::load(&harness.ctx, id)
            .expect("text edit state should exist after interacting")
            .cursor
            .char_range()
            .expect("cursor should be set after a click")
            .primary
            .index
    }

    // Reference: editor outside any scene, clicked directly at the local point.
    let expected = {
        let mut buf = text.to_owned();
        let mut harness = Harness::builder().with_size(vec2(800.0, 600.0)).build_ui(
            move |ui| {
                ui.put(editor_rect, egui::TextEdit::multiline(&mut buf).id(te_id));
            },
        );
        harness.run();
        click_and_read(&mut harness, local_click, te_id)
    };

    // Same editor inside a Scene whose view is zoomed and panned. Clicking the
    // same *visual* point (the local target carried into global space by the
    // scene transform) must resolve to the same glyph. Without undoing the
    // transform the cursor is computed from raw global coordinates and lands
    // elsewhere.
    let scene_cursor = {
        let mut buf = text.to_owned();
        // A non-identity view: smaller scene rect than the screen => zoom + pan.
        let mut scene_rect = Rect::from_min_size(pos2(0.0, 0.0), vec2(400.0, 300.0));
        let transform = Rc::new(Cell::new(TSTransform::IDENTITY));
        let captured = transform.clone();
        let mut harness = Harness::builder().with_size(vec2(800.0, 600.0)).build_ui(
            move |ui| {
                Scene::new().show(ui, &mut scene_rect, |ui| {
                    captured.set(
                        ui.ctx()
                            .layer_transform_to_global(ui.layer_id())
                            .unwrap_or(TSTransform::IDENTITY),
                    );
                    ui.put(editor_rect, egui::TextEdit::multiline(&mut buf).id(te_id));
                });
            },
        );
        harness.run();
        let to_global = transform.get();
        assert_ne!(
            to_global,
            TSTransform::IDENTITY,
            "scene should apply a non-trivial transform for this test to be meaningful"
        );
        click_and_read(&mut harness, to_global * local_click, te_id)
    };

    assert_eq!(
        scene_cursor, expected,
        "cursor inside the transformed Scene ({scene_cursor}) should match the untransformed cursor ({expected})"
    );
}
