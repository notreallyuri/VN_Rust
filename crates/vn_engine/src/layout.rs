use std::fmt;
use std::rc::Rc;

use raylib::prelude::{Rectangle, Vector2};

type PlaceFn = Rc<dyn Fn(Rectangle, &[Vector2]) -> Vec<Rectangle>>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Anchor {
    TopLeft,
    Top,
    TopRight,
    Left,
    Center,
    Right,
    BottomLeft,
    Bottom,
    BottomRight,
}

impl Anchor {
    fn fractions(self) -> (f32, f32) {
        match self {
            Anchor::TopLeft => (0.0, 0.0),
            Anchor::Top => (0.5, 0.0),
            Anchor::TopRight => (1.0, 0.0),
            Anchor::Left => (0.0, 0.5),
            Anchor::Center => (0.5, 0.5),
            Anchor::Right => (1.0, 0.5),
            Anchor::BottomLeft => (0.0, 1.0),
            Anchor::Bottom => (0.5, 1.0),
            Anchor::BottomRight => (1.0, 1.0),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Align {
    Start,
    Center,
    End,
    Stretch,
}

impl Align {
    fn fraction(self) -> f32 {
        match self {
            Align::Start | Align::Stretch => 0.0,
            Align::Center => 0.5,
            Align::End => 1.0,
        }
    }
}

#[derive(Clone)]
pub enum Arrangement {
    Column,
    Row,
    Grid { columns: usize },
    RowsOf(Vec<usize>),
    Custom(PlaceFn),
}

impl fmt::Debug for Arrangement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Arrangement::Column => write!(f, "Column"),
            Arrangement::Row => write!(f, "Row"),
            Arrangement::Grid { columns } => write!(f, "Grid {{ columns: {} }}", columns),
            Arrangement::RowsOf(pattern) => write!(f, "RowsOf({:?})", pattern),
            Arrangement::Custom(_) => write!(f, "Custom(..)"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Layout {
    pub arrangement: Arrangement,
    pub anchor: Anchor,
    pub align: Align,
    pub spacing: Vector2,
}

impl Default for Layout {
    fn default() -> Self {
        Self {
            arrangement: Arrangement::Column,
            anchor: Anchor::Center,
            align: Align::Center,
            spacing: Vector2::new(16.0, 16.0),
        }
    }
}

impl Layout {
    pub fn column(mut self) -> Self {
        self.arrangement = Arrangement::Column;
        self
    }

    pub fn row(mut self) -> Self {
        self.arrangement = Arrangement::Row;
        self
    }

    pub fn grid(mut self, columns: usize) -> Self {
        self.arrangement = Arrangement::Grid {
            columns: columns.max(1),
        };
        self
    }

    pub fn rows_of(mut self, pattern: impl IntoIterator<Item = usize>) -> Self {
        let pattern: Vec<usize> = pattern.into_iter().filter(|&n| n > 0).collect();
        self.arrangement = if pattern.is_empty() {
            Arrangement::Column
        } else {
            Arrangement::RowsOf(pattern)
        };
        self
    }

    pub fn custom(
        mut self,
        place: impl Fn(Rectangle, &[Vector2]) -> Vec<Rectangle> + 'static,
    ) -> Self {
        self.arrangement = Arrangement::Custom(Rc::new(place));
        self
    }

    pub fn anchor(mut self, anchor: Anchor) -> Self {
        self.anchor = anchor;
        self
    }

    pub fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = Vector2::new(spacing, spacing);
        self
    }

    pub fn spacing_xy(mut self, x: f32, y: f32) -> Self {
        self.spacing = Vector2::new(x, y);
        self
    }

    pub fn row_counts(&self, count: usize) -> Vec<usize> {
        let mut counts = Vec::new();
        let mut remaining = count;

        while remaining > 0 {
            let wanted = match &self.arrangement {
                Arrangement::Column | Arrangement::Custom(_) => 1,
                Arrangement::Row => remaining,
                Arrangement::Grid { columns } => *columns,
                Arrangement::RowsOf(pattern) => pattern
                    .get(counts.len())
                    .or(pattern.last())
                    .copied()
                    .unwrap_or(1),
            };
            let taken = wanted.clamp(1, remaining);
            remaining -= taken;
            counts.push(taken);
        }
        counts
    }

    pub fn columns(&self, count: usize) -> usize {
        self.row_counts(count).into_iter().max().unwrap_or(1)
    }

    pub fn rows(&self, count: usize) -> usize {
        self.row_counts(count).len()
    }

    pub fn block_size(&self, sizes: &[Vector2]) -> Vector2 {
        let grid = Grid::new(self, sizes);
        Vector2::new(grid.width(self.spacing.x), grid.height(self.spacing.y))
    }

    pub fn place(&self, area: Rectangle, sizes: &[Vector2]) -> Vec<Rectangle> {
        if let Arrangement::Custom(place) = &self.arrangement {
            return place(area, sizes);
        }

        let grid = Grid::new(self, sizes);
        let gap = Vector2::new(
            grid.rows
                .iter()
                .map(|row| fit(self.spacing.x, area.width, &row.cells))
                .fold(self.spacing.x, f32::min),
            fit(self.spacing.y, area.height, &grid.heights()),
        );
        let block = Vector2::new(grid.width(gap.x), grid.height(gap.y));

        let (ax, ay) = self.anchor.fractions();
        let left = area.x + (area.width - block.x) * ax;
        let top = area.y + (area.height - block.y) * ay;
        let align = self.align.fraction();
        let stretch = self.align == Align::Stretch;

        let mut rects = Vec::with_capacity(sizes.len());
        let mut y = top;
        for row in &grid.rows {
            let spare = block.x - span(&row.cells, gap.x);
            let (row_left, grow) = match (grid.table, stretch) {
                (true, _) => (left, 0.0),
                (false, true) => (left, spare / row.cells.len() as f32),
                (false, false) => (left + spare * align, 0.0),
            };

            let mut x = row_left;
            for (cell, size) in row.cells.iter().zip(&sizes[row.start..]) {
                let cell = cell + grow;
                let width = if stretch { cell } else { size.x };
                rects.push(Rectangle::new(
                    x + (cell - width) * align,
                    y + (row.height - size.y) / 2.0,
                    width,
                    size.y,
                ));
                x += cell + gap.x;
            }
            y += row.height + gap.y;
        }
        rects
    }
}

struct Row {
    start: usize,
    cells: Vec<f32>,
    height: f32,
}

struct Grid {
    rows: Vec<Row>,
    table: bool,
}

impl Grid {
    fn new(layout: &Layout, sizes: &[Vector2]) -> Self {
        let counts = layout.row_counts(sizes.len());
        let table = !matches!(layout.arrangement, Arrangement::RowsOf(_));

        let columns = counts.iter().copied().max().unwrap_or(0);
        let mut shared = vec![0.0_f32; columns];
        if table {
            for (i, size) in sizes.iter().enumerate() {
                shared[i % columns] = shared[i % columns].max(size.x);
            }
        }

        let mut start = 0;
        let rows = counts
            .into_iter()
            .map(|count| {
                let items = &sizes[start..start + count];
                let cells = if table {
                    shared[..count].to_vec()
                } else {
                    items.iter().map(|size| size.x).collect()
                };
                let height = items.iter().map(|size| size.y).fold(0.0, f32::max);
                let row = Row {
                    start,
                    cells,
                    height,
                };
                start += count;
                row
            })
            .collect();

        Self { rows, table }
    }

    fn heights(&self) -> Vec<f32> {
        self.rows.iter().map(|row| row.height).collect()
    }

    fn width(&self, gap: f32) -> f32 {
        self.rows
            .iter()
            .map(|row| span(&row.cells, gap))
            .fold(0.0, f32::max)
    }

    fn height(&self, gap: f32) -> f32 {
        span(&self.heights(), gap)
    }
}

fn span(lengths: &[f32], gap: f32) -> f32 {
    lengths.iter().sum::<f32>() + lengths.len().saturating_sub(1) as f32 * gap
}

fn fit(gap: f32, available: f32, lengths: &[f32]) -> f32 {
    let gaps = lengths.len().saturating_sub(1) as f32;
    if gaps == 0.0 {
        return gap;
    }
    let room = (available - lengths.iter().sum::<f32>()) / gaps;
    gap.min(room.max(0.0))
}
