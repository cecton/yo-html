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

macro_rules! log {
    ($($tt:tt)+) => {{
        log(&format!($($tt)*));
    }};
}

type Key = u64;

#[derive(Clone)]
pub enum VNode {
    Element(Rc<VNodeElement>),
    Text(Rc<VNodeText>),
    Fragment(Rc<VNodeFragment>),
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

impl VNode {
    pub fn builder(tag: &'static str) -> VNodeElementBuilder {
        VNodeElementBuilder {
            tag,
            key: Default::default(),
            class: Default::default(),
            dyn_attrs: Default::default(),
            children: Default::default(),
        }
    }

    pub fn create_dom(self) -> Self {
        match self {
            Self::Element(x) => Self::Element(Rc::new(Rc::unwrap_or_clone(x).create_dom())),
            Self::Text(x) => Self::Text(Rc::new(Rc::unwrap_or_clone(x).create_dom())),
            Self::Fragment(x) => Self::Fragment(Rc::new(Rc::unwrap_or_clone(x).create_dom())),
            _ => todo!(),
        }
    }

    pub fn update(self, new_vnode: Self) -> Self {
        match (self, new_vnode) {
            (Self::Element(a), Self::Element(b)) => Self::Element(Rc::new(
                Rc::unwrap_or_clone(a).update(Rc::unwrap_or_clone(b)),
            )),
            (Self::Text(a), Self::Text(b)) => Self::Text(Rc::new(
                Rc::unwrap_or_clone(a).update(Rc::unwrap_or_clone(b)),
            )),
            (Self::Fragment(a), Self::Fragment(b)) => Self::Fragment(Rc::new(
                Rc::unwrap_or_clone(a).update(Rc::unwrap_or_clone(b)),
            )),
            _ => todo!(),
        }
    }

    pub fn node(&self) -> Option<&web_sys::Node> {
        match self {
            Self::Element(x) => x.node(),
            Self::Text(x) => x.node(),
            Self::Fragment(x) => x.node(),
            _ => todo!(),
        }
    }

    pub fn next_sibling(&self) -> Option<web_sys::Node> {
        self.node().and_then(|x| x.next_sibling())
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
    node: Option<web_sys::Element>,
    dyn_attrs: Rc<[(IString, IString)]>,
    children: Rc<[VNode]>,
}

impl VNodeElement {
    pub fn create_dom(self) -> Self {
        if self.node.is_some() {
            return self;
        }

        let Self {
            tag,
            dyn_attrs,
            children,
            ..
        } = self;

        log!("create element: {tag}");
        let node = document().create_element(tag).unwrap();
        for (attr_name, attr_value) in dyn_attrs.iter() {
            node.set_attribute(attr_name, attr_value).unwrap();
        }
        let children = children
            .iter()
            .map(|child| {
                let child = child.clone().create_dom();
                node.append_child(child.node().unwrap()).unwrap();
                child
            })
            .collect();

        Self {
            node: Some(node),
            dyn_attrs,
            children,
            ..self
        }
    }

