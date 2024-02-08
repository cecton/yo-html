pub mod basic_web_framework;

use basic_web_framework::prelude::*;

fn unclosed_fragment() {
    let _ = html! { <><span /> };
}

fn unclosed_element() {
    let _ = html! { <div> };
}

fn invalid_token() {
    let _ = html! { / };
}

fn invalid_token_in_element() {
    let _ = html! { <div /!> };
}

fn main() {}
