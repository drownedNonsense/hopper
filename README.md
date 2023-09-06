# hopper

 [![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
 
 [![ko-fi](https://ko-fi.com/img/githubbutton_sm.svg)](https://ko-fi.com/O5O1OWKNW)

# Description
 A rust ECS library.

# Examples
## Fibonacci sequence computing
 ```cs
    let mut world = World::<u8, u8, u8>::builder()
        .with_component::<(i32, i32)>()
        .with_flag(0u8, 0u8..1u8)
        .build()?;

    world.new_entity()
        .with_component((4i32, 13i32))?
        .build();

    world.new_entity()
        .with_component((0i32, 1i32))?
        .with_flag(0u8, None)?
        .build();

        
    for _ in 0..20 {

        let query = world.new_query()
            .with_component::<(i32, i32)>()? // will only query entities with component `(i32, i32)`
            .with_flag(0u8, None)?           // will only query entities with flag `0u8` enabled
            .build();

        for component_ptr in query.get_components::<(i32, i32)>()?.iter() {

            let mut component = component_ptr.borrow_mut();
            println!("{}", component.0);

            *component = (component.1, component.0 + component.1);

        } // for ..
    } // for ..
 ```
 
