/// A span within a region of text, from `start` (inclusive) to `end` (exclusive), in chars.
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
}
