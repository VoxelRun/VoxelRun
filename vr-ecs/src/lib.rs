use std::{collections::HashMap, fmt::Debug, fmt::Display};

/// A world which holds all entities. An entity is valid per world
pub struct World {
    tables: Vec<Table>,
    entities: Vec<EntityTableData>,
    removed: Vec<usize>,
    component_lookup: ComponentLookup,
}

impl Display for World {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "Total entity count: {}",
            self.entities.len() - self.removed.len()
        )?;
        writeln!(f, "Removed indeces: {:?}", self.removed)?;
        writeln!(
            f,
            "Entity Table Data: {}",
            self.entities
                .iter()
                .enumerate()
                .filter(|(index, _)| !self.removed.contains(index))
                .map(|(_, it)| format!("{:?}", it))
                .fold(String::new(), |mut accumulator, it| {
                    accumulator.push_str("\n");
                    accumulator.push_str(&it);
                    accumulator
                })
        )?;
        writeln!(
            f,
            "Tables:{}",
            self.tables.iter().map(|it| format!("{:?}", it)).fold(
                String::new(),
                |mut accumulator, it| {
                    accumulator.push_str("\n");
                    accumulator.push_str("\n");
                    accumulator.push_str(&it);
                    accumulator
                }
            )
        )
    }
}

impl World {
    pub fn new() -> World {
        World {
            tables: vec![],
            entities: vec![],
            removed: vec![],
            component_lookup: ComponentLookup::new(),
        }
    }

    pub fn is_valid(&self, entity: &Entity) -> bool {
        self.get_valid_entity_table_data(entity).is_some()
    }

    pub fn spawn_entity<T: ComponentBundle>(&mut self, components: T) -> Entity {
        self.spawn_entity_from_vec(components.into_components())
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

    fn spawn_entity_from_vec(&mut self, components: Vec<Box<dyn DynamicComponent>>) -> Entity {
        let index = self.removed.pop().unwrap_or(self.entities.len());
        let generation = self
            .entities
            .get(index)
            .map(|it| it.generation)
            .unwrap_or(0);
        let entity = Entity { index, generation };

        let mut component_set = self.get_component_set(&components);
        component_set.sort();
        let (table_index, table) = unsafe { self.get_or_create_table(component_set.clone()) };

        let inner_table_index = unsafe { table.add(entity, &component_set, components) };
        if let Some(t) = self.entities.get_mut(index) {
            *t = EntityTableData {
                generation,
                table_index,
                inner_table_index,
                component_set,
            }
        } else {
            self.entities.push(EntityTableData {
                generation,
                table_index,
                inner_table_index,
                component_set,
            });
        }
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

    fn get_component_set(&mut self, components: &[Box<dyn DynamicComponent>]) -> Vec<usize> {
        components
            .iter()
            .map(|component| {
                self.component_lookup
                    .get_or_create(component.get_id().to_string())
            })
            .collect()
    }

    /// unsafe: components must be sorted
    unsafe fn get_or_create_table(&mut self, components: Vec<usize>) -> (usize, &mut Table) {
        let component_set = components;
        // validate uniqueness
        let mut last = None;
        for key in &component_set {
            if let Some(k) = last {
                if k == key {
                    panic!("Component contained multiple times in one bundle");
                }
            }
            last = Some(key);
        }

        // Find table with matching components or create and add a new one
        match self
            .tables
            .iter()
            .enumerate()
            .find(|(_, table)| table.component_set == component_set)
            .map(|(it, _)| it)
        {
            Some(it) => (it, &mut self.tables[it]),
            None => {
                let size = component_set.last().map(|it| it + 1).unwrap_or(0);
                let mut collumns = vec![];
                for _ in 0..size {
                    collumns.push(None);
                }
                for collumn in &component_set {
                    collumns[*collumn] = Some(vec![])
                }
                self.tables.push(Table {
                    component_set,
                    collumns,
                    entities: vec![],
                    removed: vec![],
                });
                (self.tables.len() - 1, self.tables.last_mut().unwrap())
            }
        }
    }
}

/// All entities in a table have exactly the same component
#[derive(Debug)]
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
        }
        self.entities[index] = None;
        self.removed.push(index);
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
#[derive(Debug)]
struct EntityTableData {
    generation: u32,
    table_index: usize,
    inner_table_index: usize,
    component_set: Vec<usize>,
}
struct ComponentLookup {
    next_id: usize,
    component_id_to_index: HashMap<String, usize>,
}

impl ComponentLookup {
    fn get_or_create(&mut self, id: String) -> usize {
        *self.component_id_to_index.entry(id).or_insert_with(|| {
            let current_id = self.next_id;
            self.next_id += 1;
            current_id
        })
    }

    fn new() -> ComponentLookup {
        ComponentLookup {
            next_id: 0,
            component_id_to_index: HashMap::new(),
        }
    }
}

/// An entity. Works similar to a reference
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Entity {
    index: usize,
    generation: u32,
}

/// A component which can have a run-time id: See Component if the component is known at compile time
pub trait DynamicComponent: Debug {
    fn get_id(&self) -> &str;
}

/// A component which is known at compile time
trait Component: Debug {
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
