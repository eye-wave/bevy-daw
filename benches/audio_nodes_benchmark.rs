use bevy_daw::nodes::ToneGenerator;
use bevy_daw::traits::AudioNode;
use criterion::{Bencher, Criterion, criterion_group, criterion_main};

const BUFFER_SIZE: usize = 4096;

fn audio_nodes_benchmark(c: &mut Criterion) {
    let mut buffer = [0.0; BUFFER_SIZE];
    let mut node = ToneGenerator::new(440.0, 0.5);

    c.bench_function("Tone generator", |b: &mut Bencher| {
        b.iter(|| node.process(0, &mut buffer))
    });
}

criterion_group!(benches, audio_nodes_benchmark);
criterion_main!(benches);
