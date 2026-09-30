//! Mobile soft keyboard sync: [`egui::Event::TextInputState`] into a [`egui::TextEdit`].

use egui::{Event, Id, TextEdit, TextInputState, TextSpan, text_edit::TextEditState};
use egui_kittest::Harness;

fn keyboard_state(text: &str, cursor: usize) -> Event {
    Event::TextInputState(TextInputState {
        text: text.to_owned(),
        selection: TextSpan {
            start: cursor,
            end: cursor,
        },
        compose_region: None,
    })
}

/// A harness with one single-line [`TextEdit`], focused if `focused`.
fn harness(id: Id, text: &str, focused: bool) -> Harness<'static, String> {
    let mut harness = Harness::builder()
        .with_accessibility_check(false)
        .build_ui_state(
            move |ui, text: &mut String| {
                let response = ui.add(TextEdit::singleline(text).id(id));
                if focused {
                    response.request_focus();
                }
            },
            text.to_owned(),
        );
    harness.run();
    harness
}

#[test]
fn text_input_state_edits_the_focused_text_edit() {
    let id = Id::unique("keyboard_text_edit");
    let mut harness = harness(id, "hello world", true);

    harness
        .input_mut()
        .events
        .push(keyboard_state("hello brave world", 12));
    harness.run();

    assert_eq!(harness.state(), "hello brave world");
    let cursor = TextEditState::load(&harness.ctx, id)
        .and_then(|state| state.cursor.char_range())
        .expect("the focused edit should have a cursor");
    assert_eq!(cursor.primary.index.0, 12);
    assert_eq!(cursor.secondary.index.0, 12);
}

#[test]
fn text_input_state_is_ignored_without_focus() {
    let id = Id::unique("unfocused_text_edit");
    let mut harness = harness(id, "hello world", false);

    harness
        .input_mut()
        .events
        .push(keyboard_state("something else", 3));
    harness.run();

    assert_eq!(harness.state(), "hello world");
}

#[test]
fn platform_output_append_keeps_the_latest_text_input_state() {
    let state = |text: &str| TextInputState {
        text: text.to_owned(),
        selection: TextSpan { start: 0, end: 0 },
        compose_region: None,
    };

    let mut output = egui::PlatformOutput {
        text_input_state: Some(state("older")),
        ..Default::default()
    };
    output.append(egui::PlatformOutput::default());
    assert_eq!(output.text_input_state, Some(state("older")));

    output.append(egui::PlatformOutput {
        text_input_state: Some(state("newer")),
        ..Default::default()
    });
    assert_eq!(output.text_input_state, Some(state("newer")));
}

/// A read-only buffer has nothing for the keyboard to edit, so its state does not move the cursor
/// either.
#[test]
fn text_input_state_leaves_an_immutable_buffer_alone() {
    let id = Id::unique("immutable_text_edit");
    let mut harness = Harness::builder()
        .with_accessibility_check(false)
        .build_ui(move |ui| {
            let mut text = "read only";
            ui.add(TextEdit::singleline(&mut text).id(id))
                .request_focus();
        });
    harness.run();

    harness.input_mut().events.push(keyboard_state("read", 2));
    harness.run();

    let cursor = TextEditState::load(&harness.ctx, id)
        .and_then(|state| state.cursor.char_range())
        .expect("the focused edit should have a cursor");
    assert_eq!(cursor.primary.index.0, 9);
}
