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
    Component(Rc<dyn Component>, Box<VNode>),
}

impl implicit_clone::ImplicitClone for VNode {}

impl Default for VNode {
    fn default() -> Self {
        thread_local! {
            static DEFAULT_VALUE: Rc<VNodeFragment> = Rc::new(VNodeFragment {
                key: None,
                fragment: document().create_document_fragment(),
                children: Default::default(),
                container: None,
            });
        }
        DEFAULT_VALUE.with(|x| Self::Fragment(x.clone()))
    }
}

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

impl<T: Component + 'static> From<T> for VNode {
    fn from(component: T) -> VNode {
        VNode::Component(Rc::new(component), Default::default())
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

    pub fn create_dom(self, container: &web_sys::Node) -> Self {
        match self {
            Self::Element(x) => {
                Self::Element(Rc::new(Rc::unwrap_or_clone(x).create_dom(container)))
            }
            Self::Text(x) => Self::Text(Rc::new(Rc::unwrap_or_clone(x).create_dom(container))),
            Self::Fragment(x) => {
                Self::Fragment(Rc::new(Rc::unwrap_or_clone(x).create_dom(container)))
            }
            Self::Component(component, _) => {
                let vnode = component.render().create_dom(container);
                Self::Component(component, Box::new(vnode))
            }
        }
    }

    pub fn remove_dom(self) -> Self {
        match self {
            Self::Element(x) => Self::Element(Rc::new(Rc::unwrap_or_clone(x).remove_dom())),
            Self::Text(x) => Self::Text(Rc::new(Rc::unwrap_or_clone(x).remove_dom())),
            Self::Fragment(x) => Self::Fragment(Rc::new(Rc::unwrap_or_clone(x).remove_dom())),
            Self::Component(component, vnode) => {
                Self::Component(component, Box::new(vnode.remove_dom()))
            }
        }
    }

    pub fn update(self, new_vnode: Self, container: &web_sys::Node) -> Self {
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
            (Self::Component(_comp_a, vnode_a), Self::Component(comp_b, _vnode_b)) => {
                let vnode_b = comp_b.render();
                Self::Component(comp_b, Box::new(vnode_a.update(vnode_b, container)))
            }
            (a, b) => {
                a.remove_dom();
                b.create_dom(container)
            }
        }
    }

    pub fn node(&self) -> Option<&web_sys::Node> {
        match self {
            Self::Element(x) => x.node(),
            Self::Text(x) => x.node(),
            Self::Fragment(x) => Some(x.node()),
            Self::Component(_, vnode) => vnode.node(),
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
    node: Option<web_sys::Element>,
    dyn_attrs: Rc<[(IString, IString)]>,
    fragment: VNodeFragment,
}

impl VNodeElement {
    pub fn create_dom(self, container: &web_sys::Node) -> Self {
        if self.node.is_some() {
            return self;
        }

        let Self {
            tag,
            dyn_attrs,
            fragment,
            ..
        } = self;

        log!("create element: {tag}");
        let node = document().create_element(tag).unwrap();
        for (attr_name, attr_value) in dyn_attrs.iter() {
            node.set_attribute(attr_name, attr_value).unwrap();
        }
        let fragment = fragment.create_dom(&node);
        container.append_child(&node).unwrap();

        Self {
            node: Some(node),
            dyn_attrs,
            fragment,
            ..self
        }
    }

    pub fn remove_dom(mut self) -> Self {
        if let Some(node) = self.node.take() {
            node.remove();
        }
        self
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
            fragment,
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

        let fragment = fragment.update(new_vnode.fragment);

        Self {
            tag,
            key: new_vnode.key,
            node: Some(node),
            dyn_attrs: new_vnode.dyn_attrs,
            fragment,
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
    pub fn create_dom(self, container: &web_sys::Node) -> Self {
        if self.node.is_some() {
            return self;
        }

        log!("create text: {}", self.text);
        let node = document().create_text_node(&self.text);
        container.append_child(&node).unwrap();
        VNodeText {
            node: Some(node),
            ..self
        }
    }

    pub fn remove_dom(mut self) -> Self {
        if let Some(node) = self.node.take() {
            node.remove();
        }
        self
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
    fragment: web_sys::DocumentFragment,
    container: Option<web_sys::Node>,
    children: Vec<VNode>,
}

impl VNodeFragment {
    pub fn builder() -> VNodeFragmentBuilder {
        VNodeFragmentBuilder {
            key: Default::default(),
            children: Default::default(),
        }
    }

    pub fn create_dom(self, container: &web_sys::Node) -> Self {
        let Self {
            key,
            fragment,
            children,
            container: _,
        } = self;

        log("create fragment");
        let children = children
            .into_iter()
            .map(|child| child.clone().create_dom(&fragment))
            .collect();
        container.append_child(&fragment).unwrap();

        Self {
            key,
            fragment,
            children,
            container: Some(container.clone()),
        }
    }

    pub fn remove_dom(mut self) -> Self {
        for child in self.children.iter_mut() {
            *child = child.clone().remove_dom();
        }
        self
    }

    pub fn update(self, new_vnode: Self) -> Self {
        if self.children.is_empty() {
            return self;
        }

        let Self {
            key,
            fragment,
            children: old_children,
            container,
        } = self;
        let container = container.unwrap();

        log("update fragment");
        let mut new_children: Vec<VNode> = Vec::with_capacity(new_vnode.children.len());

        let mut to_remove = vec![true; new_vnode.children.len()];
        new_vnode
            .children
            .into_iter()
            .enumerate()
            .for_each(|(new_pos, new_child)| {
                new_children.push(if let Some(key) = new_child.key() {
                    if let Some((old_pos, old_child)) = old_children
                        .iter()
                        .enumerate()
                        .find(|(_, x)| x.key() == Some(key))
                    {
                        fragment.append_child(old_child.node().unwrap()).unwrap();
                        to_remove[old_pos] = false;
                        old_child.clone().update(new_child, &fragment)
                    } else {
                        new_child.create_dom(&fragment)
                    }
                } else if let Some(old_child) = old_children.get(new_pos) {
                    fragment.append_child(old_child.node().unwrap()).unwrap();
                    to_remove[new_pos] = false;
                    old_child.clone().update(new_child, &fragment)
                } else {
                    new_child.create_dom(&fragment)
                });
            });

        old_children
            .into_iter()
            .zip(to_remove.into_iter())
            .for_each(|(child, to_remove)| {
                if to_remove {
                    child.remove_dom();
                }
            });

        container.append_child(&fragment).unwrap();

        Self {
            key,
            fragment,
            children: new_children,
            container: Some(container),
        }
    }

    pub fn node(&self) -> &web_sys::Node {
        self.fragment.unchecked_ref()
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
            fragment: VNodeFragment {
                key: None,
                fragment: document().create_document_fragment(),
                children: std::mem::take(&mut self.children),
                container: None,
            },
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
            fragment: document().create_document_fragment(),
            children: std::mem::take(&mut self.children),
            container: None,
        }))
    }
}

pub trait Component {
    fn render(&self) -> VNode;
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
}

pub mod prelude {
    pub use super::html_context;
    pub use super::log;
    pub use super::Component;
    pub use super::VNode;
    pub use implicit_clone::unsync::*;
    pub use yo_html::html;
}
