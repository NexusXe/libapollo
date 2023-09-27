use core::ops::Range;

use crate::parameters::{U24Arr, U48Arr};
use crate::telemetry::BlockData;

/// Returns byte `n` from `number`.
/// This function is designed for use in [to_24_bit], where it's [never actually called](https://godbolt.org/z/z7q1nqG99).
/// Instead, it largely exists in Rust logic.
#[inline]
const fn get_byte(n: usize, number: u32) -> u8 {
    (number >> (8 * n)) as u8
}

/// Converts a [u32] into a [U24Arr]. This conversion has **zero overhead**. Just like [get_byte], it's [never actually called](https://godbolt.org/z/z7q1nqG99).
const fn to_24_bit(n: u32) -> U24Arr {
    [get_byte(0, n), get_byte(1, n), get_byte(2, n)]
}

/// Converts a [U24Arr] back into a [u32]. This conversion is extremely fast; it just `and`s the input with `2^24 - 1`, which effectively prepends a byte of zeroes to the beginning, yielding a [u32].
/// This is why I deal with 24-bit data in this way: conversion is extremely fast. This function is 2 instrctions, a `mov` and an `and`. That's it.
/// https://godbolt.org/z/nch3qc13P
/// In addition to this, and in a similar vein to [to_24_bit], when this code is called in a function, nothing will actually happen, as the compiler will just change the type of the data in a
/// 32-bit register and call it a day.
const fn from_24_bit(n: U24Arr) -> u32 {
    let mut output: u32 = 0;
    let mut i: usize = 0;
    while i < n.len() {
        output = output | ((n[i] as u32) << 8 * i);
        i += 1;
    }
    output
}

pub struct FloatMap {
    slope64: f64,
    input_range: Range<f64>,
    output_range: Range<u32>,
}

impl FloatMap {
    /// Constructs a new [FloatMap] provided:
    ///
    /// An input [Range], where [Range::start] is the smallest (inclusive) value that will be given to be mapped and [Range::end] is the largest (inclusive).
    ///
    /// An output [Range], where [Range::start] is the output value for that smallest input, and [Range::end] is the output value for that largest input.
    pub const fn new(_input_range: Range<f64>, _output_range: Range<u32>) -> Self {
        FloatMap {
            slope64: (_output_range.start - _output_range.end) as f64
            / (_input_range.end - _input_range.start),
            input_range: _input_range,
            output_range: _output_range,
        }
    }
    /// Maps an input float between `self.output_range[0]` and `self.output_range[1]`.
    pub const fn map(&self, input: f64) -> U24Arr {
        debug_assert!(input >= self.input_range.start);
        debug_assert!(input <= self.input_range.end);

        let output: u32 = (self.output_range.start as f64
            + self.slope64 * (input - self.input_range.start))
            as u32;

        debug_assert!(output >= self.output_range.start);
        debug_assert!(output <= self.output_range.end);
        debug_assert!(output <= 2u32.pow(24));
        to_24_bit(output)
    }
    /// Undoes the mapping preformed by map.
    pub const fn demap(&self, input: U24Arr) -> f64 {
        (-(self.output_range.start as f64)
            + (self.slope64 * (self.input_range.start as f64))
            + from_24_bit(input) as f64)
            / self.slope64
    }
}

const U24_OUTPUT_START: u32 = 0;
const U24_OUTPUT_END: u32 = 2u32.pow(24) - 1u32;
const LAT_INPUT_START: isize = -90;
const LAT_INPUT_END: isize = 90;
const LONG_INPUT_START: isize = -180;
const LONG_INPUT_END: isize = 180;

/// Constructs [FloatMap]s given input and output [Range]s.
pub const fn make_floatmap_f64(
    _input_range: Range<isize>,
    _output_range: Range<u32>,
) -> FloatMap {
    FloatMap {
        slope64: (_output_range.end - _output_range.start) as f64
        / (_input_range.end - _input_range.start) as f64,
        input_range: _input_range.start as f64.._input_range.end as f64,
        output_range: _output_range,
    }
}

