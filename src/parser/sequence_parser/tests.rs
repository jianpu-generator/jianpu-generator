use super::*;

#[test]
fn absent_section_returns_none() {
    let (sequence, errors) = parse_sequence("", 0);
    assert!(sequence.is_none());
    assert!(errors.is_empty());
}

#[test]
fn blank_section_returns_none() {
    let (sequence, errors) = parse_sequence("   \n  ", 0);
    assert!(sequence.is_none());
    assert!(errors.is_empty());
}

#[test]
fn parses_comma_separated_labels() {
    let (sequence, errors) = parse_sequence("A, B, A, C", 0);
    assert!(errors.is_empty());
    let sequence = sequence.expect("expected a sequence");
    let labels: Vec<&str> = sequence.entries.iter().map(|e| e.label.as_str()).collect();
    assert_eq!(labels, vec!["A", "B", "A", "C"]);
}

#[test]
fn tolerates_surrounding_whitespace_and_newlines() {
    let (sequence, errors) = parse_sequence("  A ,\nB\n, C  \n", 0);
    assert!(errors.is_empty());
    let sequence = sequence.expect("expected a sequence");
    let labels: Vec<&str> = sequence.entries.iter().map(|e| e.label.as_str()).collect();
    assert_eq!(labels, vec!["A", "B", "C"]);
}

#[test]
fn single_entry() {
    let (sequence, errors) = parse_sequence("A", 0);
    assert!(errors.is_empty());
    let sequence = sequence.expect("expected a sequence");
    assert_eq!(sequence.entries.len(), 1);
    assert_eq!(sequence.entries[0].label, "A");
}

#[test]
fn empty_entry_is_a_recoverable_error() {
    let (sequence, errors) = parse_sequence("A,,B", 0);
    assert_eq!(errors.len(), 1);
    let sequence = sequence.expect("expected a sequence");
    let labels: Vec<&str> = sequence.entries.iter().map(|e| e.label.as_str()).collect();
    assert_eq!(labels, vec!["A", "B"]);
}

#[test]
fn trailing_comma_is_a_recoverable_error() {
    let (sequence, errors) = parse_sequence("A, B,", 0);
    assert_eq!(errors.len(), 1);
    let sequence = sequence.expect("expected a sequence");
    let labels: Vec<&str> = sequence.entries.iter().map(|e| e.label.as_str()).collect();
    assert_eq!(labels, vec!["A", "B"]);
}

#[test]
fn entry_span_points_to_absolute_offset() {
    let content = "foo, bar";
    let offset = 100;
    let (sequence, _errors) = parse_sequence(content, offset);
    let sequence = sequence.expect("expected a sequence");
    assert_eq!(sequence.entries[0].span, Span::new(100, 103));
    assert_eq!(sequence.entries[1].span, Span::new(105, 108));
}
