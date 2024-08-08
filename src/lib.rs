use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Board {
    width: usize,
    height: usize,
    cells: Vec<Vec<Cell>>,
    /// indexable by region tag
    regions: Vec<Vec<(usize, usize)>>,
}

impl Board {
    pub fn new(width: usize, height: usize, cells: Vec<Vec<Cell>>) -> Self {
        let cells_by_region = cells
            .iter()
            .flat_map(|row| row.iter().enumerate())
            .enumerate()
            .map(|(row, (col, &cell))| (cell.region, (row, col)));

        let mut regional_map: HashMap<usize, Vec<_>> = HashMap::new();
        for (region, coords) in cells_by_region {
            regional_map.entry(region).or_default().push(coords);
        }
        let mut tagged_regions = regional_map.into_iter().collect::<Vec<_>>();
        tagged_regions.sort();

        Self {
            width,
            height,
            cells,
            regions: tagged_regions
                .into_iter()
                .map(|(_region, cells)| cells)
                .collect(),
        }
    }

    pub fn solve(&mut self) {
        let mut past_self = self.clone();
        loop {
            self.blackout_cols();
            self.blackout_rows();
            self.blackout_regions();
            self.blackout_star_adjacencies();

            self.add_required_stars_cols();
            self.add_required_stars_rows();
            self.add_required_stars_region();

            if &past_self == self {
                break;
            } else {
                past_self = self.clone();
            }
        }
    }

    fn adjacencies(&self, row: usize, col: usize) -> Vec<(usize, usize)> {
        adjacencies(self.width, self.height, row, col)
    }

    fn blackout_star_adjacencies(&mut self) {
        for row in 0..self.height {
            for col in 0..self.width {
                if self.cells[row][col].state == CellState::Star {
                    for (row, col) in self.adjacencies(row, col) {
                        self.cells[row][col].shade()
                    }
                }
            }
        }
    }

    fn blackout_rows(&mut self) {
        for row in &mut self.cells {
            if row
                .iter()
                .filter(|cell| cell.state == CellState::Star)
                .count()
                == 2
            {
                for cell in row {
                    cell.shade()
                }
            }
        }
    }

    fn blackout_cols(&mut self) {
        for col in 0..self.width {
            if self
                .cells
                .iter()
                .map(|row| row[col])
                .filter(|cell| cell.state == CellState::Star)
                .count()
                == 2
            {
                for row in 0..self.height {
                    self.cells[row][col].shade()
                }
            }
        }
    }

    fn blackout_regions(&mut self) {
        for region in &self.regions {
            if region
                .iter()
                .map(|(row, col)| self.cells[*row][*col])
                .filter(|cell| cell.state == CellState::Star)
                .count()
                == 2
            {
                for (row, col) in region {
                    self.cells[*row][*col].shade()
                }
            }
        }
    }

    fn add_required_stars_rows(&mut self) {
        for row in self.cells.iter_mut() {
            let mut row = row.iter_mut().collect::<Vec<_>>();
            Self::add_required_stars_slice(&mut row)
        }
    }
    fn add_required_stars_cols(&mut self) {
        for col in 0..self.width {
            let mut col = self
                .cells
                .iter_mut()
                .map(|row| &mut row[col])
                .collect::<Vec<&mut Cell>>();
            Self::add_required_stars_slice(&mut col);
        }
    }

    fn add_required_stars_slice(row: &mut [&mut Cell]) {
        let blanks = row
            .iter()
            .enumerate()
            .filter(|(_col, cell)| cell.state == CellState::Blank)
            .collect::<Vec<_>>();
        let count = blanks.len();

        if count == 2 {
            for cell in row {
                cell.star()
            }
        } else if count == 3 {
            let cell = if blanks[1].0 - blanks[0].0 == 1 {
                Some(2)
            } else if blanks[2].0 - blanks[1].0 == 1 {
                Some(0)
            } else {
                None
            };

            if let Some(cell) = cell {
                row[cell].star();
            }
        }
    }

    fn add_required_stars_region(&mut self) {
        for region in &mut self.regions {
            let blanks = region
                .iter()
                .enumerate()
                .filter(|(_col, (row, col))| self.cells[*row][*col].state == CellState::Blank)
                .collect::<Vec<_>>();
            let count = blanks.len();

            if count == 2 {
                for (row, col) in region {
                    self.cells[*row][*col].star()
                }
            }
        }
    }
}

fn adjacencies(width: usize, height: usize, row: usize, col: usize) -> Vec<(usize, usize)> {
    if row >= height || col >= width {
        return vec![];
    }

    let mut adjacencies = vec![];

    if row > 0 {
        adjacencies.push((row - 1, col));
        if col > 0 {
            adjacencies.push((row - 1, col - 1));
        }
        if col < width - 1 {
            adjacencies.push((row - 1, col + 1));
        }
    }
    if row < height - 1 {
        adjacencies.push((row + 1, col));
        if col > 0 {
            adjacencies.push((row + 1, col - 1));
        }
        if col < width - 1 {
            adjacencies.push((row + 1, col + 1));
        }
    }
    if col > 0 {
        adjacencies.push((row, col - 1));
    }
    if col < width - 1 {
        adjacencies.push((row, col + 1));
    }

    adjacencies
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cell {
    /// indexes into the `regions` member of the board struct
    region: usize,
    state: CellState,
}

impl Cell {
    fn shade(&mut self) {
        if self.state == CellState::Blank {
            self.state = CellState::Filled;
        }
    }
    fn star(&mut self) {
        if self.state == CellState::Blank {
            self.state = CellState::Star;
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CellState {
    Blank,
    Star,
    Filled,
}

#[cfg(test)]
mod test {

    use super::*;
    #[test]
    fn test_adjacencies() {
        unordered_eq(adjacencies(10, 10, 10, 10), vec![]);
        unordered_eq(adjacencies(10, 10, 0, 0), vec![(1, 0), (0, 1), (1, 1)]);
        unordered_eq(adjacencies(10, 10, 9, 9), vec![(8, 9), (9, 8), (8, 8)]);
        unordered_eq(
            adjacencies(10, 10, 0, 5),
            vec![(0, 6), (0, 4), (1, 6), (1, 4), (1, 5)],
        );
        unordered_eq(
            adjacencies(10, 10, 5, 0),
            vec![(6, 0), (4, 0), (6, 1), (4, 1), (5, 1)],
        );
        unordered_eq(
            adjacencies(10, 10, 5, 5),
            vec![
                (4, 4),
                (4, 5),
                (4, 6),
                (5, 4),
                (5, 6),
                (6, 4),
                (6, 5),
                (6, 6),
            ],
        );
    }

    fn unordered_eq<T: Eq + Clone + Ord + core::fmt::Debug>(vec1: Vec<T>, vec2: Vec<T>) {
        let (mut v1, mut v2) = (vec1.clone(), vec2.clone());
        v1.sort();
        v2.sort();

        assert_eq!(v1, v2);
    }
}
