use implicit_clone::unsync::*;
use std::collections::HashMap;
use std::rc::Rc;
use wasm_bindgen::prelude::*;

fn window() -> web_sys::Window {
    web_sys::window().unwrap()
}

fn document() -> web_sys::Document {
    window().document().unwrap()
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(message: &str);
}

type Key = u64;

#[derive(Clone)]
pub enum VNode {
    Element(Rc<VNodeElement>),
    Text(Rc<VNodeText>),
    Fragment(Rc<[VNode]>),
    Component(Rc<dyn Component>),
}

impl implicit_clone::ImplicitClone for VNode {}

impl From<IString> for VNode {
    fn from(text: IString) -> VNode {
        VNode::Text(Rc::new(VNodeText { text, node: None }))
    }
}

impl From<String> for VNode {
    fn from(s: String) -> VNode {
        VNode::Text(Rc::new(VNodeText {
            text: s.into(),
            node: None,
        }))
    }
}

impl From<&'static str> for VNode {
    fn from(s: &'static str) -> VNode {
        VNode::Text(Rc::new(VNodeText {
            text: s.into(),
            node: None,
        }))
    }
}

impl From<std::fmt::Arguments<'_>> for VNode {
    fn from(args: std::fmt::Arguments) -> VNode {
        VNode::Text(Rc::new(VNodeText {
            text: args.into(),
            node: None,
        }))
    }
}

impl<const N: usize> From<[VNode; N]> for VNode {
    fn from(elements: [VNode; N]) -> VNode {
        VNode::Fragment(elements.into())
    }
}

impl VNode {
    pub fn builder(tag: &'static str) -> VNodeBuilder {
        VNodeBuilder {
            tag,
            key: Default::default(),
            class: Default::default(),
            dyn_attrs: Default::default(),
            children: Default::default(),
        }
    }

    pub fn create_dom_element(self) -> Self {
        match self {
            Self::Element(x) => Self::Element(Rc::new(Rc::unwrap_or_clone(x).create_dom_element())),
            Self::Text(x) => Self::Text(Rc::new(Rc::unwrap_or_clone(x).create_dom_element())),
            Self::Fragment(x) => Self::Fragment(x),
            _ => todo!(),
        }
    }

    pub fn update_dom_element(self, new_vnode: Self) -> Self {
        match (self, new_vnode) {
            (Self::Element(a), Self::Element(b)) => Self::Element(Rc::new(
                Rc::unwrap_or_clone(a).update_dom_element(Rc::unwrap_or_clone(b)),
            )),
            (Self::Text(a), Self::Text(b)) => Self::Text(Rc::new(
                Rc::unwrap_or_clone(a).update_dom_element(Rc::unwrap_or_clone(b)),
            )),
            _ => todo!(),
        }
    }

    pub fn remove_dom_element(self) -> Self {
        match self {
            Self::Element(x) => Self::Element(Rc::new(Rc::unwrap_or_clone(x).remove_dom_element())),
            Self::Text(x) => Self::Text(Rc::new(Rc::unwrap_or_clone(x).remove_dom_element())),
            _ => todo!(),
        }
    }

    pub fn node(&self) -> Option<&web_sys::Node> {
        match self {
            Self::Element(x) => x.node(),
            Self::Text(x) => x.node(),
            _ => todo!(),
        }
    }

    pub fn key(&self) -> Option<Key> {
        match self {
            Self::Element(x) => x.key(),
            _ => None,
        }
    }
}

#[derive(Clone)]
pub struct VNodeElement {
    tag: &'static str,
    key: Option<Key>,
    element: Option<web_sys::Element>,
    dyn_attrs: Rc<[(IString, IString)]>,
    children: Rc<[VNode]>,
}

impl VNodeElement {
    pub fn create_dom_element(self) -> Self {
        if self.element.is_some() {
            return self;
        }

        let Self {
            tag,
            dyn_attrs,
            children,
            ..
        } = self;

        log("create dom element");
        let element = document().create_element(tag).unwrap();
        for (attr_name, attr_value) in dyn_attrs.iter() {
            element.set_attribute(attr_name, attr_value).unwrap();
        }
        let children = children
            .iter()
            .map(|child| {
                let child = child.clone().create_dom_element();
                element
                    .append_child(child.node().unwrap().unchecked_ref())
                    .unwrap();
                child
            })
            .collect();

        Self {
            element: Some(element),
            dyn_attrs,
            children,
            ..self
        }
    }

