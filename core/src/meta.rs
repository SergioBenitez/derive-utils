use syn::{self, Meta::*, Lit::*};

use generator::Result;
use spanned::Spanned;
use proc_macro::Span;

// Spans of k/v pair, key, then value.
pub struct SpanWrapped<T> {
    pub meta_span: Span,
    pub key_span: Span,
    pub value_span: Span,
    pub value: T
}

pub trait FromMeta: Sized {
    fn from_meta(meta: &syn::Meta) -> Result<Self>;

    fn from_attr(name: &str, attr: &syn::Attribute) -> Result<Self> {
        let meta = attr.interpret_meta().ok_or_else(|| {
            attr.span()
                .error("malformed attribute")
                .help(format!("expected syntax: #[{}(key = value, ..)]", name))
        })?;

        Self::from_meta(&meta)
    }

    fn from_attrs(name: &str, attrs: &[syn::Attribute]) -> Option<Result<Self>> {
        let tokens = name.parse().expect("`name` contained invalid tokens");
        let path = syn::parse(tokens).expect("`name` was not a valid path");
        let attr = attrs.iter().filter(|attr| attr.path == path).next()?;
        Some(Self::from_attr(name, attr))
    }

    fn default() -> Option<Self> {
        None
    }
}

impl FromMeta for usize {
    fn from_meta(meta: &syn::Meta) -> Result<Self> {
        if let NameValue(nv) = meta {
            if let Int(ref i) = nv.lit {
                if i.value() <= usize::max_value() as u64 {
                    return Ok(i.value() as usize);
                }

                return Err(nv.lit.span().error("value is out of range for `usize`"));
            }

            return Err(nv.lit.span().error("invalid value: expected unsigned integer"));
        }

        Err(meta.span().error("expected key/value pair"))
    }
}

impl FromMeta for String {
    fn from_meta(meta: &syn::Meta) -> Result<Self> {
        if let NameValue(nv) = meta {
            if let Str(ref s) = nv.lit {
                return Ok(s.value());
            }

            return Err(nv.lit.span().error("invalid value: expected string"));
        }

        Err(meta.span().error("expected key/value pair"))
    }
}

impl FromMeta for bool {
    fn from_meta(meta: &syn::Meta) -> Result<Self> {
        if let NameValue(nv) = meta {
            if let Bool(ref b) = nv.lit {
                return Ok(b.value);
            }

            return Err(nv.lit.span().error("invalid value: expected boolean"));
        }

        Err(meta.span().error("expected key/value pair"))
    }
}

impl<T: FromMeta> FromMeta for Option<T> {
    fn from_meta(meta: &syn::Meta) -> Result<Self> {
        T::from_meta(meta).map(Some)
    }

    fn default() -> Option<Self> {
        Some(None)
    }
}

impl<T: FromMeta> FromMeta for SpanWrapped<T> {
    fn from_meta(meta: &syn::Meta) -> Result<Self> {
        let (key_span, value_span) = if let NameValue(nv) = meta {
            (Spanned::span(&nv.ident), nv.lit.span())
        } else {
            return Err(meta.span().error("expected key/value pair"));
        };

        T::from_meta(meta).map(|value| SpanWrapped {
            meta_span: meta.span(),
            key_span, value_span, value
        })
    }
}