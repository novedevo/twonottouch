use std::{collections::HashMap, fmt::Display};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Board {
    width: usize,
    height: usize,
    cells: Vec<Vec<Cell>>,
    /// indexable by region tag
    regions: Vec<Vec<(usize, usize)>>,
    #[cfg(test)]
    solution: Option<Box<Board>>,
}

impl Board {
    pub fn new(width: usize, height: usize, regions: Vec<Vec<usize>>) -> Self {
        let cells = Self::blank_from_regions(regions);
        let cells_by_region = cells
            .iter()
            .enumerate()
            .flat_map(|(row_index, row)| {
                row.iter()
                    .enumerate()
                    .map(move |(col_index, cell)| (cell.region, (row_index, col_index)))
            })
            .collect::<Vec<_>>();

        let mut regional_map: HashMap<usize, Vec<_>> = HashMap::new();
        for (region, coords) in cells_by_region {
            regional_map.entry(region).or_default().push(coords);
        }
        let mut tagged_regions = regional_map.into_iter().collect::<Vec<_>>();
        tagged_regions.sort();

        let result = Self {
            width,
            height,
            cells,
            regions: tagged_regions
                .into_iter()
                .map(|(_region, cells)| cells)
                .collect(),
            #[cfg(test)]
            solution: None,
        };
        result.print();
        result
    }

    #[cfg(test)]
    pub fn solved(width: usize, height: usize, stars: Vec<(usize, usize)>) -> Self {
        let mut cells = vec![
            vec![
                Cell {
                    region: 0,
                    state: CellState::Filled
                };
                10
            ];
            10
        ];
        for row in 0..height {
            let (star1, star2) = stars[row];
            cells[row][star1] = Cell {
                region: 0,
                state: CellState::Star,
            };
            cells[row][star2] = Cell {
                region: 0,
                state: CellState::Star,
            };
        }
        let result = Self {
            width,
            height,
            cells,
            regions: vec![],
            #[cfg(test)]
            solution: None,
        };
        result.print();
        result
    }

    #[cfg(test)]
    pub fn add_solution(&mut self, solution: Self) {
        self.solution = Some(Box::new(solution));
    }

    fn blank_from_regions(regions: Vec<Vec<usize>>) -> Vec<Vec<Cell>> {
        regions
            .into_iter()
            .map(|row| {
                row.into_iter()
                    .map(|region| Cell {
                        region,
                        state: CellState::Blank,
                    })
                    .collect()
            })
            .collect()
    }

    pub fn solve(&mut self) {
        let mut past_self = self.clone();
        loop {
            self.enforce_rules();
            //blackout before adding more stars
            self.add_required_stars_cols();
            self.enforce_rules();
            self.add_required_stars_rows();
            self.enforce_rules();
            self.add_required_stars_region();

            if &past_self == self {
                break;
            } else {
                past_self = self.clone();
                self.print();
            }
        }
    }

    fn enforce_rules(&mut self) {
        let mut past_self = self.clone();
        loop {
            #[cfg(test)]
            self.assert_matches_with_solution();
            self.blackout_cols();
            #[cfg(test)]
            self.assert_matches_with_solution();
            self.blackout_rows();
            #[cfg(test)]
            self.assert_matches_with_solution();
            self.blackout_regions();
            #[cfg(test)]
            self.assert_matches_with_solution();
            self.blackout_star_adjacencies();
            #[cfg(test)]
            self.assert_matches_with_solution();
            self.blackout_next_to_contiguity();
            #[cfg(test)]
            self.assert_matches_with_solution();
            self.eliminate_middle_of_small_empty_regions();

            self.regenerate_regions();

            #[cfg(test)]
            self.assert_matches_with_solution();
            if &past_self == self {
                break;
            } else if true {
                past_self = self.clone();
                self.print();
            }
        }
    }

