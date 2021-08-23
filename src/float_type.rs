const BIAS: i32 = 127;
const RADIX: f32 = 2.0;

//given a value 1.898 × 10^27
// a sign, which is implied in our two examples, would be present for negative numbers
// (-)
// the mantissa, also known as the significand, can be thought of as being the value in
// question (1.898 and 3.801)
// the radix, also known as the base, is the value that is raised to the power of the
// exponent (10 in both of our examples)
// the exponent, which describes the scale of the value (27 and -4)

//extracting the bits of those values from the container
pub fn deconstruct_f32(n: f32) -> (u32, u32, u32) {
    //convert to a u32 so as to optimally perform bitwise manipulation on num
    let n_: u32 = unsafe { std::mem::transmute(n) };
    //strip 31 unwanted bits away (since it is an f32) by shifting them into nowhere, leaving only the signed bit
    //add 31 0's at the start
    let sign = (n_ >> 31) & 1;
    //filter out the top bit (0xff is 255) with a logical AND mask, then strip 23 unwanted bits away
    //Only non-zero bits in the mask can pass through.
    let exponent = (n_ >> 23) & 0xff;
    //only retain 23 least significant bits via an AND mask
    let fraction = 0b00000000_01111111_11111111_11111111 & n_;

    //the mantissa is also called the fraction
    (sign, exponent, fraction)
}

//decode each value from its raw bit pattern to its actual value
pub fn decode_f32_parts(sign: u32, exponent: u32, fraction: u32) -> (f32, f32, f32) {
    //convert signed bit to 1.0 or -1.0
    let signed_1 = (-1.0_f32).powf(sign as f32);
    //exponrnt must be i32 incase subtracting the bias leads to a negative value
    let exponent = (exponent as i32) - BIAS;
    //cast to f32 so as to be used as exponential
    let exponent = RADIX.powf(exponent as f32);
    //We start by assuming that the implicit 24th bit is set.
    //That has the upshot of defaulting the mantissa’s value as 1.
    let mut mantissa: f32 = 1.0;

    for i in 0..23_u32 {
        //at eash iteartion, create an AND mask of a single bit in the position we are interested in
        // eg When i equals 5, the bit pattern is 0b00000000_00000000_00000000_00100000
        let one_at_bit_i = 1 << i;
        //any non zero result means the bit is within fraction
        if (one_at_bit_i & fraction) != 0 {
            // To arrive at the decimal value of the bit at i, we find 2i-23. -23 means that the result gets smaller when i is close
            // to 0, as desired.
            mantissa += 2_f32.powf((i as f32) - 23.0)
        }
    }
    (signed_1, exponent, mantissa)
}

//convert from scientific notation to an ordinary number
pub fn f32_from_parts(sign: f32, exponent: f32, mantissa: f32) -> f32 {
    sign * exponent * mantissa
}

//represent decimal numbers in a single byte using point number format
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Q7(i8);

//converting f64 to Q7
impl From<f64> for Q7 {
    fn from(n: f64) -> Self {
        //out of bounds are coereced to the max of the Q7 range 2^7
        if n >= 1.0 {
            Q7(127)
        } else if n <= -1.0 {
            Q7(-128)
        } else {
            Q7((n * 128.0) as i8)
        }
    }
}

//converting from Q7 to f64
impl From<Q7> for f64 {
    fn from(n: Q7) -> f64 {
        //this is a mathematical equivalent to iterate through the bits
        //and multiply it to it's weight
        (n.0 as f64) * 2_f64.powf(-7.0)
    }
}

//convert f32 values using rust machinery
impl From<f32> for Q7 {
    fn from(n: f32) -> Self {
        Q7::from(n as f64)
    }
}
impl From<Q7> for f32 {
    fn from(n: Q7) -> Self {
        //converting f64 to f32 reduces transition
        f64::from(n) as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn out_of_bound() {
        assert_eq!(Q7::from(10.), Q7::from(1.));
        assert_eq!(Q7::from(-10.), Q7::from(-1.))
    }

    #[test]
    fn f32_to_q7() {
        let n1: f32 = 0.7;
        let q1 = Q7::from(n1);

        let n2 = -0.4;
        let q2 = Q7::from(n2);

        let n3 = 123.0;
        let q3 = Q7::from(n3);

        assert_eq!(q1, Q7(89));
        assert_eq!(q2, Q7(-51));
        assert_eq!(q3, Q7(127))
    }

    #[test]
    fn q7_to_f32() {
        let q1 = Q7::from(0.7);
        let n1 = f32::from(q1);
        assert_eq!(n1, 0.6953125);

        let q2 = Q7::from(n1);
        let n2 = f32::from(q2);
        assert_eq!(n1, n2);
    }
}

//generating f32 that lies between 0 and 1
pub fn generate_f32(n: u8) -> f32 {
    //underscore mark the sign, mantissa, and exponent bounderies
    let base: u32 = 0b0_01111110_00000000000000000000000;
    //align n to 32 bits then increase it's value by shifting 15 places left
    let large_n = (n as u32) << 15;
    //take a bitwise or merging the base and input value
    let f32_bits = base | large_n;
    //interpret f32_bits which is of type u32 as an f32
    let m = f32::from_bits(f32_bits);
    2.0 * (m - 0.5)
}
