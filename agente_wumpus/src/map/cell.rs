#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Empty,
    Pit,
    Wumpus,
    Gold,
}

impl Cell {
    pub fn symbol(self) -> char {
        match self {
            Cell::Empty => '.',
            Cell::Pit => 'P',
            Cell::Wumpus => 'W',
            Cell::Gold => 'G',
        }
    }
}