use std::path::{Path, PathBuf};

use syn::{
    Item, ItemConst, ItemEnum, ItemFn, ItemImpl, ItemMod, ItemStatic, ItemStruct, ItemTrait,
    ItemType,
};

use crate::attrs::{has_allow_dead_code, has_cfg_test, has_test_attr_or_dead_allow, is_pub};

pub struct PubDef {
    pub name: String,
    pub file: PathBuf,
    pub line: usize,
}

pub fn collect_pub_defs(items: &[Item], file: &Path, in_test: bool, out: &mut Vec<PubDef>) {
    macro_rules! record {
        ($item:expr) => {
            if is_pub(&$item.vis) && !in_test && !has_allow_dead_code(&$item.attrs) {
                out.push(PubDef {
                    name: $item.ident.to_string(),
                    file: file.to_path_buf(),
                    line: $item.ident.span().start().line,
                });
            }
        };
    }

    for syntax_item in items {
        match syntax_item {
            Item::Fn(f) => {
                let f: &ItemFn = f;
                if is_pub(&f.vis) && !in_test && !has_test_attr_or_dead_allow(&f.attrs) {
                    out.push(PubDef {
                        name: f.sig.ident.to_string(),
                        file: file.to_path_buf(),
                        line: f.sig.ident.span().start().line,
                    });
                }
            }
            Item::Struct(s) => {
                let s: &ItemStruct = s;
                record!(s);
            }
            Item::Enum(e) => {
                let e: &ItemEnum = e;
                record!(e);
            }
            Item::Const(c) => {
                let c: &ItemConst = c;
                record!(c);
            }
            Item::Static(s) => {
                let s: &ItemStatic = s;
                record!(s);
            }
            Item::Type(t) => {
                let t: &ItemType = t;
                record!(t);
            }
            Item::Trait(t) => {
                let t: &ItemTrait = t;
                record!(t);
            }
            Item::Impl(imp) => {
                let imp: &ItemImpl = imp;
                if imp.trait_.is_none() {
                    let nested_test = in_test || has_cfg_test(&imp.attrs);
                    for impl_item in &imp.items {
                        if let syn::ImplItem::Fn(f) = impl_item {
                            if is_pub(&f.vis)
                                && !nested_test
                                && !has_test_attr_or_dead_allow(&f.attrs)
                            {
                                out.push(PubDef {
                                    name: f.sig.ident.to_string(),
                                    file: file.to_path_buf(),
                                    line: f.sig.ident.span().start().line,
                                });
                            }
                        }
                    }
                }
            }
            Item::Mod(m) => {
                let m: &ItemMod = m;
                let nested_test = in_test || has_cfg_test(&m.attrs);
                if let Some((_, inner)) = &m.content {
                    collect_pub_defs(inner, file, nested_test, out);
                }
            }
            _ => {}
        }
    }
}
