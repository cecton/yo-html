use super::*;
use syn::spanned::Spanned;

impl syn::parse::Parse for HtmlElement {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        assert!(
            !input.is_empty(),
            "attempt to parse an element from an empty input"
        );
        if input.peek(syn::Token![<]) && input.peek2(syn::Token![>]) {
            Ok(Self::Fragmented(input.parse()?))
        } else if input.peek(syn::Token![<]) {
            Ok(Self::Tagged(input.parse()?))
        } else if input.peek(syn::token::Brace) {
            Ok(Self::Block(input.parse()?))
        } else if input.peek(syn::token::Paren) {
            Ok(Self::Format(input.parse()?))
        } else {
            Err(input.error("could not parse element"))
        }
    }
}

impl syn::parse::Parse for HtmlElementTag {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let opening_tag: HtmlOpeningTag = input.parse()?;
        let span = opening_tag.tag.span();
        let self_closing = opening_tag.self_closing_slash_token.is_some();
        Ok(Self {
            opening_tag,
            children: {
                let mut children = Vec::new();
                if !self_closing {
                    while !((input.peek(syn::Token![<]) && input.peek2(syn::Token![/]))
                        || input.is_empty())
                    {
                        children.push(input.parse()?);
                    }
                }
                children
            },
            closing_tag: {
                (!self_closing)
                    .then(|| {
                        input.parse::<HtmlClosingTag>().map_err(|_| {
                            syn::parse::Error::new(span, "could not find matching close tag")
                        })
                    })
                    .transpose()?
            },
        })
    }
}

impl syn::parse::Parse for HtmlElementFragment {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let span = input.span();
        Ok(Self {
            opening_fragment: input.parse()?,
            children: {
                let mut children = Vec::new();
                while !((input.peek(syn::Token![<]) && input.peek2(syn::Token![/]))
                    || input.is_empty())
                {
                    children.push(input.parse()?);
                }
                children
            },
            closing_fragment: input.parse().map_err(|_| {
                syn::parse::Error::new(span, "could not find matching close fragment")
            })?,
        })
    }
}

impl syn::parse::Parse for HtmlOpeningFragment {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            opening_bracket_token: input.parse()?,
            closing_bracket_token: input.parse()?,
        })
    }
}

impl syn::parse::Parse for HtmlClosingFragment {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            opening_bracket_token: input.parse()?,
            closing_slash_token: input.parse()?,
            closing_bracket_token: input.parse()?,
        })
    }
}

impl syn::parse::Parse for HtmlOpeningTag {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            opening_bracket_token: input.parse()?,
            tag: input.parse()?,
            generics: input.parse()?,
            attributes: {
                let mut attrs = Vec::new();
                while !(input.peek(syn::Token![>]) || input.peek(syn::Token![/])) {
                    attrs.push(input.parse()?);
                }
                attrs
            },
            self_closing_slash_token: {
                input
                    .peek(syn::Token![/])
                    .then(|| input.parse())
                    .transpose()?
            },
            closing_bracket_token: input.parse()?,
        })
    }
}

impl syn::parse::Parse for HtmlClosingTag {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            opening_bracket_token: input.parse()?,
            closing_slash_token: input.parse()?,
            tag: input.parse()?,
            closing_bracket_token: input.parse()?,
        })
    }
}

impl syn::parse::Parse for HtmlAttribute {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name: HtmlAttributeName = input.parse()?;
        let span = match &name {
            HtmlAttributeName::Block(block) => block.span(),
            HtmlAttributeName::Ident(ident) => ident.span(),
            HtmlAttributeName::Shorthand { ident, .. } => ident.span(),
        };
        match name {
            HtmlAttributeName::Shorthand { .. } => Ok(Self {
                name,
                eq_token: None,
                value: None,
            }),
            _ => {
                Ok(Self {
                    name,
                    eq_token: Some(input.parse().map_err(|_| {
                        syn::parse::Error::new(span, "missing `=` token in attribute")
                    })?),
                    value: Some(
                        input.parse().map_err(|_| {
                            syn::parse::Error::new(span, "missing value in attribute")
                        })?,
                    ),
                })
            }
        }
    }
}

impl syn::parse::Parse for HtmlAttributeName {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(syn::Ident) {
            Ok(Self::Ident(input.parse()?))
        } else if input.peek(syn::token::Brace) && input.peek2(syn::Token![=]) {
            Ok(Self::Block(input.parse()?))
        } else if input.peek(syn::token::Brace) {
            let content;
            Ok(Self::Shorthand {
                brace_token: syn::braced!(content in input),
                ident: content.parse()?,
            })
        } else {
            Err(input.error("expected identifier, block or shorthand property"))
        }
    }
}

impl syn::parse::Parse for HtmlAttributeValue {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(syn::token::Brace) {
            Ok(Self::Block(input.parse()?))
        } else if input.peek(syn::token::Bracket) {
            Ok(Self::ExprArray(input.parse()?))
        } else if input.peek(syn::Lit) {
            Ok(Self::Lit(input.parse()?))
        } else {
            Err(input.error("expected block, array of expressions or literal"))
        }
    }
}

impl syn::parse::Parse for HtmlElementFormat {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        Ok(Self {
            paren_token: syn::parenthesized!(content in input),
            args: content.parse_terminated(syn::Expr::parse, syn::Token![,])?,
        })
    }
}

/*
fn parse<T: syn::parse::Parse>(input: syn::parse::ParseStream) -> syn::Result<(T, proc_macro2::TokenStream)> {
    let begin = input.cursor();
    let value: T = input.parse()?;
    let end = input.cursor();

    let mut cursor = begin;
    let mut tokens = proc_macro2::TokenStream::new();
    while cursor < end {
        let (token, next) = cursor.token_tree().unwrap();
        tokens.extend(std::iter::once(token));
        cursor = next;
    }

    Ok((value, tokens))
}
*/
