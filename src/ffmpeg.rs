use std::{ffi::CString, io::BufReader};

use byteorder::{ByteOrder, LittleEndian};
use ffmpeg_next::{
    codec::Parameters,
    decoder::{self, Audio},
    encoder::{self, Encoder},
    format::context,
};

pub fn ffdecode(path: &str) -> Vec<i16> {
    ffmpeg_next::init().unwrap();

    println!(
        "whaaa {:?}",
        std::env::current_dir().unwrap().with_file_name("oob.wav")
    );

    let mut input_ctx = open(&path);

    let mut output_ctx =
        ffmpeg_next::format::output(&std::env::current_dir().unwrap().with_file_name("oob.wav"))
            .unwrap();

    let input_stream = input_ctx
        .streams()
        .best(ffmpeg_next::media::Type::Audio)
        .ok_or(ffmpeg_next::Error::StreamNotFound)
        .unwrap();

    let mut decoder = get_decoder(input_stream.parameters());

    let mut output_stream = output_ctx
        .add_stream(encoder::find_by_name("pcm_s16le").unwrap())
        .unwrap();

    let parameters = input_stream.parameters();

    output_stream.set_parameters(parameters.clone());
    output_stream.set_metadata(input_stream.metadata().to_owned());
    output_ctx.write_header().unwrap();

    let encoder_ctx = ffmpeg_next::codec::context::Context::from_parameters(parameters).unwrap();

    let mut encoder = encoder_ctx.encoder().audio().unwrap();

    encoder.set_bit_rate(16);
    encoder.set_rate(8000);

    let mut encoder = encoder
        .open_as(encoder::find_by_name("pcm_s16le").unwrap())
        .unwrap();

    let mut samples: Vec<i16> = Vec::new();

    unsafe {
        let url = CString::new(path).unwrap();

        ffmpeg_next::sys::av_dump_format(input_ctx.as_mut_ptr(), 0, url.as_ptr(), 0);
    }

    let mut buffer: Vec<u8> = Vec::new();

    for (stream, packet) in input_ctx.packets() {
        decoder.send_packet(&packet).unwrap();

        let mut decoded = ffmpeg_next::frame::Audio::empty();

        while decoder.receive_frame(&mut decoded).is_ok() {
            let timestamp = decoded.timestamp();

            decoded.set_pts(timestamp);

            write_frame(&mut decoded, &mut encoder, &mut output_ctx);

            // encoder.send_frame(&decoded).unwrap();
            //
            // buffer.extend_from_slice(decoded.data(0));
            //
            // // output_ctx.
            //
            // for chunk in decoded.data(0).chunks_exact(2) {
            //     samples.push(LittleEndian::read_i16(chunk));
            // }
        }
    }

    decoder.send_eof().unwrap();

    output_ctx.write_trailer().unwrap();

    // let buf_reader = BufReader::new(buffer.as_slice());
    //
    // let spec = hound::WavSpec {
    //     channels: 1,
    //     sample_rate: 8000,
    //     bits_per_sample: 16,
    //     sample_format: hound::SampleFormat::Int,
    // };
    //
    // let reader = hound::WavReader::new(buf_reader).unwrap();
    //
    // println!("hmmm {:?}", reader.spec());

    samples
}

fn write_frame(
    frame: &mut ffmpeg_next::util::frame::Audio,
    encoder: &mut Encoder,
    output_ctx: &mut ffmpeg_next::format::context::Output,
) {
    let timestamp = frame.timestamp();

    frame.set_pts(timestamp);

    println!("sending frame {:?}", frame);

    encoder.send_frame(frame).unwrap();

    let mut packet = ffmpeg_next::Packet::empty();

    while encoder.receive_packet(&mut packet).is_ok() {
        packet.set_stream(0);
        packet.write_interleaved(output_ctx).unwrap();
    }
}

fn get_decoder(stream_params: Parameters) -> Audio {
    unsafe {
        let codec = decoder::find_by_name("pcm_mulaw").unwrap();

        let mut params = stream_params.clone();

        (*params.as_mut_ptr()).codec_id = (*codec.as_ptr()).id;
        (*params.as_mut_ptr()).bit_rate = 16;

        let mut codec_ctx = ffmpeg_next::codec::Context::from_parameters(params).unwrap();

        let result = ffmpeg_next::sys::avcodec_open2(
            codec_ctx.as_mut_ptr(),
            codec.as_ptr(),
            std::ptr::null_mut(),
        );

        if result != 0 {
            println!("failed to open codec {}", result);
        }

        codec_ctx.decoder().audio().unwrap()
    }
}

fn open<P: AsRef<std::path::Path>>(path: &P) -> context::Input {
    unsafe {
        let mut fmt_ctx = std::ptr::null_mut();

        let file_path = CString::new(path.as_ref().as_os_str().to_str().unwrap()).unwrap();

        let fmt_name = CString::new("s16le").unwrap();

        let fmt = ffmpeg_next::sys::av_find_input_format(fmt_name.as_ptr());

        let mut options = ffmpeg_next::Dictionary::new();

        options.set("codec:a", "pcm_mulaw");
        options.set("ar", "8k");
        options.set("ac", "1");

        let mut opts = options.disown();

        let open_result =
            ffmpeg_next::sys::avformat_open_input(&mut fmt_ctx, file_path.as_ptr(), fmt, &mut opts);

        if open_result != 0 {
            // TODO: an error, fail
        }

        let info_result = ffmpeg_next::sys::avformat_find_stream_info(fmt_ctx, &mut opts);

        if info_result != 0 {
            // TODO: an error... fail
            ffmpeg_next::sys::avformat_close_input(&mut fmt_ctx);
        }

        context::Input::wrap(fmt_ctx)
    }
}
