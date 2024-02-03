use std::fmt::Debug;


/// A component which can have a run-time id: See Component if the component is known at compile time
pub trait DynamicComponent: Debug {
    fn get_id(&self) -> &str;
}

/// A component which is known at compile time
pub trait Component: Debug {
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

impl<T, U, V, W, X> ComponentBundle for (T, U, V, W,X)
where
    T: DynamicComponent + 'static,
    U: DynamicComponent + 'static,
    V: DynamicComponent + 'static,
    W: DynamicComponent + 'static,
    X: DynamicComponent + 'static,
{
    fn into_components(self: Self) -> Vec<Box<dyn DynamicComponent>> {
        vec![
            Box::new(self.0),
            Box::new(self.1),
            Box::new(self.2),
            Box::new(self.3),
            Box::new(self.4),
        ]
    }
}

impl<T, U, V, W, X, Y> ComponentBundle for (T, U, V, W,X,Y)
where
    T: DynamicComponent + 'static,
    U: DynamicComponent + 'static,
    V: DynamicComponent + 'static,
    W: DynamicComponent + 'static,
    X: DynamicComponent + 'static,
    Y: DynamicComponent + 'static,
{
    fn into_components(self: Self) -> Vec<Box<dyn DynamicComponent>> {
        vec![
            Box::new(self.0),
            Box::new(self.1),
            Box::new(self.2),
            Box::new(self.3),
            Box::new(self.4),
            Box::new(self.5),
        ]
    }
}
impl<T, U, V, W, X, Y, Z> ComponentBundle for (T, U, V, W,X,Y, Z)
where
    T: DynamicComponent + 'static,
    U: DynamicComponent + 'static,
    V: DynamicComponent + 'static,
    W: DynamicComponent + 'static,
    X: DynamicComponent + 'static,
    Y: DynamicComponent + 'static,
    Z: DynamicComponent + 'static,
{
    fn into_components(self: Self) -> Vec<Box<dyn DynamicComponent>> {
        vec![
            Box::new(self.0),
            Box::new(self.1),
            Box::new(self.2),
            Box::new(self.3),
            Box::new(self.4),
            Box::new(self.5),
            Box::new(self.6),
        ]
    }
}
