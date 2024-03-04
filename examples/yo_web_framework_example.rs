use implicit_clone::unsync::*;
use std::any::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::*;
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

#[macro_export]
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
    Component(Rc<VNodeComponent>),
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
        VNode::Text(Rc::new(VNodeText {
            text,
            node: None,
            container: None,
        }))
    }
}

impl From<String> for VNode {
    fn from(s: String) -> VNode {
        VNode::Text(Rc::new(VNodeText {
            text: s.into(),
            node: None,
            container: None,
        }))
    }
}

impl From<&'static str> for VNode {
    fn from(s: &'static str) -> VNode {
        VNode::Text(Rc::new(VNodeText {
            text: s.into(),
            node: None,
            container: None,
        }))
    }
}

impl From<std::fmt::Arguments<'_>> for VNode {
    fn from(args: std::fmt::Arguments) -> VNode {
        VNode::Text(Rc::new(VNodeText {
            text: args.into(),
            node: None,
            container: None,
        }))
    }
}

impl<T: Component + 'static> From<T> for VNode {
    fn from(component: T) -> VNode {
        VNode::Component(Rc::new(VNodeComponent {
            component: Rc::new(component),
            type_id: TypeId::of::<T>(),
            vnode: Default::default(),
            refresh_callback: Default::default(),
            refresh_callback_id: Default::default(),
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
            handlers: Default::default(),
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
            Self::Component(x) => Self::Component(x.create_dom(container)),
        }
    }

    pub fn remove_dom(self) -> Self {
        match self {
            Self::Element(x) => Self::Element(Rc::new(Rc::unwrap_or_clone(x).remove_dom())),
            Self::Text(x) => Self::Text(Rc::new(Rc::unwrap_or_clone(x).remove_dom())),
            Self::Fragment(x) => Self::Fragment(Rc::new(Rc::unwrap_or_clone(x).remove_dom())),
            Self::Component(x) => Self::Component(x.remove_dom()),
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
            (Self::Component(a), Self::Component(b)) if a.type_id == b.type_id => {
                Self::Component(a.update(Rc::unwrap_or_clone(b)))
            }
            (a, b) => b.create_dom(&a.remove_dom().container().unwrap()),
        }
    }

    pub fn node(&self) -> Option<web_sys::Node> {
        match self {
            Self::Element(x) => x.node(),
            Self::Text(x) => x.node(),
            Self::Fragment(x) => Some(x.node()),
            Self::Component(x) => x.node(),
        }
    }

    pub fn key(&self) -> Option<Key> {
        match self {
            Self::Element(x) => x.key(),
            _ => None,
        }
    }

    pub fn container(&self) -> Option<web_sys::Node> {
        match self {
            Self::Element(x) => x.container(),
            Self::Text(x) => x.container(),
            Self::Fragment(x) => x.container(),
            Self::Component(x) => x.container(),
        }
    }
}

#[derive(Clone)]
pub struct VNodeElement {
    tag: &'static str,
    key: Option<Key>,
    node: Option<web_sys::Element>,
    class: IArray<IString>,
    dyn_attrs: Rc<[(IString, IString)]>,
    fragment: VNodeFragment,
    handlers: Rc<[(&'static str, Callback)]>,
    container: Option<web_sys::Node>,
}

impl VNodeElement {
    pub fn create_dom(self, container: &web_sys::Node) -> Self {
        if self.node.is_some() {
            return self;
        }

        let Self {
            tag,
            class,
            dyn_attrs,
            fragment,
            handlers,
            ..
        } = self;

        log!("create element: {tag}");
        let node = document().create_element(tag).unwrap();

        Self::set_class(&node, &class);

        for (attr_name, attr_value) in dyn_attrs.iter() {
            node.set_attribute(attr_name, attr_value).unwrap();
        }

        let fragment = fragment.create_dom(&node);
        container.append_child(&node).unwrap();

        for (event_type, callback) in handlers.iter() {
            node.add_event_listener_with_callback(
                event_type,
                callback.closure.as_ref().as_ref().unchecked_ref(),
            )
            .unwrap();
        }

        Self {
            node: Some(node),
            class,
            dyn_attrs,
            fragment,
            handlers,
            container: Some(container.clone()),
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
            class,
            dyn_attrs,
            fragment,
            handlers,
            container,
        } = self;
        let node = node.unwrap();

        log!("update element: {tag}");

        if class != new_vnode.class {
            Self::set_class(&node, &new_vnode.class);
        }

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

        handlers.iter().for_each(|(event_type, callback)| {
            node.remove_event_listener_with_callback(
                event_type,
                callback.closure.as_ref().as_ref().unchecked_ref(),
            )
            .unwrap();
        });

        new_vnode
            .handlers
            .iter()
            .for_each(|(event_type, callback)| {
                node.add_event_listener_with_callback(
                    event_type,
                    callback.closure.as_ref().as_ref().unchecked_ref(),
                )
                .unwrap();
            });

        Self {
            tag,
            key: new_vnode.key,
            node: Some(node),
            class: new_vnode.class,
            dyn_attrs: new_vnode.dyn_attrs,
            fragment,
            handlers: new_vnode.handlers,
            container,
        }
    }

    pub fn node(&self) -> Option<web_sys::Node> {
        self.node.clone().map(|x| x.unchecked_into())
    }

    pub fn key(&self) -> Option<Key> {
        self.key
    }

    pub fn container(&self) -> Option<web_sys::Node> {
        self.container.clone()
    }

    pub fn set_class(node: &web_sys::Element, class: &[IString]) {
        let mut it = class.iter();
        if let Some(first_class) = it.next() {
            let mut class = first_class.to_string();
            it.for_each(|x| {
                class.push(' ');
                class.push_str(x);
            });
            node.set_attribute("class", &class).unwrap();
        }
    }
}

#[derive(Clone)]
pub struct VNodeText {
    text: IString,
    node: Option<web_sys::Text>,
    container: Option<web_sys::Node>,
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
            container: Some(container.clone()),
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
                container: self.container,
            }
        } else {
            self
        }
    }

    pub fn node(&self) -> Option<web_sys::Node> {
        self.node.clone().map(|x| x.unchecked_into())
    }

    pub fn container(&self) -> Option<web_sys::Node> {
        self.container.clone()
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
        self.children = self.children.into_iter().map(|x| x.remove_dom()).collect();
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
                        fragment.append_child(&old_child.node().unwrap()).unwrap();
                        to_remove[old_pos] = false;
                        old_child.clone().update(new_child)
                    } else {
                        new_child.create_dom(&fragment)
                    }
                } else if let Some(old_child) = old_children.get(new_pos) {
                    fragment.append_child(&old_child.node().unwrap()).unwrap();
                    to_remove[new_pos] = false;
                    old_child.clone().update(new_child)
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

    pub fn node(&self) -> web_sys::Node {
        self.fragment.clone().unchecked_into()
    }

    pub fn container(&self) -> Option<web_sys::Node> {
        self.container.clone()
    }
}

