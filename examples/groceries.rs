pub mod yo_web_framework_example;

#[xtask_wasm::run_example]
fn run_app() {
    use yo_web_framework_example::prelude::*;

    #[derive(Default)]
    struct ItemBuilder {
        key: Option<Key>,
        onchange: Option<Box<dyn Fn((IString, u32))>>,
    }

    impl ItemBuilder {
        pub fn set_attr_key(&mut self, key: impl std::hash::Hash) -> &mut Self {
            use std::hash::Hasher;
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            key.hash(&mut hasher);
            self.key.replace(hasher.finish().into());
            self
        }
        fn set_attr_onchange(&mut self, f: impl Fn((IString, u32)) + 'static) -> &mut Self {
            self.onchange.replace(Box::new(f));
            self
        }
        fn finish(&mut self) -> VNode {
            VNode::from(VNodeStatefulComponent::new(Item {
                key: self.key,
                onchange: self.onchange.take().map(Rc::from),
            }))
        }
    }

    #[derive(Clone)]
    struct Item {
        key: Option<Key>,
        onchange: Option<Rc<dyn Fn((IString, u32))>>,
    }

    impl Item {
        fn builder(_tag: &'static str) -> ItemBuilder {
            Default::default()
        }
    }

    impl StatefulComponent for Item {
        fn key(&self) -> Option<Key> {
            self.key
        }
        fn update(&mut self, _other: Self) -> bool {
            false
        }
        fn render(&self, ctx: &StatefulComponentHandler<Self>) -> VNode {
            let (name, count) =
                ctx.with_state(|state: &ItemState| (state.name.clone(), state.count));
            let inc = {
                let onchange = self.onchange.clone();
                ctx.callback(move |_, state: &mut ItemState, _ev: web_sys::Event| {
                    state.count = state.count.saturating_add(1);
                    if let Some(onchange) = onchange.clone() {
                        (onchange)((state.name.clone(), state.count));
                    }
                })
            };
            let dec = {
                let onchange = self.onchange.clone();
                ctx.callback(move |_, state: &mut ItemState, _ev: web_sys::Event| {
                    state.count = state.count.saturating_sub(1);
                    if let Some(onchange) = onchange.clone() {
                        (onchange)((state.name.clone(), state.count));
                    }
                })
            };
            let onchange = {
                let onchange = self.onchange.clone();
                ctx.callback(move |_, state: &mut ItemState, event: web_sys::Event| {
                    let Some(input) = event
                        .target()
                        .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
                    else {
                        return;
                    };
                    state.name = input.value().into();
                    if let Some(onchange) = onchange.clone() {
                        (onchange)((state.name.clone(), state.count));
                    }
                })
            };

            html! {
                <>
                    <input {onchange}>{name}</input>
                    {" ("}("{}", count){")"}
                    <button onclick={dec}>{"-"}</button>
                    <button onclick={inc}>{"+"}</button>
                </>
            }
        }
    }

    #[derive(Default)]
    struct ItemState {
        name: IString,
        count: u32,
    }

    struct AppBuilder;

    impl AppBuilder {
        fn finish(&mut self) -> VNode {
            VNode::from(VNodeStatefulComponent::new(App))
        }
    }

    #[derive(Clone)]
    struct App;

    impl App {
        fn builder(_tag: &'static str) -> AppBuilder {
            AppBuilder
        }
    }

    impl StatefulComponent for App {
        fn update(&mut self, _other: Self) -> bool {
            false
        }
        fn render(&self, ctx: &StatefulComponentHandler<Self>) -> VNode {
            let add_item = ctx.callback(|_, state: &mut AppState, _ev: web_sys::Event| {
                state.list.push(Default::default());
            });
            let list = ctx.with_state(|state: &AppState| {
                state
                    .list
                    .iter()
                    .enumerate()
                    .map(|(i, _)| {
                        let ctx = ctx.clone();
                        let onchange = move |(new_name, new_count): (IString, u32)| {
                            ctx.with_state_mut(|state: &mut AppState| {
                                if let Some((name, count)) = state.list.get_mut(i) {
                                    *name = new_name;
                                    *count = new_count;
                                    log!("Item #{i}: {name} ({count})");
                                }
                            });
                        };
                        html! {
                            <li key={i}><Item {onchange} /></li>
                        }
                    })
                    .collect::<VNode>()
            });

            html! {
                <>
                    <h1>{"Groceries"}</h1>
                    <ul>
                        {list}
                    </ul>
                    <button onclick={add_item}>{"Add item"}</button>
                </>
            }
        }
    }

    #[derive(Default)]
    struct AppState {
        list: Vec<(IString, u32)>,
    }

    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let body = document.body().unwrap();
    let _ = html!(<App />).create_dom(&body);
}
