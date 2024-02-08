pub mod basic_web_framework;

use basic_web_framework::prelude::*;

fn invalid_token_in_attr_value() {
    let _ = html! { <span class=* /> };
}

fn invalid_token_in_attr_name() {
    let _ = html! { <span ()=42 /> };
}

fn missing_value() {
    let _ = html! { <span class= /> };
    let _ = html! { <span {class}= /> };
}

fn missing_eq() {
    let _ = html! { <span class /> };
}

fn duplicate_attributes() {
    let class = "46";
    let _ = html! { <span class="42" class="43" class=["44", "45"] {class} /> };
}

fn invalid_shorthand() {
    let _ = html! { <span {"class"} /> };
}

fn main() {}
