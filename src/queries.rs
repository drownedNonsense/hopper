//#########################
// D E P E N D E N C I E S
//#########################

    use std::fmt::Debug;
    use std::rc::Rc;
    use std::cell::RefCell;
    use std::hash::Hash;

    use crate::worlds::{World, EcsErr};
    use crate::components::Component;
    use crate::entities::Entity;

    use rusty_toolkit::BitField;


//#######################
// D E F I N I T I O N S
//#######################

    pub struct Query<'world, B: BitField, F: BitField, P: Hash + Eq + Debug> {
        entities: Vec<Entity>,
        world:    &'world World<B, F, P>,
    } // struct ..


    pub struct QueryBuilder<'world, B: BitField, F: BitField, P: Hash + Eq + Debug> {
        pub(crate) bit_mask: B,
        pub(crate) world:    &'world World<B, F, P>,
    } // struct ..


//###############################
// I M P L E M E N T A T I O N S
//###############################

    impl<'world, B: BitField, F: BitField, P: Hash + Eq + Debug> Query<'world, B, F, P> {
        pub fn get_components<C: Component>(&self) -> Result<Vec<&Rc<RefCell<C>>>, EcsErr<B, F, P>> {
            self.world.get_entity_group_component(&self.entities)
        } // fn ..


        pub fn get_entities(&self) -> Vec<Entity> { self.entities.clone() }

    } // impl ..


    impl<'world, B: BitField, F: BitField, P: Hash + Eq + Debug> QueryBuilder<'world, B, F, P> {
        pub fn with_component<C: Component>(mut self) -> Result<Self, EcsErr<B, F, P>> {

            self.bit_mask |= self.world.component_bit_mask::<C>()?;
            Ok(self)

        } // fn ..


        pub fn with_flag<T: Into<F>>(mut self, flag: T, variant: Option<B>) -> Result<Self, EcsErr<B, F, P>> {

            self.bit_mask |= self.world.flag_bit_mask(flag.into(), variant)?;
            Ok(self)
            
        } // fn ..


        pub fn build(self) -> Query<'world, B, F, P> {

            let entities = self.world.get_entities(self.bit_mask);

            Query {
                entities,
                world: self.world,
            } // Query
        } // fn ..
    } // impl ..
