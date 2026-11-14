use crate::{app::App, config::keybinds::KeybindsConfig, ui::default_block};
use ratatui::{
    layout::Flex,
    prelude::*,
    widgets::{Padding, Paragraph},
};

/// Minimum width of the key column, including the gap before the description.
const KEY_MIN_WIDTH: u16 = 7;
/// Blank cells between page columns.
const COLUMN_GAP: u16 = 2;

/// A titled group of `(keys, description)` rows on the help page.
struct Section {
    title: &'static str,
    rows: Vec<(String, &'static str)>,
}

impl Section {
    const fn new(title: &'static str, rows: Vec<(String, &'static str)>) -> Self {
        Self { title, rows }
    }

    /// Rows taken by the title and the key rows.
    const fn height(&self) -> usize {
        1 + self.rows.len()
    }
}

/// Help page content as columns of sections, left to right.
///
/// This is the only place that lists keys for the help page: adding a keybind means adding one
/// `(keys, description)` row to a section here. Configurable keys are read from `kb` so custom
/// bindings show up; keys that cannot be configured are literal strings.
fn help_columns(kb: &KeybindsConfig) -> Vec<Vec<Section>> {
    let key = display_key;
    let pair = |first: &str, second: &str| format!("{}/{}", key(first), key(second));
    let literal = |value: &str| value.to_owned();

    vec![
        vec![
            Section::new(
                "Navigation",
                vec![
                    (pair(&kb.prev_torrent, &kb.next_torrent), "Up/down"),
                    (pair(&kb.first_torrent, &kb.last_torrent), "First/last"),
                    (pair(&kb.page_up, &kb.page_down), "Page"),
                    (pair(&kb.prev_tab, &kb.next_tab), "Prev/next tab"),
                    (literal("1-9, 0"), "Switch tab"),
                ],
            ),
            Section::new(
                "Search",
                vec![
                    (key(&kb.filter), "Filter"),
                    (key(&kb.clear_filter), "Clear filter"),
                ],
            ),
            Section::new(
                "General",
                vec![
                    (key(&kb.toggle_help), "Close help"),
                    (key(&kb.quit), "Quit"),
                ],
            ),
        ],
        vec![
            Section::new(
                "Actions",
                vec![
                    (key(&kb.toggle_torrent), "Start/stop"),
                    (key(&kb.toggle_all), "Start/stop all"),
                    (key(&kb.select), "Multi-select"),
                    (key(&kb.select_all), "Select visible"),
                    (key(&kb.clear_selection), "Clear selection"),
                    (key(&kb.move_torrent), "Move"),
                    (key(&kb.rename_torrent), "Rename"),
                    (key(&kb.delete), "Remove"),
                    (key(&kb.delete_force), "Delete with data"),
                ],
            ),
            Section::new(
                "Sort",
                vec![
                    (key(&kb.sort_column), "Next sort column"),
                    (key(&kb.sort_reverse), "Reverse sort"),
                ],
            ),
        ],
        vec![Section::new(
            "In prompts",
            vec![
                (literal("←/→"), "Move cursor"),
                (literal("^A/^E"), "Start/end"),
                (literal("Del"), "Delete forward"),
                (literal("^W"), "Delete word"),
                (literal("^U"), "Delete to start"),
                (literal("Tab"), "Complete path"),
                (literal("Enter"), "Submit"),
                (literal("Esc"), "Cancel"),
            ],
        )],
    ]
}

/// Renders the full help page into `area`.
///
/// Uses as many of the designed columns as fit side by side. On narrower areas adjacent columns
/// are stacked, and whatever still does not fit is clipped.
pub fn render_help(frame: &mut Frame, app: &App, area: Rect) {
    let block = default_block().padding(Padding::horizontal(1));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let columns = help_columns(app.config.keybinds());
    let layout = (1..=columns.len())
        .rev()
        .map(|count| stack(&columns, count))
        .find(|layout| total_width(layout) <= inner.width)
        .unwrap_or_else(|| stack(&columns, 1));

    let constraints = layout.iter().map(|column| Constraint::Length(column.width));
    let rects = Layout::horizontal(constraints)
        .spacing(COLUMN_GAP)
        .flex(Flex::Start)
        .split(inner);

    for (column, rect) in layout.into_iter().zip(rects.iter()) {
        frame.render_widget(Paragraph::new(column.lines), *rect);
    }
}

