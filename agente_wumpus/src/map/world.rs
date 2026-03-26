use rand::seq::SliceRandom;
use std::fmt;

use crate::map::Cell;

#[derive(Debug)]
pub struct World {
    size: usize,
    grid: Vec<Vec<Cell>>,
    start: (usize, usize),
}

impl World {
    pub fn random(size: usize, pits: usize, wumpus: usize) -> Self {
        assert!(size >= 2, "size must be at least 2");

        let total_cells = size * size;
        // Reserve (0,0) for the agent start and place exactly one gold.
        assert!(
            pits + wumpus + 1 <= total_cells - 1,
            "pits, wumpus, and gold must fit while keeping start cell safe"
        );

        let mut grid = vec![vec![Cell::Empty; size]; size];

        // Candidate positions exclude the start cell.
        let mut positions = Vec::with_capacity(total_cells - 1);
        for r in 0..size {
            for c in 0..size {
                if (r, c) != (0, 0) {
                    positions.push((r, c));
                }
            }
        }

        let mut rng = rand::rng();
        positions.shuffle(&mut rng);

        let mut idx = 0;

        for _ in 0..pits {
            let (r, c) = positions[idx];
            grid[r][c] = Cell::Pit;
            idx += 1;
        }

        for _ in 0..wumpus {
            let (r, c) = positions[idx];
            grid[r][c] = Cell::Wumpus;
            idx += 1;
        }

        // One gold.
        let (r, c) = positions[idx];
        grid[r][c] = Cell::Gold;

        // Extra safety: keep start always empty.
        grid[0][0] = Cell::Empty;

        Self {
            size,
            grid,
            start: (0, 0),
        }
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn start(&self) -> (usize, usize) {
        self.start
    }

    pub fn in_bounds(&self, row: isize, col: isize) -> bool {
        row >= 0 && col >= 0 && (row as usize) < self.size && (col as usize) < self.size
    }

    pub fn cell(&self, row: usize, col: usize) -> Option<Cell> {
        self.grid.get(row).and_then(|line| line.get(col)).copied()
    }

    pub fn set_cell(&mut self, row: usize, col: usize, cell: Cell) -> bool {
        if row < self.size && col < self.size {
            self.grid[row][col] = cell;
            true
        } else {
            false
        }
    }

    pub fn is_pit(&self, row: usize, col: usize) -> bool {
        matches!(self.cell(row, col), Some(Cell::Pit))
    }

    pub fn is_wumpus(&self, row: usize, col: usize) -> bool {
        matches!(self.cell(row, col), Some(Cell::Wumpus))
    }

    pub fn is_gold(&self, row: usize, col: usize) -> bool {
        matches!(self.cell(row, col), Some(Cell::Gold))
    }

    pub fn is_hazard(&self, row: usize, col: usize) -> bool {
        self.is_pit(row, col) || self.is_wumpus(row, col)
    }

    pub fn neighbors(&self, row: usize, col: usize) -> Vec<(usize, usize)> {
        let candidates = [
            (row as isize - 1, col as isize),
            (row as isize + 1, col as isize),
            (row as isize, col as isize - 1),
            (row as isize, col as isize + 1),
        ];

        candidates
            .into_iter()
            .filter(|(r, c)| self.in_bounds(*r, *c))
            .map(|(r, c)| (r as usize, c as usize))
            .collect()
    }

    pub fn has_adjacent_pit(&self, row: usize, col: usize) -> bool {
        self.neighbors(row, col)
            .into_iter()
            .any(|(r, c)| self.is_pit(r, c))
    }

    pub fn has_adjacent_wumpus(&self, row: usize, col: usize) -> bool {
        self.neighbors(row, col)
            .into_iter()
            .any(|(r, c)| self.is_wumpus(r, c))
    }

    pub fn pickup_gold_at(&mut self, row: usize, col: usize) -> bool {
        if self.is_gold(row, col) {
            self.grid[row][col] = Cell::Empty;
            true
        } else {
            false
        }
    }

    /// Shoots in straight line from `from` toward `dir`:
    /// (-1,0)=up, (1,0)=down, (0,-1)=left, (0,1)=right.
    /// Returns true if a Wumpus was hit (and removed).
    pub fn shoot_arrow(&mut self, from: (usize, usize), dir: (isize, isize)) -> bool {
        let (dr, dc) = dir;
        if !matches!((dr, dc), (-1, 0) | (1, 0) | (0, -1) | (0, 1)) {
            return false;
        }

        let mut r = from.0 as isize + dr;
        let mut c = from.1 as isize + dc;

        while self.in_bounds(r, c) {
            let ur = r as usize;
            let uc = c as usize;

            if self.is_wumpus(ur, uc) {
                self.grid[ur][uc] = Cell::Empty;
                return true;
            }

            r += dr;
            c += dc;
        }

        false
    }

    pub fn count_cell(&self, target: Cell) -> usize {
        self.grid
            .iter()
            .flatten()
            .filter(|&&cell| cell == target)
            .count()
    }
}

impl fmt::Display for World {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in &self.grid {
            for cell in row {
                write!(f, "{} ", cell.symbol())?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
