pub mod yo_web_framework_example;

#[xtask_wasm::run_example]
fn run_app() {
    use yo_web_framework_example::prelude::*;

    log("Hello World!");

    fn render(name: &str, swap: bool) -> VNode {
        let mut text1 = html! { <span style="font-weight: bold;">{"Hello"}</span> };
        let mut text2 = html! { <span>{name.to_string()}</span> };
        if swap {
            std::mem::swap(&mut text1, &mut text2);
        }
        html! {
            <div>{text1}<br/>{text2}{"!"}</div>
        }
    }

    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let body = document.body().unwrap();
    let app = render("World", false).create_dom_element();
    body.append_child(&app.node().unwrap()).unwrap();
    let app = app.update_dom_element(render("Q", true));
    drop(app);
}
