use std::{sync::Arc, time::Instant};

use ndarray::{Array3, Axis};
use ort::{inputs, Error, Session};
use serde::Serialize;

use crate::g711;

#[derive(Serialize)]
enum SpeechEvent {
    SpeechStarted,
    SpeechEnded,
}

#[derive(Serialize)]
pub struct Marker {
    kind: SpeechEvent,
    time_in_ms: i32,
}

pub fn load_model() -> Result<Session, Error> {
    Session::builder()?
        .with_optimization_level(ort::GraphOptimizationLevel::Level1)?
        .with_intra_threads(1)?
        .with_model_from_file("./models/silero_vad.onnx")
}

// returns a chunk of size 100ms
fn get_window_size(sample_rate: i32) -> usize {
    sample_rate as usize / 10
}

// format ms into hh:mm:ss.mmm
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

pub fn analyse_data(model: Arc<Session>, input: Vec<u8>) -> anyhow::Result<Vec<Marker>> {
    let samples = g711::decode(input);

    // create buffer of normalised floats
    let data: Vec<f32> = samples
        .into_iter()
        .map(|sample| (sample as f32) / 32767.0)
        .collect();

    let mut speech_started = false;

    let mut elapsed_time_ms = 0;

    let sample_rate = 8000;

    let mut markers: Vec<Marker> = Vec::new();

    for chunk in data.chunks(get_window_size(sample_rate)) {
        let chunk_array = ndarray::Array1::from_iter(chunk.to_owned());

        // making it a 2D array
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
            if value > &0.5 && !speech_started {
                markers.push(Marker {
                    kind: SpeechEvent::SpeechStarted,
                    time_in_ms: elapsed_time_ms,
                });

                println!(
                    "{} detected speech start ({})",
                    format_time(elapsed_time_ms),
                    value
                );

                speech_started = true;
            } else if value < &0.35 && speech_started {
                markers.push(Marker {
                    kind: SpeechEvent::SpeechEnded,
                    time_in_ms: elapsed_time_ms,
                });

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

    Ok(markers)
}

pub fn run_file(path: &str) -> anyhow::Result<()> {
    let start_time = Instant::now();

    let file = std::fs::read(std::path::Path::new(&path)).expect("Unable to read file");

    let model = load_model().expect("failed to load model");

    analyse_data(Arc::new(model), file).unwrap();

    println!("Elapsed time: {:.2?}", start_time.elapsed());

    Ok(())
}
