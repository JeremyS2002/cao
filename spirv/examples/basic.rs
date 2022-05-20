use rspirv::binary::{Disassemble, Assemble};


fn main() {
    // let src = "
    //     #version 450

    //     void main() {
    //         vec2 y = vec2(0.0);
    //         float x = y.x;
    //     }
    // ";

    // let compiler = shaderc::Compiler::new().unwrap();
    // let spv = compiler.compile_into_spirv(
    //     src, 
    //     shaderc::ShaderKind::Vertex,
    //     "", "main", None,
    // ).unwrap();

    // let mut loader = rspirv::dr::Loader::new();
    // rspirv::binary::parse_words(spv.as_binary(), &mut loader).unwrap();
    // let module = loader.module();

    // println!("{}", module.disassemble());


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


    let vertex = {
        let builder = spirv::VertexBuilder::new();
    
        builder.main(|b| {
            
        });
    
        builder.compile()
    };

    let mut loader = rspirv::dr::Loader::new();
    rspirv::binary::parse_words(vertex, &mut loader).unwrap();
    let module = loader.module();

    println!("{}", module.disassemble());
}