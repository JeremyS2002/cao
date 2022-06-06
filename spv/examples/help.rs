fn main() {
    use rspirv::binary::Disassemble;

    let src = "
        #version 450

        layout(push_constant) uniform Data {
            float x;
            float y;
        } p_data;

        void main() {
            float z = p_data.x + p_data.y;
        }
    ";

    let compiler = shaderc::Compiler::new().unwrap();
    let spv = compiler
        .compile_into_spirv(src, shaderc::ShaderKind::Fragment, "", "main", None)
        .unwrap();

    let mut loader = rspirv::dr::Loader::new();
    rspirv::binary::parse_words(spv.as_binary(), &mut loader).unwrap();
    let module = loader.module();

    println!("{}", module.disassemble());

    // ===================================================================
    // ===================================================================
    // ===================================================================

    // let mut builder = rspirv::dr::Builder::new();

    // let void = builder.type_void();
    // let void_f = builder.type_function(void, []);
    // let main = builder.begin_function(
    //     void,
    //     None,
    //     rspirv::spirv::FunctionControl::empty(),
    //     void_f,
    // ).unwrap();
    // builder.name(main, "main");

    // builder.entry_point(
    //     rspirv::spirv::ExecutionModel::Vertex,
    //     main,
    //     "main",
    //     [],
    // );

    // builder.begin_block(None).unwrap();

    // let float_ty = builder.type_float(32);
    // let constant = builder.constant_f32(float_ty, 0.0);

    // let pointer_ty = builder.type_pointer(
    //     None,
    //     rspirv::spirv::StorageClass::Function,
    //     float_ty,
    // );
    // let variable = builder.variable(
    //     pointer_ty,
    //     None,
    //     rspirv::spirv::StorageClass::Function,
    //     None,
    // );

    // builder.store(variable, constant, None, None).unwrap();
    // builder.ret().unwrap();
    // builder.end_function().unwrap();
    // //let code = builder.module().assemble();
    // println!("{}", builder.module().disassemble());

    // ===================================================================
    // ===================================================================
    // ===================================================================

    // let vertex = {
    //     let builder = spirv::VertexBuilder::new();

    //     let position = builder.position();

    //     builder.main(|b| {
    //         // b.spv_if(true, |b| {
    //         //     b.store_out(position, glam::vec4(0.0, 0.0, 0.0, 0.0));
    //         // }).spv_else_if(true, |b| {
    //         //     b.store_out(position, glam::vec4(1.0, 1.0, 1.0, 0.0))
    //         // });

    //         b.spv_while(true, |b| {

    //         });
    //     });

    //     builder.compile()
    // };

    // let mut loader = rspirv::dr::Loader::new();
    // rspirv::binary::parse_words(vertex, &mut loader).unwrap();
    // let module = loader.module();

    // println!("{}", module.disassemble());
}
