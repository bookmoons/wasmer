use hashbrown::HashMap;
use inkwell::{
    builder::Builder,
    context::Context,
    module::Module,
    types::{BasicType, FloatType, IntType, PointerType, StructType, VoidType},
    values::{BasicValue, BasicValueEnum, FloatValue, FunctionValue, IntValue, PointerValue},
    AddressSpace,
};
use std::marker::PhantomData;
use wasmer_runtime_core::{
    memory::MemoryType,
    module::ModuleInfo,
    structures::TypedIndex,
    types::{LocalOrImport, MemoryIndex},
};

pub struct Intrinsics {
    pub ctlz_i32: FunctionValue,
    pub ctlz_i64: FunctionValue,

    pub cttz_i32: FunctionValue,
    pub cttz_i64: FunctionValue,

    pub ctpop_i32: FunctionValue,
    pub ctpop_i64: FunctionValue,

    pub sqrt_f32: FunctionValue,
    pub sqrt_f64: FunctionValue,

    pub minimum_f32: FunctionValue,
    pub minimum_f64: FunctionValue,

    pub maximum_f32: FunctionValue,
    pub maximum_f64: FunctionValue,

    pub ceil_f32: FunctionValue,
    pub ceil_f64: FunctionValue,

    pub floor_f32: FunctionValue,
    pub floor_f64: FunctionValue,

    pub trunc_f32: FunctionValue,
    pub trunc_f64: FunctionValue,

    pub nearbyint_f32: FunctionValue,
    pub nearbyint_f64: FunctionValue,

    pub fabs_f32: FunctionValue,
    pub fabs_f64: FunctionValue,

    pub copysign_f32: FunctionValue,
    pub copysign_f64: FunctionValue,

    pub void_ty: VoidType,
    pub i1_ty: IntType,
    pub i8_ty: IntType,
    pub i16_ty: IntType,
    pub i32_ty: IntType,
    pub i64_ty: IntType,
    pub f32_ty: FloatType,
    pub f64_ty: FloatType,

    pub i8_ptr_ty: PointerType,
    pub i16_ptr_ty: PointerType,
    pub i32_ptr_ty: PointerType,
    pub i64_ptr_ty: PointerType,
    pub f32_ptr_ty: PointerType,
    pub f64_ptr_ty: PointerType,

    pub i1_zero: IntValue,
    pub i32_zero: IntValue,
    pub i64_zero: IntValue,
    pub f32_zero: FloatValue,
    pub f64_zero: FloatValue,

    // VM intrinsics.
    pub memory_grow_dynamic_local: FunctionValue,
    pub memory_grow_static_local: FunctionValue,
    pub memory_grow_shared_local: FunctionValue,
    pub memory_grow_dynamic_import: FunctionValue,
    pub memory_grow_static_import: FunctionValue,
    pub memory_grow_shared_import: FunctionValue,

    pub memory_size_dynamic_local: FunctionValue,
    pub memory_size_static_local: FunctionValue,
    pub memory_size_shared_local: FunctionValue,
    pub memory_size_dynamic_import: FunctionValue,
    pub memory_size_static_import: FunctionValue,
    pub memory_size_shared_import: FunctionValue,

    ctx_ty: StructType,
    pub ctx_ptr_ty: PointerType,
}

