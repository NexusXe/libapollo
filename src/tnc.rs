use crate::parameters::*;
use core::fmt;

const FEND: u8 = 0xC0;  // 192
const FESC: u8 = 0xDB;  // 219
const TFEND: u8 = 0xDC; // 220
const TFESC: u8 = 0xDD; // 221

// impl OneOrTwoBytes {
//     #[rustc_do_not_const_check]
//     pub const fn len(&self) -> usize {
//         match self {
//             OneOrTwoBytes::OneByte(_) => 1usize,
//             OneOrTwoBytes::TwoBytes(_) => 2usize,
//         }
//     }
// }

#[derive(Clone, Copy, Debug)]
pub struct PacketDelimitingBuffer {
    data: [u8; MAX_KISS_FRAME_SIZE],
    current_len: usize,
}

impl fmt::Display for PacketDelimitingBuffer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:X?}", &self.data[0..self.current_len])
    }
}

impl PacketDelimitingBuffer {
    fn new() -> Self {
        Self {
            data: [0u8; MAX_KISS_FRAME_SIZE],
            current_len: 0usize,
        }
    }

    fn add_bytes(&mut self, _bytes: &[u8]) {
        for _byte in _bytes {
            self.data[self.current_len] = *_byte;
            self.current_len += 1;
        }
    }

    pub fn add_data(&mut self, _data: &[u8]) {
        for _byte in _data {
            match *_byte {
                FEND => self.add_bytes(&[FESC, TFEND]),
                FESC => self.add_bytes(&[FESC, TFESC]),
                _ => self.add_bytes(&[*_byte])
                
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
    //use libc_print::std_name::println;
    
    #[test]
    fn test_delimit_packet() {
        let _delimitedpacketbuffer = delimit_packet(&[0x94, FEND, 0x11, FESC]);
        let _delimitedpacketdata = &_delimitedpacketbuffer.data[0.._delimitedpacketbuffer.current_len];
        // for i in _delimitedpacketdata {
        //     match *i {
        //         FEND => println!("FEND"),
        //         FESC => println!("FESC"),
        //         TFEND => println!("TFEND"),
        //         TFESC => println!("TFESC"),
        //         _ => println!("0x{:02X}", i),
        //     }
        // }
        const EXPECTED_DATA: &[u8] = &[0x94, FESC, TFEND, 0x11, FESC, TFESC];
        assert!(_delimitedpacketdata == EXPECTED_DATA, "packet delimiting went wrong! expected {:X?}, found {}", EXPECTED_DATA, _delimitedpacketbuffer)
    }
}