use std::iter;

use itertools::Itertools;
use strum::{Display, EnumString};
use unicode_width::UnicodeWidthStr;

use crate::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect, SegmentSize},
    style::{Style, Styled},
    text::Text,
    widgets::{Block, StatefulWidget, Widget},
};

/// A [`Cell`] contains the [`Text`] to be displayed in a [`Row`] of a [`Table`].
///
/// It can be created from anything that can be converted to a [`Text`].
/// ```rust
/// use std::borrow::Cow;
/// use ratatui::{prelude::*, widgets::*};
///
/// Cell::from("simple string");
///
/// Cell::from(Span::from("span"));
///
/// Cell::from(Line::from(vec![
///     Span::raw("a vec of "),
///     Span::styled("spans", Style::default().add_modifier(Modifier::BOLD))
/// ]));
///
/// Cell::from(Text::from("a text"));
///
/// Cell::from(Text::from(Cow::Borrowed("hello")));
/// ```
///
/// You can apply a [`Style`] on the entire [`Cell`] using [`Cell::style`] or rely on the styling
/// capabilities of [`Text`].
#[derive(Debug, Default, Clone, Eq, PartialEq, Hash)]
pub struct Cell<'a> {
    content: Text<'a>,
    style: Style,
}

impl<'a> Cell<'a> {
    /// Set the `Style` of this cell.
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

impl<'a, T> From<T> for Cell<'a>
where
    T: Into<Text<'a>>,
{
    fn from(content: T) -> Cell<'a> {
        Cell {
            content: content.into(),
            style: Style::default(),
        }
    }
}

impl<'a> Styled for Cell<'a> {
    type Item = Cell<'a>;

    fn style(&self) -> Style {
        self.style
    }

    fn set_style(self, style: Style) -> Self::Item {
        self.style(style)
    }
}

/// Holds data to be displayed in a [`Table`] widget.
///
/// A [`Row`] is a collection of cells. It can be created from simple strings:
/// ```rust
/// use ratatui::{prelude::*, widgets::*};
///
/// Row::new(vec!["Cell1", "Cell2", "Cell3"]);
/// ```
///
/// But if you need a bit more control over individual cells, you can explicitly create [`Cell`]s:
/// ```rust
/// use ratatui::{prelude::*, widgets::*};
///
/// Row::new(vec![
///     Cell::from("Cell1"),
///     Cell::from("Cell2").style(Style::default().fg(Color::Yellow)),
/// ]);
/// ```
///
/// You can also construct a row from any type that can be converted into [`Text`]:
/// ```rust
/// use std::borrow::Cow;
/// use ratatui::{prelude::*, widgets::*};
///
/// Row::new(vec![
///     Cow::Borrowed("hello"),
///     Cow::Owned("world".to_uppercase()),
/// ]);
/// ```
///
/// By default, a row has a height of 1 but you can change this using [`Row::height`].
#[derive(Debug, Default, Clone, Eq, PartialEq, Hash)]
pub struct Row<'a> {
    cells: Vec<Cell<'a>>,
    height: u16,
    style: Style,
    bottom_margin: u16,
}

impl<'a> Row<'a> {
    /// Creates a new [`Row`] from an iterator where items can be converted to a [`Cell`].
    pub fn new<T>(cells: T) -> Self
    where
        T: IntoIterator,
        T::Item: Into<Cell<'a>>,
    {
        Self {
            height: 1,
            cells: cells.into_iter().map(Into::into).collect(),
            style: Style::default(),
            bottom_margin: 0,
        }
    }

    /// Set the fixed height of the [`Row`]. Any [`Cell`] whose content has more lines than this
    /// height will see its content truncated.
    pub fn height(mut self, height: u16) -> Self {
        self.height = height;
        self
    }

    /// Set the [`Style`] of the entire row. This [`Style`] can be overridden by the [`Style`] of a
    /// any individual [`Cell`] or event by their [`Text`] content.
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Set the bottom margin. By default, the bottom margin is `0`.
    pub fn bottom_margin(mut self, margin: u16) -> Self {
        self.bottom_margin = margin;
        self
    }

