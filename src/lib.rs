#![deny(warnings)]

use lazy_static::lazy_static;
use rutie::{class, methods, wrappable_struct, AnyObject, Class, Fixnum, Object, Symbol};
use wasmer_runtime::{self as runtime, imports};

static WASM: &'static [u8] = &[
    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x06, 0x01, 0x60, 0x01, 0x7f, 0x01, 0x7f,
    0x03, 0x02, 0x01, 0x00, 0x07, 0x0b, 0x01, 0x07, 0x61, 0x64, 0x64, 0x5f, 0x6f, 0x6e, 0x65, 0x00,
    0x00, 0x0a, 0x09, 0x01, 0x07, 0x00, 0x20, 0x00, 0x41, 0x01, 0x6a, 0x0b, 0x00, 0x1a, 0x04, 0x6e,
    0x61, 0x6d, 0x65, 0x01, 0x0a, 0x01, 0x00, 0x07, 0x61, 0x64, 0x64, 0x5f, 0x6f, 0x6e, 0x65, 0x02,
    0x07, 0x01, 0x00, 0x01, 0x00, 0x02, 0x70, 0x30,
];

pub struct ExportedFunctions {}

impl ExportedFunctions {
    pub fn new() -> Self {
        Self {}
    }

    pub fn method_missing(&self, method_name: String) -> i32 {
        println!("----> {}", method_name);
        42
    }
}

wrappable_struct!(
    ExportedFunctions,
    ExportedFunctionsWrapper,
    EXPORTED_FUNCTIONS_WRAPPER
);

class!(RubyExportedFunctions);

#[rustfmt::skip]
methods!(
    RubyExportedFunctions,
    _itself,

    fn ruby_exported_functions_new() -> AnyObject {
        let exported_functions = ExportedFunctions::new();

        Class::from_existing("ExportedFunctions").wrap_data(exported_functions, &*EXPORTED_FUNCTIONS_WRAPPER)
    }

    fn ruby_exported_functions_method_missing(method_name: Symbol) -> Fixnum {
        let result = _itself.get_data(&*EXPORTED_FUNCTIONS_WRAPPER).method_missing(method_name.unwrap().to_string());

        Fixnum::new(result as i64)
    }
);

pub struct Instance {
    instance: runtime::Instance,
}

impl Instance {
    pub fn new() -> Self {
        let import_object = imports! {};
        let instance = runtime::instantiate(WASM, &import_object).unwrap();

        Self { instance }
    }

    pub fn call(&self) -> i32 {
        let result = &self
            .instance
            .dyn_func("add_one")
            .unwrap()
            .call(&[runtime::Value::I32(1)])
            .unwrap()[0];

        if let runtime::Value::I32(result) = result {
            *result
        } else {
            0
        }
    }
}

wrappable_struct!(Instance, InstanceWrapper, INSTANCE_WRAPPER);

class!(RubyInstance);

#[rustfmt::skip]
methods!(
    RubyInstance,
    _itself,

    fn ruby_instance_new() -> AnyObject {
        let instance = Instance::new();
        let mut ruby_instance: AnyObject = Class::from_existing("Instance").wrap_data(instance, &*INSTANCE_WRAPPER);
        let exported_functions = ExportedFunctions::new();
        let ruby_exported_functions: RubyExportedFunctions = Class::from_existing("ExportedFunctions").wrap_data(exported_functions, &*EXPORTED_FUNCTIONS_WRAPPER);

        ruby_instance.instance_variable_set("@foo", ruby_exported_functions);

        ruby_instance
    }

    fn ruby_instance_call() -> Fixnum {
        let result = _itself.get_data(&*INSTANCE_WRAPPER).call();

        Fixnum::new(result as i64)
    }

    fn ruby_instance_exported_functions() -> RubyExportedFunctions {
        unsafe {
            _itself.instance_variable_get("@foo").to::<RubyExportedFunctions>()
        }
    }
);

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn Init_wasmer() {
    let instance_data_class = Class::from_existing("Object");
    let exported_functions_data_class = Class::from_existing("Object");

    Class::new("Instance", Some(&instance_data_class)).define(|itself| {
        itself.def_self("new", ruby_instance_new);
        itself.def("call", ruby_instance_call);
        itself.def("exports", ruby_instance_exported_functions);
    });

    Class::new("ExportedFunctions", Some(&exported_functions_data_class)).define(|itself| {
        itself.def_self("new", ruby_exported_functions_new);
        itself.def("method_missing", ruby_exported_functions_method_missing);
    });
}