/// Constant FloatMap data for converting an [f64] latitude into a [U24Arr]
const LATITUDE_MAP: FloatMap = make_floatmap_f64(
    LAT_INPUT_START..LAT_INPUT_END,
    U24_OUTPUT_START..U24_OUTPUT_END,
);
/// Constant FloatMap data for converting an [f64] longitude into a [U24Arr]
const LONGITUDE_MAP: FloatMap = make_floatmap_f64(
    LONG_INPUT_START..LONG_INPUT_END,
    U24_OUTPUT_START..U24_OUTPUT_END,
);

pub const fn coords_to_u48_arr(lat: f64, long: f64) -> U48Arr {
    [LATITUDE_MAP.map(lat), LONGITUDE_MAP.map(long)]
}

pub const fn u48_arr_to_coords(data: U48Arr) -> (f64, f64) {
    (LATITUDE_MAP.demap(data[0]), LONGITUDE_MAP.demap(data[1]))
}

pub type StatusBoolsArray = [bool; 8];

pub struct StatusFlagsLat {
    lat_sign: bool,
    long_sign: bool,
    voltage_sign: bool,
    gps_lock: bool,
    altitude: [bool; 4], // in hundreds of meters
}

pub type StatusFlagsLong = u8; // battery voltage
// 0: lat sign
// 1: long sign
// 2: voltage sign
// 3: gps lock state
// 4..7: altitude in hundreds of meters

// second byte: battery voltage

pub enum StatusFlags {
    StatusFlagsLat(StatusFlagsLat),
    StatusFlagsLong(StatusFlagsLat),
}

impl Into<u8> for StatusFlags {
    /// TODO: finish
    fn into(self) -> u8 {
        let mut output: u8 = 0u8;
        match self {
            StatusFlags::StatusFlagsLat(_flagslat) => {
                todo!()
            }

            StatusFlags::StatusFlagsLong(_flagslong) => {
                todo!()
            }
        }

        output
    }
}