#[derive(Clone)]
pub struct VNodeComponent {
    component: Rc<dyn Component>,
    type_id: TypeId,
    vnode: Rc<RefCell<VNode>>,
    refresh_callback: Rc<RefCell<Option<Closure<dyn FnMut()>>>>,
    refresh_callback_id: Rc<RefCell<Option<std::num::NonZeroI32>>>,
}

impl VNodeComponent {
    pub fn create_dom(self: Rc<Self>, container: &web_sys::Node) -> Rc<Self> {
        self.vnode
            .replace(self.component.render(self.clone()).create_dom(container));
        let vnode_comp = self.clone();
        self.refresh_callback.replace(Some(Closure::new(move || {
            let new_vnode = vnode_comp.component.render(vnode_comp.clone());
            assert!(matches!(new_vnode, VNode::Element(_)));
            let this = &vnode_comp;
            assert!(new_vnode.node().is_none());
            assert!(this.vnode.borrow().node().is_some());
            assert!(this.vnode.borrow().container().is_some());
            let vnode = this.vnode.borrow().clone().update(new_vnode);
            this.vnode.replace(vnode);
        })));
        self
    }

    pub fn remove_dom(self: Rc<Self>) -> Rc<Self> {
        self.vnode.replace(self.vnode.borrow().clone().remove_dom());
        self
    }

