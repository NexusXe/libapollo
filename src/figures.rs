use crate::parameters::{FIGURES_FRAME_SIZE, FiguresFrameArray};

pub struct FiguresFrame {
    data: FiguresFrameArray,
    pos: usize,
}

pub fn make_figuresframe(data: &[u8]) -> FiguresFrame {
    todo!();
}



const OUTPUT_END: isize = 16777215;
const OUTPUT_START: isize = 0;
const INPUT_END: isize = 180;
const INPUT_START: isize = -180;

const SLOPE64: f64 = (OUTPUT_END - OUTPUT_START) as f64 / (INPUT_END - INPUT_START) as f64;
const SLOPE32: f32 = SLOPE64 as f32;

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

const fn map_float(input: f32) -> U24Arr {
    use core::f32;
    // output = a + b * (c - d)
    let output: u32 = (OUTPUT_START as f32 + SLOPE32 * (input - INPUT_START as f32)) as u32;
    debug_assert!(output <= 2u32.pow(24));
    to_24_bit(output)
}

#[inline]
const fn demap_float(input: U24Arr) -> f32 {
    let x = from_24_bit(input);
    let a = OUTPUT_START as f64;
    let b = SLOPE64;
    let d = INPUT_START as f64;
    let c = (-a + (b * d) + x as f64)/b;
    c as f32
}

pub fn coords_to_u48_arr(lat: f32, long: f32) -> U48Arr {
    [map_float(lat), map_float(long)]
}

pub const fn u48_arr_to_coords(data: U48Arr) -> (f32, f32) {
    (demap_float(data[0]), demap_float(data[1]))
}
