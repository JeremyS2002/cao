use spv::prelude::*;

fn main() {
    let src = "
        #version 450

        layout(set = 0, binding = 0) buffer Storage {
            float xs[];
        } u_storage;

        void main() {
            int i = gl_InstanceIndex;
            uint j = uint(i);
            //gl_Position = vec4(in_pos, 0.0, 1.0);
        }
    ";

    let compiler = shaderc::Compiler::new().unwrap();
    let spv = compiler
        .compile_into_spirv(src, shaderc::ShaderKind::Vertex, "", "main", None)
        .unwrap();

    use rspirv::binary::Disassemble;

    let mut loader = rspirv::dr::Loader::new();
    rspirv::binary::parse_words(spv.as_binary(), &mut loader).unwrap();
    let module = loader.module();

    println!("{}", module.disassemble());

    println!("");
    println!("");

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

    let vertex_spv = {
        let builder = spv::VertexBuilder::new();

        let in_pos = builder.in_vec2(0, false, Some("in_pos"));

        let position = builder.position();

        builder.main(|b| {
            let pos = b.load_in(in_pos);
            let x = pos.x(b);
            let y = pos.y(b);
            let pos = b.vec4(&x, &y, &0.0, &1.0);
            b.store_out(position, pos);
        });

        builder.compile()
    };

    let mut loader = rspirv::dr::Loader::new();
    rspirv::binary::parse_words(vertex_spv, &mut loader).unwrap();
    let module = loader.module();

    println!("{}", module.disassemble());
}
