pub mod yo_web_framework_example;

#[xtask_wasm::run_example]
fn run_app() {
    use yo_web_framework_example::prelude::*;

    log("Hello World!");
    let app = html! {
        <div><span style="font-weight: bold;">{"Hello"}</span><br/>{"World!"}</div>
    };
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let body = document.body().unwrap();
    app.create_dom_element(&body.into());
}
