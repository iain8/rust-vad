const BIAS: u8 = 0x84; /* Bias for linear code. */
const QUANT_MASK: u8 = 0xf;
const SEG_MASK: u8 = 0x70; /* Segment field mask. */
const SEG_SHIFT: u8 = 4; /* Left shift for segment number. */
const SIGN_BIT: u8 = 0x80; /* Sign bit for a byte. */

// mulaw byte to PCM sample magic
fn ulaw_to_linear(mut u_val: u8) -> i16 {
    /* Complement to obtain normal u-law value. */
    u_val = !u_val;

    /*
     * Extract and bias the quantization bits. Then
     * shift up by the segment number and subtract out the bias.
     */
    let mut t: i16 = (((u_val & QUANT_MASK) << 3) + BIAS) as i16;

    let shift = (u_val & SEG_MASK) >> SEG_SHIFT;

    t <<= shift;

    if (u_val & SIGN_BIT) > 0 {
        t = (BIAS as i16) - t;
    } else {
        t -= BIAS as i16;
    }

    t
}

// Decode a PCM mulaw file into raw PCM samples
pub fn decode(data: Vec<u8>) -> Vec<i16> {
    let mut output_buffer: Vec<i16> = Vec::new();

    for byte in data.into_iter() {
        let decoded_val = ulaw_to_linear(byte);

        output_buffer.push(decoded_val);
    }

    output_buffer
}