    /// Returns the total height of the row.
    fn total_height(&self) -> u16 {
        self.height.saturating_add(self.bottom_margin)
    }
}

impl<'a> Styled for Row<'a> {
    type Item = Row<'a>;

    fn style(&self) -> Style {
        self.style
    }

    fn set_style(self, style: Style) -> Self::Item {
        self.style(style)
    }
}

/// This option allows the user to configure the "highlight symbol" column width spacing
#[derive(Debug, Display, EnumString, PartialEq, Eq, Clone, Default, Hash)]
pub enum HighlightSpacing {
    /// Always add spacing for the selection symbol column
    ///
    /// With this variant, the column for the selection symbol will always be allocated, and so the
    /// table will never change size, regardless of if a row is selected or not
    Always,
    /// Only add spacing for the selection symbol column if a row is selected
    ///
    /// With this variant, the column for the selection symbol will only be allocated if there is a
    /// selection, causing the table to shift if selected / unselected
    #[default]
    WhenSelected,
    /// Never add spacing to the selection symbol column, regardless of whether something is
    /// selected or not
    ///
    /// This means that the highlight symbol will never be drawn
    Never,
}

impl HighlightSpacing {
    /// Determine if a selection should be done, based on variant
    /// Input "selection_state" should be similar to `state.selected.is_some()`
    pub fn should_add(&self, selection_state: bool) -> bool {
        match self {
            HighlightSpacing::Always => true,
            HighlightSpacing::WhenSelected => selection_state,
            HighlightSpacing::Never => false,
        }
    }
}

/// A widget to display data in formatted columns.
///
/// It is a collection of [`Row`]s, themselves composed of [`Cell`]s:
/// ```rust
/// use ratatui::{prelude::*, widgets::*};
///
/// Table::new(
///     vec![
///         // Row can be created from simple strings.
///         Row::new(vec!["Row11", "Row12", "Row13"]),
///         // You can style the entire row.
///         Row::new(vec!["Row21", "Row22", "Row23"]).style(Style::default().fg(Color::Blue)),
///         // If you need more control over the styling you may need to create Cells directly
///         Row::new(vec![
///             Cell::from("Row31"),
///             Cell::from("Row32").style(Style::default().fg(Color::Yellow)),
///             Cell::from(Line::from(vec![
///                 Span::raw("Row"),
///                 Span::styled("33", Style::default().fg(Color::Green))
///             ])),
///         ]),
///         // If a Row need to display some content over multiple lines, you just have to change
///         // its height.
///         Row::new(vec![
///             Cell::from("Row\n41"),
///             Cell::from("Row\n42"),
///             Cell::from("Row\n43"),
///         ]).height(2),
///     ],
///     // Columns widths are constrained in the same way as Layout...
///     [Constraint::Length(5), Constraint::Length(5), Constraint::Length(10)])
/// // ...and they can be separated by a fixed spacing.
/// .column_spacing(1)
/// // You can set the style of the entire Table.
/// .style(Style::default().fg(Color::White))
/// // It has an optional header, which is simply a Row always visible at the top.
/// .header(
///     Row::new(vec!["Col1", "Col2", "Col3"])
///         .style(Style::default().fg(Color::Yellow))
///         // If you want some space between the header and the rest of the rows, you can always
///         // specify some margin at the bottom.
///         .bottom_margin(1)
/// )
/// // As any other widget, a Table can be wrapped in a Block.
/// .block(Block::default().title("Table"))
/// // If you wish to highlight a row in any specific way when it is selected...
/// .highlight_style(Style::default().add_modifier(Modifier::BOLD))
/// // ...and potentially show a symbol in front of the selection.
/// .highlight_symbol(">>");
/// ```
#[derive(Debug, Default, Clone, Eq, PartialEq, Hash)]
pub struct Table<'a> {
    /// A block to wrap the widget in
    block: Option<Block<'a>>,
    /// Base style for the widget
    style: Style,
    /// Width constraints for each column
    widths: Vec<Constraint>,
    /// Space between each column
    column_spacing: u16,
    /// Style used to render the selected row
    highlight_style: Style,
    /// Symbol in front of the selected rom
    highlight_symbol: Option<&'a str>,
    /// Optional header
    header: Option<Row<'a>>,
    /// Data to display in each row
    rows: Vec<Row<'a>>,
    /// Decides when to allocate spacing for the row selection
    highlight_spacing: HighlightSpacing,
    /// Controls how to distribute extra space among the columns
    segment_size: SegmentSize,
}

