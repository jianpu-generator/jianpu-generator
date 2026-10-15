use std::fmt;

/// Longest allowed whole path, in UTF-8 bytes (same as an S3 object key).
/// Bytes, not characters: a CJK character counts as 3.
pub const MAX_PATH_BYTES: usize = 1024;

const SEPARATOR: char = '/';

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathError {
    Empty,
    /// Leading, trailing or doubled `/`, or a segment that is only whitespace.
    EmptySegment,
    /// A `.` or `..` segment.
    RelativeSegment {
        segment: String,
    },
    TooLong {
        bytes: usize,
    },
}

impl fmt::Display for PathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "path is empty"),
            Self::EmptySegment => write!(f, "path has an empty segment"),
            Self::RelativeSegment { segment } => {
                write!(f, "path segment `{segment}` is not allowed")
            }
            Self::TooLong { bytes } => {
                write!(f, "path is {bytes} bytes, the limit is {MAX_PATH_BYTES}")
            }
        }
    }
}

impl std::error::Error for PathError {}

/// A validated slash-separated path. It names either a file or a folder;
/// which one is decided by where it is used, not by the path itself.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TreePath(String);

impl TreePath {
    pub fn parse(raw: &str) -> Result<Self, PathError> {
        if raw.is_empty() {
            return Err(PathError::Empty);
        }
        if raw.len() > MAX_PATH_BYTES {
            return Err(PathError::TooLong { bytes: raw.len() });
        }
        raw.split(SEPARATOR).try_for_each(validate_segment)?;
        Ok(Self(raw.to_string()))
    }

    /// `parent` joined with a single new segment `name` (which must not
    /// contain `/`). `parent: None` means the top level.
    pub fn child(parent: Option<&Self>, name: &str) -> Result<Self, PathError> {
        if name.contains(SEPARATOR) {
            return Err(PathError::EmptySegment);
        }
        match parent {
            Some(parent) => Self::parse(&format!("{}{SEPARATOR}{name}", parent.0)),
            None => Self::parse(name),
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn segments(&self) -> impl Iterator<Item = &str> {
        self.0.split(SEPARATOR)
    }

    /// The last segment.
    pub fn name(&self) -> &str {
        self.0.rsplit(SEPARATOR).next().unwrap_or(&self.0)
    }

    /// The containing folder, or `None` for a top-level path.
    pub fn parent(&self) -> Option<Self> {
        self.0
            .rsplit_once(SEPARATOR)
            .map(|(parent, _)| Self(parent.to_string()))
    }

    /// Every proper ancestor folder, outermost first.
    pub fn ancestors(&self) -> Vec<Self> {
        let mut ancestors: Vec<Self> = std::iter::successors(self.parent(), Self::parent).collect();
        ancestors.reverse();
        ancestors
    }
}

fn validate_segment(segment: &str) -> Result<(), PathError> {
    match segment {
        "." | ".." => Err(PathError::RelativeSegment {
            segment: segment.to_string(),
        }),
        _ if segment.trim().is_empty() => Err(PathError::EmptySegment),
        _ => Ok(()),
    }
}
