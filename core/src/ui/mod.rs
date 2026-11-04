mod header;
mod input;
mod peer_list;
mod task_card;

pub mod theme;

pub use header::header;
pub use input::{TextInput, key_bindings};
pub use peer_list::{peer_section, peer_table};
pub use task_card::{progress_block, task_card};

use crate::ui::theme::DARK;
use gpui::{Div, SharedString, Stateful, div, prelude::*, rgb};

pub fn button(id: String, label: impl Into<SharedString>) -> Stateful<Div> {
    let text: SharedString = label.into();
    div()
        .id(id)
        .px_4()
        .py_2()
        .rounded_md()
        .cursor_pointer()
        .bg(rgb(DARK.accent))
        .text_color(rgb(DARK.accent_text))
        .hover(|style| style.bg(rgb(DARK.accent_hover)))
        .active(|style| style.bg(rgb(DARK.accent)))
        .child(text)
}

pub fn quiet_button(id: String, label: impl Into<SharedString>) -> Stateful<Div> {
    let text: SharedString = label.into();
    div()
        .id(id)
        .px_4()
        .py_2()
        .rounded_md()
        .cursor_pointer()
        .bg(rgb(DARK.quiet))
        .text_color(rgb(DARK.text))
        .hover(|style| style.bg(rgb(DARK.quiet_hover)))
        .active(|style| style.bg(rgb(DARK.quiet)))
        .child(text)
}

pub fn danger_button(id: String, label: impl Into<SharedString>) -> Stateful<Div> {
    let text: SharedString = label.into();
    div()
        .id(id)
        .px_4()
        .py_2()
        .rounded_md()
        .cursor_pointer()
        .bg(rgb(DARK.danger))
        .text_color(rgb(DARK.accent_text))
        .hover(|style| style.bg(rgb(DARK.danger_hover)))
        .active(|style| style.bg(rgb(DARK.danger)))
        .child(text)
}