    pub fn update(self: Rc<Self>, new_vnode: Self) -> Rc<Self> {
        if self.component.update(new_vnode.component) {
            self.vnode.replace(
                self.vnode
                    .borrow()
                    .clone()
                    .update(self.component.render(self.clone())),
            );
        }
        self
    }

    pub fn node(&self) -> Option<web_sys::Node> {
        self.vnode.borrow().node()
    }

    pub fn container(&self) -> Option<web_sys::Node> {
        self.vnode.borrow().container()
    }

    pub fn refresh(&self) {
        let window = window();
        if let Some(id) = self.refresh_callback_id.take() {
            window.cancel_animation_frame(id.into()).unwrap();
        }
        let id = window
            .request_animation_frame(
                self.refresh_callback
                    .borrow()
                    .as_ref()
                    .unwrap()
                    .as_ref()
                    .unchecked_ref(),
            )
            .unwrap();
        self.refresh_callback_id
            .replace(Some(id.try_into().unwrap()));
    }
}

pub struct VNodeElementBuilder {
    tag: &'static str,
    key: Option<Key>,
    class: Vec<IString>,
    dyn_attrs: HashMap<IString, IString>,
    children: Vec<VNode>,
    handlers: Vec<(&'static str, Callback)>,
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

    pub fn set_attr_onclick(&mut self, callback: impl Into<Callback>) -> &mut Self {
        self.handlers.push(("click", callback.into()));
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
            class: IArray::from(std::mem::take(&mut self.class)),
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
            handlers: Rc::from(std::mem::take(&mut self.handlers)),
            container: None,
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

pub trait Component: AsAnyRc {
    fn update(&self, other: Rc<dyn Component>) -> bool;
    fn render(&self, vnode_comp: Rc<VNodeComponent>) -> VNode;
}

pub trait AsAnyRc {
    fn as_any_rc(self: Rc<Self>) -> Rc<dyn Any>;
}

impl<T: 'static> AsAnyRc for T {
    fn as_any_rc(self: Rc<Self>) -> Rc<dyn Any>
    where
        Self: Sized,
    {
        self
    }
}

pub trait PureComponent {
    fn render(&self) -> VNode;
}

#[derive(Clone)]
pub struct VNodePureComponent<T: PureComponent> {
    component: RefCell<T>,
}

impl<T: PureComponent + 'static> VNodePureComponent<T> {
    pub fn new(component: T) -> Self {
        Self {
            component: RefCell::new(component),
        }
    }
}

impl<T: PureComponent + Clone + PartialEq + 'static> Component for VNodePureComponent<T> {
    fn update(&self, other: Rc<dyn Component>) -> bool {
        let other: Rc<Self> = Rc::downcast(other.as_any_rc()).unwrap();
        let other = Rc::unwrap_or_clone(other);
        if self.component != other.component {
            self.component.replace(other.component.into_inner());
            true
        } else {
            false
        }
    }

    fn render(&self, _: Rc<VNodeComponent>) -> VNode {
        self.component.borrow().render()
    }
}

pub trait StatefulComponent {
    fn update(&mut self, other: Self) -> bool;
    fn render(&self, context: &StatefulComponentHandler<Self>) -> VNode
    where
        Self: Sized;
}