impl<'a> Table<'a> {
    /// Creates a new [`Table`] widget with the given rows.
    ///
    /// The `rows` parameter is a Vector of [`Row`], this holds the data to be displayed by the
    /// table.
    ///
    /// The `widths` parameter is an array (or any other type that implements IntoIterator) of
    /// [`Constraint`]s, this holds the widths of each column. This parameter was added in 0.25.0.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ratatui::{prelude::*, widgets::*};
    /// let table = Table::new(
    ///     vec![
    ///         Row::new(vec![
    ///             Cell::from("Cell1"),
    ///             Cell::from("Cell2")
    ///         ]),
    ///         Row::new(vec![
    ///             Cell::from("Cell3"),
    ///             Cell::from("Cell4")
    ///         ]),
    ///     ],
    ///     [Constraint::Length(5), Constraint::Length(5)]
    /// );
    /// ```
    pub fn new<R, C>(rows: R, widths: C) -> Self
    where
        R: IntoIterator<Item = Row<'a>>,
        C: IntoIterator,
        C::Item: AsRef<Constraint>,
    {
        let widths = widths.into_iter().map(|c| *c.as_ref()).collect_vec();
        ensure_percentages_less_than_100(&widths);
        Self {
            block: None,
            style: Style::default(),
            widths,
            column_spacing: 1,
            highlight_style: Style::default(),
            highlight_symbol: None,
            header: None,
            rows: rows.into_iter().collect(),
            highlight_spacing: HighlightSpacing::default(),
            segment_size: SegmentSize::None,
        }
    }

    /// Creates a custom block around a [`Table`] widget.
    ///
    /// The `block` parameter is of type [`Block`]. This holds the specified block to be
    /// created around the [`Table`]
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ratatui::{prelude::*, widgets::*};
    /// let table = Table::new(
    ///     vec![
    ///         Row::new(vec![
    ///             Cell::from("Cell1"),
    ///             Cell::from("Cell2")
    ///         ]),
    ///         Row::new(vec![
    ///             Cell::from("Cell3"),
    ///             Cell::from("Cell4")
    ///         ]),
    ///     ],
    ///     [Constraint::Length(5), Constraint::Length(5)]
    /// )
    /// .block(Block::default().title("Table"));
    /// ```
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    /// Creates a header for a [`Table`] widget.
    ///
    /// The `header` parameter is of type [`Row`] and this holds the cells to be displayed at the
    /// top of the [`Table`]
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ratatui::{prelude::*, widgets::*};
    /// let table = Table::new(
    ///     vec![
    ///         Row::new(vec![Cell::from("Cell1"), Cell::from("Cell2")])
    ///     ],
    ///     [Constraint::Length(20), Constraint::Length(20)]
    /// )
    /// .header(
    ///     Row::new(vec![
    ///         Cell::from("Header Cell 1"),
    ///         Cell::from("Header Cell 2")
    ///     ])
    /// );
    /// ```
    pub fn header(mut self, header: Row<'a>) -> Self {
        self.header = Some(header);
        self
    }

