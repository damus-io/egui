/// A span within a region of text, from `start` (inclusive) to `end` (exclusive).
///
/// Both indices count chars (Unicode scalar values), like [`crate::text::CCursor`], not bytes
/// or UTF-16 code units. A platform whose soft keyboard counts differently converts at the
/// integration boundary: Android's counts UTF-16 code units, as Java strings do, and
/// `egui-winit` converts them.
///
/// An empty span or cursor position is specified with `start == end`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct TextSpan {
    /// The start of the span (inclusive).
    pub start: usize,

    /// The end of the span (exclusive).
    pub end: usize,
}

/// The state of a mobile soft keyboard's text input.
///
/// On Android the soft keyboard keeps its own copy of the text being edited. The integration
/// sends the keyboard's copy to egui as [`crate::Event::TextInputState`], and a focused
/// [`crate::TextEdit`] sends its own back through [`crate::PlatformOutput::text_input_state`],
/// so the two stay in sync.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct TextInputState {
    /// The full text being edited.
    pub text: String,

    /// A selection defined on the text.
    pub selection: TextSpan,

    /// A composing region defined on the text.
    pub compose_region: Option<TextSpan>,

    /// Orders the states egui sends against the ones the keyboard sends back.
    ///
    /// On a state egui sends, this is its version, and each one sent is greater than the last.
    ///
    /// On a state the keyboard sends, this is the version of the last state egui sent that the
    /// keyboard had been given when it made this one, or 0 if it had none. The keyboard edits its
    /// copy while egui's latest state is still on its way to it, so a version older than the
    /// last one egui sent means the keyboard made the state before it saw egui's latest edit
    /// (e.g. clearing a message box after sending): it is stale, and egui's own state replaces
    /// it on the keyboard's side, so a [`crate::TextEdit`] ignores it.
    pub version: u64,
}