/// Packs 8 bools into a u8, where the first bit in the input is the least-significant
/// bit in the output u8. For example, a bool array of `[TFFFFFTF]` would result in a
/// u8 whose binary representation (LE) would be `0b01000001`.
///
/// The fact that this could potentially be "backwards" makes it much faster at runtime.
///
/// TODO: Ensure portability, especially when the transmitter and receiver differ in
/// endianness.
pub const fn pack_bools_to_byte(bools: StatusBoolsArray) -> u8 {
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
pub const fn unpack_bools(packed_bools: u8) -> StatusBoolsArray {
    let mut i: usize = 0;
    let mut bools: StatusBoolsArray = [false; 8];
    const MASK_SET: [u8; 8] = [1, 2, 4, 8, 16, 32, 64, 128];

    while i < 8 {
        let x = (packed_bools & MASK_SET[i]) >> i;

        if (x != 0) & (x != 1) { // using the proper logical and can cause a SIGILL..?
            unreachable!();
        }
        bools[i] = x != 0;
        i += 1;
    }
    bools
}

/// Packs a [U24Arr] and a packed status [u8] into a [BlockData::DynData] object,
/// ready to be incorporated into a [crate::telemetry::Block]
const fn make_packed_status(data: U24Arr, status: u8) -> BlockData {
    BlockData::DynData(Some([data[0], data[1], data[2], status]))
}

/// Provided a latitude [f64], longitude [f64], and 2 [StatusBools] arrays,
/// make two [BlockData::DynData]s that are ready to be incorporated into a
/// [crate::telemetry::Block].
pub const fn make_status_data(
    lat: f64,
    long: f64,
    status_bools: [StatusBoolsArray; 2],
) -> [BlockData; 2] {
    let packed_coords = coords_to_u48_arr(lat, long);

    [
        make_packed_status(packed_coords[0], pack_bools_to_byte(status_bools[0])),
        make_packed_status(packed_coords[1], pack_bools_to_byte(status_bools[1])),
    ]
}

/// Unpacks a 4-byte block into its 24-bit number (represented as a [U24Arr]) and its [StatusBools].
pub const fn unpack_status_data(status_block: [u8; 4]) -> (U24Arr, StatusBoolsArray) {
    let mut data: [u8; 3] = [0u8; 3];
    data[0] = status_block[0];
    data[1] = status_block[1];
    data[2] = status_block[2];

    (data, unpack_bools(status_block[3]))
}

/// Unpacks the packed latitude and longitude blocks into their recovered floating-point
/// values and their packed flags
pub const fn unpack_status_blocks(status_blocks: [[u8; 4]; 2]) -> [(f64, StatusBoolsArray); 2] {
    let (lat, status_1): (U24Arr, StatusBoolsArray) = unpack_status_data(status_blocks[0]);
    let (long, status_2 ): (U24Arr, StatusBoolsArray) = unpack_status_data(status_blocks[1]);

    [(LATITUDE_MAP.demap(lat), status_1), (LONGITUDE_MAP.demap(long), status_2)]
}


#[cfg(test)]
mod tests {
    const EXAMPLE_STATUSES: [StatusBoolsArray; 2] = [
        [true, true, false, true, true, true, true, false],
        [false, true, false, false, false, false, false, false],
    ];
    const LAT: f64 = 38.897957;
    const LONG: f64 = -77.036560;
    use super::*;

    const fn make_statuses(status_bools: [StatusBoolsArray; 2]) -> [u8; 2] {
        [
            pack_bools_to_byte(status_bools[0]),
            pack_bools_to_byte(status_bools[1]),
        ]
    }

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

    #[test]
    fn check_floatmap_accuracy() {
        const MAPPED_LAT: U24Arr = LATITUDE_MAP.map(LAT);
        const MAPPED_LONG: U24Arr = LONGITUDE_MAP.map(LONG);
        const RCV_LAT: f64 = LATITUDE_MAP.demap(MAPPED_LAT);
        const RCV_LONG: f64 = LONGITUDE_MAP.demap(MAPPED_LONG);

        let lat_delta: f64 = f64::abs(LAT - RCV_LAT);
        let long_delta: f64 = f64::abs(LONG - RCV_LONG);

        assert!(
            lat_delta < 0.0001,
            "Latitude imprecision error, delta is {}",
            lat_delta
        );
        assert!(
            long_delta < 0.0001,
            "Longitude imprecision error, delta is {}",
            long_delta
        );
    }

    #[test]
    fn test_lat_long_packing() {
        const MAPPED_COORDS: U48Arr = coords_to_u48_arr(LAT, LONG);
        const DEMAPPED_COORDS: (f64, f64) = u48_arr_to_coords(MAPPED_COORDS);
        let lat_delta: f64 = f64::abs(LAT - DEMAPPED_COORDS.0);
        let long_delta: f64 = f64::abs(LONG - DEMAPPED_COORDS.1);

        assert!(
            lat_delta < 0.0001,
            "Latitude imprecision error, delta is {}",
            lat_delta
        );
        assert!(
            long_delta < 0.0001,
            "Longitude imprecision error, delta is {}",
            long_delta
        );
    }

    #[test]
    fn test_coord_status_packing() {
        const EXPECTED_STATUS_INTS: [u8; 2] = make_statuses(EXAMPLE_STATUSES);
        const EXPECTED_LAT: U24Arr = LATITUDE_MAP.map(LAT);
        const EXPECTED_LONG: U24Arr = LONGITUDE_MAP.map(LONG);

        const PACKED_STATUS_DATA: [BlockData; 2] = make_status_data(LAT, LONG, EXAMPLE_STATUSES);
        const LAT_BLOCK: BlockData = PACKED_STATUS_DATA[0];
        const LONG_BLOCK: BlockData = PACKED_STATUS_DATA[1];

        assert!(
            &LAT_BLOCK.get_data()[0..3] == &EXPECTED_LAT,
            "Expected: {:08X?}\n Found:   {:08X?}",
            &EXPECTED_LAT,
            &LAT_BLOCK.get_data()[0..3]
        );
        assert!(
            &LONG_BLOCK.get_data()[0..3] == &EXPECTED_LONG,
            "Expected: {:08X?}\n Found:   {:08X?}",
            &EXPECTED_LONG,
            &LONG_BLOCK.get_data()[0..3]
        );
        assert!(
            &LAT_BLOCK.get_data()[3] == &EXPECTED_STATUS_INTS[0],
            "Expected: {:08X?}\n Found:   {:08X?}",
            &EXPECTED_STATUS_INTS[0],
            &LAT_BLOCK.get_data()[3]
        );
        assert!(
            &LONG_BLOCK.get_data()[3] == &EXPECTED_STATUS_INTS[1],
            "Expected: {:08X?}\n Found:   {:08X?}",
            &EXPECTED_STATUS_INTS[1],
            &LONG_BLOCK.get_data()[3]
        );
    }
}
