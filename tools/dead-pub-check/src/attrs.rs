use syn::Attribute;

pub fn has_attr_named(attrs: &[Attribute], name: &str) -> bool {
    attrs.iter().any(|attr| attr.path().is_ident(name))
}

pub fn has_cfg_test(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.path().is_ident("cfg")
            && matches!(
                &attr.meta,
                syn::Meta::List(list) if list.tokens.to_string().replace(' ', "").contains("test")
            )
    })
}

pub fn has_allow_dead_code(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.path().is_ident("allow")
            && matches!(
                &attr.meta,
                syn::Meta::List(list) if list.tokens.to_string().replace(' ', "").contains("dead_code")
            )
    })
}

pub fn has_test_attr_or_dead_allow(attrs: &[Attribute]) -> bool {
    has_attr_named(attrs, "test") || has_allow_dead_code(attrs)
}

pub fn is_pub(vis: &syn::Visibility) -> bool {
    matches!(vis, syn::Visibility::Public(_))
}

pub fn path_attr_value(attrs: &[Attribute]) -> Option<String> {
    attrs.iter().find_map(|attr| {
        if !attr.path().is_ident("path") {
            return None;
        }
        let syn::Meta::NameValue(nv) = &attr.meta else {
            return None;
        };
        let syn::Expr::Lit(expr_lit) = &nv.value else {
            return None;
        };
        let syn::Lit::Str(s) = &expr_lit.lit else {
            return None;
        };
        Some(s.value())
    })
}