    #[cfg(test)]
    fn assert_matches_with_solution(&self) {
        if let Some(solution) = &self.solution {
            for row in 0..self.height {
                for col in 0..self.width {
                    if self.cells[row][col].state != CellState::Blank
                        && self.cells[row][col].state != solution.cells[row][col].state
                    {
                        eprintln!(
                            "failed to match state: self followed by solution at {row}, {col}"
                        );
                        self.print();
                        solution.print();
                        panic!();
                    }
                }
            }
        }
    }

    pub fn print(&self) {
        for row in &self.cells {
            for cell in row {
                match cell.state {
                    CellState::Star | CellState::Filled => print!("{} ", cell.state),
                    CellState::Blank => print!("{} ", cell.region),
                }
            }
            println!();
        }
        println!();
    }

    fn adjacencies(&self, row: usize, col: usize) -> Vec<(usize, usize)> {
        adjacencies(self.width, self.height, row, col)
    }

    fn blackout_star_adjacencies(&mut self) {
        for row in 0..self.height {
            for col in 0..self.width {
                if self.cells[row][col].state == CellState::Star {
                    for (row, col) in self.adjacencies(row, col) {
                        if self.cells[row][col].state == CellState::Star {
                            unreachable!();
                        }
                        self.shade_coords(row, col);
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

    fn blackout_next_to_contiguity(&mut self) {
        for row in 0..self.height {
            let blanks = self.cells[row]
                .iter()
                .enumerate()
                .filter(|(_col, cell)| cell.state == CellState::Blank)
                .map(|(i, _)| i)
                .collect::<Vec<_>>();
            let starcount = self.cells[row]
                .iter()
                .filter(|cell| cell.state == CellState::Star)
                .count();

            if blanks.len() == 2 && starcount == 1 && blanks[1] - blanks[0] == 1 {
                if row != 0 {
                    self.cells[row - 1][blanks[0]].shade();
                    self.cells[row - 1][blanks[1]].shade();
                }
                if row < self.height - 1 {
                    self.cells[row + 1][blanks[0]].shade();
                    self.cells[row + 1][blanks[1]].shade();
                }
            } else if blanks.len() == 3 && starcount == 1 && blanks[2] - blanks[0] == 2 {
                if row != 0 {
                    self.cells[row - 1][blanks[1]].shade();
                }
                if row < self.height - 1 {
                    self.cells[row + 1][blanks[1]].shade();
                }
            } else if blanks.len() == 4 && starcount == 0 {
                if blanks[1] - blanks[0] == 1 {
                    if row != 0 {
                        self.cells[row - 1][blanks[0]].shade();
                        self.cells[row - 1][blanks[1]].shade();
                    }
                    if row < self.height - 1 {
                        self.cells[row + 1][blanks[0]].shade();
                        self.cells[row + 1][blanks[1]].shade();
                    }
                }
                if blanks[3] - blanks[2] == 1 {
                    if row != 0 {
                        self.cells[row - 1][blanks[2]].shade();
                        self.cells[row - 1][blanks[3]].shade();
                    }
                    if row < self.height - 1 {
                        self.cells[row + 1][blanks[2]].shade();
                        self.cells[row + 1][blanks[3]].shade();
                    }
                }
            }
        }

        for col in 0..self.width {
            let blanks = self
                .cells
                .iter_mut()
                .map(|row| &mut row[col])
                .enumerate()
                .filter(|(_col, cell)| cell.state == CellState::Blank)
                .map(|(i, _)| i)
                .collect::<Vec<_>>();
            let starcount = self
                .cells
                .iter_mut()
                .map(|row| &mut row[col])
                .filter(|cell| cell.state == CellState::Star)
                .count();

            if blanks.len() == 2 && starcount == 1 && blanks[1] - blanks[0] == 1 {
                if col != 0 {
                    self.cells[blanks[0]][col - 1].shade();
                    self.cells[blanks[1]][col - 1].shade();
                }
                if col < self.width - 1 {
                    self.cells[blanks[0]][col + 1].shade();
                    self.cells[blanks[1]][col + 1].shade();
                }
            } else if blanks.len() == 3 && starcount == 1 && blanks[2] - blanks[0] == 2 {
                if col != 0 {
                    self.cells[blanks[1]][col - 1].shade();
                }
                if col < self.width - 1 {
                    self.cells[blanks[1]][col + 1].shade();
                }
            } else if blanks.len() == 4 && starcount == 0 {
                if blanks[1] - blanks[0] == 1 {
                    if col != 0 {
                        self.cells[blanks[0]][col - 1].shade();
                        self.cells[blanks[1]][col - 1].shade();
                    }
                    if col < self.width - 1 {
                        self.cells[blanks[0]][col + 1].shade();
                        self.cells[blanks[1]][col + 1].shade();
                    }
                }
                if blanks[3] - blanks[2] == 1 {
                    if col != 0 {
                        self.cells[blanks[2]][col - 1].shade();
                        self.cells[blanks[3]][col - 1].shade();
                    }
                    if col < self.width - 1 {
                        self.cells[blanks[2]][col + 1].shade();
                        self.cells[blanks[3]][col + 1].shade();
                    }
                }
            }
        }
    }

    fn add_star_coords(&mut self, row: usize, col: usize) {
        self.cells[row][col].star();
        #[cfg(test)]
        self.assert_matches_with_solution();
        self.enforce_rules();
    }

    fn shade_coords(&mut self, row: usize, col: usize) {
        self.cells[row][col].shade();
        #[cfg(test)]
        self.assert_matches_with_solution();
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
        let starcount = row
            .iter()
            .filter(|cell| cell.state == CellState::Star)
            .count();
        let count = blanks.len();

        if starcount == 0 {
            if count <= 2 {
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
                    row[blanks[cell].0].star();
                }
            }
        } else if starcount == 1 && count == 1 {
            for cell in row {
                cell.star()
            }
        }
    }

    fn add_required_stars_region(&mut self) {
        for region in self.regions.clone() {
            let blanks = region
                .iter()
                .filter(|(row, col)| self.cells[*row][*col].state == CellState::Blank)
                .collect::<Vec<_>>();
            let starcount = region
                .iter()
                .filter(|(row, col)| self.cells[*row][*col].state == CellState::Star)
                .count();
            let count = blanks.len();

            if starcount == 0 {
                if count <= 2 {
                    for (row, col) in region {
                        self.add_star_coords(row, col);
                    }
                } else if count == 3 {
                    if adjacencies(self.width, self.height, blanks[0].0, blanks[0].1)
                        .contains(blanks[1])
                    {
                        self.add_star_coords(blanks[2].0, blanks[2].1);
                    } else if adjacencies(self.width, self.height, blanks[1].0, blanks[1].1)
                        .contains(blanks[2])
                    {
                        self.add_star_coords(blanks[0].0, blanks[0].1);
                    } else if adjacencies(self.width, self.height, blanks[0].0, blanks[0].1)
                        .contains(blanks[2])
                    {
                        self.add_star_coords(blanks[1].0, blanks[1].1);
                    }
                }
            } else if starcount == 1 && count == 1 {
                for (row, col) in region {
                    self.add_star_coords(row, col);
                }
            }
        }
    }

    fn eliminate_middle_of_small_empty_regions(&mut self) {
        self.print();
        for region in self.regions.clone() {
            let starcount = self.regional_stars(&region);
            if region.is_empty() || starcount != 0 {
                continue;
            }

            let mut min_row = usize::MAX;
            let mut max_row = usize::MIN;
            let mut min_col = usize::MAX;
            let mut max_col = usize::MIN;

            for (row, col) in region {
                min_row = min_row.min(row);
                min_col = min_col.min(col);
                max_row = max_row.max(row);
                max_col = max_col.max(col);
            }
            let width = max_col - min_col + 1;
            let height = max_row - min_row + 1;
            let area = width * height;
            if area <= 6 && width <= 3 && height <= 3 {
                //small region detected :)
                //time to find the middle
                if width <= height {
                    let mid_row = max_row - 1;
                    for col in min_col..=max_col {
                        self.shade_coords(mid_row, col);
                    }
                    if min_row != 0 {
                        for col in min_col..=max_col {
                            self.shade_coords(min_row - 1, col);
                        }
                    }
                    if max_row < self.height - 1 {
                        for col in min_col..=max_col {
                            self.shade_coords(max_row + 1, col);
                        }
                    }
                } else {
                    let mid_col = max_col - 1;
                    for row in min_row..=max_row {
                        self.shade_coords(row, mid_col);
                    }
                    if min_col != 0 {
                        for row in min_row..=max_row {
                            self.shade_coords(row, min_col - 1);
                        }
                    }
                    if max_col < self.width - 1 {
                        for row in min_row..=max_row {
                            self.shade_coords(row, max_col + 1);
                        }
                    }
                }
            }
        }
    }

    fn regional_stars(&self, region: &[(usize, usize)]) -> usize {
        region
            .iter()
            .filter(|(row, col)| self.cells[*row][*col].state == CellState::Star)
            .count()
    }

    fn regenerate_regions(&mut self) {
        self.regions = self
            .regions
            .iter()
            .map(|region| {
                region
                    .iter()
                    .filter(|(row, col)| self.cells[*row][*col].state != CellState::Filled)
                    .copied()
                    .collect::<Vec<(usize, usize)>>()
            })
            .collect()
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

impl Display for CellState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Blank => '-',
                Self::Star => 'X',
                Self::Filled => '#',
            }
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn test_board_sample() -> Board {
        Board::new(
            10,
            10,
            vec![
                vec![0, 0, 0, 1, 1, 1, 1, 2, 2, 3],
                vec![0, 0, 0, 1, 2, 2, 1, 2, 2, 3],
                vec![0, 0, 0, 1, 2, 2, 2, 2, 2, 3],
                vec![0, 0, 0, 0, 2, 2, 4, 4, 3, 3],
                vec![5, 5, 4, 4, 4, 4, 4, 4, 3, 3],
                vec![5, 5, 5, 5, 4, 6, 6, 6, 6, 3],
                vec![5, 5, 7, 5, 5, 6, 6, 6, 6, 3],
                vec![8, 8, 7, 7, 6, 6, 6, 6, 6, 3],
                vec![8, 9, 9, 7, 7, 7, 7, 6, 6, 3],
                vec![8, 9, 9, 9, 9, 7, 6, 6, 6, 6],
            ],
        )
    }
    fn test_board_stolen_1() -> Board {
        Board::new(
            10,
            10,
            vec![
                vec![0, 0, 1, 1, 2, 2, 2, 3, 3, 3],
                vec![0, 0, 1, 2, 2, 2, 2, 3, 2, 3],
                vec![0, 0, 1, 1, 1, 1, 2, 2, 2, 3],
                vec![0, 1, 1, 1, 4, 4, 5, 2, 2, 2],
                vec![0, 1, 1, 1, 4, 5, 5, 5, 5, 2],
                vec![0, 6, 6, 6, 4, 4, 4, 4, 2, 2],
                vec![7, 6, 7, 7, 4, 4, 4, 8, 8, 2],
                vec![7, 7, 7, 7, 4, 9, 9, 9, 8, 2],
                vec![7, 7, 7, 8, 8, 8, 8, 8, 8, 2],
                vec![7, 7, 7, 8, 8, 8, 8, 8, 8, 8],
            ],
        )
    }
    fn solved_board_stolen_1() -> Board {
        Board::solved(
            10,
            10,
            vec![
                (1, 3),
                (5, 7),
                (2, 9),
                (4, 6),
                (0, 8),
                (3, 6),
                (1, 9),
                (5, 7),
                (0, 2),
                (4, 8),
            ],
        )
    }

    #[test]
    fn test_constructor() {
        test_board_stolen_1();
    }

    #[test]
    fn test_solve() {
        // let mut board = test_board_sample();
        // board.solve();
        // board.print();
        let mut board = test_board_stolen_1();
        let solution = solved_board_stolen_1();
        board.add_solution(solution);
        board.solve();
        board.print();
        // let mut board = test_board_sample();
        // board.solve();
        // board.print();
    }

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
