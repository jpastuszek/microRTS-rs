use std::ptr;
use std::hash::{Hash, Hasher};
use std::slice::Iter as SliceIter;
use std::iter::Zip as ZipIter;
use std::iter::Map as MapIter;
use std::iter::Enumerate;
use std::option::IntoIter as OptionIntoIter;
use std::iter::Cycle;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Coordinates(pub usize, pub usize);

impl Coordinates {
    pub fn in_direction(&self, direction: Direction) -> Option<Coordinates> {
        match direction {
            Direction::Up if self.1 > 0 => Some(Coordinates(self.0, self.1 - 1)),
            Direction::Right if self.0 < usize::max_value() => Some(
                Coordinates(self.0 + 1, self.1),
            ),
            Direction::Down if self.1 < usize::max_value() => Some(Coordinates(self.0, self.1 + 1)),
            Direction::Left if self.0 > 0 => Some(Coordinates(self.0 - 1, self.1)),
            _ => None,
        }
    }
}

#[test]
fn in_direction_overflow() {
    assert!(Coordinates(0, 0).in_direction(Direction::Up).is_none());
    assert!(Coordinates(0, 0).in_direction(Direction::Left).is_none());
    assert!(Coordinates(0, 0).in_direction(Direction::Down).is_some());
    assert!(Coordinates(0, 0).in_direction(Direction::Right).is_some());

    assert!(Coordinates(1, 1).in_direction(Direction::Up).is_some());
    assert!(Coordinates(1, 1).in_direction(Direction::Left).is_some());
    assert!(Coordinates(1, 1).in_direction(Direction::Down).is_some());
    assert!(Coordinates(1, 1).in_direction(Direction::Right).is_some());

    assert!(
        Coordinates(usize::max_value(), usize::max_value())
            .in_direction(Direction::Up)
            .is_some()
    );
    assert!(
        Coordinates(usize::max_value(), usize::max_value())
            .in_direction(Direction::Left)
            .is_some()
    );
    assert!(
        Coordinates(usize::max_value(), usize::max_value())
            .in_direction(Direction::Down)
            .is_none()
    );
    assert!(
        Coordinates(usize::max_value(), usize::max_value())
            .in_direction(Direction::Right)
            .is_none()
    );
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Direction {
    Up,
    Right,
    Down,
    Left,
}

impl Direction {
    fn clockwise() -> DirectionClockwiseIter {
        DirectionClockwiseIter { direction: Some(Direction::Up) }
    }
}

#[derive(Debug)]
pub struct DirectionClockwiseIter {
    direction: Option<Direction>,
}

impl Iterator for DirectionClockwiseIter {
    type Item = Direction;

    fn next(&mut self) -> Option<Direction> {
        let next_direction = match self.direction {
            Some(Direction::Up) => Some(Direction::Right),
            Some(Direction::Right) => Some(Direction::Down),
            Some(Direction::Down) => Some(Direction::Left),
            Some(Direction::Left) => None,
            None => None,
        };
        let ret = self.direction;
        self.direction = next_direction;
        ret
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Location<'m> {
    map: &'m Map,
    pub coordinates: Coordinates,
    pub tile: &'m Tile,
}

impl<'m> PartialEq for Location<'m> {
    fn eq(&self, other: &Location<'m>) -> bool {
        ptr::eq(self.tile, other.tile)
    }
}

impl<'m> Eq for Location<'m> {}

impl<'m> Hash for Location<'m> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_usize((self.tile as *const Tile) as usize)
    }
}

impl<'m> Location<'m> {
    pub fn neighbours(&self) -> NeighboursIter<'m> {
        NeighboursIter {
            location: self.clone(),
            directions: Direction::clockwise(),
        }
    }

    pub fn in_direction(&self, direction: Direction) -> Option<Location<'m>> {
        self.coordinates.in_direction(direction).and_then(
            |coordinates| {
                self.map.location(coordinates)
            },
        )
    }

    pub fn walkable(&self) -> bool {
        match *self.tile {
            Tile::Empty => true,
            _ => false,
        }
    }
}

pub struct NeighboursIter<'m> {
    location: Location<'m>,
    directions: DirectionClockwiseIter,
}

