use std::collections::HashMap;

use syn::visit::{self, Visit};
use syn::{Ident, ItemFn, ItemMod, ItemUse};

use crate::attrs::{has_attr_named, has_cfg_test, is_pub};

pub struct UsageVisitor {
    pub counts: HashMap<String, usize>,
    pub test_depth: usize,
}

impl<'ast> Visit<'ast> for UsageVisitor {
    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        if has_cfg_test(&node.attrs) || has_attr_named(&node.attrs, "test") {
            self.test_depth += 1;
            visit::visit_item_fn(self, node);
            self.test_depth -= 1;
        } else {
            visit::visit_item_fn(self, node);
        }
    }

    fn visit_item_mod(&mut self, node: &'ast ItemMod) {
        if has_cfg_test(&node.attrs) {
            self.test_depth += 1;
            visit::visit_item_mod(self, node);
            self.test_depth -= 1;
        } else {
            visit::visit_item_mod(self, node);
        }
    }

    fn visit_item_use(&mut self, node: &'ast ItemUse) {
        if is_pub(&node.vis) {
            // A `pub use` re-export declares the export surface — it isn't a
            // real call/reference site, so don't let it count as usage.
            return;
        }
        visit::visit_item_use(self, node);
    }

    fn visit_ident(&mut self, ident: &'ast Ident) {
        if self.test_depth == 0 {
            *self.counts.entry(ident.to_string()).or_insert(0) += 1;
        }
    }

    // Macro bodies (`format!(...)`, `assert!(...)`, `vec![...]`, ...) are
    // opaque token streams to syn, not parsed AST — a call made only inside
    // one would otherwise be invisible to `visit_ident`. Walk the raw tokens
    // by hand so those call sites still count as usage.
    fn visit_macro(&mut self, node: &'ast syn::Macro) {
        if self.test_depth == 0 {
            record_token_stream_idents(node.tokens.clone(), &mut self.counts);
        }
    }
}

fn record_token_stream_idents(
    tokens: proc_macro2::TokenStream,
    counts: &mut HashMap<String, usize>,
) {
    for tt in tokens {
        match tt {
            proc_macro2::TokenTree::Ident(ident) => {
                *counts.entry(ident.to_string()).or_insert(0) += 1;
            }
            proc_macro2::TokenTree::Group(group) => {
                record_token_stream_idents(group.stream(), counts);
            }
            proc_macro2::TokenTree::Punct(_) | proc_macro2::TokenTree::Literal(_) => {}
        }
    }
}
