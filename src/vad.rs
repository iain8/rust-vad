use std::time::Instant;

use ndarray::{Array3, Axis};
use ort::{inputs, Error, Session};

use crate::ffmpeg::ffdecode;

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

pub fn run_file(path: &str) -> anyhow::Result<()> {
    let start_time = Instant::now();

    let model = load_model().expect("failed to load model");

    let samples = ffdecode(path);
    //
    // let samples = decode(path);
    println!("samples max {:?}", samples.iter().max().unwrap());
    // let samples = load_and_decode_s16le(path);
    let modifier = 1.0 / samples.iter().max().unwrap().abs() as f32;
    // let max = samples.iter().max().unwrap();

    let mut data_max = 0.0; // 0.0;

    // create buffer of normalised(?) floats
    let data: Vec<f32> = samples
        .iter()
        .map(|input| {
            let normal = (input.abs() as f32) * modifier;

            if normal > data_max {
                data_max = normal;
            }

            if normal > 1.0 {
                return 1.0;
            }

            normal
        })
        .collect();
    let mut speech_started = false;
    // println!("data {:?}", data);
    println!("data max {:?}", data_max);

    let mut elapsed_time_ms = 0;

    let sample_rate = 8000; // TODO: hmm

    for chunk in data.chunks(get_window_size(sample_rate)) {
        // println!("chunk {:?}", chunk);
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
            } else {
                // println!("silence!");
            }
        }

        elapsed_time_ms += 100;
    }

    println!("Elapsed time: {:.2?}", start_time.elapsed());

    Ok(())
}
