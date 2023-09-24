use core::ops::Range;

use crate::parameters::{U24Arr, U48Arr};

/// Returns byte `n` from `number`.
const fn get_byte(n: usize, number: u32) -> u8 {
    (number >> (8 * n)) as u8
}

const fn to_24_bit(n: u32) -> U24Arr {
    [get_byte(0, n), get_byte(1, n), get_byte(2, n)]
}

pub struct FloatMap<TI, TO> {
    slope32: f32,
    slope64: f64,
    input_range: Range<TI>,
    output_range: Range<TO>,
}

impl FloatMap<f32, u32> {
    /// Constructs a new [FloatMap] provided:
    /// 
    /// An input [Range], where [Range::start] is the smallest (inclusive) value that will be given to be mapped and [Range::end] is the largest (inclusive).
    /// 
    /// An output [Range], where [Range::start] is the output value for that smallest input, and [Range::end] is the output value for that largest input.
    pub const fn new(_input_range: Range<f32>, _output_range: Range<u32>) -> Self {
        let _slope64: f64 = (_output_range.start - _output_range.end) as f64
            / (_input_range.end - _input_range.start) as f64;
        FloatMap {
            slope32: _slope64 as f32,
            slope64: _slope64,
            input_range: _input_range,
            output_range: _output_range,
        }
    }
    /// Maps an input float between `self.output_range[0]` and `self.output_range[1]`.
    pub const fn map(&self, input: f32) -> U24Arr {
        debug_assert!(input >= self.input_range.start);
        debug_assert!(input <= self.input_range.end);

        let output: u32 = (self.output_range.start as f32
            + self.slope32 * (input - self.input_range.start as f32))
            as u32;

        debug_assert!(output >= self.output_range.start);
        debug_assert!(output <= self.output_range.end);
        debug_assert!(output <= 2u32.pow(24));
        to_24_bit(output)
    }
    /// Undoes the mapping preformed by [map]
    pub const fn demap(&self, input: U24Arr) -> f32 {
        ((-(self.output_range.start as f64)
            + (self.slope64 * (self.input_range.start as f64))
            + from_24_bit(input) as f64)
            / self.slope64) as f32
    }
}


const U24_OUTPUT_START: u32 = 0;
const U24_OUTPUT_END: u32 = 2u32.pow(24) - 1u32;
const LAT_INPUT_START: isize = -90;
const LAT_INPUT_END: isize = 90;
const LONG_INPUT_START: isize = -180;
const LONG_INPUT_END: isize = 180;

/// Constructs our constant [LATITUDE_MAP] and [LONGITUDE_MAP].
pub const fn make_floatmap_f32(
    _input_range: Range<isize>,
    _output_range: Range<u32>,
) -> FloatMap<f32, u32> {
    let _slope64: f64 = (_output_range.start - _output_range.end) as f64
        / (_input_range.end - _input_range.start) as f64;
    FloatMap {
        slope32: _slope64 as f32,
        slope64: _slope64,
        input_range: _input_range.start as f32.._input_range.end as f32,
        output_range: _output_range,
    }
}

/// Constant FloatMap data for converting an [f32] latitude into a [U24Arr]
const LATITUDE_MAP: FloatMap<f32, u32> = make_floatmap_f32(LAT_INPUT_START..LAT_INPUT_END, U24_OUTPUT_START..U24_OUTPUT_END);
/// Constant FloatMap data for converting an [f32] longitude into a [U24Arr]
const LONGITUDE_MAP: FloatMap<f32, u32> = make_floatmap_f32(LONG_INPUT_START..LONG_INPUT_END, U24_OUTPUT_START..U24_OUTPUT_END);

/// Converts a [U24Arr] back into a [u32]. This conversion is extremely fast; it just `and`s the input with `16777215`.
/// https://godbolt.org/z/nch3qc13P
const fn from_24_bit(n: U24Arr) -> u32 {
    let mut output: u32 = 0;
    let mut i: usize = 0;
    while i < n.len() {
        output = output | ((n[i] as u32) << 8 * i);
        i += 1;
    }
    output
}



