use ndarray::{Array3, Axis};
use ort::{inputs, Error, Session};
use wavers::{Samples, Wav};

fn load_model() -> Result<Session, Error> {
    Session::builder()?
        .with_optimization_level(ort::GraphOptimizationLevel::Level1)?
        .with_intra_threads(1)?
        .with_model_from_file("./models/silero_vad.onnx")
}

// returns a chunk of size 100ms
fn get_window_size(sample_rate: i32) -> usize {
    sample_rate as usize / 10
}

fn format_time(ms: i32) -> String {
    let seconds = ms / 1000;
    let minutes = (seconds / 60) % 60;
    let hours = (minutes / 60) % 60;

    format!(
        "{:0>2}:{:0>2}:{:0>2}.{:0>3}",
        hours,
        minutes,
        seconds,
        ms % 1000
    )
}

fn main() -> anyhow::Result<()> {
    let model = load_model().expect("failed to load model");
    println!("{}", model.metadata()?.description()?);

    let mut wav: Wav<i16> = Wav::from_path("./en.wav").expect("failed to load audio file");
    let samples: Samples<i16> = wav.read()?;
    let (sample_rate, channels, duration, encoding) = wav.wav_spec();

    println!(
        "loaded audio file: {}Hz, {}ch, {}s, {}bit",
        sample_rate, channels, duration, encoding
    );

    // create buffer of normalised(?) floats
    let data: Vec<f32> = samples
        .iter()
        .map(|input| {
            let mut val = *input as f32;

            val = 32767.0 / val.abs();

            val
        })
        .collect();

    let mut speech_started = false;

    let mut elapsed_time_ms = 0;

    for chunk in data.chunks(get_window_size(sample_rate)) {
        let chunk_array = ndarray::Array1::from_iter(chunk.to_owned());

        // making it a 2D array?
        let input_array = chunk_array.view().insert_axis(Axis(0));

        let input = ort::Value::from_array(input_array).expect("failed to make input array");
        let sr = ort::Value::from_array(ndarray::array![sample_rate as i64])?;
        let h = ort::Value::from_array(Array3::<f32>::zeros((2, 1, 64)))?;
        let c = ort::Value::from_array(Array3::<f32>::zeros((2, 1, 64)))?;

        let input_data = inputs![
            "input" => input,
            "sr" => sr,
            "h" => h,
            "c" => c
        ]?;

        let outputs = model.run(input_data)?;

        let tensor = outputs["output"].extract_tensor::<f32>()?;

        for value in tensor.view().iter() {
            if value.to_owned() > 0.5 && !speech_started {
                println!(
                    "{} detected speech start ({})",
                    format_time(elapsed_time_ms),
                    value
                );

                speech_started = true;
            } else if value.to_owned() < 0.35 && speech_started {
                println!(
                    "{} detected speech end ({})",
                    format_time(elapsed_time_ms),
                    value
                );

                speech_started = false;
            }
        }

        elapsed_time_ms += 100;
    }

    Ok(())
}