impl<'m> Iterator for NeighboursIter<'m> {
    type Item = (Direction, Location<'m>);

    fn next(&mut self) -> Option<(Direction, Location<'m>)> {
        while let Some(direction) = self.directions.next() {
            if let ret @ Some(_) = self.location.in_direction(direction).map(|location| {
                (direction, location)
            })
            {
                return ret;
            }
        }
        None
    }
}

#[test]
fn location_neighbours() {
    let map = Map::new(8, 8);

    assert_eq!(
        map.location(Coordinates(0, 0))
            .unwrap()
            .neighbours()
            .collect::<Vec<_>>(),
        vec![
            (Direction::Right, map.location(Coordinates(1, 0)).unwrap()),
            (Direction::Down, map.location(Coordinates(0, 1)).unwrap()),
        ]
    )
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub enum Tile {
    Empty,
    Wall,
}

#[derive(Debug)]
pub struct Map {
    tiles: Vec<Vec<Tile>>, // TODO: Matix
}

pub struct Dimension(usize);

impl Dimension {
    pub fn new(dimension: usize) -> Option<Dimension> {
        if dimension > 0 {
            Some(Dimension(dimension))
        } else {
            None
        }
    }

    pub fn unwrap(self) -> usize {
        self.0
    }
}


impl Map {
    pub fn new(width: Dimension, height: Dimension) -> Map {
        let width = width.unwrap();
        let height = height.unwrap();

        Map {
            tiles: (0..height)
                .map(|_row| (0..width).map(|_col| Tile::Empty).collect())
                .collect(),
        }
    }

    pub fn width(&self) -> usize {
        self.tiles.iter().next().unwrap().len()
    }

    pub fn height(&self) -> usize {
        self.tiles.len()
    }

    pub fn get_mut_tile(&mut self, coordinates: Coordinates) -> Option<&mut Tile> {
        self.tiles.get_mut(coordinates.1).and_then(|row| {
            row.get_mut(coordinates.0)
        })
    }

    pub fn location(&self, coordinates: Coordinates) -> Option<Location> {
        self.tiles
            .get(coordinates.1)
            .and_then(|row| row.get(coordinates.0))
            .map(|tile| {
                Location {
                    map: &self,
                    coordinates: coordinates,
                    tile: tile,
                }
            })
    }

    pub fn rows(&self) -> RowIter {
        fn to_row<'m>((row_no, (tiles, map)): (usize, (&'m Vec<Tile>, &'m Map))) -> Row<'m> {
            Row {
                map: map,
                row_no: row_no,
                tiles: tiles.as_slice(),
            }
        }
        let maps = Some(self).into_iter().cycle();
        RowIter(self.tiles.iter().zip(maps).enumerate().map(to_row))
    }
}

pub struct Row<'m> {
    map: &'m Map,
    pub row_no: usize,
    pub tiles: &'m [Tile],
}

impl<'m> IntoIterator for Row<'m> {
    type Item = Location<'m>;
    type IntoIter = RowLocationsIter<'m>;

    fn into_iter(self) -> RowLocationsIter<'m> {
        let maps = Some(self.map).into_iter().cycle();
        RowLocationsIter {
            row_no: self.row_no,
            row_iter: self.tiles.iter().zip(maps).enumerate(),
        }
    }
}

type ESIter<'i, T> = Enumerate<ZipIter<SliceIter<'i, T>, Cycle<OptionIntoIter<&'i Map>>>>;
type ToRowFn<'m> = fn((usize, (&'m Vec<Tile>, &'m Map))) -> Row<'m>;
pub struct RowIter<'m>(MapIter<ESIter<'m, Vec<Tile>>, ToRowFn<'m>>);

impl<'m> Iterator for RowIter<'m> {
    type Item = Row<'m>;

    fn next(&mut self) -> Option<Row<'m>> {
        self.0.next()
    }
}

pub struct RowLocationsIter<'m> {
    row_no: usize,
    row_iter: ESIter<'m, Tile>,
}

impl<'m> Iterator for RowLocationsIter<'m> {
    type Item = Location<'m>;

    fn next(&mut self) -> Option<Location<'m>> {
        self.row_iter.next().map(|(col_no, (tile, map))| {
            Location {
                map: map,
                coordinates: Coordinates(col_no, self.row_no),
                tile: tile,
            }
        })
    }
}


#[test]
fn test_direction_iter() {
    assert_eq!(
        Direction::clockwise().collect::<Vec<_>>(),
        vec![
            Direction::Up,
            Direction::Right,
            Direction::Down,
            Direction::Left,
        ]
    )
}
