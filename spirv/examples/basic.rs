
fn main() {
    let builder = spirv::Builder::new();

    builder.spv_main(|b| {
        let t = b.new_bool(true);
        let f = b.new_float(1.0);


        let x = b.new_int(10);
        let y = b.new_int(100);

        b.spv_if(&t, |b| {
            b.spv_store(f, 0.0);
        });

        let z = b.spv_add(x, y);
    });

    let instructions = builder.instructions();

    for instructon in instructions {
        println!("{:?}", instructon)
    }
}