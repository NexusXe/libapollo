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

// https://www.ax25.net/kiss.aspx
const CMD_DATAFRAME: u8 = 0;
const CMD_TXDELAY: u8 = 1;
const CMD_P: u8 = 2;
const CMD_SLOTTIME: u8 = 3;
const CMD_TXTAIL: u8 = 4;
const CMD_FULLDUPLEX: u8 = 5;
const CMD_SETHARDWARE: u8 = 6;
const CMD_RETURN: u8 = 0xFF;

struct SettingsArray{
    settings: [u8; 6],
}

static mut SETTINGS_ARRAY: SettingsArray = SettingsArray {
    settings: [
    50,
    63,
    10,
    0,
    false as u8,
    0
    ]
};

trait SettingsConfiguration {
    fn txdelay(&self) -> u8;
    fn p(&self) -> u8;
    fn slottime(&self) -> u8;
    fn txtail(&self) -> u8;
    fn fullduplex(&self) -> u8;
    fn sethardware(&self) -> u8;
}

impl SettingsConfiguration for SettingsArray {
    fn txdelay(&self) -> u8 {self.settings[0]}
    fn p(&self) -> u8 {self.settings[1]}
    fn slottime(&self) -> u8 {self.settings[2]}
    fn txtail(&self) -> u8 {self.settings[3]}
    fn fullduplex(&self) -> u8 {self.settings[4]}
    fn sethardware(&self) -> u8 {self.settings[5]}
}

pub struct TncFrame {
    command: u8,
    data: Option<u8>,
}

fn delimit_packet(_packet: &[u8]) -> PacketDelimitingBuffer {
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

pub fn return_frame(_type: u8, _data: Option<u8>) -> Option<TncFrame> {
    match _type {
        CMD_DATAFRAME => (),
        CMD_TXDELAY => (),
        CMD_P => (),
        CMD_SLOTTIME => (),
        CMD_TXTAIL => (),
        CMD_FULLDUPLEX => (),
        CMD_SETHARDWARE => (),
        CMD_RETURN | _ => (), // do nothing
    }
    Some(TncFrame {
        command: _type,
        data: _data,
    })
}

pub fn change_option(_type: u8, _data: u8) -> Option<TncFrame> {
    match _type {
        CMD_DATAFRAME => return None,
        CMD_TXDELAY => (),
        CMD_P => (),
        CMD_SLOTTIME => (),
        CMD_TXTAIL => (),
        CMD_FULLDUPLEX => (),
        CMD_SETHARDWARE => (),
        CMD_RETURN | _ => return None, // do nothing
    }
    unsafe {
        SETTINGS_ARRAY.settings[(_type-1) as usize] = _data;
        Some(TncFrame { command: _type, data: Some(SETTINGS_ARRAY.settings[(_type-1) as usize]) })
    }
}
