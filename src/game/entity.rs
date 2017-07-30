use std::collections::HashMap;
use std::ops::RangeFrom;
use std::collections::hash_map::Iter as HashMapIter;

use game::player::Player;
use game::map::Location;

#[derive(Debug, Clone)]
pub enum Unit {
    Worker,
    Light,
    Heavy,
}

#[derive(Debug, Clone)]
pub struct Resources(pub u64);

#[derive(Debug, Clone)]
pub enum Building {
    Base(Resources),
    Barracks,
}

#[derive(Debug, Clone)]
pub enum Object<'p> {
    Unit(&'p Player, Unit),
    Building(&'p Player, Building),
    Resource(u64),
}

// Using Copy object to reference entities to allow them to be modified, removed
// etc. while AI are holding this
#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
pub struct EntityID(pub usize);

#[derive(Debug, Clone)]
pub struct Entity<'m, 'p> {
    pub id: EntityID,
    pub location: Location<'m>,
    pub object: Object<'p>,
}

type LocationIndex<'m> = HashMap<Location<'m>, EntityID>;

#[derive(Debug, Clone)]
pub struct Entities<'p, 'm> {
    entities: HashMap<EntityID, Entity<'m, 'p>>,
    entity_id_seq: RangeFrom<usize>,
    location_index: LocationIndex<'m>,
}

#[derive(Debug)]
pub enum EntitiesError<'m> {
    LocationNotWalkable(Location<'m>),
    LocationAlreadyTaken(Location<'m>, EntityID),
}

pub struct EntityMutator<'p: 'e, 'm: 'e, 'e> {
    pub entity: &'e mut Entity<'m, 'p>,
    location_index: &'e mut LocationIndex<'m>,
}

impl<'p: 'e, 'm: 'e, 'e> EntityMutator<'p, 'm, 'e> {
    pub fn set_location(&mut self, location: Location<'m>) -> Result<(), EntitiesError<'m>> {
        // Check if new location is valid for entity to be placed on
        if !location.walkable() {
            return Err(EntitiesError::LocationNotWalkable(location));
        }

        if let Some(entity_id) = self.location_index.get(&location) {
            return Err(EntitiesError::LocationAlreadyTaken(location, *entity_id));
        }

        let entity_location = &mut self.entity.location;

        // Update indexes first
        self.location_index.remove(&entity_location).expect(
            "bad location_index",
        );
        self.location_index.insert(location, self.entity.id);

        // Update entity
        *entity_location = location;

        Ok(())
    }
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
            location: location,
            object: object,
        };

        if self.entities.insert(entity_id, entity).is_some() {
            panic!("duplicate ID");
        }

        self.location_index.insert(location, entity_id);

        Ok(entity_id)
    }

    pub fn get<'e>(&'e self, entity_id: EntityID) -> Option<&'e Entity<'m, 'p>> {
        self.entities.get(&entity_id)
    }

    pub fn get_mutator<'e>(&'e mut self, entity_id: EntityID) -> Option<EntityMutator<'p, 'm, 'e>> {
        let entities = &mut self.entities;
        let location_index = &mut self.location_index;

        entities.get_mut(&entity_id).map(move |e| {
            EntityMutator {
                entity: e,
                location_index: location_index,
            }
        })
    }

    pub fn get_by_location<'e>(&'e self, location: Location<'m>) -> Option<&'e Entity<'m, 'p>> {
        self.location_index.get(&location).and_then(|entity_id| {
            self.get(*entity_id)
        })
    }

    pub fn iter<'e>(&'e self) -> Iter<'p, 'm, 'e> {
        Iter { iter: self.entities.iter() }
    }
}

impl<'p: 'e, 'm: 'e, 'e> IntoIterator for &'e Entities<'p, 'm> {
    type IntoIter = Iter<'p, 'm, 'e>;
    type Item = (EntityID, &'e Entity<'m, 'p>);

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct Iter<'p: 'e, 'm: 'e, 'e> {
    iter: HashMapIter<'e, EntityID, Entity<'m, 'p>>,
}

impl<'p: 'e, 'm: 'e, 'e> Iterator for Iter<'p, 'm, 'e> {
    type Item = (EntityID, &'e Entity<'m, 'p>);

    fn next(&mut self) -> Option<(EntityID, &'e Entity<'m, 'p>)> {
        self.iter.next().map(
            |(entity_id, entity)| (*entity_id, entity),
        )
    }
}