impl Intrinsics {
    pub fn declare(module: &Module, context: &Context) -> Self {
        let void_ty = context.void_type();
        let i1_ty = context.bool_type();
        let i8_ty = context.i8_type();
        let i16_ty = context.i16_type();
        let i32_ty = context.i32_type();
        let i64_ty = context.i64_type();
        let f32_ty = context.f32_type();
        let f64_ty = context.f64_type();

        let i8_ptr_ty = i8_ty.ptr_type(AddressSpace::Generic);
        let i16_ptr_ty = i16_ty.ptr_type(AddressSpace::Generic);
        let i32_ptr_ty = i32_ty.ptr_type(AddressSpace::Generic);
        let i64_ptr_ty = i64_ty.ptr_type(AddressSpace::Generic);
        let f32_ptr_ty = f32_ty.ptr_type(AddressSpace::Generic);
        let f64_ptr_ty = f64_ty.ptr_type(AddressSpace::Generic);

        let opaque_ptr_ty = void_ty.ptr_type(AddressSpace::Generic);

        let i1_zero = i1_ty.const_int(0, false);
        let i32_zero = i32_ty.const_int(0, false);
        let i64_zero = i64_ty.const_int(0, false);
        let f32_zero = f32_ty.const_float(0.0);
        let f64_zero = f64_ty.const_float(0.0);

        let i1_ty_basic = i1_ty.as_basic_type_enum();
        let i32_ty_basic = i32_ty.as_basic_type_enum();
        let i64_ty_basic = i64_ty.as_basic_type_enum();
        let f32_ty_basic = f32_ty.as_basic_type_enum();
        let f64_ty_basic = f64_ty.as_basic_type_enum();
        let i8_ptr_ty_basic = i8_ptr_ty.as_basic_type_enum();
        let opaque_ptr_ty_basic = opaque_ptr_ty.as_basic_type_enum();

        let ctx_ty = context.opaque_struct_type("ctx");
        let ctx_ptr_ty = ctx_ty.ptr_type(AddressSpace::Generic);

        let local_memory_ty =
            context.struct_type(&[i8_ptr_ty_basic, i64_ty_basic, opaque_ptr_ty_basic], false);
        let local_table_ty = local_memory_ty;
        let local_global_ty = i64_ty;
        let imported_func_ty = context.struct_type(
            &[opaque_ptr_ty_basic, ctx_ptr_ty.as_basic_type_enum()],
            false,
        );
        ctx_ty.set_body(
            &[
                local_memory_ty
                    .ptr_type(AddressSpace::Generic)
                    .ptr_type(AddressSpace::Generic)
                    .as_basic_type_enum(),
                local_table_ty
                    .ptr_type(AddressSpace::Generic)
                    .ptr_type(AddressSpace::Generic)
                    .as_basic_type_enum(),
                local_global_ty
                    .ptr_type(AddressSpace::Generic)
                    .ptr_type(AddressSpace::Generic)
                    .as_basic_type_enum(),
                local_memory_ty
                    .ptr_type(AddressSpace::Generic)
                    .ptr_type(AddressSpace::Generic)
                    .as_basic_type_enum(),
                local_table_ty
                    .ptr_type(AddressSpace::Generic)
                    .ptr_type(AddressSpace::Generic)
                    .as_basic_type_enum(),
                local_global_ty
                    .ptr_type(AddressSpace::Generic)
                    .ptr_type(AddressSpace::Generic)
                    .as_basic_type_enum(),
                imported_func_ty
                    .ptr_type(AddressSpace::Generic)
                    .ptr_type(AddressSpace::Generic)
                    .as_basic_type_enum(),
            ],
            false,
        );

        let ret_i32_take_i32_i1 = i32_ty.fn_type(&[i32_ty_basic, i1_ty_basic], false);
        let ret_i64_take_i64_i1 = i64_ty.fn_type(&[i64_ty_basic, i1_ty_basic], false);

        let ret_i32_take_i32 = i32_ty.fn_type(&[i32_ty_basic], false);
        let ret_i64_take_i64 = i64_ty.fn_type(&[i64_ty_basic], false);

        let ret_f32_take_f32 = f32_ty.fn_type(&[f32_ty_basic], false);
        let ret_f64_take_f64 = f64_ty.fn_type(&[f64_ty_basic], false);

        let ret_f32_take_f32_f32 = f32_ty.fn_type(&[f32_ty_basic, f32_ty_basic], false);
        let ret_f64_take_f64_f64 = f64_ty.fn_type(&[f64_ty_basic, f64_ty_basic], false);

        let ret_i32_take_i64_i32_i32 =
            i32_ty.fn_type(&[i64_ty_basic, i32_ty_basic, i32_ty_basic], false);
        let ret_i32_take_i64_i32 = i32_ty.fn_type(&[i64_ty_basic, i32_ty_basic], false);

        Self {
            ctlz_i32: module.add_function("llvm.ctlz.i32", ret_i32_take_i32_i1, None),
            ctlz_i64: module.add_function("llvm.ctlz.i64", ret_i64_take_i64_i1, None),

            cttz_i32: module.add_function("llvm.cttz.i32", ret_i32_take_i32_i1, None),
            cttz_i64: module.add_function("llvm.cttz.i64", ret_i64_take_i64_i1, None),

            ctpop_i32: module.add_function("llvm.ctpop.i32", ret_i32_take_i32, None),
            ctpop_i64: module.add_function("llvm.ctpop.i64", ret_i64_take_i64, None),

            sqrt_f32: module.add_function("llvm.sqrt.f32", ret_f32_take_f32, None),
            sqrt_f64: module.add_function("llvm.sqrt.f64", ret_f64_take_f64, None),

            minimum_f32: module.add_function("llvm.minimum.f32", ret_f32_take_f32_f32, None),
            minimum_f64: module.add_function("llvm.minimum.f64", ret_f64_take_f64_f64, None),

            maximum_f32: module.add_function("llvm.maximum.f32", ret_f32_take_f32_f32, None),
            maximum_f64: module.add_function("llvm.maximum.f64", ret_f64_take_f64_f64, None),

            ceil_f32: module.add_function("llvm.ceil.f32", ret_f32_take_f32, None),
            ceil_f64: module.add_function("llvm.ceil.f64", ret_f64_take_f64, None),

            floor_f32: module.add_function("llvm.floor.f32", ret_f32_take_f32, None),
            floor_f64: module.add_function("llvm.floor.f64", ret_f64_take_f64, None),

            trunc_f32: module.add_function("llvm.trunc.f32", ret_f32_take_f32, None),
            trunc_f64: module.add_function("llvm.trunc.f64", ret_f64_take_f64, None),

            nearbyint_f32: module.add_function("llvm.nearbyint.f32", ret_f32_take_f32, None),
            nearbyint_f64: module.add_function("llvm.nearbyint.f64", ret_f64_take_f64, None),

            fabs_f32: module.add_function("llvm.fabs.f32", ret_f32_take_f32, None),
            fabs_f64: module.add_function("llvm.fabs.f64", ret_f64_take_f64, None),

            copysign_f32: module.add_function("llvm.copysign.f32", ret_f32_take_f32_f32, None),
            copysign_f64: module.add_function("llvm.copysign.f64", ret_f64_take_f64_f64, None),

            void_ty,
            i1_ty,
            i8_ty,
            i16_ty,
            i32_ty,
            i64_ty,
            f32_ty,
            f64_ty,

            i8_ptr_ty,
            i16_ptr_ty,
            i32_ptr_ty,
            i64_ptr_ty,
            f32_ptr_ty,
            f64_ptr_ty,

            i1_zero,
            i32_zero,
            i64_zero,
            f32_zero,
            f64_zero,

            // VM intrinsics.
            memory_grow_dynamic_local: module.add_function(
                "vm.memory.grow.dynamic.local",
                ret_i32_take_i64_i32_i32,
                None,
            ),
            memory_grow_static_local: module.add_function(
                "vm.memory.grow.static.local",
                ret_i32_take_i64_i32_i32,
                None,
            ),
            memory_grow_shared_local: module.add_function(
                "vm.memory.grow.shared.local",
                ret_i32_take_i64_i32_i32,
                None,
            ),
            memory_grow_dynamic_import: module.add_function(
                "vm.memory.grow.dynamic.import",
                ret_i32_take_i64_i32_i32,
                None,
            ),
            memory_grow_static_import: module.add_function(
                "vm.memory.grow.static.import",
                ret_i32_take_i64_i32_i32,
                None,
            ),
            memory_grow_shared_import: module.add_function(
                "vm.memory.grow.shared.import",
                ret_i32_take_i64_i32_i32,
                None,
            ),

            memory_size_dynamic_local: module.add_function(
                "vm.memory.size.dynamic.local",
                ret_i32_take_i64_i32,
                None,
            ),
            memory_size_static_local: module.add_function(
                "vm.memory.size.static.local",
                ret_i32_take_i64_i32,
                None,
            ),
            memory_size_shared_local: module.add_function(
                "vm.memory.size.shared.local",
                ret_i32_take_i64_i32,
                None,
            ),
            memory_size_dynamic_import: module.add_function(
                "vm.memory.size.dynamic.import",
                ret_i32_take_i64_i32,
                None,
            ),
            memory_size_static_import: module.add_function(
                "vm.memory.size.static.import",
                ret_i32_take_i64_i32,
                None,
            ),
            memory_size_shared_import: module.add_function(
                "vm.memory.size.shared.import",
                ret_i32_take_i64_i32,
                None,
            ),

            ctx_ty,
            ctx_ptr_ty,
        }
    }

