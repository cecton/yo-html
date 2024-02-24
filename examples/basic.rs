pub mod yo_web_framework_example;

#[xtask_wasm::run_example]
fn run_app() {
    use yo_web_framework_example::prelude::*;

    log("Hello World!");

    fn render(name: &str) -> VNode {
        html! {
            <div><span style="font-weight: bold;">{"Hello"}</span><br/>{name.to_string()}{"!"}</div>
        }
    }

    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let body = document.body().unwrap();
    let app = render("World").create_dom_element();
    body.append_child(&app.node().unwrap()).unwrap();
    let app = app.update_dom_element(render("Q"));
    drop(app);
}
