use crate::parameters::*;

const FEND: u8 = 0xC0;
const FESC: u8 = 0xDB;
const TFEND: u8 = 0xDC;
const TFESC: u8 = 0xDD;

#[derive(Clone, Copy, Debug)]
enum OneOrTwoBytes {
    OneByte([u8; 1]),
    TwoBytes([u8; 2])
}

// impl OneOrTwoBytes {
//     #[rustc_do_not_const_check]
//     pub const fn len(&self) -> usize {
//         match self {
//             OneOrTwoBytes::OneByte(_) => 1usize,
//             OneOrTwoBytes::TwoBytes(_) => 2usize,
//         }
//     }
// }

impl Iterator for OneOrTwoBytes {
    type Item = u8;
    
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            OneOrTwoBytes::OneByte(_) => None,
            OneOrTwoBytes::TwoBytes(data) => Some(data[1]),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PacketDelimitingBuffer {
    data: [u8; MAX_KISS_FRAME_SIZE],
    current_len: usize,
}

impl PacketDelimitingBuffer {
    fn new() -> Self {
        Self {
            data: [0u8; MAX_KISS_FRAME_SIZE],
            current_len: 0usize,
        }
    }

    pub fn add_data(&mut self, _data: &[u8]) {
        fn delimit_byte(_byte: u8) -> OneOrTwoBytes {
            match _byte {
                FEND => OneOrTwoBytes::TwoBytes([FESC, TFEND]),
                FESC => OneOrTwoBytes::TwoBytes([FESC, TFESC]),
                _ => OneOrTwoBytes::OneByte([_byte])
            }
        }
        for i in _data {
            for x in delimit_byte(*i) {
                self.data[self.current_len + 1] = x;
                self.current_len += 1;
            }
        }
    }
}

pub fn delimit_packet(_packet: &[u8]) -> PacketDelimitingBuffer {
    let mut delimited_packet_buffer = PacketDelimitingBuffer::new();
    delimited_packet_buffer.add_data(_packet);
    delimited_packet_buffer
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate libc_print;
    use libc_print::std_name::{println, dbg};
    
    #[test]
    fn test_delimit_packet() {
        let x = &delimit_packet(&[0x94, FEND, 0x11, FESC]).data[0..4];
        for i in x {
            println!("{:02x}", i);
        }
    }
}