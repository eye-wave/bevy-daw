#[cfg(test)]
pub mod test {
    use crate::engine::SAMPLE_RATE;
    use hound;
    use plotters::prelude::*;
    use rustfft::{FftPlanner, num_complex::Complex};
    use std::path::{Path, PathBuf};

    pub fn node_test_suite(samples: &[f32], window_size: usize, name: &str) {
        let base_dir: PathBuf = "target/node-plots/".into();

        let path_wave = base_dir.join(format!("{name}_wave.png"));
        let path_spectr = base_dir.join(format!("{name}_spectr.png"));
        let path_sound = base_dir.join(format!("{name}.wav"));

        let spectr = compute_spectrogram(samples, window_size, 1);

        plot_waveform(samples, &path_wave);
        plot_spectrogram(&spectr, window_size, &path_spectr);
        save_tone(&path_sound, samples).ok();
    }

    pub fn save_tone<P: AsRef<Path>>(
        path: &P,
        samples: &[f32],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: SAMPLE_RATE,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let mut writer = hound::WavWriter::create(path, spec)?;

        for &sample in samples {
            let s = (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
            writer.write_sample(s)?;
        }

        writer.finalize()?;
        Ok(())
    }

    pub fn plot_waveform<P: AsRef<Path>>(samples: &[f32], path: &P) {
        let root = BitMapBackend::new(path, (800, 300)).into_drawing_area();
        root.fill(&WHITE).unwrap();

        let len = samples.len();
        let duration = len as f32 / SAMPLE_RATE as f32;

        let mut chart = ChartBuilder::on(&root)
            .margin(10)
            .x_label_area_size(40)
            .y_label_area_size(40)
            .build_cartesian_2d(0f32..duration, -1f32..1f32)
            .unwrap();

        chart
            .configure_mesh()
            .x_desc("Time (seconds)")
            .y_desc("Amplitude")
            .draw()
            .unwrap();

        chart
            .draw_series(LineSeries::new(
                (0..len).map(|i| (i as f32 / SAMPLE_RATE as f32, samples[i])),
                &BLACK,
            ))
            .unwrap();
    }

    pub fn compute_spectrogram(
        samples: &[f32],
        window_size: usize,
        hop_size: usize,
    ) -> Vec<Vec<f32>> {
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(window_size);

        let mut spec = Vec::new();

        for i in (0..samples.len() - window_size).step_by(hop_size) {
            let window: Vec<Complex<f32>> = samples[i..i + window_size]
                .iter()
                .map(|&s| Complex { re: s, im: 0.0 })
                .collect();

            let mut buffer = window.clone();
            fft.process(&mut buffer);

            let half_size = window_size / 2;
            let mags = buffer.iter().take(half_size).map(|c| c.norm()).collect();

            spec.push(mags);
        }

        spec
    }

    fn interp_linear(mags: &[f32], idx: f32) -> f32 {
        let idx_floor = idx.floor() as usize;
        let idx_ceil = idx.ceil() as usize;
        if idx_ceil >= mags.len() {
            return mags[mags.len() - 1];
        }
        let w = idx - idx_floor as f32;
        mags[idx_floor] * (1.0 - w) + mags[idx_ceil] * w
    }

    pub fn plot_spectrogram<P: AsRef<Path>>(spec: &[Vec<f32>], window_size: usize, path: &P) {
        let width = spec.len();
        let height = spec[0].len();

        let f_min = 20.0;
        let f_max = SAMPLE_RATE as f32 / 2.0;
        let octaves = (f_max / f_min).log2();

        let image_width = width as u32 + 100;
        let image_height = height as u32 + 100;

        let root = BitMapBackend::new(path, (image_width, image_height)).into_drawing_area();
        root.fill(&WHITE).unwrap();

        let (upper, _) = root.split_vertically(image_height - 80);
        let (_, right) = upper.split_horizontally(80);

        let drawing_area = right;

        let max_val = spec.iter().flatten().cloned().fold(f32::MIN, f32::max);
        let min_val = spec.iter().flatten().cloned().fold(f32::MAX, f32::min);

        let mut chart = ChartBuilder::on(&drawing_area)
            .margin(5)
            .x_label_area_size(40)
            .y_label_area_size(80)
            .build_cartesian_2d(0..width, (f_min.ln())..(f_max.ln()))
            .unwrap();

        chart
            .configure_mesh()
            .disable_mesh()
            .x_desc("Time (frames)")
            .y_desc("Frequency (Hz)")
            .y_label_formatter(&|v| format!("{:.0}", v.exp()))
            .draw()
            .unwrap();

        for (x, column) in spec.iter().enumerate() {
            for y in 0..height {
                let y_norm = y as f32 / (height as f32 - 1.0);
                let f_y = f_min * 2f32.powf(y_norm * octaves);
                let ln_f_y = f_y.ln();

                let b_y = f_y * window_size as f32 / SAMPLE_RATE as f32;
                let val = interp_linear(column, b_y);

                let intensity = (val - min_val) / (max_val - min_val);
                let intensity = 1.0 - (intensity - 1.0).powf(2.0);

                let int8 = (intensity * 255.0) as u8;
                let color = RGBColor(int8, int8, int8);

                chart
                    .draw_series(std::iter::once(Rectangle::new(
                        [
                            (x, ln_f_y),
                            (x + 1, ln_f_y + (f_max.ln() - f_min.ln()) / height as f32),
                        ],
                        color.filled(),
                    )))
                    .unwrap();
            }
        }
    }
}
