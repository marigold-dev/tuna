use std::{borrow::Cow, ptr::NonNull};

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use mimalloc::MiMalloc;
use serde::{Deserialize, Serialize};
use vm_library::{
    arena::ARENA,
    compile_store,
    env::Context,
    managed::{imports, value::Value},
};
use wasmer::{imports, wat2wasm, Instance, Module};
use wasmer_middlewares::metering::set_remaining_points;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn deser(json: &str) -> T {
    serde_json::from_str(json).unwrap()
}
fn ser(res: T) -> String {
    serde_json::ser::to_string(&res).unwrap()
}
fn benchmark(num: i64, json: &str) -> (Module, String, i64) {
    let x = deser(json);

    let module = Module::new(
        &compile_store::new_compile_store(),
        wat2wasm(x.mod_.as_bytes()).unwrap(),
    )
    .unwrap();
    let mut env = Box::new(Context {
        instance: None,
        pusher: None,
        gas_limit: 10000,
    });
    let store = module.store();

    let imports = imports::make_imports(&env, store);
    let mut instance = Box::new(Instance::new(&module, &imports).unwrap());
    env.instance = NonNull::new(instance.as_mut());

    set_remaining_points(&instance, 100000);
    let caller = instance
        .exports
        .get_native_function::<i64, i64>("fac")
        .unwrap();
    let result = caller.call(num).expect("error");
    let serialized = ser(x);
    unsafe { ARENA.clear() };
    (module, serialized, result)
}
//RUSTFLAGS='-C target-cpu=native' cargo bench
#[derive(Deserialize, Serialize)]
struct T<'a> {
    #[serde(borrow)]
    mod_: Cow<'a, str>,
    arg: Value,
    initial_storage: Value,
}

fn criterion_benchmark(c: &mut Criterion) {
    let json = r#"{"mod_":" (module (type $t0 (func (param i64) (result i64))) (func $fac (export \"fac\") (type $t0) (param $p0 i64) (result i64) (if $I0 (result i64) (i64.lt_s (local.get $p0) (i64.const 1)) (then (i64.const 1)) (else (i64.mul (local.get $p0) (call $fac (i64.sub (local.get $p0) (i64.const 1))))))))","arg":["Union",["Left",["List",[["Int","0"],["Int","1"],["Int","2"],["Int","3"],["Int","4"],["Int","5"],["Int","6"],["Int","7"],["Int","8"],["Int","9"],["Int","10"],["Int","11"],["Int","12"],["Int","13"],["Int","14"],["Int","15"],["Int","16"],["Int","17"],["Int","18"],["Int","19"],["Int","20"],["Int","21"],["Int","22"],["Int","23"],["Int","24"],["Int","25"],["Int","26"],["Int","27"],["Int","28"],["Int","29"],["Int","30"],["Int","31"],["Int","32"],["Int","33"],["Int","34"],["Int","35"],["Int","36"],["Int","37"],["Int","38"],["Int","39"],["Int","40"],["Int","41"],["Int","42"],["Int","43"],["Int","44"],["Int","45"],["Int","46"],["Int","47"],["Int","48"],["Int","49"],["Int","50"],["Int","51"],["Int","52"],["Int","53"],["Int","54"],["Int","55"],["Int","56"],["Int","57"],["Int","58"],["Int","59"],["Int","60"],["Int","61"],["Int","62"],["Int","63"],["Int","64"],["Int","65"],["Int","66"],["Int","67"],["Int","68"],["Int","69"],["Int","70"],["Int","71"],["Int","72"],["Int","73"],["Int","74"],["Int","75"],["Int","76"],["Int","77"],["Int","78"],["Int","79"],["Int","80"],["Int","81"],["Int","82"],["Int","83"],["Int","84"],["Int","85"],["Int","86"],["Int","87"],["Int","88"],["Int","89"],["Int","90"],["Int","91"],["Int","92"],["Int","93"],["Int","94"],["Int","95"],["Int","96"],["Int","97"],["Int","98"],["Int","99"]]]]],"initial_storage":["List",[["Int","0"],["Int","1"],["Int","2"],["Int","3"],["Int","4"],["Int","5"],["Int","6"],["Int","7"],["Int","8"],["Int","9"],["Int","10"],["Int","11"],["Int","12"],["Int","13"],["Int","14"],["Int","15"],["Int","16"],["Int","17"],["Int","18"],["Int","19"],["Int","20"],["Int","21"],["Int","22"],["Int","23"],["Int","24"],["Int","25"],["Int","26"],["Int","27"],["Int","28"],["Int","29"],["Int","30"],["Int","31"],["Int","32"],["Int","33"],["Int","34"],["Int","35"],["Int","36"],["Int","37"],["Int","38"],["Int","39"],["Int","40"],["Int","41"],["Int","42"],["Int","43"],["Int","44"],["Int","45"],["Int","46"],["Int","47"],["Int","48"],["Int","49"],["Int","50"],["Int","51"],["Int","52"],["Int","53"],["Int","54"],["Int","55"],["Int","56"],["Int","57"],["Int","58"],["Int","59"],["Int","60"],["Int","61"],["Int","62"],["Int","63"],["Int","64"],["Int","65"],["Int","66"],["Int","67"],["Int","68"],["Int","69"],["Int","70"],["Int","71"],["Int","72"],["Int","73"],["Int","74"],["Int","75"],["Int","76"],["Int","77"],["Int","78"],["Int","79"],["Int","80"],["Int","81"],["Int","82"],["Int","83"],["Int","84"],["Int","85"],["Int","86"],["Int","87"],["Int","88"],["Int","89"],["Int","90"],["Int","91"],["Int","92"],["Int","93"],["Int","94"],["Int","95"],["Int","96"],["Int","97"],["Int","98"],["Int","99"]]]}"#;

    let input = (json, 100);

    c.bench_with_input(
        BenchmarkId::new("test", "simple module"),
        &input,
        |b, (json, num)| b.iter(|| benchmark(*num, json)),
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
