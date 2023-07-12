//#########################
// D E P E N D E N C I E S
//#########################

    use std::collections::HashMap;
    use std::any::Any;
    use std::rc::Rc;
    use std::cell::RefCell;

    use crate::entities::Entity;


//#######################
// D E F I N I T I O N S
//#######################

    pub(crate) trait ComponentCell {
        fn as_any(&self) -> &dyn Any;
    } // trait ComponentCell


    pub(crate) trait ComponentColumn: Any {
        fn as_any(&self)         -> &dyn Any;
        fn as_any_mut(&mut self) -> &mut dyn Any;
        fn remove_entity(&mut self, entity: Entity);
    } // trait ComponentColumn


    pub trait Component: Any + Clone {}


//###############################
// I M P L E M E N T A T I O N S
//###############################

    impl<C: 'static + Component> ComponentCell for Rc<RefCell<C>> {
        fn as_any(&self) -> &dyn Any { self }
    } // impl ComponentCell ..


    impl<C: 'static + Component> ComponentColumn for HashMap<Entity, Rc<RefCell<C>>> {
        fn as_any(&self)         -> &dyn Any        { self }
        fn as_any_mut(&mut self) -> &mut dyn Any    { self }
        fn remove_entity(&mut self, entity: Entity) { self.remove(&entity); }
    } // impl ComponentColumn ..
