use symphonia::{
    core::{
        audio::{Layout, SampleBuffer},
        codecs::{CodecParameters, Decoder, CODEC_TYPE_PCM_MULAW, CODEC_TYPE_PCM_S16LE},
        errors::Error,
        formats::Packet,
    },
    default::codecs::PcmDecoder,
};

use crate::g711;

// ulaw files are:
// PCM signed 16-bit little-endian
// 8kHz
// 1 channel
// buffer size 65536?

pub fn get_s16le_decoder() -> Result<PcmDecoder, Error> {
    let codec_params = CodecParameters {
        bits_per_sample: Some(16),
        bits_per_coded_sample: Some(16),
        channels: None,
        channel_layout: Some(Layout::Mono),
        codec: CODEC_TYPE_PCM_S16LE,
        delay: None,
        extra_data: None,
        frames_per_block: None,
        max_frames_per_packet: Some(3000),
        n_frames: None,
        packet_data_integrity: false,
        padding: None,
        sample_rate: Some(8000),
        start_ts: 0,
        time_base: None,
        verification_check: None,
        ..CodecParameters::default()
    };

    PcmDecoder::try_new(&codec_params, &Default::default())
}

pub fn get_mulaw_decoder() -> Result<PcmDecoder, Error> {
    let codec_params = CodecParameters {
        bits_per_sample: Some(16),
        bits_per_coded_sample: Some(16),
        channels: None,
        channel_layout: Some(Layout::Mono),
        codec: CODEC_TYPE_PCM_MULAW,
        delay: None,
        extra_data: None,
        frames_per_block: None,
        max_frames_per_packet: Some(3000),
        n_frames: None,
        packet_data_integrity: false,
        padding: None,
        sample_rate: Some(8000),
        start_ts: 0,
        time_base: None,
        verification_check: None,
        ..CodecParameters::default()
    };

    PcmDecoder::try_new(&codec_params, &Default::default())
}

pub fn load_and_decode_s16le(path: &str) -> Vec<f32> {
    let mut pcm_decoder = get_s16le_decoder().unwrap();

    let data = g711::decode(path);
    // TODO: just make this a boolean or something or skip it altogether
    let mut sample_buffer = None;
    let mut audio_buffer: Vec<f32> = Vec::new();

    for chunk in data {
        let packet = Packet::new_from_slice(0, 0, 0, &chunk.to_be_bytes());

        match pcm_decoder.decode(&packet) {
            Ok(audio_buf) => {
                if sample_buffer.is_none() {
                    let spec = *audio_buf.spec();
                    let duration = audio_buf.capacity() as u64;

                    sample_buffer = Some(SampleBuffer::<f32>::new(duration, spec));
                }

                if let Some(buf) = &mut sample_buffer {
                    buf.copy_interleaved_ref(audio_buf);
                    // println!("samples {:?}", buf.samples());
                    audio_buffer.append(&mut buf.samples().to_vec());
                }
            }
            Err(e) => {
                println!("decoding error: {}", e);
            }
        }
    }

    audio_buffer
}

pub fn load_and_decode(path: &str) -> Vec<f32> {
    let mut pcm_decoder = get_mulaw_decoder().unwrap();

    let file = std::fs::read(path).unwrap();
    // TODO: just make this a boolean or something or skip it altogether
    let mut sample_buffer = None;
    let mut audio_buffer: Vec<f32> = Vec::new();

    for chunk in file.iter() {
        let packet = Packet::new_from_slice(0, 0, 0, &chunk.to_le_bytes());

        match pcm_decoder.decode(&packet) {
            Ok(audio_buf) => {
                if sample_buffer.is_none() {
                    let spec = *audio_buf.spec();
                    let duration = audio_buf.capacity() as u64;

                    sample_buffer = Some(SampleBuffer::<f32>::new(duration, spec));
                }

                if let Some(buf) = &mut sample_buffer {
                    buf.copy_interleaved_ref(audio_buf);
                    // println!("samples {:?}", buf.samples());
                    audio_buffer.append(&mut buf.samples().to_vec());
                }
            }
            Err(e) => {
                println!("decoding error: {}", e);
            }
        }
    }

    audio_buffer
}