    /// Set the widths of the columns of the [`Table`] widget.
    ///
    /// The `widths` parameter accepts anything which be converted to an Iterator of Constraints
    /// which can be an array, slice, Vec etc.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ratatui::{prelude::*, widgets::*};
    /// let table = Table::default().widths([Constraint::Length(5), Constraint::Length(5)]);
    /// let table = Table::default().widths(&[Constraint::Length(5), Constraint::Length(5)]);
    ///
    /// // widths could also be computed at runtime
    /// let widths = vec![Constraint::Length(5), Constraint::Length(5)];
    /// let table = Table::default().widths(widths.clone());
    /// let table = Table::default().widths(&widths);
    /// ```
    pub fn widths<I, C>(mut self, widths: I) -> Self
    where
        I: IntoIterator<Item = C>,
        C: AsRef<Constraint>,
    {
        let widths = widths.into_iter().map(|c| *c.as_ref()).collect_vec();
        ensure_percentages_less_than_100(&widths);
        self.widths = widths;
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn highlight_symbol(mut self, highlight_symbol: &'a str) -> Self {
        self.highlight_symbol = Some(highlight_symbol);
        self
    }

    pub fn highlight_style(mut self, highlight_style: Style) -> Self {
        self.highlight_style = highlight_style;
        self
    }

    /// Set when to show the highlight spacing
    ///
    /// See [`HighlightSpacing`] about which variant affects spacing in which way
    pub fn highlight_spacing(mut self, value: HighlightSpacing) -> Self {
        self.highlight_spacing = value;
        self
    }

    pub fn column_spacing(mut self, spacing: u16) -> Self {
        self.column_spacing = spacing;
        self
    }

    /// Get all offsets and widths of all user specified columns
    /// Returns (x, width)
    fn get_columns_widths(&self, max_width: u16, selection_width: u16) -> Vec<(u16, u16)> {
        let constraints = iter::once(Constraint::Length(selection_width))
            .chain(Itertools::intersperse(
                self.widths.iter().cloned(),
                Constraint::Length(self.column_spacing),
            ))
            .collect_vec();
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints)
            .segment_size(self.segment_size)
            .split(Rect::new(0, 0, max_width, 1));
        layout
            .iter()
            .skip(1) // skip selection column
            .step_by(2) // skip spacing between columns
            .map(|c| (c.x, c.width))
            .collect()
    }

    fn get_row_bounds(
        &self,
        selected: Option<usize>,
        offset: usize,
        max_height: u16,
    ) -> (usize, usize) {
        let offset = offset.min(self.rows.len().saturating_sub(1));
        let mut start = offset;
        let mut end = offset;
        let mut height = 0;
        for item in self.rows.iter().skip(offset) {
            if height + item.height > max_height {
                break;
            }
            height += item.total_height();
            end += 1;
        }

        let selected = selected.unwrap_or(0).min(self.rows.len() - 1);
        while selected >= end {
            height = height.saturating_add(self.rows[end].total_height());
            end += 1;
            while height > max_height {
                height = height.saturating_sub(self.rows[start].total_height());
                start += 1;
            }
        }
        while selected < start {
            start -= 1;
            height = height.saturating_add(self.rows[start].total_height());
            while height > max_height {
                end -= 1;
                height = height.saturating_sub(self.rows[end].total_height());
            }
        }
        (start, end)
    }

    /// Builder method to control how extra space is distributed amongst columns.
    ///
    /// This determines how the space is distributed when the constraints are satisfied. By default,
    /// the extra space is not distributed at all.  But this can be changed to distribute all extra
    /// space to the last column or to distribute it equally.
    ///
    /// # Examples
    ///
    /// Create a table that needs at least 30 columns to display.  Any extra space will be assigned
    /// to the last column.
    #[cfg_attr(feature = "unstable", doc = " ```")]
    #[cfg_attr(not(feature = "unstable"), doc = " ```ignore")]
    /// # use ratatui::layout::Constraint;
    /// # use ratatui::layout::SegmentSize;
    /// # use ratatui::widgets::Table;
    /// let widths = [Constraint::Min(10), Constraint::Min(10), Constraint::Min(10)];
    /// let table = Table::new([], widths)
    ///     .segment_size(SegmentSize::LastTakesRemainder);
    /// ```
    #[stability::unstable(
        feature = "segment-size",
        reason = "The name for this feature is not final and may change in the future",
        issue = "https://github.com/ratatui-org/ratatui/issues/536"
    )]
    pub const fn segment_size(mut self, segment_size: SegmentSize) -> Self {
        self.segment_size = segment_size;
        self
    }
}

