use super::*;

fn path(raw: &str) -> TreePath {
    TreePath::parse(raw).unwrap()
}

fn strings(raw: &[&str]) -> Vec<String> {
    raw.iter().map(|raw| (*raw).to_string()).collect()
}

/// One line per row: indent, name, `/` + count for folders, markers.
fn render(rows: &[TreeViewRow]) -> Vec<String> {
    rows.iter()
        .map(|row| {
            let marker = match row.kind {
                TreeViewRowKind::Folder | TreeViewRowKind::DemoFolder => format!(
                    "/ {} {}",
                    row.file_count,
                    if row.expanded { "open" } else { "closed" }
                ),
                _ => String::new(),
            };
            format!(
                "{}{}{}{}",
                "  ".repeat(row.depth),
                row.name,
                marker,
                if row.is_active { " *" } else { "" }
            )
        })
        .collect()
}

fn tree() -> FileTree {
    FileTree::build(
        &[
            path("Hymns/Advent/O Come.jianpu"),
            path("Hymns/Advent/Rejoice.jianpu"),
            path("Hymns/Amazing.jianpu"),
            path("Scratch.jianpu"),
        ],
        &[path("Empty")],
    )
    .unwrap()
}

#[test]
fn folders_are_collapsed_by_default_and_show_recursive_file_counts() {
    let rows = tree().view_rows(&TreeViewInput {
        active: "Scratch.jianpu",
        expanded: &[],
        demo_file_names: &strings(&["01.jianpu", "02.jianpu"]),
    });
    assert_eq!(
        render(&rows),
        vec![
            "Empty/ 0 closed",
            "Hymns/ 3 closed",
            "Scratch.jianpu *",
            "Demo/ 2 closed",
        ]
    );
}

#[test]
fn an_expanded_folder_shows_its_children_and_remembers_nothing_else() {
    let rows = tree().view_rows(&TreeViewInput {
        active: "Scratch.jianpu",
        expanded: &strings(&["Hymns"]),
        demo_file_names: &[],
    });
    assert_eq!(
        render(&rows),
        vec![
            "Empty/ 0 closed",
            "Hymns/ 3 open",
            "  Advent/ 2 closed",
            "  Amazing.jianpu",
            "Scratch.jianpu *",
        ]
    );
}

#[test]
fn ancestors_of_the_active_file_are_open_even_when_not_expanded() {
    let rows = tree().view_rows(&TreeViewInput {
        active: "Hymns/Advent/Rejoice.jianpu",
        expanded: &[],
        demo_file_names: &[],
    });
    assert_eq!(
        render(&rows),
        vec![
            "Empty/ 0 closed",
            "Hymns/ 3 open",
            "  Advent/ 2 open",
            "    O Come.jianpu",
            "    Rejoice.jianpu *",
            "  Amazing.jianpu",
            "Scratch.jianpu",
        ]
    );
}

#[test]
fn an_expanded_empty_folder_gets_a_hint_row() {
    let rows = tree().view_rows(&TreeViewInput {
        active: "Scratch.jianpu",
        expanded: &strings(&["Empty"]),
        demo_file_names: &[],
    });
    assert_eq!(rows[0].kind, TreeViewRowKind::Folder);
    assert_eq!(rows[1].kind, TreeViewRowKind::EmptyFolderHint);
    assert_eq!(rows[1].depth, 1);
}

#[test]
fn demo_is_last_collapsed_by_default_and_opens_by_key_or_active_file() {
    let demo = strings(&["01.jianpu", "02.jianpu"]);
    let opened = tree().view_rows(&TreeViewInput {
        active: "Scratch.jianpu",
        expanded: &strings(&[DEMO_EXPANSION_KEY]),
        demo_file_names: &demo,
    });
    assert_eq!(
        &render(&opened)[3..],
        ["Demo/ 2 open", "  01.jianpu", "  02.jianpu"]
    );
    assert_eq!(opened[4].kind, TreeViewRowKind::DemoFile);

    let active_demo = tree().view_rows(&TreeViewInput {
        active: "02.jianpu",
        expanded: &[],
        demo_file_names: &demo,
    });
    assert_eq!(
        &render(&active_demo)[3..],
        ["Demo/ 2 open", "  01.jianpu", "  02.jianpu *"]
    );
}

#[test]
fn active_label_splits_folders_from_the_file_name() {
    assert_eq!(
        active_label("Hymns/Advent/O Come.jianpu", &[]),
        ActiveLabel {
            folder_names: strings(&["Hymns", "Advent"]),
            file_name: "O Come.jianpu".to_string(),
        }
    );
    assert_eq!(
        active_label("a.jianpu", &[]).folder_names,
        Vec::<String>::new()
    );
    assert_eq!(
        active_label("01.jianpu", &strings(&["01.jianpu"])).folder_names,
        strings(&["Demo"])
    );
}
