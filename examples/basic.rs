pub mod yo_web_framework_example;

#[xtask_wasm::run_example]
fn run_app() {
    use yo_web_framework_example::prelude::*;

    log("Hello World!");

    fn render(name: impl Into<IString>, swap: bool) -> VNode {
        let mut text1 = html! { <span key="hello" style="font-weight: bold;">{"Hello"}</span> };
        let mut text2 = html! { <span key="name">{name.into()}</span> };
        if swap {
            std::mem::swap(&mut text1, &mut text2);
        }

        fn make_stuff(i: i32) -> VNode {
            html! { <li key={i}>{"Stuff #"}("{}", i)</li> }
        }

        let mut stuff1 = make_stuff(1);
        let mut stuff2 = make_stuff(2);
        if swap {
            std::mem::swap(&mut stuff1, &mut stuff2);
        }

        html! {
            <div>
            <p>{"Welcome"}</p>
            <>{text1}<br/>{text2}{"!"}</>
            <ul>
            {stuff1}
            {stuff2}
            </ul>
            </div>
        }
    }

    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let body = document.body().unwrap();
    let app = render("World", false).create_dom();
    body.append_child(app.node().unwrap()).unwrap();
    let app = app.update(render("Q", true));
    drop(app);
}