fn ensure_percentages_less_than_100(widths: &[Constraint]) {
    let between_0_and_100 = |&w| match w {
        Constraint::Percentage(p) => p <= 100,
        _ => true,
    };
    assert!(
        widths.iter().all(between_0_and_100),
        "Percentages should be between 0 and 100 inclusively."
    );
}

impl<'a> Styled for Table<'a> {
    type Item = Table<'a>;

    fn style(&self) -> Style {
        self.style
    }

    fn set_style(self, style: Style) -> Self::Item {
        self.style(style)
    }
}

#[derive(Debug, Default, Clone, Eq, PartialEq, Hash)]
pub struct TableState {
    offset: usize,
    selected: Option<usize>,
}

impl TableState {
    pub fn offset(&self) -> usize {
        self.offset
    }

    pub fn offset_mut(&mut self) -> &mut usize {
        &mut self.offset
    }

    pub fn with_selected(mut self, selected: Option<usize>) -> Self {
        self.selected = selected;
        self
    }

    pub fn with_offset(mut self, offset: usize) -> Self {
        self.offset = offset;
        self
    }

    pub fn selected(&self) -> Option<usize> {
        self.selected
    }

    pub fn select(&mut self, index: Option<usize>) {
        self.selected = index;
        if index.is_none() {
            self.offset = 0;
        }
    }
}

impl<'a> StatefulWidget for Table<'a> {
    type State = TableState;

    fn render(mut self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if area.area() == 0 {
            return;
        }
        buf.set_style(area, self.style);
        let table_area = match self.block.take() {
            Some(b) => {
                let inner_area = b.inner(area);
                b.render(area, buf);
                inner_area
            }
            None => area,
        };

        let selection_width = if self.highlight_spacing.should_add(state.selected.is_some()) {
            self.highlight_symbol.map_or(0, |s| s.width() as u16)
        } else {
            0
        };
        let columns_widths = self.get_columns_widths(table_area.width, selection_width);
        let highlight_symbol = self.highlight_symbol.unwrap_or("");
        let mut current_height = 0;
        let mut rows_height = table_area.height;

        // Draw header
        if let Some(ref header) = self.header {
            let max_header_height = table_area.height.min(header.total_height());
            buf.set_style(
                Rect {
                    x: table_area.left(),
                    y: table_area.top(),
                    width: table_area.width,
                    height: table_area.height.min(header.height),
                },
                header.style,
            );
            let inner_offset = table_area.left();
            for ((x, width), cell) in columns_widths.iter().zip(header.cells.iter()) {
                render_cell(
                    buf,
                    cell,
                    Rect {
                        x: inner_offset + x,
                        y: table_area.top(),
                        width: *width,
                        height: max_header_height,
                    },
                );
            }
            current_height += max_header_height;
            rows_height = rows_height.saturating_sub(max_header_height);
        }

        // Draw rows
        if self.rows.is_empty() {
            return;
        }
        let (start, end) = self.get_row_bounds(state.selected, state.offset, rows_height);
        state.offset = start;
        for (i, table_row) in self
            .rows
            .iter_mut()
            .enumerate()
            .skip(state.offset)
            .take(end - start)
        {
            let (row, inner_offset) = (table_area.top() + current_height, table_area.left());
            current_height += table_row.total_height();
            let table_row_area = Rect {
                x: inner_offset,
                y: row,
                width: table_area.width,
                height: table_row.height,
            };
            buf.set_style(table_row_area, table_row.style);
            let is_selected = state.selected.map_or(false, |s| s == i);
            if selection_width > 0 && is_selected {
                // this should in normal cases be safe, because "get_columns_widths" allocates
                // "highlight_symbol.width()" space but "get_columns_widths"
                // currently does not bind it to max table.width()
                buf.set_stringn(
                    inner_offset,
                    row,
                    highlight_symbol,
                    table_area.width as usize,
                    table_row.style,
                );
            };
            for ((x, width), cell) in columns_widths.iter().zip(table_row.cells.iter()) {
                render_cell(
                    buf,
                    cell,
                    Rect {
                        x: inner_offset + x,
                        y: row,
                        width: *width,
                        height: table_row.height,
                    },
                );
            }
            if is_selected {
                buf.set_style(table_row_area, self.highlight_style);
            }
        }
    }
}

