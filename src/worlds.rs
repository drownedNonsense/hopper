//#########################
// D E P E N D E N C I E S
//#########################

    use std::collections::HashMap;
    use std::any::TypeId;
    use std::rc::Rc;
    use std::cell::RefCell;
    use std::ops::Range;
    use std::hash::Hash;
    use std::fmt::{Debug, Display};
    use std::fmt;
    use std::error::Error;

    use crate::components::{Component, ComponentCell, ComponentColumn};
    use crate::entities::{Entity, EntityBuilder, EntityId};
    use crate::queries::QueryBuilder;

    use rusty_toolkit::BitField;


//#######################
// D E F I N I T I O N S
//#######################

    pub struct World<B: BitField, F: BitField, P: Hash + Eq + Debug> {
        components:         Vec<TypeId>,
        flags:              HashMap<F, Range<u8>>,
        component_columns:  HashMap<B, Box<dyn ComponentColumn>>,
        component_pointers: HashMap<P, Box<dyn ComponentCell>>,
        entities:           HashMap<Entity, B>,
        next_entity_id:     EntityId,
    } // struct ..


    pub struct WorldBuilder<B: BitField, F: BitField, P: Hash + Eq + Debug> {
        components:         Vec<TypeId>,
        flags:              HashMap<F, Range<u8>>,
        component_count:    usize,
        component_columns:  HashMap<B, Box<dyn ComponentColumn>>,
        component_pointers: HashMap<P, Box<dyn ComponentCell>>,
    } // struct ..


    #[derive(Debug)]
    pub enum EcsErr<B: BitField, F: BitField, P> {
        MissingEntity(Entity),
        MissingComponent(TypeId),
        MissingComponentPtr(P),
        MissingFlag(F),
        MissingComponentToEntity(TypeId, Entity),
        FailedToDowncastComponentCol(B),
        FailedToDowncastPtr(P),
        BitFieldRangeTooSmall(usize, usize),
    } // enum ..
    

