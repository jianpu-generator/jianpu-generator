use super::*;

#[test]
fn absent_section_returns_none() {
    let (section, errors) = parse_group("", 0);
    assert!(section.is_none());
    assert!(errors.is_empty());
}

#[test]
fn parses_named_group_with_abbreviation() {
    let (section, errors) = parse_group("Vocal [v] = s1 s2\n", 0);
    assert!(errors.is_empty());
    let section = section.unwrap();
    assert_eq!(section.groups.len(), 1);
    assert_eq!(section.groups[0].display_name, "Vocal");
    assert_eq!(section.groups[0].abbreviation, "v");
    assert_eq!(section.groups[0].members, vec!["s1", "s2"]);
}

#[test]
fn parses_group_without_bracket_abbreviation() {
    let (section, errors) = parse_group("Vocal = s1 s2\n", 0);
    assert!(errors.is_empty());
    let section = section.unwrap();
    assert_eq!(section.groups[0].display_name, "Vocal");
    assert_eq!(section.groups[0].abbreviation, "Vocal");
}

#[test]
fn malformed_line_without_equals_is_recoverable() {
    let (section, errors) = parse_group("not a declaration\n", 0);
    assert_eq!(section.unwrap().groups.len(), 0);
    assert_eq!(errors.len(), 1);
}

#[test]
fn empty_member_list_is_recoverable() {
    let (section, errors) = parse_group("Vocal [v] = \n", 0);
    assert_eq!(section.unwrap().groups.len(), 0);
    assert_eq!(errors.len(), 1);
}
