pub mod yo_web_framework_example;

#[xtask_wasm::run_example]
fn run_app() {
    use yo_web_framework_example::prelude::*;

    log("Hello World!");

    pub struct MyComponentBuilder<T> {
        value: Option<T>,
    }

    impl<T: 'static> MyComponentBuilder<T> {
        pub fn set_attr_value(&mut self, value: T) -> &mut Self {
            self.value.replace(value.into());
            self
        }
        pub fn finish(&mut self) -> VNodePureComponent<MyComponent<T>> {
            VNodePureComponent::new(MyComponent {
                value: self.value.take().expect("missing property value"),
            })
        }
    }

    #[derive(PartialEq)]
    pub struct MyComponent<T = ()> {
        value: T,
    }

    impl<T> MyComponent<T> {
        pub fn builder(_tag: &'static str) -> MyComponentBuilder<T> {
            MyComponentBuilder { value: None }
        }
    }

    impl<T> PureComponent for MyComponent<T> {
        fn render(&self) -> VNode {
            html! {
                <p>{"My component"}</p>
            }
        }
    }

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
            <>
            <>{"Header"}</>
            <br/>
            <>{text1}<br/>{text2}{"!"}</>
            <ul>
            {stuff1}
            {stuff2}
            </ul>
            <MyComponent<u32> value=42 />
            <>{"Footer"}</>
            </>
        }
    }

    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let body = document.body().unwrap();
    let app = render("World", false).create_dom(&body);
    let app = app.update(render("Q", true), &body);
    let app = app.update(render("World", false), &body);
    drop(app);
}
