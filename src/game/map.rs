use std::slice::Iter as SliceIter;
use std::iter::Map as MapIter;
use std::iter::Enumerate;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Coordinates(pub usize, pub usize);

impl Coordinates {
    pub fn in_direction(&self, direction: Direction) -> Coordinates {
        match direction {
            Direction::Up => Coordinates(self.0, self.1 - 1),
            Direction::Down => Coordinates(self.0, self.1 + 1),
            Direction::Left => Coordinates(self.0 - 1, self.1),
            Direction::Right => Coordinates(self.0 + 1, self.1),
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
    Void, // TODO: Option<Tile>
    Plain, // TODO: Empty
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

type ESIter<'i, T> = Enumerate<SliceIter<'i, T>>;
pub struct RowIter<'m>(MapIter<ESIter<'m, Vec<Tile>>, fn((usize, &'m Vec<Tile>)) -> Row<'m>>);

impl<'m> Iterator for RowIter<'m> {
    type Item = Row<'m>;

    fn next(&mut self) -> Option<Row<'m>> {
        self.0.next()
    }
}

pub struct Row<'m> {
    pub row_no: usize,
    pub tiles: &'m [Tile],
}

pub struct RowLocationsIter<'m> {
    row_no: usize,
    row_iter: ESIter<'m, Tile>,
}

impl<'m> Iterator for RowLocationsIter<'m> {
    type Item = Location<'m>;

    fn next(&mut self) -> Option<Location<'m>> {
        self.row_iter.next().map(|(col_no, tile)| {
            Location(Coordinates(col_no, self.row_no), tile)
        })
    }
}

impl<'m> IntoIterator for Row<'m> {
    type Item = Location<'m>;
    type IntoIter = RowLocationsIter<'m>;

    fn into_iter(self) -> RowLocationsIter<'m> {
        RowLocationsIter {
            row_no: self.row_no,
            row_iter: self.tiles.iter().enumerate(),
        }
    }
}

impl Map {
    pub fn new(width: usize, height: usize) -> Map {
        //TODO: make Dimension type that need to by build and unwrapped
        if width == 0 || height == 0 {
            panic!("Map cannot have 0 dimension!");
        }

        Map {
            tiles: (0..height)
                .map(|_row| (0..width).map(|_col| Tile::Plain).collect())
                .collect(),
            void: Tile::Void,
        }
    }

    pub fn width(&self) -> usize {
        self.tiles.iter().next().unwrap().len()
    }

    pub fn height(&self) -> usize {
        self.tiles.len()
    }

    pub fn rows(&self) -> RowIter {
        fn map(row: (usize, &Vec<Tile>)) -> Row {
            Row {
                row_no: row.0,
                tiles: row.1.as_slice(),
            }
        }
        RowIter(self.tiles.iter().enumerate().map(map))
    }

    pub fn get_mut_tile(&mut self, coordinates: Coordinates) -> Option<&mut Tile> {
        self.tiles.get_mut(coordinates.0).and_then(|row| {
            row.get_mut(coordinates.1)
        })
    }

    pub fn location(&self, coordinates: Coordinates) -> Location {
        let tile = self.tiles
            .get(coordinates.0)
            .and_then(|row| row.get(coordinates.1))
            .unwrap_or(&self.void);
        Location(coordinates, tile)
    }
}