//###############################
// I M P L E M E N T A T I O N S
//###############################

    impl<B: BitField + Debug, F: BitField + Debug, P: Debug> Error for EcsErr<B, F, P> {}
    impl<B: BitField, F: BitField + Debug, P: Debug> Display for EcsErr<B, F, P> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", match self {
                EcsErr::MissingEntity(e)                => format!("The entity `{:?}` is not registered!", e),
                EcsErr::MissingComponent(c)             => format!("The component `{:?}` is not registered!", c),
                EcsErr::MissingComponentPtr(p)          => format!("The component pointer `{:?}` is not registered!", p),
                EcsErr::MissingFlag(b)                  => format!("The flag `{:x}` is not registerd!", b),
                EcsErr::FailedToDowncastComponentCol(b) => format!("Failed to downcast the `{:x}` component column!", b),
                EcsErr::FailedToDowncastPtr(p)          => format!("Failed to downcast the `{:?}` component pointer!", p),
                EcsErr::MissingComponentToEntity(c, e)  => format!("The entity `{:?}` has no registered component `{:?}`!", e, c),
                EcsErr::BitFieldRangeTooSmall(s, r)     => format!("The bitfield's range is too low: `{} > {}`!", s, r),
            }) // write()
        } // fn ..
    } // impl ..


    impl<B: BitField, F: BitField, P: Hash + Eq + Debug> World<B, F, P> {
        pub fn builder() -> WorldBuilder<B, F, P> {
            WorldBuilder {
                components:         Vec::default(),
                flags:              HashMap::default(),
                component_count:    0usize,
                component_columns:  HashMap::default(),
                component_pointers: HashMap::default(),
            } // WorldBuilder
        } // fn ..


        fn get_mut_raw_component_column<C: Component>(&mut self) -> Result<&mut Box<dyn ComponentColumn>, EcsErr<B, F, P>> {

            let bit_mask = self.component_bit_mask::<C>()?;
            match self.component_columns.get_mut(&bit_mask) {
                None                   => Err(EcsErr::MissingComponent(TypeId::of::<C>())),
                Some(component_column) => Ok(component_column),
            } // match ..
        } // fn ..


        pub(crate) fn get_component_column<C: Component>(&self) -> Result<&HashMap<Entity, Rc<RefCell<C>>>, EcsErr<B, F, P>> {

            let bit_mask = self.component_bit_mask::<C>()?;
            match self.component_columns.get(&bit_mask) {
                None                   => Err(EcsErr::MissingComponent(TypeId::of::<C>())),
                Some(component_column) => match component_column
                    .as_any()
                    .downcast_ref::<HashMap<Entity, Rc<RefCell<C>>>>() {

                        None                    => Err(EcsErr::FailedToDowncastComponentCol(bit_mask)),
                        Some(downcasted_column) => Ok(downcasted_column),

                    } // match ..
                } // match ..
        } // fn ..


        fn get_mut_component_column<C: Component>(&mut self) -> Result<&mut HashMap<Entity, Rc<RefCell<C>>>, EcsErr<B, F, P>> {

            let bit_mask = self.component_bit_mask::<C>()?;
            match self.component_columns.get_mut(&bit_mask) {
                None                   => Err(EcsErr::MissingComponent(TypeId::of::<C>())),
                Some(component_column) => match component_column
                    .as_any_mut()
                    .downcast_mut::<HashMap<Entity, Rc<RefCell<C>>>>() {

                        None                    => Err(EcsErr::FailedToDowncastComponentCol(bit_mask)),
                        Some(downcasted_column) => Ok(downcasted_column),

                    } // match ..
                } // match ..
        } // fn ..


        pub fn get_pointer_component<C: Component>(
            &self,
            id: P,
        ) -> Result<&Rc<RefCell<C>>, EcsErr<B, F, P>> {
            match self.component_pointers.get(&id) {
                None          => Err(EcsErr::MissingComponentPtr(id)),
                Some(raw_ptr) => match raw_ptr.as_any().downcast_ref::<Rc<RefCell<C>>>() {
                    Some(ptr) => Ok(ptr),
                    None      => Err(EcsErr::FailedToDowncastPtr(id))
                } // match ..
            } // match ..
        } // fn ..


        pub(crate) fn get_entities(&self, bit_mask_filter: B) -> Vec<Entity> {
            self.entities
                .iter()
                .filter(|(_, bit_mask)| bit_mask.has_bit_mask(bit_mask_filter))
                .map(|(entity, _)| *entity)
                .collect()
        } // fn ..


        pub(crate) fn component_bit_mask<C: Component>(&self) -> Result<B, EcsErr<B, F, P>> {

            let type_id = TypeId::of::<C>();
            match self.components
                .iter()
                .enumerate()
                .find_map(|(index, id)| {
                    match id == &type_id {
                        true  => Some(B::bit(index as u8)),
                        false => None,
                    } // match ..
                }) {
                    Some(bit_mask) => Ok(bit_mask),
                    None           => Err(EcsErr::MissingComponent(type_id))
                } // match ..
        } // fn ..


        pub(crate) fn flag_bit_mask(
            &self,
            flag:    F,
            variant: Option<B>,
        ) -> Result<B, EcsErr<B, F, P>> {

            match self.flags
                .iter()
                .find_map(|(id, range)| {
                    return match id == &flag {
                        true => Some(
                                match variant {
                                Some(variant) => ((variant << range.start) & B::bit_mask(range.clone())) << self.components.len() as u8,
                                None          => B::bit_mask(range.clone()),
                            } // match ..
                        ), // => ..
                        false => None,
                    } // return ..
                }) {
                    Some(bit_mask) => Ok(bit_mask),
                    None           => Err(EcsErr::MissingFlag(flag)),
                } // match ..
        } // fn ..


        fn get_entity_bit_mask(&self, entity: Entity) -> Result<B, EcsErr<B, F, P>> {
            match self.entities.get(&entity) {
                Some(bit_mask) => Ok(*bit_mask),
                None           => Err(EcsErr::MissingEntity(entity)),
            } // match ..
        } // fn ..


        fn get_mut_entity_bit_mask(&mut self, entity: Entity) -> Result<&mut B, EcsErr<B, F, P>> {
            match self.entities.get_mut(&entity) {
                Some(bit_mask) => Ok(bit_mask),
                None           => Err(EcsErr::MissingEntity(entity)),
            } // match ..
        } // fn ..


        pub(crate) fn add_component_to_entity_builder<C: Component>(
            &mut self,
            component:       C,
            entity:          Entity,
            entity_bit_mask: &mut B,
        ) -> Result<(), EcsErr<B, F, P>> {

            entity_bit_mask.set_bit_mask(self.component_bit_mask::<C>()?);

            self.get_mut_component_column::<C>()?.insert(entity, Rc::new(RefCell::new(component)));
            Ok(())

        } // fn ..


        pub(crate) fn add_shared_component_to_entity_builder<C: Component>(
            &mut self,
            component:       &Rc<RefCell<C>>,
            entity:          Entity,
            entity_bit_mask: &mut B,
        ) -> Result<(), EcsErr<B, F, P>> {
            
            entity_bit_mask.set_bit_mask(self.component_bit_mask::<C>()?);

            self.get_mut_component_column::<C>()?.insert(entity, component.clone());
            Ok(())

        } // fn ..


        pub(crate) fn add_flag_to_entity_builder(
            &mut self,
            flag:            F,
            variant:         Option<B>,
            entity_bit_mask: &mut B,
        ) -> Result<(), EcsErr<B, F, P>> {

            entity_bit_mask.set_bit_mask(self.flag_bit_mask(flag, variant)?);
            Ok(())

        } // fn ..


        pub fn entity_has_component<C: Component>(
            &self,
            entity: Entity,
        ) -> Result<bool, EcsErr<B, F, P>> {
            Ok(self.get_entity_bit_mask(entity)?.has_bit_mask(self.component_bit_mask::<C>()?))
        } // fn ..


        pub fn entity_group_has_component<C: Component>(
            &self,
            entity_group: &[Entity],
        ) -> Result<Vec<bool>, EcsErr<B, F, P>> {

            let bit_mask = self.component_bit_mask::<C>()?;
            entity_group.iter()
                .map(|entity| match self.get_entity_bit_mask(*entity) {
                    Ok(entity_bit_mask) => Ok(entity_bit_mask.has_bit_mask(bit_mask)),
                    Err(err)            => Err(err),
                }).collect()

        } // fn ..


        pub fn entity_has_flag(
            &self,
            entity:  Entity,
            flag:    F,
            variant: Option<B>,
        ) ->Result<bool, EcsErr<B, F, P>> {
            Ok(self.get_entity_bit_mask(entity)?.has_bit_mask(self.flag_bit_mask(flag, variant)?))
        } // fn ..


        pub fn entity_group_has_flag(
            &self,
            entity_group: &[Entity],
            flag:         F,
            variant:      Option<B>,
        ) -> Result<Vec<bool>, EcsErr<B, F, P>> {

            let bit_mask = self.flag_bit_mask(flag, variant)?;
            entity_group.iter()
                .map(|entity| match self.get_entity_bit_mask(*entity) {
                    Ok(entity_bit_mask) => Ok(entity_bit_mask.has_bit_mask(bit_mask)),
                    Err(err)            => Err(err),
                }).collect()

        } // fn ..


        pub fn add_component_to_entity<C: Component>(
            &mut self,
            component: C,
            entity:    Entity,
        ) -> Result<(), EcsErr<B, F, P>> {

            self.get_entity_bit_mask(entity)?.set_bit_mask(self.component_bit_mask::<C>()?);
            self.get_mut_component_column::<C>()?.insert(entity, Rc::new(RefCell::new(component)));

            Ok(())

        } // fn ..


        pub fn add_component_to_entity_group<C: Component>(
            &mut self,
            component:    C,
            entity_group: &[Entity],
        ) -> Result<(), EcsErr<B, F, P>> {

            let bit_mask             = self.component_bit_mask::<C>()?;
            let mut component_column = self.get_mut_component_column::<C>()?.clone();

            entity_group
                .iter()
                .map(|entity| {

                    component_column.insert(*entity, Rc::new(RefCell::new(component.clone())));
                    
                    match self.get_mut_entity_bit_mask(*entity) {
                        Ok(entity_bit_mask) => { entity_bit_mask.set_bit_mask(bit_mask); Ok(()) },
                        Err(err)            => Err(err),
                    } // match ..
                }).collect::<Result<_, EcsErr<B, F, P>>>()?; // map()


            Ok(())

        } // fn ..


        pub fn add_shared_component_to_entity<C: Component>(
            &mut self,
            component: &Rc<RefCell<C>>,
            entity:    Entity,
        ) -> Result<(), EcsErr<B, F, P>> {

            self.get_entity_bit_mask(entity)?.set_bit_mask(self.component_bit_mask::<C>()?);
            self.get_mut_component_column::<C>()?.insert(entity, component.clone());

            Ok(())

        } // fn ..


        pub fn add_shared_component_to_entity_group<C: Component>(
            &mut self,
            component:    &Rc<RefCell<C>>,
            entity_group: &[Entity],
        ) -> Result<(), EcsErr<B, F, P>> {

            let bit_mask             = self.component_bit_mask::<C>()?;
            let mut component_column = self.get_mut_component_column::<C>()?.clone();

            entity_group.iter()
                .map(|entity| {

                    component_column.insert(*entity, component.clone());
                    
                    match self.get_mut_entity_bit_mask(*entity) {
                        Ok(entity_bit_mask) => { entity_bit_mask.set_bit_mask(bit_mask); Ok(()) },
                        Err(err)            => Err(err),
                    } // match ..
                }).collect::<Result<_, EcsErr<B, F, P>>>()?; // map()


            Ok(())

        } // fn ..


        pub fn get_entity_component<C: Component>(
            &self,
            entity: Entity,
        ) -> Result<Option<&Rc<RefCell<C>>>, EcsErr<B, F, P>> {
            Ok(self.get_component_column::<C>()?.get(&entity))
        } // fn ..


        pub fn get_some_entity_group_component<C: Component>(
            &self,
            entity_group: &[Entity],
        ) -> Result<Vec<Option<&Rc<RefCell<C>>>>, EcsErr<B, F, P>> {

            let component_column = self.get_component_column::<C>()?;

            Ok(entity_group
                .iter()
                .map(|entity| component_column.get(entity))
                .collect())

        } // fn ..


        pub fn get_entity_group_component<C: Component>(
            &self,
            entity_group: &[Entity],
        ) -> Result<Vec<&Rc<RefCell<C>>>, EcsErr<B, F, P>> {

            let component_column = self.get_component_column::<C>()?;

            entity_group
                .iter()
                .map(|entity| match component_column.get(entity) {
                    Some(component) => Ok(component),
                    None            => Err(EcsErr::MissingComponentToEntity(TypeId::of::<C>(), *entity))
                }).collect()
        } // fn ..


        pub fn delete_entity_component<C: Component>(
            &mut self,
            entity: Entity,
        ) -> Result<(), EcsErr<B, F, P>> {

            let bit_mask = self.component_bit_mask::<C>()?;
            self.get_mut_entity_bit_mask(entity)?.unset_bit_mask(bit_mask);
            self.get_mut_raw_component_column::<C>()?.remove_entity(entity);

            Ok(())

        } // fn ..


        pub fn delete_entity_group_component<C: Component>(
            &mut self,
            entity_group: &[Entity],
        ) -> Result<(), EcsErr<B, F, P>> {

            let bit_mask = self.component_bit_mask::<C>()?;

            entity_group
                .iter()
                .map(|entity| {

                    self.get_mut_raw_component_column::<C>()?.remove_entity(*entity);
                    match self.get_mut_entity_bit_mask(*entity) {
                        Ok(entity_bit_mask) => { entity_bit_mask.unset_bit_mask(bit_mask); Ok(()) },
                        Err(err)            => Err(err),
                    } // match ..
                }).collect::<Result<_, EcsErr<B, F, P>>>()?;

            Ok(())

        } // fn ..


        pub fn set_entity_flag(
            &mut self,
            entity:  Entity,
            flag:    F,
            variant: Option<B>,
        ) -> Result<(), EcsErr<B, F, P>> {

            let bit_mask = self.flag_bit_mask(flag, variant)?;
            self.get_mut_entity_bit_mask(entity)?.set_bit_mask(bit_mask);
            Ok(())

        } // fn ..


        pub fn set_entity_group_flag(
            &mut self,
            entity_group: &[Entity],
            flag:         F,
            variant:      Option<B>,
        ) -> Result<(), EcsErr<B, F, P>> {

            let bit_mask = self.flag_bit_mask(flag, variant)?;

            entity_group
                .iter()
                .map(|entity|
                    match self.get_mut_entity_bit_mask(*entity) {
                        Ok(entity_bit_mask) => { entity_bit_mask.set_bit_mask(bit_mask); Ok(()) },
                        Err(err)            => Err(err),
                    } // match ..
                ).collect::<Result<_, EcsErr<B, F, P>>>()?;

            Ok(())

        } // fn ..


        pub fn remove_entity_flag(
            &mut self,
            entity:  Entity,
            flag:    F,
            variant: Option<B>,
        ) -> Result<(), EcsErr<B, F, P>> {

            let bit_mask = self.flag_bit_mask(flag, variant)?;
            self.get_mut_entity_bit_mask(entity)?.unset_bit_mask(bit_mask);
            Ok(())

        } // fn ..


        pub fn remove_entity_group_flag(
            &mut self,
            entity_group: &[Entity],
            flag:         F,
            variant:      Option<B>,
        ) -> Result<(), EcsErr<B, F, P>> {

            let bit_mask = self.flag_bit_mask(flag, variant)?;

            entity_group
                .iter()
                .map(|entity|
                    match self.get_mut_entity_bit_mask(*entity) {
                        Ok(entity_bit_mask) => { entity_bit_mask.unset_bit_mask(bit_mask); Ok(()) },
                        Err(err)            => Err(err),
                    } // match ..
                ).collect::<Result<_, EcsErr<B, F, P>>>()?;

            Ok(())

        } // fn ..


        pub(crate) fn add_entity(
            &mut self,
            entity:          Entity,
            entity_bit_mask: B,
        ) { self.entities.insert(entity, entity_bit_mask); }



        pub fn delete_entity(&mut self, entity: Entity) -> Result<(), EcsErr<B, F, P>> {

            let entity_bit_mask = self.get_entity_bit_mask(entity)?;
            self.component_columns
                .iter_mut()
                .map(|(bit_mask, component_column)| if entity_bit_mask.has_bit_mask(*bit_mask) {

                        component_column.remove_entity(entity);
                        Ok(())

                    } else { Err(EcsErr::MissingEntity(entity)) }
                ).collect::<Result<_, EcsErr<B, F, P>>>()?;
            
            self.entities.remove(&entity);

            Ok(())
            
        } // fn ..


        pub fn delete_entity_group(&mut self, entity_group: &[Entity]) -> Result<(), EcsErr<B, F, P>> {
            entity_group.iter().map(|entity| self.delete_entity(*entity)).collect::<Result<_, EcsErr<B, F, P>>>()?;
            Ok(())
        } // fn ..


        pub fn new_entity(&mut self) -> EntityBuilder<B, F, P> {

            self.next_entity_id += 1;
            EntityBuilder::new(self.next_entity_id - 1, self)

        } // fn ..


        pub const fn new_query(&self) -> QueryBuilder<B, F, P> { QueryBuilder { bit_mask: B::MIN, world: self }}

    } // impl ..


    impl<B: BitField, F: BitField, P: Hash + Eq + Debug> WorldBuilder<B, F, P> {
        pub fn with_component_pointer<C: Component, T: Into<P>>(mut self, id: T, component: C) -> Self {

            let id = id.into();
            match self.component_pointers.contains_key(&id) {
                true =>  { println!("The component pointer {:?} has been discarded as it was already registered!", id) },
                false => { self.component_pointers.insert(id, Box::new(Rc::new(RefCell::new(component)))); },
            } // match ..

            self

        } // fn ..


        pub fn with_shared_component_pointer<C: Component, T: Into<P>>(mut self, id: T, component: &Rc<RefCell<C>>) -> Self {

            let id = id.into();
            match self.component_pointers.contains_key(&id) {
                true =>  { println!("The component pointer {:?} has been discarded as it was already registered!", id) },
                false => { self.component_pointers.insert(id, Box::new(component.clone())); },
            } // match ..

            self
            
        } // fn ..


        pub fn with_component<C: Component>(mut self) -> Self {

            match self.components.contains(&TypeId::of::<C>()) {
                true =>  { println!("The component no.{} has been discarded as it was already registered!", self.component_count ) },
                false => {
                    self.components.push(TypeId::of::<C>());
                    self.component_columns.insert(
                        B::bit(self.component_count as u8),
                        Box::new(HashMap::<Entity, Rc<RefCell<C>>>::new()
                    )); // insert()
                    self.component_count += 1;
                }, // => ..
            } // match ..

            self

        } // fn ..


        pub fn with_flag<T: Into<F>>(mut self, flag: T, range: Range<u8>) -> Self {

            self.flags.insert(flag.into(), range);
            self

        } // fn ..


        pub fn build(self) -> Result<World<B, F, P>, EcsErr<B, F, P>> {

            let range = usize::from(B::BITS);
            let size  = self.component_count + usize::from(
                match self.flags.iter().max_by(|(_, a), (_, b)| a.start.cmp(&b.end)) {
                    Some(max) => max.1.end,
                    None      => 0u8,
                } // match ..
            ); // let ..


            match size > range {
                true  => Err(EcsErr::BitFieldRangeTooSmall(size, range)),
                false => Ok(World {
                    components:         self.components,
                    flags:              self.flags,
                    component_columns:  self.component_columns,
                    component_pointers: self.component_pointers,
                    entities:           HashMap::default(),
                    next_entity_id:     0usize,
                }) // => ..
            } // match ..
        } // fn ..
    } // impl ..
