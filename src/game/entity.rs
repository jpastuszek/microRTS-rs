use std::collections::HashMap;
use std::ops::RangeFrom;
use std::collections::hash_map::Iter as HashMapIter;

use game::player::Player;
use game::map::{Location, Tile};

#[derive(Debug)]
pub enum Unit {
    Worker,
    Light,
    Heavy,
}

#[derive(Debug)]
pub struct Resources(pub u64);

#[derive(Debug)]
pub enum Building {
    Base(Resources),
    Barracks,
}

#[derive(Debug)]
pub enum EntityType<'p> {
    Unit(&'p Player, Unit),
    Building(&'p Player, Building),
    Resource(u64),
}

// Using Copy object to reference entities to allow them to be modified, removed
// etc. while AI are holding this
#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
pub struct EntityID(usize);

#[derive(Debug)]
pub struct Entity<'m, 'p>(pub Location<'m>, pub EntityType<'p>);

#[derive(Debug)]
pub struct Entities<'p, 'm> {
    entities: HashMap<EntityID, Entity<'m, 'p>>,
    entity_id_seq: RangeFrom<usize>,
    location_index: HashMap<Location<'m>, EntityID>,
}

#[derive(Debug)]
pub enum EntitiesError<'m> {
    NoEntity(EntityID),
    InvalidPlacementLocation(Location<'m>),
    LocationAlreadyTaken(Location<'m>, EntityID),
}

impl<'p, 'm> Entities<'p, 'm> {
    pub fn new() -> Entities<'p, 'm> {
        Entities {
            entities: Default::default(),
            location_index: Default::default(),
            entity_id_seq: 0..,
        }
    }

    pub fn place(
        &mut self,
        location: Location<'m>,
        entity: EntityType<'p>,
    ) -> Result<Option<Entity<'m, 'p>>, EntitiesError> {
        match location {
            location @ Location(_, &Tile::Plain) => Ok({
                let entity_id = EntityID(self.entity_id_seq.next().expect("out of IDs"));

                if let Some(entity_id) = self.location_index.get(&location) {
                    return Err(EntitiesError::LocationAlreadyTaken(location, *entity_id));
                }

                self.location_index.insert(location.clone(), entity_id);
                self.entities.insert(entity_id, Entity(location, entity))
            }),
            _ => Err(EntitiesError::InvalidPlacementLocation(location)),
        }
    }

    pub fn get_by_entity_id<'e>(&'e self, entity_id: &EntityID) -> Option<&'e Entity<'m, 'p>> {
        self.entities.get(entity_id)
    }

    pub fn get_by_location<'e>(&'e self, location: &Location<'m>) -> Option<&'e Entity<'m, 'p>> {
        self.location_index.get(&location).and_then(|entity_id| {
            self.get_by_entity_id(entity_id)
        })
    }

    pub fn set_location_by_entity_id<'e>(
        &'e mut self,
        entity_id: &EntityID,
        location: Location<'m>,
    ) -> Result<(), EntitiesError> {
        if let Some(entity_id) = self.location_index.get(&location) {
            return Err(EntitiesError::LocationAlreadyTaken(location, *entity_id));
        }

        if let Some(ref mut entity) = self.entities.get_mut(&entity_id) {
            entity.0 = location;
            Ok(())
        } else {
            Err(EntitiesError::NoEntity(*entity_id))
        }
    }

    pub fn iter<'e>(&'e self) -> EntitiesIter<'p, 'm, 'e> {
        EntitiesIter { iter: self.entities.iter() }
    }
}

impl<'p: 'e, 'm: 'e, 'e> IntoIterator for &'e Entities<'p, 'm> {
    type IntoIter = EntitiesIter<'p, 'm, 'e>;
    type Item = (&'e EntityID, &'e Entity<'m, 'p>);

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct EntitiesIter<'p: 'e, 'm: 'e, 'e> {
    iter: HashMapIter<'e, EntityID, Entity<'m, 'p>>,
}

impl<'p: 'e, 'm: 'e, 'e> Iterator for EntitiesIter<'p, 'm, 'e> {
    type Item = (&'e EntityID, &'e Entity<'m, 'p>);

    fn next(&mut self) -> Option<(&'e EntityID, &'e Entity<'m, 'p>)> {
        self.iter.next()
    }
}
