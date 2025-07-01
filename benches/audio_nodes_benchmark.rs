use assert_no_alloc::*;
use bevy_daw::nodes::{DelayNode, ToneGenerator};
use bevy_daw::traits::AudioNode;
use criterion::{Bencher, Criterion, criterion_group, criterion_main};

#[cfg(debug_assertions)]
#[global_allocator]
static A: AllocDisabler = AllocDisabler;

const BUFFER_SIZE: usize = 4096;

macro_rules! bench_nodes_group {
    (
        $group_name:ident,
        [
            $( $bench_name:ident => ($node_type:ty, $($ctor_arg:expr),*) ),* $(,)?
        ]
    ) => {
        $(
            fn $bench_name(c: &mut Criterion) {
                let mut buffer = [0.0f32; BUFFER_SIZE];
                let mut node = <$node_type>::new($($ctor_arg),*);

                c.bench_function(stringify!($bench_name), |b: &mut Bencher| {
                    b.iter(|| assert_no_alloc(|| node.process(0, &mut buffer)))
                });
            }
        )*

        criterion_group!($group_name, $( $bench_name ),*);
        criterion_main!($group_name);
    };
}

bench_nodes_group!(benches, [
    tone_generator_bench => (ToneGenerator, 440.0, 0.5),
    delay_generator_bench => (DelayNode, 11025),
]);
