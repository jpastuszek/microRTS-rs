use std::collections::HashMap;
use std::ops::RangeFrom;
use std::collections::hash_map::Iter as HashMapIter;

use game::player::Player;
use game::map::Location;

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
pub enum Object<'p> {
    Unit(&'p Player, Unit),
    Building(&'p Player, Building),
    Resource(u64),
}

// Using Copy object to reference entities to allow them to be modified, removed
// etc. while AI are holding this
#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
pub struct EntityID(pub usize);

#[derive(Debug)]
pub struct Entity<'m, 'p> {
    pub id: EntityID,
    pub location: Location<'m>,
    pub object: Object<'p>,
}

#[derive(Debug)]
pub struct Entities<'p, 'm> {
    entities: HashMap<EntityID, Entity<'m, 'p>>,
    entity_id_seq: RangeFrom<usize>,
    location_index: HashMap<Location<'m>, EntityID>,
}

#[derive(Debug)]
pub enum EntitiesError<'m> {
    NoEntity(EntityID),
    LocationNotWalkable(Location<'m>),
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
        object: Object<'p>,
    ) -> Result<EntityID, EntitiesError<'m>> {
        if !location.walkable() {
            return Err(EntitiesError::LocationNotWalkable(location));
        }

        if let Some(entity_id) = self.location_index.get(&location) {
            return Err(EntitiesError::LocationAlreadyTaken(location, *entity_id));
        }

        let entity_id = EntityID(self.entity_id_seq.next().expect("out of IDs"));
        let entity = Entity {
            id: entity_id,
            location: location.clone(),
            object: object
        };

        if self.entities.insert(entity_id, entity).is_some() {
            panic!("duplicate ID");
        }

        self.location_index.insert(location, entity_id);

        Ok(entity_id)
    }

    pub fn get_by_entity_id<'e>(&'e self, entity_id: &EntityID) -> Option<&'e Entity<'m, 'p>> {
        self.entities.get(entity_id)
    }

    pub fn get_by_location<'e>(&'e self, location: &Location<'m>) -> Option<&'e Entity<'m, 'p>> {
        self.location_index.get(&location).and_then(|entity_id| {
            self.get_by_entity_id(entity_id)
        })
    }

    pub fn set_location_by_entity_id(
        &mut self,
        entity_id: &EntityID,
        location: Location<'m>,
    ) -> Result<(), EntitiesError<'m>> {
        // Check if new location is valid for entity to be placed on
        if !location.walkable() {
            return Err(EntitiesError::LocationNotWalkable(location));
        }

        if let Some(entity_id) = self.location_index.get(&location) {
            return Err(EntitiesError::LocationAlreadyTaken(location, *entity_id));
        }

        match self.entities.get_mut(&entity_id) {
            None => Err(EntitiesError::NoEntity(*entity_id)),
            Some(&mut Entity { location: ref mut entity_location, .. }) => {
                // Update indexes first
                self.location_index.remove(&entity_location).expect(
                    "bad location_index",
                    );
                self.location_index.insert(location.clone(), *entity_id);

                // Update entity
                *entity_location = location;

                Ok(())
            }
        }
    }

    pub fn iter<'e>(&'e self) -> Iter<'p, 'm, 'e> {
        Iter { iter: self.entities.iter() }
    }
}

impl<'p: 'e, 'm: 'e, 'e> IntoIterator for &'e Entities<'p, 'm> {
    type IntoIter = Iter<'p, 'm, 'e>;
    type Item = (&'e EntityID, &'e Entity<'m, 'p>);

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct Iter<'p: 'e, 'm: 'e, 'e> {
    iter: HashMapIter<'e, EntityID, Entity<'m, 'p>>,
}

impl<'p: 'e, 'm: 'e, 'e> Iterator for Iter<'p, 'm, 'e> {
    type Item = (&'e EntityID, &'e Entity<'m, 'p>);

    fn next(&mut self) -> Option<(&'e EntityID, &'e Entity<'m, 'p>)> {
        self.iter.next()
    }
}