    pub fn update_dom_element(self, new_vnode: Self) -> Self {
        if self.element.is_none() {
            return self;
        }

        let Self {
            tag,
            key: _,
            element,
            dyn_attrs,
            children,
        } = self;
        let element = element.unwrap();

        log("update dom element");
        dyn_attrs
            .iter()
            .map(|(x, _)| x)
            .filter(|attr_name| !new_vnode.dyn_attrs.iter().any(|(x, _)| x == *attr_name))
            .for_each(|attr_name| {
                element.remove_attribute(&attr_name).unwrap();
            });

        new_vnode
            .dyn_attrs
            .iter()
            .filter(|(attr_name, attr_value)| {
                if let Some((_, old_value)) = dyn_attrs.iter().find(|(x, _)| x == attr_name) {
                    attr_value != old_value
                } else {
                    true
                }
            })
            .for_each(|(attr_name, attr_value)| {
                element.set_attribute(attr_name, attr_value).unwrap();
            });

        let mut new_children: Vec<VNode> = Vec::with_capacity(new_vnode.children.len());

        let mut next_sibling = element.first_child();
        let mut to_remove = vec![true; new_children.capacity()];
        new_vnode
            .children
            .iter()
            .enumerate()
            .for_each(|(new_pos, new_child)| {
                new_children.push(if let Some(key) = new_child.key() {
                    if let Some((old_pos, old_child)) = children
                        .iter()
                        .enumerate()
                        .find(|(_, x)| x.key() == Some(key))
                    {
                        let child = old_child.clone().update_dom_element(new_child.clone());
                        if old_pos != new_pos {
                            // NOTE: this is "insert_after"
                            // http://stackoverflow.com/questions/4793604/ddg#4793630
                            element
                                .insert_before(child.node().unwrap(), next_sibling.as_ref())
                                .unwrap();
                            log("relocate");
                        }
                        next_sibling = child.node().unwrap().next_sibling();
                        to_remove[old_pos] = false;
                        child
                    } else {
                        let child = new_child.clone().create_dom_element();
                        // NOTE: this is "insert_after"
                        // http://stackoverflow.com/questions/4793604/ddg#4793630
                        element
                            .insert_before(child.node().unwrap(), next_sibling.as_ref())
                            .unwrap();
                        child
                    }
                } else if let Some(old_child) = children.get(new_pos) {
                    let child = old_child.clone().update_dom_element(new_child.clone());
                    next_sibling = child.node().unwrap().next_sibling();
                    to_remove[new_pos] = false;
                    child
                } else {
                    let child = new_child.clone().create_dom_element();
                    element.append_child(child.node().unwrap()).unwrap();
                    child
                });
            });

        children
            .iter()
            .zip(to_remove.into_iter())
            .for_each(|(child, to_remove)| {
                if to_remove {
                    child.clone().remove_dom_element();
                }
            });

        Self {
            tag,
            key: new_vnode.key,
            element: Some(element),
            dyn_attrs: new_vnode.dyn_attrs,
            children: Rc::from(new_children),
        }
    }

    pub fn remove_dom_element(mut self) -> Self {
        if let Some(element) = self.element.take() {
            log("remove dom element");
            element.remove();
        }
        self
    }

    pub fn node(&self) -> Option<&web_sys::Node> {
        self.element.as_ref().map(|x| x.unchecked_ref())
    }

    pub fn key(&self) -> Option<Key> {
        self.key
    }
}

#[derive(Clone)]
pub struct VNodeText {
    text: IString,
    node: Option<web_sys::Text>,
}

impl VNodeText {
    pub fn create_dom_element(self) -> Self {
        if self.node.is_some() {
            return self;
        }

        log("create dom text");
        let node = document().create_text_node(&self.text);
        VNodeText {
            node: Some(node),
            ..self
        }
    }

    pub fn update_dom_element(self, new_vnode: Self) -> Self {
        if let Some(node) = self.node {
            if self.text != new_vnode.text {
                log("update dom text");
                node.set_node_value(Some(&new_vnode.text));
            }

            Self {
                text: new_vnode.text,
                node: Some(node),
            }
        } else {
            self
        }
    }

    pub fn remove_dom_element(mut self) -> Self {
        if let Some(node) = self.node.take() {
            log("remove dom text");
            node.remove();
        }
        self
    }

    pub fn node(&self) -> Option<&web_sys::Node> {
        self.node.as_ref().map(|x| x.unchecked_ref())
    }
}

pub struct VNodeBuilder {
    tag: &'static str,
    key: Option<Key>,
    class: Vec<IString>,
    dyn_attrs: HashMap<IString, IString>,
    children: Vec<VNode>,
}

impl VNodeBuilder {
    pub fn set_attr_key(&mut self, key: impl std::hash::Hash) -> &mut Self {
        use std::hash::Hasher;
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        key.hash(&mut hasher);
        self.key.replace(hasher.finish().into());
        self
    }

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

    pub fn add_child(&mut self, vnode: impl Into<VNode>, additional: usize) -> &mut Self {
        self.children.reserve_exact(additional);
        self.children.push(vnode.into());
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
        VNode::Element(Rc::new(VNodeElement {
            tag: self.tag,
            key: self.key,
            element: Default::default(),
            dyn_attrs: Rc::from(
                std::mem::take(&mut self.dyn_attrs)
                    .into_iter()
                    .collect::<Vec<_>>(),
            ),
            children: Rc::from(std::mem::take(&mut self.children)),
        }))
    }
}

pub trait Component {}

pub struct MyComponent<T = ()> {
    phantom: std::marker::PhantomData<T>,
}

impl<T> MyComponent<T> {
    pub fn builder(_tag: &'static str) -> MyComponentBuilder<T> {
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
    pub type ul = super::VNode;
    pub type li = super::VNode;
    pub type br = super::VNode;
    pub type Text = super::VNode;
    pub type Fragment = super::VNode;
    pub use super::MyComponent;
}

pub mod prelude {
    pub use super::html_context;
    pub use super::log;
    pub use super::VNode;
    pub use implicit_clone::unsync::*;
    pub use yo_html::html;
}
