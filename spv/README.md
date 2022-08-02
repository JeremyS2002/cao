# spirv

Note: Requires nightly compiler until TypeId::of::<T>() is const

# examples
run from insed `cao/spirv`
```
cargo run --example basic
```

# TODO
- Error handling
- Lift restriction on names being static
- For Loops
- More intuitive loops (allow b.spv_while(x < 3, |b| { .. }) syntax)
- Testing
- Stop parent builders being used while they have children
- utility user facing macros
- Examples 
- Remove 2 stage compilation and compile as user commands are recoreded