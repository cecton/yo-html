pub mod yo_web_framework_example;

#[xtask_wasm::run_example]
fn run_app() {
    use yo_web_framework_example::prelude::*;

    log("Hello World!");

    // Pure component

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

    #[derive(PartialEq, Clone)]
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

    // Stateful component

    pub struct CounterBuilder {
        min: Option<i32>,
        max: Option<i32>,
    }

    impl CounterBuilder {
        pub fn set_attr_min(&mut self, min: i32) -> &mut Self {
            self.min.replace(min);
            self
        }
        pub fn set_attr_max(&mut self, max: i32) -> &mut Self {
            self.max.replace(max);
            self
        }
        pub fn finish(&mut self) -> VNodeStatefulComponent<Counter> {
            VNodeStatefulComponent::new(Counter {
                min: self.min,
                max: self.max,
            })
        }
    }

    #[derive(PartialEq, Clone)]
    pub struct Counter {
        min: Option<i32>,
        max: Option<i32>,
    }

    impl Counter {
        pub fn builder(_tag: &'static str) -> CounterBuilder {
            CounterBuilder {
                min: None,
                max: None,
            }
        }
    }

    impl StatefulComponent for Counter {
        fn update(&mut self, other: Self) -> bool {
            let should_update = *self != other;
            *self = other;
            should_update
        }
        fn render(&self, context: &StatefulComponentHandler<Self>) -> VNode {
            let range: IString = match (self.min, self.max) {
                (Some(min), Some(max)) => format!("{min}-{max}").into(),
                (Some(min), _) => format!("{min}-").into(),
                (_, Some(max)) => format!("-{max}").into(),
                (None, None) => "-".into(),
            };
            let value = context.with_state(|x: &i32| *x);
            let inc_callback = context.callback(|context| {
                log("click!");
                context.with_state_mut(|x: &mut i32| *x += 1);
            });
            let dec_callback = context.callback(|context| {
                log("click!");
                context.with_state_mut(|x: &mut i32| *x -= 1);
            });
            html! {
                <div>
                    <p>{"Counter ("}{range}{"): "}("{}", value)</p>
                    <button onclick={inc_callback}>{"Increase"}</button>
                    <button onclick={dec_callback}>{"Decrease"}</button>
                </div>
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
            <Counter min=0 />
            <>{"Footer"}</>
            </>
        }
    }

    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let body = document.body().unwrap();
    let app = render("World", false).create_dom(&body);
    let app = app.update(render("Q", true));
    let _ = app.update(render("World", false));
}