fn render_cell(buf: &mut Buffer, cell: &Cell, area: Rect) {
    buf.set_style(area, cell.style);
    for (i, line) in cell.content.lines.iter().enumerate() {
        if i as u16 >= area.height {
            break;
        }

        let x_offset = match line.alignment {
            Some(Alignment::Center) => (area.width / 2).saturating_sub(line.width() as u16 / 2),
            Some(Alignment::Right) => area.width.saturating_sub(line.width() as u16),
            _ => 0,
        };

        buf.set_line(area.x + x_offset, area.y + i as u16, line, area.width);
    }
}

impl<'a> Widget for Table<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut state = TableState::default();
        StatefulWidget::render(self, area, buf, &mut state);
    }
}

#[cfg(test)]
mod tests {
    use std::vec;

    use super::*;
    use crate::{
        layout::Constraint::*,
        style::{Color, Modifier, Style, Stylize},
        text::Line,
    };
    #[test]
    #[should_panic]
    fn table_invalid_percentages() {
        Table::new(vec![], [Constraint::Percentage(110)]);
    }

    #[test]
    fn widths_conversions() {
        let array = [Constraint::Percentage(100)];
        let table = Table::new(vec![], array);
        assert_eq!(table.widths, vec![Constraint::Percentage(100)], "array");

        let array_ref = &[Constraint::Percentage(100)];
        let table = Table::new(vec![], array_ref);
        assert_eq!(table.widths, vec![Constraint::Percentage(100)], "array ref");

        let vec = vec![Constraint::Percentage(100)];
        let slice = vec.as_slice();
        let table = Table::new(vec![], slice);
        assert_eq!(table.widths, vec![Constraint::Percentage(100)], "slice");

        let vec = vec![Constraint::Percentage(100)];
        let table = Table::new(vec![], vec);
        assert_eq!(table.widths, vec![Constraint::Percentage(100)], "vec");

        let vec_ref = &vec![Constraint::Percentage(100)];
        let table = Table::new(vec![], vec_ref);
        assert_eq!(table.widths, vec![Constraint::Percentage(100)], "vec ref");
    }

    // test how constraints interact with table column width allocation
    mod table_column_widths {
        use super::*;

        /// Construct a a new table with the given constraints, available and selection widths and
        /// tests that the widths match the expected list of (x, width) tuples.
        #[track_caller]
        fn test(
            constraints: &[Constraint],
            segment_size: SegmentSize,
            available_width: u16,
            selection_width: u16,
            expected: &[(u16, u16)],
        ) {
            let table = Table::new(vec![], constraints).segment_size(segment_size);

            let widths = table.get_columns_widths(available_width, selection_width);
            assert_eq!(widths, expected);
        }

