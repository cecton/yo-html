use implicit_clone::unsync::*;
use std::collections::HashMap;
use std::rc::Rc;

pub enum VNode {
    Tagged {
        tag: &'static str,
        children: Rc<[VNode]>,
        // attributes...
    },
    Text(IString),
    Fragment(Rc<[VNode]>),
    Component(Rc<dyn Component>),
}

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
    pub fn builder(_tag: &'static str) -> VNodeBuilder {
        VNodeBuilder::default()
    }
}

#[derive(Default)]
pub struct VNodeBuilder {
    tag: &'static str,
    class: Vec<IString>,
    style: Option<IString>,
    numeric: Option<f64>,
    dyn_attrs: HashMap<IString, IString>,
    children: Vec<VNode>,
}

impl VNodeBuilder {
    pub fn set_attr_class(&mut self, class: impl Into<IString>) -> &mut Self {
        self.add_attr_class(class, 1)
    }

    pub fn add_attr_class(
        &mut self,
        class: impl Into<IString>,
        additional: usize,
    ) -> &mut Self {
        //dbg!(additional);
        self.class.reserve_exact(additional);
        self.class.push(class.into());
        self
    }

    pub fn set_attr_style(&mut self, style: impl Into<IString>) -> &mut Self {
        self.style.replace(style.into());
        self
    }

    pub fn set_attr_numeric(&mut self, numeric: impl Into<f64>) -> &mut Self {
        self.numeric.replace(numeric.into());
        self
    }

    pub fn add_child(&mut self, element: impl Into<VNode>, additional: usize) -> &mut Self {
        //dbg!(additional);
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
        //dbg!(additional);
        // NOTE: it's actually better to use reserve() here instead of reserve_exact() because
        //       there can be multiple different keys that will have their own number of items.
        self.dyn_attrs.reserve(additional);
        self.dyn_attrs.insert(name.into(), value.into());
        self
    }

    pub fn finish(&mut self) -> VNode {
        VNode::Tagged {
            tag: self.tag,
            children: Rc::from(std::mem::take(&mut self.children)),
        }
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
    pub type br = super::VNode;
    pub type Text = super::VNode;
    pub type Fragment = super::VNode;
    pub use super::MyComponent;
}

pub mod prelude {
    pub use super::html_context;
    pub use yo_html::html;
}
