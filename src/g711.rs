const BIAS: u8 = 0x84; /* Bias for linear code. */
const QUANT_MASK: u8 = 0xf;
const SEG_MASK: u8 = 0x70; /* Segment field mask. */
const SEG_SHIFT: u8 = 4; /* Left shift for segment number. */
const SIGN_BIT: u8 = 0x80; /* Sign bit for a A-law byte. */

fn ulaw_to_linear(mut u_val: u8) -> i16 {
    /* Complement to obtain normal u-law value. */
    u_val = !u_val;

    /*
     * Extract and bias the quantization bits. Then
     * shift up by the segment number and subtract out the bias.
     */
    let mut t: i16 = (((u_val & QUANT_MASK) << 3) + BIAS) as i16;

    let shift = (u_val & SEG_MASK) >> SEG_SHIFT;

    t = t << shift;

    if (u_val & SIGN_BIT) > 0 {
        t = (BIAS as i16) - t;
    } else {
        t = t - (BIAS as i16);
    }

    t
}

pub fn decode(path: &str) -> Vec<i16> {
    let file = std::fs::read(std::path::Path::new(&path)).expect("Unable to read file");

    let mut output_buffer: Vec<i16> = Vec::new();

    // let mut max: i16 = 0;

    for byte in file.into_iter() {
        let decoded_val = ulaw_to_linear(byte);
        // println!("decoded: {}", decoded_val);
        // if decoded_val > max {
        //     max = decoded_val;
        // }

        output_buffer.push(decoded_val);
    }

    // output_buffer.iter().map(|val| (128 / val) as f32).collect()
    output_buffer
}