        #[test]
        fn length_constraint() {
            // without selection, more than needed width
            test(
                &[Length(4), Length(4)],
                SegmentSize::None,
                20,
                0,
                &[(0, 4), (5, 4)],
            );

            // with selection, more than needed width
            test(
                &[Length(4), Length(4)],
                SegmentSize::None,
                20,
                3,
                &[(3, 4), (8, 4)],
            );

            // without selection, less than needed width
            test(
                &[Length(4), Length(4)],
                SegmentSize::None,
                7,
                0,
                &[(0, 4), (5, 2)],
            );

            // with selection, less than needed width
            test(
                &[Length(4), Length(4)],
                SegmentSize::None,
                7,
                3,
                &[(3, 4), (7, 0)],
            );
        }

        #[test]
        fn max_constraint() {
            // without selection, more than needed width
            test(
                &[Max(4), Max(4)],
                SegmentSize::None,
                20,
                0,
                &[(0, 4), (5, 4)],
            );

            // with selection, more than needed width
            test(
                &[Max(4), Max(4)],
                SegmentSize::None,
                20,
                3,
                &[(3, 4), (8, 4)],
            );

            // without selection, less than needed width
            test(
                &[Max(4), Max(4)],
                SegmentSize::None,
                7,
                0,
                &[(0, 4), (5, 2)],
            );

            // with selection, less than needed width
            test(
                &[Max(4), Max(4)],
                SegmentSize::None,
                7,
                3,
                &[(3, 3), (7, 0)],
            );
        }

        #[test]
        fn min_constraint() {
            // in its currently stage, the "Min" constraint does not grow to use the possible
            // available length and enabling "expand_to_fill" will just stretch the last
            // constraint and not split it with all available constraints

            // without selection, more than needed width
            test(
                &[Min(4), Min(4)],
                SegmentSize::None,
                20,
                0,
                &[(0, 4), (5, 4)],
            );

            // with selection, more than needed width
            test(
                &[Min(4), Min(4)],
                SegmentSize::None,
                20,
                3,
                &[(3, 4), (8, 4)],
            );

            // without selection, less than needed width
            // allocates no spacer
            test(
                &[Min(4), Min(4)],
                SegmentSize::None,
                7,
                0,
                &[(0, 4), (4, 3)],
            );

            // with selection, less than needed width
            // allocates no selection and no spacer
            test(
                &[Min(4), Min(4)],
                SegmentSize::None,
                7,
                3,
                &[(0, 4), (4, 3)],
            );
        }

        #[test]
        fn percentage_constraint() {
            // without selection, more than needed width
            test(
                &[Percentage(30), Percentage(30)],
                SegmentSize::None,
                20,
                0,
                &[(0, 6), (7, 6)],
            );

            // with selection, more than needed width
            test(
                &[Percentage(30), Percentage(30)],
                SegmentSize::None,
                20,
                3,
                &[(3, 6), (10, 6)],
            );

            // without selection, less than needed width
            // rounds from positions: [0.0, 0.0, 2.1, 3.1, 5.2, 7.0]
            test(
                &[Percentage(30), Percentage(30)],
                SegmentSize::None,
                7,
                0,
                &[(0, 2), (3, 2)],
            );

            // with selection, less than needed width
            // rounds from positions: [0.0, 3.0, 5.1, 6.1, 7.0, 7.0]
            test(
                &[Percentage(30), Percentage(30)],
                SegmentSize::None,
                7,
                3,
                &[(3, 2), (6, 1)],
            );
        }

        #[test]
        fn ratio_constraint() {
            // without selection, more than needed width
            // rounds from positions: [0.00, 0.00, 6.67, 7.67, 14.33]
            test(
                &[Ratio(1, 3), Ratio(1, 3)],
                SegmentSize::None,
                20,
                0,
                &[(0, 7), (8, 6)],
            );

            // with selection, more than needed width
            // rounds from positions: [0.00, 3.00, 10.67, 17.33, 20.00]
            test(
                &[Ratio(1, 3), Ratio(1, 3)],
                SegmentSize::None,
                20,
                3,
                &[(3, 7), (11, 6)],
            );

            // without selection, less than needed width
            // rounds from positions: [0.00, 2.33, 3.33, 5.66, 7.00]
            test(
                &[Ratio(1, 3), Ratio(1, 3)],
                SegmentSize::None,
                7,
                0,
                &[(0, 2), (3, 3)],
            );

            // with selection, less than needed width
            // rounds from positions: [0.00, 3.00, 5.33, 6.33, 7.00, 7.00]
            test(
                &[Ratio(1, 3), Ratio(1, 3)],
                SegmentSize::None,
                7,
                3,
                &[(3, 2), (6, 1)],
            );
        }

