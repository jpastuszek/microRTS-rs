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
            Direction::Right if self.0 < usize::max_value() => {
                Some(Coordinates(self.0 + 1, self.1))
            }
            Direction::Down if self.1 < usize::max_value() => Some(Coordinates(self.0, self.1 + 1)),
            Direction::Left if self.0 > 0 => Some(Coordinates(self.0 - 1, self.1)),
            _ => None,
        }
    }

    pub fn direction_to(&self, to: Coordinates) -> Option<Direction> {
        if self.0 == to.0 && self.1 == to.1 {
            return None;
        }

        Some(
            if to.1 + to.0 >= self.1 + self.0 && to.1 + self.0 > to.0 + self.1 {
                Direction::Down
            } else if to.1 + self.0 <= to.0 + self.1 && to.1 + to.0 > self.1 + self.0 {
                Direction::Right
            } else if to.1 + to.0 <= self.1 + self.0 && to.1 + self.0 < to.0 + self.1 {
                Direction::Up
            } else {
                Direction::Left
            },
        )
    }
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
        DirectionClockwiseIter {
            direction: Some(Direction::Up),
        }
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
pub struct Location<'t> {
    terrain: &'t Terrain,
    pub coordinates: Coordinates,
    pub tile: &'t Tile,
}

impl<'t> PartialEq for Location<'t> {
    fn eq(&self, other: &Location<'t>) -> bool {
        ptr::eq(self.tile, other.tile)
    }
}

impl<'t> Eq for Location<'t> {}

impl<'t> Hash for Location<'t> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_usize((self.tile as *const Tile) as usize)
    }
}

impl<'t> Location<'t> {
    pub fn neighbours(&self) -> NeighboursIter<'t> {
        NeighboursIter {
            location: self.clone(),
            directions: Direction::clockwise(),
        }
    }

    pub fn in_direction(&self, direction: Direction) -> Option<Location<'t>> {
        self.coordinates
            .in_direction(direction)
            .and_then(|coordinates| self.terrain.location(coordinates))
    }

    pub fn direction_to(&self, to: Location<'t>) -> Option<Direction> {
        self.coordinates.direction_to(to.coordinates)
    }

    pub fn walkable(&self) -> bool {
        match *self.tile {
            Tile::Empty => true,
            _ => false,
        }
    }
}

pub struct NeighboursIter<'t> {
    location: Location<'t>,
    directions: DirectionClockwiseIter,
}

impl<'t> Iterator for NeighboursIter<'t> {
    type Item = (Direction, Location<'t>);

    fn next(&mut self) -> Option<(Direction, Location<'t>)> {
        while let Some(direction) = self.directions.next() {
            if let ret @ Some(_) = self.location
                .in_direction(direction)
                .map(|location| (direction, location))
            {
                return ret;
            }
        }
        None
    }
}

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub enum Tile {
    Empty,
    Wall,
}

#[derive(Debug)]
pub struct Terrain {
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

impl Terrain {
    pub fn width(&self) -> usize {
        self.tiles.iter().next().unwrap().len()
    }

    pub fn height(&self) -> usize {
        self.tiles.len()
    }

    pub fn location(&self, coordinates: Coordinates) -> Option<Location> {
        self.tiles
            .get(coordinates.1)
            .and_then(|row| row.get(coordinates.0))
            .map(|tile| {
                Location {
                    terrain: &self,
                    coordinates: coordinates,
                    tile: tile,
                }
            })
    }

    pub fn rows(&self) -> RowIter {
        fn to_row<'t>((row_no, (tiles, terrain)): (usize, (&'t Vec<Tile>, &'t Terrain))) -> Row<'t> {
            Row {
                terrain: terrain,
                row_no: row_no,
                tiles: tiles.as_slice(),
            }
        }
        let terrains = Some(self).into_iter().cycle();
        RowIter(self.tiles.iter().zip(terrains).enumerate().map(to_row))
    }
}

//TODO: Error
#[derive(Debug)]
pub enum TerrainBuilderError {
    OutOfTerrain(Coordinates),
    CoordinatesAlreadyOccupied(Coordinates, Tile),
}

pub struct TerrainBuilder {
    tiles: Vec<Vec<Tile>>, // TODO: Matix
}

impl TerrainBuilder {
    pub fn new(width: Dimension, height: Dimension) -> TerrainBuilder {
        let width = width.unwrap();
        let height = height.unwrap();

        TerrainBuilder {
            tiles: (0..height)
                .map(|_row| (0..width).map(|_col| Tile::Empty).collect())
                .collect(),
        }
    }

