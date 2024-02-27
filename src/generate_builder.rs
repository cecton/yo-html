use super::*;

impl HtmlElement {
    pub(crate) fn generate_builder(&self) -> proc_macro2::TokenStream {
        use syn::spanned::Spanned;
        use HtmlElement::*;

        match self {
            Tagged(element) => {
                let tag = &element.opening_tag.tag;
                let tag_str = tag.to_string();
                let generics = &element.opening_tag.generics;

                let mut visited_attrs = Vec::new();
                let attributes = element
                    .opening_tag
                    .attributes
                    .iter()
                    .map(|x| {
                        let many;
                        let mut values = match &x.value {
                            Some(HtmlAttributeValue::Block(value)) => {
                                many = false;
                                vec![quote_block(value)]
                            }
                            Some(HtmlAttributeValue::Lit(lit)) => {
                                many = false;
                                vec![quote::ToTokens::to_token_stream(lit)]
                            }
                            Some(HtmlAttributeValue::ExprArray(array)) => {
                                many = true;
                                array
                                    .elems
                                    .iter()
                                    .map(quote::ToTokens::to_token_stream)
                                    .collect::<Vec<_>>()
                            }
                            None => {
                                many = false;
                                vec![]
                            }
                        };
                        let mut attrs_count = vec![values.len()];
                        attrs_count.resize(values.len(), 0);

                        match &x.name {
                            HtmlAttributeName::Block(name) => {
                                let name = quote_block(name);
                                quote::quote_spanned! {name.span()=>
                                    #(.add_attr(#name, #values, #attrs_count))*
                                }
                            }
                            HtmlAttributeName::Ident(name) => {
                                if visited_attrs.contains(&name) {
                                    values.clear();
                                    values.push(quote::quote_spanned! {name.span()=>
                                        compile_error!("attribute already defined")
                                    });
                                }
                                visited_attrs.push(name);
                                if many {
                                    let method = quote::format_ident!("add_attr_{}", name);
                                    quote::quote_spanned! {name.span()=>
                                        #(.#method(#values, #attrs_count))*
                                    }
                                } else {
                                    let method = quote::format_ident!("set_attr_{}", name);
                                    quote::quote_spanned! {name.span()=>
                                        #(.#method(#values))*
                                    }
                                }
                            }
                            HtmlAttributeName::Shorthand { ident, .. } => {
                                let mut name = quote::ToTokens::to_token_stream(&ident);
                                if visited_attrs.contains(&ident) {
                                    name = quote::quote_spanned! {ident.span()=>
                                        compile_error!("attribute already defined")
                                    };
                                }
                                visited_attrs.push(ident);
                                let method = quote::format_ident!("set_attr_{}", ident);
                                quote::quote_spanned! {ident.span()=>
                                    .#method(#name)
                                }
                            }
                        }
                    })
                    .collect::<Vec<_>>();

                let children = element
                    .children
                    .iter()
                    .enumerate()
                    .map(|(i, x)| {
                        let children_count = if i == 0 { element.children.len() } else { 0 };
                        let child = x.generate_builder();
                        quote::quote_spanned! {child.span()=>
                            .add_child(#child, #children_count)
                        }
                    })
                    .collect::<Vec<_>>();

                quote::quote! {
                    <#tag #generics>::builder(#tag_str)
                        #(#attributes)*
                        #(#children)*
                        .finish()
                }
            }
            Fragmented(fragment) => {
                let children = fragment
                    .children
                    .iter()
                    .enumerate()
                    .map(|(i, x)| {
                        let children_count = if i == 0 { fragment.children.len() } else { 0 };
                        let child = x.generate_builder();
                        quote::quote_spanned! {child.span()=>
                            .add_child(#child, #children_count)
                        }
                    })
                    .collect::<Vec<_>>();

                quote::quote! {
                    Fragment::builder()
                        #(#children)*
                        .finish()
                }
            }
            Block(block) => quote_block(block),
            Format(format) => {
                let args = &format.args;

                quote::quote_spanned! {args.span()=>
                    Text::from(format_args!(#args))
                }
            }
        }
    }
}

fn quote_block(block: &syn::Block) -> proc_macro2::TokenStream {
    if block.stmts.len() == 1 {
        quote::ToTokens::to_token_stream(&block.stmts[0])
    } else {
        quote::ToTokens::to_token_stream(block)
    }
}
