//! Maps terminal cells to UI elements using the areas recorded during the last render.

use crate::ui::{TAB_DIVIDER, TAB_PADDING, table::HEADER_BOTTOM_MARGIN};
use ratatui::{
    layout::{Margin, Position, Rect},
    text::Span,
};

/// Rows between the top border and the first torrent row: the header plus its bottom margin.
const TABLE_HEADER_ROWS: u16 = 1 + HEADER_BOTTOM_MARGIN;

/// Returns how many torrent rows fit in a bordered table rendered in `area`.
#[must_use]
pub fn table_body_rows(area: Rect) -> usize {
    let inner = area.inner(Margin::new(1, 1));
    usize::from(inner.height.saturating_sub(TABLE_HEADER_ROWS))
}

/// Returns the index of the torrent row at `position` in a bordered table rendered in `area`.
///
/// `offset` is the [`TableState`](ratatui::widgets::TableState) scroll offset and `row_count`
/// the number of rows in the table. Clicks on the border, header, or empty space below the last
/// row return [`None`].
#[must_use]
pub fn table_row_at(
    area: Rect,
    offset: usize,
    row_count: usize,
    position: Position,
) -> Option<usize> {
    let inner = area.inner(Margin::new(1, 1));
    if !inner.contains(position) {
        return None;
    }
    let row_offset = position.y.checked_sub(inner.y + TABLE_HEADER_ROWS)?;
    let row = offset + usize::from(row_offset);
    (row < row_count).then_some(row)
}

/// Returns the index of the tab title at `position` in bordered tabs rendered in `area`.
///
/// `title_widths` are the display widths of the tab titles in order. Each tab spans its left
/// padding, title, and right padding; clicks on the border or a divider return [`None`].
#[must_use]
pub fn tab_at(
    area: Rect,
    title_widths: impl IntoIterator<Item = usize>,
    position: Position,
) -> Option<usize> {
    let inner = area.inner(Margin::new(1, 1));
    if !inner.contains(position) || position.y != inner.y {
        return None;
    }
    let column = usize::from(position.x - inner.x);
    let padding = 2 * Span::raw(TAB_PADDING).width();
    let divider = Span::raw(TAB_DIVIDER).width();
    let mut start = 0;
    for (index, title_width) in title_widths.into_iter().enumerate() {
        let end = start + padding + title_width;
        if column < end {
            return (column >= start).then_some(index);
        }
        start = end + divider;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use claims::{assert_none, assert_some_eq};

    /// Table occupying rows 3..=12: border at 3, header at 4, margin at 5, first row at 6.
    const TABLE_AREA: Rect = Rect::new(0, 3, 40, 10);

    #[test]
    fn table_body_rows_matches_last_clickable_row() {
        let rows = table_body_rows(TABLE_AREA);

        assert_eq!(rows, 6);
        // The bottom inner row (just above the border at 12) is the last row that clicks hit.
        assert_some_eq!(
            table_row_at(TABLE_AREA, 0, 100, Position::new(1, 11)),
            rows - 1
        );
        assert_none!(table_row_at(TABLE_AREA, 0, 100, Position::new(1, 12)));
    }

    #[test]
    fn table_body_rows_is_zero_without_room_below_header() {
        assert_eq!(table_body_rows(Rect::new(0, 0, 40, 3)), 0);
    }

    /// Tabs occupying rows 0..=2 with titles on row 1.
    const TABS_AREA: Rect = Rect::new(0, 0, 40, 3);

    #[test]
    fn table_row_without_offset() {
        assert_some_eq!(table_row_at(TABLE_AREA, 0, 5, Position::new(5, 6)), 0);
        assert_some_eq!(table_row_at(TABLE_AREA, 0, 5, Position::new(5, 8)), 2);
    }

    #[test]
    fn table_row_with_offset() {
        assert_some_eq!(table_row_at(TABLE_AREA, 10, 20, Position::new(5, 6)), 10);
        assert_some_eq!(table_row_at(TABLE_AREA, 10, 20, Position::new(5, 11)), 15);
    }

    #[test]
    fn table_header_and_border_ignored() {
        for y in [3, 4, 5, 12] {
            assert_none!(table_row_at(TABLE_AREA, 0, 50, Position::new(5, y)));
        }
        assert_none!(table_row_at(TABLE_AREA, 0, 50, Position::new(0, 6)));
        assert_none!(table_row_at(TABLE_AREA, 0, 50, Position::new(39, 6)));
    }

    #[test]
    fn table_click_below_last_row_ignored() {
        assert_none!(table_row_at(TABLE_AREA, 0, 2, Position::new(5, 8)));
        assert_none!(table_row_at(TABLE_AREA, 0, 0, Position::new(5, 6)));
    }

    #[test]
    fn table_click_outside_area_ignored() {
        assert_none!(table_row_at(TABLE_AREA, 0, 50, Position::new(5, 1)));
        assert_none!(table_row_at(TABLE_AREA, 0, 50, Position::new(50, 6)));
    }

    #[test]
    fn tab_index_from_column() {
        // Inner row starts at x = 1: " Ab |" spans 1..=5, " Cde |" spans 6..=11, " F " 12..=14.
        let widths = [2, 3, 1];
        assert_some_eq!(tab_at(TABS_AREA, widths, Position::new(1, 1)), 0);
        assert_some_eq!(tab_at(TABS_AREA, widths, Position::new(4, 1)), 0);
        assert_some_eq!(tab_at(TABS_AREA, widths, Position::new(6, 1)), 1);
        assert_some_eq!(tab_at(TABS_AREA, widths, Position::new(10, 1)), 1);
        assert_some_eq!(tab_at(TABS_AREA, widths, Position::new(12, 1)), 2);
        assert_some_eq!(tab_at(TABS_AREA, widths, Position::new(14, 1)), 2);
    }

    #[test]
    fn tab_divider_and_trailing_space_ignored() {
        let widths = [2, 3, 1];
        assert_none!(tab_at(TABS_AREA, widths, Position::new(5, 1)));
        assert_none!(tab_at(TABS_AREA, widths, Position::new(11, 1)));
        assert_none!(tab_at(TABS_AREA, widths, Position::new(15, 1)));
    }

    #[test]
    fn tab_border_ignored() {
        let widths = [2, 3, 1];
        assert_none!(tab_at(TABS_AREA, widths, Position::new(0, 1)));
        assert_none!(tab_at(TABS_AREA, widths, Position::new(2, 0)));
        assert_none!(tab_at(TABS_AREA, widths, Position::new(2, 2)));
    }
}