#[derive(Clone)]
pub struct VNodeStatefulComponent<T: StatefulComponent> {
    component: RefCell<T>,
    state: Rc<RefCell<HashMap<TypeId, Box<dyn Any>>>>,
}

impl<T: StatefulComponent + 'static> VNodeStatefulComponent<T> {
    pub fn new(component: T) -> Self {
        Self {
            component: RefCell::new(component),
            state: Default::default(),
        }
    }
}

impl<T: StatefulComponent + Clone + 'static> Component for VNodeStatefulComponent<T> {
    fn update(&self, other: Rc<dyn Component>) -> bool {
        let other: Rc<Self> = Rc::downcast(other.as_any_rc()).unwrap();
        let other = Rc::unwrap_or_clone(other);
        if self
            .component
            .borrow_mut()
            .update(other.component.into_inner())
        {
            true
        } else {
            false
        }
    }

    fn render(&self, vnode_comp: Rc<VNodeComponent>) -> VNode {
        self.component
            .borrow()
            .render(&StatefulComponentHandler::new(vnode_comp))
    }
}

pub struct StatefulComponentHandler<C> {
    vnode_comp: Rc<VNodeComponent>,
    phantom: std::marker::PhantomData<C>,
}

impl<C> Clone for StatefulComponentHandler<C> {
    fn clone(&self) -> Self {
        Self::new(self.vnode_comp.clone())
    }
}

impl<C> StatefulComponentHandler<C> {
    fn new(vnode_comp: Rc<VNodeComponent>) -> Self {
        Self {
            vnode_comp,
            phantom: Default::default(),
        }
    }
}

impl<C: StatefulComponent + 'static> StatefulComponentHandler<C> {
    pub fn update(&self) {
        self.vnode_comp.refresh();
    }

    pub fn with_state<T: Default + 'static, R>(&self, f: impl FnOnce(&T) -> R) -> R {
        let component: Rc<VNodeStatefulComponent<C>> =
            Rc::downcast(self.vnode_comp.component.clone().as_any_rc()).unwrap();
        let mut guard = component.state.borrow_mut();
        let state = guard
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::new(T::default()))
            .downcast_ref::<T>()
            .unwrap();
        (f)(state)
    }

    pub fn with_state_mut<T: Default + 'static, R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        let component: Rc<VNodeStatefulComponent<C>> =
            Rc::downcast(self.vnode_comp.component.clone().as_any_rc()).unwrap();
        let mut guard = component.state.borrow_mut();
        let state = guard
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::new(T::default()))
            .downcast_mut::<T>()
            .unwrap();
        let res = (f)(state);
        self.update();
        res
    }

    pub fn callback<T: Default + 'static>(
        &self,
        mut f: impl FnMut(&Self, &mut T) + 'static,
    ) -> Callback {
        let context = self.clone();
        Callback::from(move || context.with_state_mut(|state: &mut T| (f)(&context, state)))
    }
}

#[derive(Clone)]
pub struct Callback {
    closure: Rc<Closure<dyn FnMut()>>,
}

impl implicit_clone::ImplicitClone for Callback {}

impl<F: FnMut() + 'static> From<F> for Callback {
    fn from(f: F) -> Callback {
        Callback {
            closure: Rc::new(Closure::new(f)),
        }
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
    pub type button = super::VNode;
    pub type Text = super::VNode;
    pub type Fragment = super::VNodeFragment;
}

pub mod prelude {
    pub use super::html_context;
    pub use super::log;
    pub use super::AsAnyRc;
    pub use super::Callback;
    pub use super::Component;
    pub use super::PureComponent;
    pub use super::StatefulComponent;
    pub use super::StatefulComponentHandler;
    pub use super::VNode;
    pub use super::VNodePureComponent;
    pub use super::VNodeStatefulComponent;
    pub use implicit_clone::unsync::*;
    pub use yo_html::html;
}
