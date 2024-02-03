mod component;
mod world;
pub use component::{Component, ComponentBundle, DynamicComponent};
pub use world::World;

/// An entity. Works similar to a reference
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Entity {
    index: usize,
    generation: u32,
}

#[cfg(test)]
mod tests {
    use proc_macros::Component;

    use super::*;

    #[derive(Component, Debug)]
    struct Position(u8, u8, u8);

    #[derive(Component, Debug)]
    struct Velocity(u8, u8, u8);

    #[test]
    fn cycle_test() {
        let mut world = World::new();
        let e = world.spawn_entity(Position(0, 0, 0));
        world.spawn_entity(Position(0, 0, 0));
        world.remove(&e);
        world.spawn_entity(Position(0, 0, 0));
        world.spawn_entity(Position(0, 0, 0));
        let e = world.spawn_entity(Velocity(0, 0, 0));
        world.spawn_entity(Velocity(0, 0, 0));
        world.remove(&e);
        world.spawn_entity(Velocity(0, 0, 0));
        world.spawn_entity((Velocity(0, 0, 0), Position(0, 0, 0)));
        world.spawn_entity((Velocity(0, 0, 0), Position(0, 0, 0)));
        world.spawn_entity((Velocity(0, 0, 0), Position(0, 0, 0)));
        world.spawn_entity((Position(0, 0, 0), Velocity(0, 0, 0)));
        world.spawn_entity((Position(0, 0, 0), Velocity(0, 0, 0)));
        let e = world.spawn_entity((Position(0, 0, 0), Velocity(0, 0, 0)));
        world.remove(&e);
        println!("{}", world);
        println!("{}", world.is_valid(&e));
    }
}
