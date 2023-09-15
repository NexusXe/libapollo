// use crate::parameters::{FIGURES_FRAME_SIZE, FiguresFrameArray};

// pub struct FiguresFrame {
//     data: FiguresFrameArray,
//     pos: usize,
// }

// pub fn make_figuresframe(data: &[u8]) -> FiguresFrame {
//     todo!();
// }

pub struct FloatMap<TI, TO> {
    slope32: f32,
    slope64: f64,
    input_range: [TI; 2],
    output_range: [TO; 2]
}

const U24_OUTPUT_START: u32 = 0;
const U24_OUTPUT_END: u32 = 2u32.pow(24) - 1u32;
const LAT_INPUT_START: isize = -90;
const LAT_INPUT_END: isize = 90;
const LONG_INPUT_START: isize = -180;
const LONG_INPUT_END: isize = 180;

const LATITUDE_MAP: FloatMap<f32, u32> = FloatMap {
    slope32: ((U24_OUTPUT_END - U24_OUTPUT_START) as f64 / (LAT_INPUT_END - LAT_INPUT_START) as f64) as f32,
    slope64: ((U24_OUTPUT_END - U24_OUTPUT_START) as f64 / (LAT_INPUT_END - LAT_INPUT_START) as f64),
    input_range: [LAT_INPUT_START as f32, LAT_INPUT_END as f32],
    output_range: [U24_OUTPUT_START, U24_OUTPUT_END],
};

const LONGITUDE_MAP: FloatMap<f32, u32> = FloatMap {
    slope32: ((U24_OUTPUT_END - U24_OUTPUT_START) as f64 / (LONG_INPUT_END - LONG_INPUT_START) as f64) as f32,
    slope64: ((U24_OUTPUT_END - U24_OUTPUT_START) as f64 / (LONG_INPUT_END - LONG_INPUT_START) as f64),
    input_range: [LONG_INPUT_START as f32, LONG_INPUT_END as f32],
    output_range: [U24_OUTPUT_START, U24_OUTPUT_END],
};

pub fn main() {
    // let x = map_float(180.0);
    // println!("{:02x?}", x);
    // dbg!(from_24_bit(x));
}

// pub fn sphere_to_48bit(latitude: f32, longitude: f32) -> (u64, f32, f32) {
//     let latitude_rad = latitude * PI / 180.0;
//     let longitude_rad = longitude * PI / 180.0;
//     let x = latitude_rad.cos() * longitude_rad.cos();
//     let y = latitude_rad.cos() * longitude_rad.sin();
//     let z = latitude_rad.sin();
//     ((x * 100_000_000.0) as u64, y, z)
// }

const fn get_byte(n: usize, number: u32) -> u8 {
    (number >> (8 * n)) as u8
}

type U24Arr = [u8; 3];
type U48Arr = [U24Arr; 2];

const fn to_24_bit(n: u32) -> U24Arr {
    [get_byte(0, n), get_byte(1, n), get_byte(2, n)]
}

const fn from_24_bit(n: U24Arr) -> u32 {
    let mut output: u32 = 0;
    let mut i: usize = 0;
    while i < n.len() {
        output = output | ((n[i] as u32) << 8*i);
        i += 1;
    }
    output
}

const fn map_float(_floatmap: FloatMap<f32, u32>, input: f32) -> U24Arr {
    let input_start = _floatmap.input_range[0];
    //let input_end = _floatmap.input_range[1];
    let output_start = _floatmap.output_range[0];
    //let output_end = _floatmap.output_range[1];
    let slope = _floatmap.slope32;

    let output: u32 = (output_start as f32 + slope * (input - input_start as f32)) as u32;
    debug_assert!(output <= 2u32.pow(24));
    to_24_bit(output)
}

#[inline]
const fn demap_float(_floatmap: FloatMap<f32, u32>, input: U24Arr) -> f32 {
    ((-(_floatmap.output_range[0] as f64) + (_floatmap.slope64 * (_floatmap.input_range[0] as f64)) + from_24_bit(input) as f64)/_floatmap.slope64) as f32
}

pub const fn coords_to_u48_arr(lat: f32, long: f32) -> U48Arr {
    [map_float(LATITUDE_MAP, lat), map_float(LONGITUDE_MAP, long)]
}

pub const fn u48_arr_to_coords(data: U48Arr) -> (f32, f32) {
    (demap_float(LATITUDE_MAP, data[0]), demap_float(LONGITUDE_MAP, data[1]))
}
