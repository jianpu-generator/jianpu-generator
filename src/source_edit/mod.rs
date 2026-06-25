pub enum PartMode {
    Chords,
    Notes,
    NotesLyrics,
    Follow { target: String },
}

impl PartMode {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "chords" => Some(Self::Chords),
            "notes" => Some(Self::Notes),
            "notes+lyrics" => Some(Self::NotesLyrics),
            _ if s.starts_with("follow[") && s.ends_with(']') => {
                let target = s["follow[".len()..s.len() - 1].to_owned();
                Some(Self::Follow { target })
            }
            _ => None,
        }
    }

    pub fn to_rhs_str(&self) -> String {
        match self {
            Self::Chords => "chords".to_owned(),
            Self::Notes => "notes".to_owned(),
            Self::NotesLyrics => "notes+lyrics".to_owned(),
            Self::Follow { target } => format!("follow[{target}]"),
        }
    }
}

pub fn update_part_declaration(
    source: &str,
    abbreviation: &str,
    new_mode: &PartMode,
    new_soundfont: Option<&str>,
) -> Option<String> {
    let lines: Vec<&str> = source.split('\n').collect();

    let parts_index = lines.iter().position(|line| line.trim() == "# parts")?;

    let target_index = lines
        .iter()
        .enumerate()
        .skip(parts_index + 1)
        .find(|(_, line)| {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                return false;
            }
            if trimmed.starts_with("# ") {
                return false;
            }
            let Some(eq_pos) = line.find('=') else {
                return false;
            };
            let lhs = line[..eq_pos].trim();
            let line_abbr = if let Some(bracket_start) = lhs.rfind('[') {
                lhs[bracket_start + 1..].trim_end_matches(']')
            } else {
                lhs
            };
            line_abbr == abbreviation
        })
        .map(|(index, _)| index)?;

    let line = lines.get(target_index)?;
    let eq_pos = line.find('=')?;
    let lhs_with_eq = &line[..eq_pos + 1];

    let soundfont_suffix = new_soundfont
        .map(|sf| format!(" \"{sf}\""))
        .unwrap_or_default();

    let new_rhs = new_mode.to_rhs_str();
    let new_line = format!("{lhs_with_eq} {new_rhs}{soundfont_suffix}");

    let result = lines
        .iter()
        .enumerate()
        .map(|(i, line)| {
            if i == target_index {
                new_line.as_str()
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    Some(result)
}

#[cfg(test)]
mod tests;