    pub fn update(self, new_vnode: Self) -> Self {
        if self.node.is_none() {
            return self;
        }

        let Self {
            tag,
            key: _,
            node,
            dyn_attrs,
            children,
        } = self;
        let node = node.unwrap();

        log!("update element: {tag}");
        dyn_attrs
            .iter()
            .map(|(x, _)| x)
            .filter(|attr_name| !new_vnode.dyn_attrs.iter().any(|(x, _)| x == *attr_name))
            .for_each(|attr_name| {
                node.remove_attribute(&attr_name).unwrap();
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
                node.set_attribute(attr_name, attr_value).unwrap();
            });

        let mut new_children: Vec<VNode> = Vec::with_capacity(new_vnode.children.len());

        let mut next_sibling = node.first_child();
        let mut to_remove = vec![true; new_vnode.children.len()];
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
                        let child = old_child.clone().update(new_child.clone());
                        if old_pos != new_pos {
                            // NOTE: this is "insert_after"
                            // http://stackoverflow.com/questions/4793604/ddg#4793630
                            node.insert_before(child.node().unwrap(), next_sibling.as_ref())
                                .unwrap();
                            log("relocate");
                        }
                        next_sibling = child.next_sibling();
                        to_remove[old_pos] = false;
                        child
                    } else {
                        let child = new_child.clone().create_dom();
                        // NOTE: this is "insert_after"
                        // http://stackoverflow.com/questions/4793604/ddg#4793630
                        node.insert_before(child.node().unwrap(), next_sibling.as_ref())
                            .unwrap();
                        child
                    }
                } else if let Some(old_child) = children.get(new_pos) {
                    let child = old_child.clone().update(new_child.clone());
                    next_sibling = child.next_sibling();
                    to_remove[new_pos] = false;
                    child
                } else {
                    let child = new_child.clone().create_dom();
                    node.append_child(child.node().unwrap()).unwrap();
                    child
                });
            });

        children
            .iter()
            .zip(to_remove.into_iter())
            .for_each(|(child, to_remove)| {
                if to_remove {
                    if let Some(node) = child.node() {
                        if let Some(parent) = node.parent_node() {
                            log("remove node");
                            parent.remove_child(node).unwrap();
                        }
                    }
                }
            });

        Self {
            tag,
            key: new_vnode.key,
            node: Some(node),
            dyn_attrs: new_vnode.dyn_attrs,
            children: Rc::from(new_children),
        }
    }

    pub fn node(&self) -> Option<&web_sys::Node> {
        self.node.as_deref()
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
    pub fn create_dom(self) -> Self {
        if self.node.is_some() {
            return self;
        }

        log!("create text: {}", self.text);
        let node = document().create_text_node(&self.text);
        VNodeText {
            node: Some(node),
            ..self
        }
    }

    pub fn update(self, new_vnode: Self) -> Self {
        if let Some(node) = self.node {
            if self.text != new_vnode.text {
                log!("update text: {} -> {}", self.text, new_vnode.text);
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

    pub fn node(&self) -> Option<&web_sys::Node> {
        // TODO why this one cant use as_deref?
        self.node.as_ref().map(|x| x.unchecked_ref())
    }
}

#[derive(Clone)]
pub struct VNodeFragment {
    key: Option<Key>,
    node: Option<web_sys::DocumentFragment>,
    children: Vec<VNode>,
}

impl VNodeFragment {
    pub fn builder() -> VNodeFragmentBuilder {
        VNodeFragmentBuilder {
            key: Default::default(),
            children: Default::default(),
        }
    }

    pub fn create_dom(self) -> Self {
        let Self {
            key,
            node: _,
            children,
        } = self;

        log("create fragment");
        let node = document().create_document_fragment();
        let children = children
            .into_iter()
            .map(|child| {
                let child = child.clone().create_dom();
                node.append_child(child.node().unwrap()).unwrap();
                child
            })
            .collect();

        Self {
            key,
            node: Some(node),
            children,
        }
    }

    pub fn update(self, new_vnode: Self) -> Self {
        if self.children.is_empty() {
            return self;
        }

        let Self {
            key,
            node: _,
            children,
        } = self;

        log("update fragment");
        let mut new_children: Vec<VNode> = Vec::with_capacity(new_vnode.children.len());

        // TODO logic is identical to element's children. this should be generalized
        let mut next_sibling = children[0].node().map(Clone::clone);
        let node = next_sibling.as_ref().unwrap().parent_node().unwrap();
        log!("first child is some: {:?}", next_sibling.is_some());
        let mut to_remove = vec![true; new_vnode.children.len()];
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
                        let child = old_child.clone().update(new_child.clone());
                        if old_pos != new_pos {
                            // NOTE: this is "insert_after"
                            // http://stackoverflow.com/questions/4793604/ddg#4793630
                            node.insert_before(child.node().unwrap(), next_sibling.as_ref())
                                .unwrap();
                            log("relocate");
                        }
                        next_sibling = child.next_sibling();
                        to_remove[old_pos] = false;
                        child
                    } else {
                        let child = new_child.clone().create_dom();
                        // NOTE: this is "insert_after"
                        // http://stackoverflow.com/questions/4793604/ddg#4793630
                        node.insert_before(child.node().unwrap(), next_sibling.as_ref())
                            .unwrap();
                        child
                    }
                } else if let Some(old_child) = children.get(new_pos) {
                    let child = old_child.clone().update(new_child.clone());
                    next_sibling = child.next_sibling();
                    to_remove[new_pos] = false;
                    child
                } else {
                    let child = new_child.clone().create_dom();
                    node.append_child(child.node().unwrap()).unwrap();
                    child
                });
            });

        children
            .iter()
            .zip(to_remove.into_iter())
            .for_each(|(child, to_remove)| {
                if to_remove {
                    // TODO things should be removed recursively because of the fragments
                    if let Some(node) = child.node() {
                        if let Some(parent) = node.parent_node() {
                            log("remove node");
                            parent.remove_child(node).unwrap();
                        }
                    }
                }
            });

        Self {
            key,
            node: None,
            children,
        }
    }

    pub fn node(&self) -> Option<&web_sys::Node> {
        self.node.as_deref()
    }
}

pub struct VNodeElementBuilder {
    tag: &'static str,
    key: Option<Key>,
    class: Vec<IString>,
    dyn_attrs: HashMap<IString, IString>,
    children: Vec<VNode>,
}

impl VNodeElementBuilder {
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
            node: Default::default(),
            dyn_attrs: Rc::from(
                std::mem::take(&mut self.dyn_attrs)
                    .into_iter()
                    .collect::<Vec<_>>(),
            ),
            children: Rc::from(std::mem::take(&mut self.children)),
        }))
    }
}

pub struct VNodeFragmentBuilder {
    key: Option<Key>,
    children: Vec<VNode>,
}

impl VNodeFragmentBuilder {
    pub fn set_attr_key(&mut self, key: impl std::hash::Hash) -> &mut Self {
        use std::hash::Hasher;
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        key.hash(&mut hasher);
        self.key.replace(hasher.finish().into());
        self
    }

    pub fn add_child(&mut self, vnode: impl Into<VNode>, additional: usize) -> &mut Self {
        self.children.reserve_exact(additional);
        self.children.push(vnode.into());
        self
    }

    pub fn finish(&mut self) -> VNode {
        VNode::Fragment(Rc::new(VNodeFragment {
            key: self.key,
            node: Default::default(),
            children: std::mem::take(&mut self.children),
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
    pub type p = super::VNode;
    pub type Text = super::VNode;
    pub type Fragment = super::VNodeFragment;
    pub use super::MyComponent;
}

pub mod prelude {
    pub use super::html_context;
    pub use super::log;
    pub use super::VNode;
    pub use implicit_clone::unsync::*;
    pub use yo_html::html;
}
