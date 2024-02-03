/// A world which holds all entities. An entity is valid per world
pub struct World {
    tables: Vec<Table>,
    entities: Vec<EntityTableData>,
    removed: Vec<usize>,
}

impl World {
    pub fn new() -> World {
        World {
            tables: vec![],
            entities: vec![],
            removed: vec![],
        }
    }

    pub fn is_valid(&self, entity: &Entity) -> bool {
        self.get_valid_entity_table_data(entity).is_some()
    }

    pub fn add_entity<T: ComponentBundle>(&mut self, components: T) -> Entity {
        self.add_entity_from_vec(components.into_components())
    }

    pub fn remove(&mut self, entity: &Entity) {
        if let Some(entity_table_data) = self.get_valid_entity_table_data_mut(entity) {
            // Invalidate all references
            entity_table_data.generation += 1;

            let table_index = entity_table_data.table_index;
            let inner_table_index = entity_table_data.inner_table_index;

            // remove from table
            self.tables
                .get_mut(table_index)
                .unwrap()
                .remove(inner_table_index);

            // mark as free
            self.removed.push(entity.index);
        }
    }

    fn add_entity_from_vec(&mut self, components: Vec<Box<dyn DynamicComponent>>) -> Entity {
        let index = self.removed.pop().unwrap_or(self.entities.len());
        let generation = self
            .entities
            .get(index)
            .map(|it| it.generation)
            .unwrap_or(0);
        let entity = Entity { index, generation };

        let component_set = self.get_component_set(&components);
        let (table, table_index) = self.get_or_create_table(&component_set);

        let inner_table_index = unsafe { table.add(entity, &component_set, components) };
        self.entities[index] = EntityTableData {
            generation,
            table_index,
            inner_table_index,
            component_set,
        };
        entity
    }

    fn get_valid_entity_table_data(&self, entity: &Entity) -> Option<&EntityTableData> {
        match self.entities.get(entity.index).as_ref() {
            Some(entity_table_data) if entity.generation == entity_table_data.generation => {
                Some(entity_table_data)
            }
            _ => None,
        }
    }

    fn get_valid_entity_table_data_mut(&mut self, entity: &Entity) -> Option<&mut EntityTableData> {
        match self.entities.get_mut(entity.index) {
            Some(entity_table_data) if entity.generation == entity_table_data.generation => {
                Some(entity_table_data)
            }
            _ => None,
        }
    }

    fn get_component_set(&self, components: &[Box<dyn DynamicComponent>]) -> Vec<usize> {
        todo!()
    }

    fn get_or_create_table(&self, components: &[usize]) -> (&mut Table, usize) {
        todo!()
    }
}

/// All entities in a table have exactly the same component
struct Table {
    component_set: Vec<usize>,
    collumns: Vec<Option<Vec<Option<Box<dyn DynamicComponent>>>>>,
    entities: Vec<Option<Entity>>,
    removed: Vec<usize>,
}

impl Table {
    fn remove(&mut self, index: usize) {
        for component_id in &self.component_set {
            self.collumns[component_id.clone()].as_mut().unwrap()[index] = None;
            self.entities[index] = None;
            self.removed.push(index);
        }
    }

    /// unsafe: Assumes components and component_set have the same length, and that sorted(component_set) == self.component_set
    unsafe fn add(
        &mut self,
        entity: Entity,
        component_set: &[usize],
        components: Vec<Box<dyn DynamicComponent>>,
    ) -> usize {
        match self.removed.pop() {
            Some(index) => {
                for (component_set_index, component) in components.into_iter().enumerate() {
                    self.collumns[component_set[component_set_index]]
                        .as_mut()
                        .unwrap()[index] = Some(component)
                }
                self.entities[index] = Some(entity);
                index
            }
            None => {
                for (component_set_index, component) in components.into_iter().enumerate() {
                    self.collumns[component_set[component_set_index]]
                        .as_mut()
                        .unwrap()
                        .push(Some(component))
                }
                self.entities.push(Some(entity));
                self.entities.len() - 1
            }
        }
    }
}

/// The data necessarry to find the right row in the right table, as well as to validate entities
struct EntityTableData {
    generation: u32,
    table_index: usize,
    inner_table_index: usize,
    component_set: Vec<usize>,
}

/// An entity. Works similar to a reference
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Entity {
    index: usize,
    generation: u32,
}

/// A component which can have a run-time id: See Component if the component is known at compile time
pub trait DynamicComponent {
    fn get_id(&self) -> &str;
}

/// A component which is known at compile time
trait Component {
    /// If the id is equal, it can be cast to this component. Must be save
    fn get_id() -> &'static str;
}

impl<T> DynamicComponent for T
where
    T: Component,
{
    fn get_id(&self) -> &str {
        T::get_id()
    }
}

/// A bundle of components. Used when adding a new entity
pub trait ComponentBundle {
    fn into_components(self: Self) -> Vec<Box<dyn DynamicComponent + 'static>>;
}
impl<T: DynamicComponent + 'static> ComponentBundle for T {
    fn into_components(self) -> Vec<Box<dyn DynamicComponent>> {
        vec![Box::new(self)]
    }
}

impl<T, U> ComponentBundle for (T, U)
where
    T: DynamicComponent + 'static,
    U: DynamicComponent + 'static,
{
    fn into_components(self: Self) -> Vec<Box<dyn DynamicComponent>> {
        vec![Box::new(self.0), Box::new(self.1)]
    }
}

impl<T, U, V> ComponentBundle for (T, U, V)
where
    T: DynamicComponent + 'static,
    U: DynamicComponent + 'static,
    V: DynamicComponent + 'static,
{
    fn into_components(self: Self) -> Vec<Box<dyn DynamicComponent>> {
        vec![Box::new(self.0), Box::new(self.1), Box::new(self.2)]
    }
}

impl<T, U, V, W> ComponentBundle for (T, U, V, W)
where
    T: DynamicComponent + 'static,
    U: DynamicComponent + 'static,
    V: DynamicComponent + 'static,
    W: DynamicComponent + 'static,
{
    fn into_components(self: Self) -> Vec<Box<dyn DynamicComponent>> {
        vec![
            Box::new(self.0),
            Box::new(self.1),
            Box::new(self.2),
            Box::new(self.3),
        ]
    }
}

#[cfg(test)]
mod tests {
    use proc_macros::Component;

    use super::*;

    #[derive(Component)]
    struct Position(u8, u8, u8);

    #[derive(Component)]
    struct Velocity(u8, u8, u8);

    #[test]
    fn cycle_test() {
        let mut world = World::new();
        println!("{}", Position(0, 0, 0).get_id());
        let e = world.add_entity(Position(0, 0, 0));
        world.add_entity(Position(0, 0, 0));
        world.remove(&e);
        world.add_entity(Position(0, 0, 0));
        world.add_entity(Position(0, 0, 0));
        let e = world.add_entity(Velocity(0, 0, 0));
        world.add_entity(Velocity(0, 0, 0));
        world.remove(&e);
        world.add_entity(Velocity(0, 0, 0));
        world.add_entity((Velocity(0, 0, 0), Position(0, 0, 0)));
        world.add_entity((Velocity(0, 0, 0), Position(0, 0, 0)));
        world.add_entity((Velocity(0, 0, 0), Position(0, 0, 0)));
        world.add_entity((Position(0, 0, 0), Velocity(0, 0, 0)));
        world.add_entity((Position(0, 0, 0), Velocity(0, 0, 0)));
        let e = world.add_entity((Position(0, 0, 0), Velocity(0, 0, 0)));
        world.remove(&e);
        println!("{}", world.is_valid(&e));
    }
}