    pub fn place(mut self, coordinates: Coordinates, tile: Tile) -> Result<TerrainBuilder, TerrainBuilderError> {
        self.tiles
            .get_mut(coordinates.1)
            .and_then(|row| row.get_mut(coordinates.0))
            .ok_or(TerrainBuilderError::OutOfTerrain(coordinates))
            .and_then(|terrain_tile| if *terrain_tile != Tile::Empty {
                Err(TerrainBuilderError::CoordinatesAlreadyOccupied(
                    coordinates,
                    terrain_tile.clone(),
                ))
            } else {
                Ok(terrain_tile)
            })
            .map(|terrain_tile| *terrain_tile = tile)
            .map(|_| self)
    }

    pub fn build(self) -> Terrain {
        Terrain {
            tiles: self.tiles
        }
    }

    pub fn terrain_8x8_wall1() -> Terrain {
        TerrainBuilder::new(Dimension::new(8).unwrap(), Dimension::new(8).unwrap())
            .place(Coordinates(2, 5), Tile::Wall).unwrap()
            .place(Coordinates(3, 4), Tile::Wall).unwrap()
            .place(Coordinates(4, 3), Tile::Wall).unwrap()
            .place(Coordinates(5, 2), Tile::Wall).unwrap()
            .build()
    }
}
    
pub struct Row<'t> {
    terrain: &'t Terrain,
    pub row_no: usize,
    pub tiles: &'t [Tile],
}

impl<'t> IntoIterator for Row<'t> {
    type Item = Location<'t>;
    type IntoIter = RowLocationsIter<'t>;

    fn into_iter(self) -> RowLocationsIter<'t> {
        let terrains = Some(self.terrain).into_iter().cycle();
        RowLocationsIter {
            row_no: self.row_no,
            row_iter: self.tiles.iter().zip(terrains).enumerate(),
        }
    }
}

type ESIter<'i, T> = Enumerate<ZipIter<SliceIter<'i, T>, Cycle<OptionIntoIter<&'i Terrain>>>>;
type ToRowFn<'t> = fn((usize, (&'t Vec<Tile>, &'t Terrain))) -> Row<'t>;
pub struct RowIter<'t>(MapIter<ESIter<'t, Vec<Tile>>, ToRowFn<'t>>);

impl<'t> Iterator for RowIter<'t> {
    type Item = Row<'t>;

    fn next(&mut self) -> Option<Row<'t>> {
        self.0.next()
    }
}

pub struct RowLocationsIter<'t> {
    row_no: usize,
    row_iter: ESIter<'t, Tile>,
}

impl<'t> Iterator for RowLocationsIter<'t> {
    type Item = Location<'t>;

    fn next(&mut self) -> Option<Location<'t>> {
        self.row_iter.next().map(|(col_no, (tile, terrain))| {
            Location {
                terrain: terrain,
                coordinates: Coordinates(col_no, self.row_no),
                tile: tile,
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_direction_to() {
        assert_eq!(Coordinates(1, 1).direction_to(Coordinates(1, 1)), None);

        assert_eq!(
            Coordinates(0, 0).direction_to(Coordinates(1, 0)),
            Some(Direction::Right)
        );
        assert_eq!(
            Coordinates(2, 0).direction_to(Coordinates(1, 0)),
            Some(Direction::Left)
        );

        assert_eq!(
            Coordinates(0, 0).direction_to(Coordinates(0, 1)),
            Some(Direction::Down)
        );
        assert_eq!(
            Coordinates(0, 2).direction_to(Coordinates(0, 1)),
            Some(Direction::Up)
        );

        assert_eq!(
            Coordinates(0, 0).direction_to(Coordinates(2, 1)),
            Some(Direction::Right)
        );
        assert_eq!(
            Coordinates(4, 0).direction_to(Coordinates(2, 1)),
            Some(Direction::Left)
        );

        assert_eq!(
            Coordinates(0, 2).direction_to(Coordinates(2, 1)),
            Some(Direction::Right)
        );
        assert_eq!(
            Coordinates(4, 2).direction_to(Coordinates(2, 1)),
            Some(Direction::Left)
        );

        assert_eq!(
            Coordinates(0, 0).direction_to(Coordinates(1, 2)),
            Some(Direction::Down)
        );
        assert_eq!(
            Coordinates(0, 4).direction_to(Coordinates(1, 2)),
            Some(Direction::Up)
        );
    }

    #[test]
    fn location_neighbours() {
        let terrain = TerrainBuilder::new(Dimension::new(8).unwrap(), Dimension::new(8).unwrap()).build();

        assert_eq!(
            terrain.location(Coordinates(0, 0))
                .unwrap()
                .neighbours()
                .collect::<Vec<_>>(),
            vec![
                (Direction::Right, terrain.location(Coordinates(1, 0)).unwrap()),
                (Direction::Down, terrain.location(Coordinates(0, 1)).unwrap()),
            ]
        )
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
}
