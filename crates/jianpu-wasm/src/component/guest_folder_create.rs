//! The New folder `Guest` methods: convert WIT values to
//! `jianpu_generator::file_tree` types, call it, convert back.
#![allow(clippy::needless_pass_by_value)]
use super::*;
use jianpu_generator::file_tree as tree;

pub(super) fn file_store_create_folder(
    state: FileStoreState,
    parent: Option<String>,
    name: String,
    context: FileOperationContext,
) -> FileStoreResponse {
    let outcome = parse_optional(parent.as_deref()).and_then(|parent| {
        tree::create_folder(&state.into(), parent.as_ref(), &name, &context.into())
    });
    store_response(outcome)
}

pub(super) fn file_store_create_folder_error(
    state: FileStoreState,
    parent: Option<String>,
    name: String,
    context: FileOperationContext,
) -> Option<String> {
    tree::create_folder_error(&state.into(), parent.as_deref(), &name, &context.into())
}
