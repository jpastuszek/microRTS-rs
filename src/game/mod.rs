mod terrain;
mod entity;
mod player;
mod game_state;

// Flat structure for AI
pub use game::terrain::{Terrain, TerrainBuilder, TerrainBuilderError, Dimension, Direction, Coordinates, Location, Tile};
pub use game::entity::{Entity, Object, Iter as EntitiesIter, EntitiesError, Unit, Building,
                       Resource, Entities, EntityID};
pub use game::player::{Player, Colour, AI, EmptyPersistentState, Owned};
pub use game::game_state::{GameState, GameStateBuilder, Order};