use implicit_clone::unsync::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(message: &str);
}

#[derive(Clone)]
pub enum VNode {
    Tagged {
        tag: &'static str,
        id: u64,
        element: RefCell<Option<web_sys::Element>>,
        children: Rc<[VNode]>,
        // attributes...
    },
    Text(IString),
    Fragment(Rc<[VNode]>),
    Component(Rc<dyn Component>),
}

impl implicit_clone::ImplicitClone for VNode {}

impl From<String> for VNode {
    fn from(s: String) -> VNode {
        VNode::Text(s.into())
    }
}

impl From<&'static str> for VNode {
    fn from(s: &'static str) -> VNode {
        VNode::Text(s.into())
    }
}

impl From<std::fmt::Arguments<'_>> for VNode {
    fn from(args: std::fmt::Arguments) -> VNode {
        VNode::Text(args.into())
    }
}

impl<const N: usize> From<[VNode; N]> for VNode {
    fn from(elements: [VNode; N]) -> VNode {
        VNode::Fragment(elements.into())
    }
}

impl VNode {
    pub fn builder(tag: &'static str, id: u64) -> VNodeBuilder {
        VNodeBuilder {
            tag,
            id,
            class: Default::default(),
            dyn_attrs: Default::default(),
            children: Default::default(),
        }
    }

    pub fn create_dom_element(&self, container: &web_sys::Element) {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        match self {
            Self::Tagged { tag, element, .. } => {
                element.replace(Some(document.create_element(tag).unwrap()));
                container
                    .append_child(&element.borrow().clone().unwrap().unchecked_into())
                    .unwrap();
                self.update_dom_element();
            }
            Self::Text(s) => {
                let node = document.create_text_node(s);
                container.append_child(&node).unwrap();
            }
            _ => todo!(),
        }
    }

    pub fn update_dom_element(&self) {
        match self {
            Self::Tagged {
                element, children, ..
            } => {
                let element = element.borrow();
                let Some(element) = element.as_ref() else {
                    return;
                };
                // TODO optimize children creation/removal/re-ordering
                element.set_inner_html("");
                for child in children.iter() {
                    child.create_dom_element(element);
                }
            }
            _ => todo!(),
        }
    }
}

pub struct VNodeBuilder {
    tag: &'static str,
    id: u64,
    class: Vec<IString>,
    dyn_attrs: HashMap<IString, IString>,
    children: Vec<VNode>,
}

impl VNodeBuilder {
    pub fn set_attr_class(&mut self, class: impl Into<IString>) -> &mut Self {
        self.add_attr_class(class, 1)
    }

    pub fn add_attr_class(&mut self, class: impl Into<IString>, additional: usize) -> &mut Self {
        self.class.reserve_exact(additional);
        self.class.push(class.into());
        self
    }

    pub fn set_attr_style(&mut self, style: impl Into<IString>) -> &mut Self {
        self.add_attr("style", style, 1);
        self
    }

    pub fn add_child(&mut self, element: impl Into<VNode>, additional: usize) -> &mut Self {
        self.children.reserve_exact(additional);
        self.children.push(element.into());
        self
    }

    pub fn add_attr(
        &mut self,
        name: impl Into<IString>,
        value: impl Into<IString>,
        additional: usize,
    ) -> &mut Self {
        // NOTE: it's actually better to use reserve() here instead of reserve_exact() because
        //       there can be multiple different keys that will have their own number of items.
        self.dyn_attrs.reserve(additional);
        self.dyn_attrs.insert(name.into(), value.into());
        self
    }

    pub fn finish(&mut self) -> VNode {
        VNode::Tagged {
            tag: self.tag,
            id: self.id,
            element: Default::default(),
            children: Rc::from(std::mem::take(&mut self.children)),
        }
    }
}

pub trait Component {}

pub struct MyComponent<T = ()> {
    phantom: std::marker::PhantomData<T>,
}

impl<T> MyComponent<T> {
    pub fn builder(_tag: &'static str, _id: u64) -> MyComponentBuilder<T> {
        MyComponentBuilder {
            phantom: std::marker::PhantomData,
        }
    }
}

pub struct MyComponentBuilder<T> {
    phantom: std::marker::PhantomData<T>,
}

impl<T> MyComponentBuilder<T> {
    pub fn finish(&mut self) -> MyComponentProps<T> {
        MyComponentProps {
            phantom: std::marker::PhantomData,
        }
    }
}

pub struct MyComponentProps<T> {
    phantom: std::marker::PhantomData<T>,
}

impl<T> Component for MyComponentProps<T> {}

impl<T: 'static> From<MyComponentProps<T>> for VNode {
    fn from(component: MyComponentProps<T>) -> VNode {
        VNode::Component(Rc::new(component))
    }
}

#[doc(hidden)]
#[allow(non_camel_case_types)]
pub mod html_context {
    pub type div = super::VNode;
    pub type span = super::VNode;
    pub type br = super::VNode;
    pub type Text = super::VNode;
    pub type Fragment = super::VNode;
    pub use super::MyComponent;
}

pub mod prelude {
    pub use super::html_context;
    pub use super::log;
    pub use yo_html::html;
}