/// A rendered page column: its lines and the width they need.
struct PageColumn {
    lines: Vec<Line<'static>>,
    width: u16,
}

fn total_width(layout: &[PageColumn]) -> u16 {
    let gaps = u16::try_from(layout.len().saturating_sub(1)).unwrap_or(u16::MAX);
    layout
        .iter()
        .map(|column| column.width)
        .fold(gaps.saturating_mul(COLUMN_GAP), u16::saturating_add)
}

/// Stacks adjacent `columns` into `count` page columns, keeping their order and choosing the
/// split with the shortest tallest column.
fn stack(columns: &[Vec<Section>], count: usize) -> Vec<PageColumn> {
    split(columns, count)
        .into_iter()
        .map(|group| page_column(&group.iter().flatten().collect::<Vec<_>>()))
        .collect()
}

fn split(columns: &[Vec<Section>], count: usize) -> Vec<&[Vec<Section>]> {
    if count <= 1 || columns.len() <= 1 {
        return vec![columns];
    }
    (1..columns.len())
        .map(|at| {
            let (head, tail) = columns.split_at(at);
            let mut groups = vec![head];
            groups.extend(split(tail, count - 1));
            groups
        })
        .min_by_key(|groups| groups.iter().map(|group| group_height(group)).max())
        .unwrap_or_else(|| vec![columns])
}

/// Rows taken by `group` stacked as one column, with a blank row between sections.
fn group_height(group: &[Vec<Section>]) -> usize {
    let sections = group.iter().flatten();
    let count = sections.clone().count();
    sections.map(Section::height).sum::<usize>() + count.saturating_sub(1)
}

fn page_column(sections: &[&Section]) -> PageColumn {
    let key_style = Style::default().fg(Color::Yellow).bold();
    let title_style = Style::default().fg(Color::Magenta).bold();
    let desc_style = Style::default().fg(Color::White);

    let rows = sections.iter().flat_map(|section| &section.rows);
    let key_width = rows
        .clone()
        .map(|(keys, _)| Span::raw(keys.as_str()).width() + 1)
        .max()
        .map_or(KEY_MIN_WIDTH, |width| {
            u16::try_from(width).unwrap_or(u16::MAX).max(KEY_MIN_WIDTH)
        });
    let desc_width = rows
        .map(|(_, desc)| Span::raw(*desc).width())
        .max()
        .unwrap_or_default();
    let title_width = sections
        .iter()
        .map(|section| Span::raw(section.title).width())
        .max()
        .unwrap_or_default();
    let width = (usize::from(key_width) + desc_width).max(title_width);

    let mut lines = Vec::new();
    for (index, section) in sections.iter().enumerate() {
        if index > 0 {
            lines.push(Line::default());
        }
        lines.push(Line::styled(section.title, title_style));
        lines.extend(section.rows.iter().map(|(keys, desc)| {
            let padding = usize::from(key_width).saturating_sub(Span::raw(keys.as_str()).width());
            Line::from(vec![
                Span::styled(keys.clone(), key_style),
                Span::raw(" ".repeat(padding)),
                Span::styled(*desc, desc_style),
            ])
        }));
    }

    PageColumn {
        lines,
        width: u16::try_from(width).unwrap_or(u16::MAX),
    }
}

/// Formats a configured key string for display, e.g. `" "` as `Space`.
pub fn display_key(key: &str) -> String {
    match key.to_ascii_lowercase().as_str() {
        " " => "Space".to_string(),
        "enter" => "Enter".to_string(),
        "esc" | "escape" => "Esc".to_string(),
        "pageup" | "page_up" => "PgUp".to_string(),
        "pagedown" | "page_down" => "PgDn".to_string(),
        _ => key.to_string(),
    }
}