pub const fn coords_to_u48_arr(lat: f32, long: f32) -> U48Arr {
    [LATITUDE_MAP.map(lat), LONGITUDE_MAP.map(long)]
}

pub const fn u48_arr_to_coords(data: U48Arr) -> (f32, f32) {
    (
        LATITUDE_MAP.demap(data[0]),
        LONGITUDE_MAP.demap(data[1]),
    )
}

pub type StatusBools = [bool; 8];

pub const fn pack_bools_to_byte(bools: StatusBools) -> u8 {
    let mut i: usize = 0;
    let mut packed_bools: u8 = 0b00000000;
    while i < bools.len() {
        packed_bools |= (bools[i] as u8) << i;
        i += 1;
    }
    packed_bools
}

/// Unpacks a u8 into a [StatusBools], rather quickly.
///
/// Some implementation info:
///
/// unpacked bools will always be either 0x00 or 0x01*, so we can
/// tell the compiler that it doesn't have to convert to a bool
/// (which it would do with a `cmp` to 0). this is bit-identical
/// to `bools[i] = std::mem::transmute(x);` but more code-safe
///
/// *technically, rust checks the value of a bool by only checking
/// its least-significant bit; therefore, a bool with the value
/// `0b1110` is `false`, whereas `0b1111` is `true`. however, the
/// compiler is not fond of this and will lead to `SIGILL`s,
/// i think because the compiler will put unaligned values into
/// the "excess" space of the bool, yet the computer is still
/// writing the "excess" bits of the non-standard bool
/// this causes problems, as one may expect, and is technically
/// a memory access violation (fun!)
pub const fn unpack_bools(packed_bools: u8) -> StatusBools {
    let mut i: usize = 0;
    let mut bools: StatusBools = [false; 8];
    const MASK_SET: [u8; 8] = [1, 2, 4, 8, 16, 32, 64, 128];

    while i < 8 {
        let x = (packed_bools & MASK_SET[i]) >> i;

        if (x != 0) && (x != 1) {
            unreachable!();
        }
        bools[i] = x != 0;
        i += 1;
    }
    bools
}

pub const fn make_statuses(status_bools: [StatusBools; 2]) -> [u8; 2] {
    [
        pack_bools_to_byte(status_bools[0]),
        pack_bools_to_byte(status_bools[1]),
    ]
}

pub const fn make_status_data(lat: f32, long: f32, status: [u8; 2]) -> [[u8; 4]; 2] {
    let mut packed_status = [[0u8; 4]; 2];
    let packed_coords = coords_to_u48_arr(lat, long);

    let mut i: usize = 0;
    let mut x: usize = 0;

    while x < packed_coords.len() {
        while i < packed_coords[x].len() {
            if x >= packed_coords.len() || i >= packed_coords[x].len() {
                // ensures that the compiler does not generate code for an
                // index out of range error, which is not possible here
                unreachable!();
            }

            packed_status[x][i] = packed_coords[x][i];
            i += 1;
        }
        packed_status[x][i] = status[x];
        x += 1
    }
    packed_status
}

#[cfg(test)]
mod tests {
    const EXAMPLE_STATUSES: [StatusBools; 2] = [
        [true, true, false, true, true, true, true, false],
        [false, true, false, false, false, false, false, false],
    ];
    use super::*;

    #[test]
    fn check_status_packing() {
        const ATTEMPT: [u8; 2] = make_statuses(EXAMPLE_STATUSES);
        const EXPECTED_OUTPUT: [u8; 2] = [0b01111011u8, 0b00000010u8];
        assert_eq!(
            ATTEMPT, EXPECTED_OUTPUT,
            "\nexpected: [{:08b}, {:08b}]\nfound:    [{:08b}, {:08b}]",
            EXPECTED_OUTPUT[0], EXPECTED_OUTPUT[1], ATTEMPT[0], ATTEMPT[1]
        );
    }
}
