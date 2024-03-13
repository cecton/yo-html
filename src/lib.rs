//! JSX-like macro similar to what you can find in React or Yew but without framework nor trait.
//!
//! ```ignore
//! let name = "Tippsie";
//! let class = "class";
//! let onclick = todo!();
//! let dynamic_attribute = "style";
//! html! {
//!     <>      // Support fragments
//!         <div class="important">{"Hello "}<strong>{name}</strong></div>
//!         <ul class=["list", "of", class]>
//!             <li><button {onclick}>{"Click here"}</button></li>
//!             <li {dynamic_attribute}="color:red"></li>
//!             <li>("%x", 42)</li>     // Shorthand for: format_args!("%x", 42)
//!         </ul>
//!     </>
//! }
//! ```
//!
//! An example of web framework is provided in the `examples` directory but you need to make your
//! own for this macro to be usable.

mod generate_builder;
mod parser;

#[proc_macro]
pub fn html(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    if input.is_empty() {
        return quote::quote! {
            {
                use html_context::*;

                Fragment::builder("").finish()
            }
        }
        .into();
    }

    // Parse the input tokens into a syn AST
    let item = syn::parse_macro_input!(input as HtmlElement);

    let builder = item.generate_builder();

    quote::quote! {
        {
            use html_context::*;

            #builder
        }
    }
    .into()
}

enum HtmlElement {
    Tagged(HtmlElementTag),
    Fragmented(HtmlElementFragment),
    Block(syn::Block),
    Format(HtmlElementFormat),
}

#[allow(dead_code)]
struct HtmlElementTag {
    opening_tag: HtmlOpeningTag,
    children: Vec<HtmlElement>,
    closing_tag: Option<HtmlClosingTag>,
}

#[allow(dead_code)]
struct HtmlElementFragment {
    opening_fragment: HtmlOpeningFragment,
    children: Vec<HtmlElement>,
    closing_fragment: HtmlClosingFragment,
}

#[allow(dead_code)]
struct HtmlOpeningFragment {
    opening_bracket_token: syn::Token![<],
    closing_bracket_token: syn::Token![>],
}

#[allow(dead_code)]
struct HtmlClosingFragment {
    opening_bracket_token: syn::Token![<],
    closing_slash_token: syn::Token![/],
    closing_bracket_token: syn::Token![>],
}

#[allow(dead_code)]
struct HtmlOpeningTag {
    opening_bracket_token: syn::Token![<],
    tag: syn::Ident,
    generics: syn::Generics,
    attributes: Vec<HtmlAttribute>,
    self_closing_slash_token: Option<syn::Token![/]>,
    closing_bracket_token: syn::Token![>],
}

#[allow(dead_code)]
struct HtmlClosingTag {
    opening_bracket_token: syn::Token![<],
    closing_slash_token: syn::Token![/],
    tag: syn::Ident,
    closing_bracket_token: syn::Token![>],
}

#[allow(dead_code)]
struct HtmlAttribute {
    name: HtmlAttributeName,
    eq_token: Option<syn::Token![=]>,
    value: Option<HtmlAttributeValue>,
}

#[allow(dead_code)]
enum HtmlAttributeName {
    Block(syn::Block),
    Ident(syn::Ident),
    Shorthand {
        brace_token: syn::token::Brace,
        ident: syn::Ident,
    },
}

#[allow(dead_code)]
enum HtmlAttributeValue {
    Block(syn::Block),
    ExprArray(syn::ExprArray),
    Lit(syn::Lit),
}

#[allow(dead_code)]
struct HtmlElementFormat {
    paren_token: syn::token::Paren,
    args: syn::punctuated::Punctuated<syn::Expr, syn::Token![,]>,
}