    pub fn ctx<'a>(
        &'a self,
        info: &'a ModuleInfo,
        builder: &'a Builder,
        func_value: &'a FunctionValue,
    ) -> CtxType<'a> {
        CtxType {
            ctx_ty: self.ctx_ty,
            ctx_ptr_ty: self.ctx_ptr_ty,

            ctx_ptr_value: func_value.get_nth_param(0).unwrap().into_pointer_value(),

            builder,
            intrinsics: self,
            info,

            cached_memories: HashMap::new(),

            _phantom: PhantomData,
        }
    }
}

enum MemoryCache {
    /// The memory moves around.
    Dynamic {
        ptr_to_base_ptr: PointerValue,
        ptr_to_bounds: PointerValue,
    },
    /// The memory is always in the same place.
    Static {
        base_ptr: PointerValue,
        bounds: IntValue,
    },
}

pub struct CtxType<'a> {
    ctx_ty: StructType,
    ctx_ptr_ty: PointerType,

    ctx_ptr_value: PointerValue,

    builder: &'a Builder,
    intrinsics: &'a Intrinsics,
    info: &'a ModuleInfo,

    cached_memories: HashMap<MemoryIndex, MemoryCache>,

    _phantom: PhantomData<&'a FunctionValue>,
}

impl<'a> CtxType<'a> {
    pub fn basic(&self) -> BasicValueEnum {
        self.ctx_ptr_value.as_basic_value_enum()
    }

