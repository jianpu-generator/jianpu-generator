//! Lowering `parts [ … ]`.

use anyfix_expr::{Node, NodeKind};
use itertools::Itertools;

use super::{as_call, as_field, Lowerer};
use crate::sound_label::sound_label;

impl Lowerer<'_> {
    // ── parts ───────────────────────────────────────────────────────────

    pub(super) fn parts(&mut self, node: &Node) -> Option<String> {
        let parts = self.elements(node, "[")?;
        Some(
            parts
                .iter()
                .filter_map(|part| self.part(part))
                .map(|line| format!("{line}\n"))
                .collect(),
        )
    }

    /// `"Name" [Abbr] : kind` as `Name [Abbr] = kind …`.
    fn part(&mut self, node: &Node) -> Option<String> {
        let shape = as_call(node).map(|(call, name)| (name, call.arguments.as_slice()));
        let (display, abbreviation, kind) = match shape {
            Some((name, [display, abbreviation, kind])) if name == "_ _ _" => {
                (display, Some(abbreviation), kind)
            }
            Some((name, [display, kind])) if name == "_ _" => (display, None, kind),
            _ => {
                self.error(node.span, "expected a part, like `\"Soprano\" [S] : notes`");
                return None;
            }
        };
        let NodeKind::String(display) = &display.kind else {
            self.error(display.span, "a part's display name must be a string");
            return None;
        };
        let left = match abbreviation {
            Some(abbreviation) => {
                format!(
                    "{} [{}]",
                    display.value,
                    self.bracketed_abbreviation(abbreviation)?
                )
            }
            None => display.value.clone(),
        };
        Some(format!("{left} = {}", self.part_kind(kind)?))
    }

    fn part_kind(&mut self, node: &Node) -> Option<String> {
        let Some((call, name)) = as_call(node) else {
            self.error(
                node.span,
                "expected a part kind: notes, chords, percussion, follow [X] or lyrics [X]",
            );
            return None;
        };
        let arguments = call.arguments.as_slice();
        let (head, options) = match (name.as_str(), arguments) {
            ("notes" | "chords" | "percussion", []) => (name.clone(), None),
            ("notes _" | "chords _" | "percussion _", [options]) => {
                (name.trim_end_matches(" _").to_string(), Some(options))
            }
            ("follow _", [target]) => (
                format!("follow[{}]", self.bracketed_abbreviation(target)?),
                None,
            ),
            ("follow _ _", [target, options]) => (
                format!("follow[{}]", self.bracketed_abbreviation(target)?),
                Some(options),
            ),
            ("lyrics _", [target]) => (
                format!("lyrics[{}]", self.bracketed_abbreviation(target)?),
                None,
            ),
            _ => {
                self.error(
                    node.span,
                    format!("found `{name}`; expected notes, chords, percussion, follow [X] or lyrics [X], each optionally followed by `{{ options }}`"),
                );
                return None;
            }
        };
        let is_percussion = head == "percussion";
        let suffixes = match options {
            Some(options) => self.part_options(options, is_percussion)?,
            None => Vec::new(),
        };
        Some(std::iter::once(head).chain(suffixes).join(" "))
    }

    /// `{ instrument : violin, sound : 41, volume : 80%, octave : +1 }` as the
    /// suffixes the current syntax tells apart by shape, in its order.
    fn part_options(&mut self, node: &Node, is_percussion: bool) -> Option<Vec<String>> {
        let fields = self.elements(node, "{")?;
        let mut instrument = None;
        let mut sound = None;
        let mut volume = None;
        let mut octave = None;
        for field in fields {
            let Some((words, value)) = as_field(field) else {
                self.error(field.span, "expected an option, like `volume : 80%`");
                continue;
            };
            match words.as_slice() {
                ["instrument"] => instrument = self.flat_text(value),
                ["sound"] => sound = self.sound(value, is_percussion),
                ["volume"] => volume = self.flat_text(value),
                ["octave"] => octave = self.flat_text(value),
                _ => self.error(
                    field.span,
                    format!(
                        "unknown part option `{}`; expected instrument, sound, volume or octave",
                        words.join(" ")
                    ),
                ),
            }
        }
        Some(
            [instrument, sound, volume, octave]
                .into_iter()
                .flatten()
                .collect(),
        )
    }

    /// `sound : 41` as the quoted `"41: Violin"` label the current syntax reads.
    fn sound(&mut self, node: &Node, is_percussion: bool) -> Option<String> {
        let program = match &node.kind {
            NodeKind::Number(text) => text.parse::<u8>().ok(),
            _ => None,
        };
        match program {
            Some(program) => Some(format!(
                "\"{}\"",
                sound_label(program, is_percussion, self.instruments)
            )),
            None => {
                self.error(
                    node.span,
                    "a sound is a General MIDI number, like `sound : 41`",
                );
                None
            }
        }
    }
}