        /// When more width is available than requested, the behavior is controlled by segment_size
        #[test]
        fn underconstrained() {
            let widths = [Min(10), Min(10), Min(1)];
            test(
                &widths[..],
                SegmentSize::None,
                62,
                0,
                &[(0, 10), (11, 10), (22, 1)],
            );
            test(
                &widths[..],
                SegmentSize::LastTakesRemainder,
                62,
                0,
                &[(0, 10), (11, 10), (22, 40)],
            );
            test(
                &widths[..],
                SegmentSize::EvenDistribution,
                62,
                0,
                &[(0, 20), (21, 20), (42, 20)],
            );
        }
    }

    #[test]
    fn test_render_table_with_alignment() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 20, 3));
        let table = Table::new(
            vec![
                Row::new(vec![Line::from("Left").alignment(Alignment::Left)]),
                Row::new(vec![Line::from("Center").alignment(Alignment::Center)]),
                Row::new(vec![Line::from("Right").alignment(Alignment::Right)]),
            ],
            [Percentage(100)],
        );

        Widget::render(table, Rect::new(0, 0, 20, 3), &mut buf);

        let expected = Buffer::with_lines(vec![
            "Left                ",
            "       Center       ",
            "               Right",
        ]);

        assert_eq!(buf, expected);
    }

    #[test]
    fn cell_can_be_stylized() {
        assert_eq!(
            Cell::from("").black().on_white().bold().not_dim().style,
            Style::default()
                .fg(Color::Black)
                .bg(Color::White)
                .add_modifier(Modifier::BOLD)
                .remove_modifier(Modifier::DIM)
        )
    }

    #[test]
    fn row_can_be_stylized() {
        assert_eq!(
            Row::new(vec![Cell::from("")])
                .black()
                .on_white()
                .bold()
                .not_italic()
                .style,
            Style::default()
                .fg(Color::Black)
                .bg(Color::White)
                .add_modifier(Modifier::BOLD)
                .remove_modifier(Modifier::ITALIC)
        )
    }

    #[test]
    fn table_can_be_stylized() {
        assert_eq!(
            Table::new(vec![Row::new(vec![Cell::from("")])], [Percentage(100)])
                .black()
                .on_white()
                .bold()
                .not_crossed_out()
                .style,
            Style::default()
                .fg(Color::Black)
                .bg(Color::White)
                .add_modifier(Modifier::BOLD)
                .remove_modifier(Modifier::CROSSED_OUT)
        )
    }

    #[test]
    fn highlight_spacing_to_string() {
        assert_eq!(HighlightSpacing::Always.to_string(), "Always".to_string());
        assert_eq!(
            HighlightSpacing::WhenSelected.to_string(),
            "WhenSelected".to_string()
        );
        assert_eq!(HighlightSpacing::Never.to_string(), "Never".to_string());
    }

    #[test]
    fn highlight_spacing_from_str() {
        assert_eq!(
            "Always".parse::<HighlightSpacing>(),
            Ok(HighlightSpacing::Always)
        );
        assert_eq!(
            "WhenSelected".parse::<HighlightSpacing>(),
            Ok(HighlightSpacing::WhenSelected)
        );
        assert_eq!(
            "Never".parse::<HighlightSpacing>(),
            Ok(HighlightSpacing::Never)
        );
        assert_eq!(
            "".parse::<HighlightSpacing>(),
            Err(strum::ParseError::VariantNotFound)
        );
    }
}
