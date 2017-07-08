#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Coordinates(pub usize, pub usize);

impl Coordinates {
    pub fn in_direction(&self, direction: Direction) -> Coordinates {
        match direction {
            Direction::Up => Coordinates(self.0, self.1 - 1),
            Direction::Down => Coordinates(self.0, self.1 + 1),
            Direction::Left => Coordinates(self.0 - 1, self.1),
            Direction::Right => Coordinates(self.0 + 1, self.1)
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub enum Tile {
    Void,
    Plain,
    Wall,
}

#[derive(Debug)]
pub struct Map {
    tiles: Vec<Vec<Tile>>, // TODO: Matix
    void: Tile,
}

// TODO: both Coordinates and Tile pointer values are key here... hmm
#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct Location<'m>(pub Coordinates, pub &'m Tile);

impl Map {
    pub fn new(width: usize, height: usize) -> Map {
        Map {
            tiles: (0..height)
                .map(|_row| (0..width).map(|_col| Tile::Plain).collect())
                .collect(),
            void: Tile::Void,
        }
    }

    pub fn location(&self, coordinates: Coordinates) -> Location {
        let tile = self.tiles
            .get(coordinates.0)
            .and_then(|row| row.get(coordinates.1))
            .unwrap_or(&self.void);
        Location(coordinates, tile)
    }
}