    pub fn memory(&mut self, index: MemoryIndex) -> (PointerValue, IntValue) {
        let (cached_memories, builder, info, ctx_ptr_value, intrinsics) = (
            &mut self.cached_memories,
            self.builder,
            self.info,
            self.ctx_ptr_value,
            self.intrinsics,
        );

        let memory_cache = cached_memories.entry(index).or_insert_with(|| {
            let (memory_array_ptr_ptr, index, memory_type) = match index.local_or_import(info) {
                LocalOrImport::Local(local_mem_index) => (
                    unsafe { builder.build_struct_gep(ctx_ptr_value, 0, "memory_array_ptr_ptr") },
                    local_mem_index.index() as u64,
                    info.memories[local_mem_index].memory_type(),
                ),
                LocalOrImport::Import(import_mem_index) => (
                    unsafe { builder.build_struct_gep(ctx_ptr_value, 3, "memory_array_ptr_ptr") },
                    import_mem_index.index() as u64,
                    info.imported_memories[import_mem_index].1.memory_type(),
                ),
            };

            let memory_array_ptr = builder
                .build_load(memory_array_ptr_ptr, "memory_array_ptr")
                .into_pointer_value();
            let const_index = intrinsics.i32_ty.const_int(index, false);
            let memory_ptr_ptr = unsafe {
                builder.build_in_bounds_gep(memory_array_ptr, &[const_index], "memory_ptr_ptr")
            };
            let memory_ptr = builder
                .build_load(memory_ptr_ptr, "memory_ptr")
                .into_pointer_value();

            let (ptr_to_base_ptr, ptr_to_bounds) = unsafe {
                (
                    builder.build_struct_gep(memory_ptr, 0, "base_ptr"),
                    builder.build_struct_gep(memory_ptr, 1, "bounds_ptr"),
                )
            };

            match memory_type {
                MemoryType::Dynamic => MemoryCache::Dynamic {
                    ptr_to_base_ptr,
                    ptr_to_bounds,
                },
                MemoryType::Static | MemoryType::SharedStatic => MemoryCache::Static {
                    base_ptr: builder
                        .build_load(ptr_to_base_ptr, "base")
                        .into_pointer_value(),
                    bounds: builder.build_load(ptr_to_bounds, "bounds").into_int_value(),
                },
            }
        });

        match memory_cache {
            MemoryCache::Dynamic {
                ptr_to_base_ptr,
                ptr_to_bounds,
            } => {
                let base = builder
                    .build_load(*ptr_to_base_ptr, "base")
                    .into_pointer_value();
                let bounds = builder
                    .build_load(*ptr_to_bounds, "bounds")
                    .into_int_value();

                (base, bounds)
            }
            MemoryCache::Static { base_ptr, bounds } => (*base_ptr, *bounds),
        }
    }
}

// pub struct Ctx {
//     /// A pointer to an array of locally-defined memories, indexed by `MemoryIndex`.
//     pub(crate) memories: *mut *mut LocalMemory,

//     /// A pointer to an array of locally-defined tables, indexed by `TableIndex`.
//     pub(crate) tables: *mut *mut LocalTable,

//     /// A pointer to an array of locally-defined globals, indexed by `GlobalIndex`.
//     pub(crate) globals: *mut *mut LocalGlobal,

//     /// A pointer to an array of imported memories, indexed by `MemoryIndex,
//     pub(crate) imported_memories: *mut *mut LocalMemory,

//     /// A pointer to an array of imported tables, indexed by `TableIndex`.
//     pub(crate) imported_tables: *mut *mut LocalTable,

//     /// A pointer to an array of imported globals, indexed by `GlobalIndex`.
//     pub(crate) imported_globals: *mut *mut LocalGlobal,

//     /// A pointer to an array of imported functions, indexed by `FuncIndex`.
//     pub(crate) imported_funcs: *mut ImportedFunc,

//     local_backing: *mut LocalBacking,
//     import_backing: *mut ImportBacking,
//     module: *const ModuleInner,

//     pub data: *mut c_void,
//     pub data_finalizer: Option<extern "C" fn(data: *mut c_void)>,
// }
