pub mod new_layout;
pub mod new_types;

/// Margin on every edge of the page in points (~9 mm).
/// Applied to all four sides: left/right for column fitting, top/bottom for row fitting.
pub(crate) const PAGE_MARGIN: f32 = 25.0;

#[cfg(test)]
mod tests;
