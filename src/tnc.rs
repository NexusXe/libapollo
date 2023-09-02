use crate::parameters::*;
use core::fmt;

// https://www.ax25.net/kiss.aspx
const FEND: u8 = 0xC0;  // 192
const FESC: u8 = 0xDB;  // 219
const TFEND: u8 = 0xDC; // 220
const TFESC: u8 = 0xDD; // 221

const CMD_DATAFRAME: u8 = 0;
const CMD_TXDELAY: u8 = 1;
const CMD_P: u8 = 2;
const CMD_SLOTTIME: u8 = 3;
const CMD_TXTAIL: u8 = 4;
const CMD_FULLDUPLEX: u8 = 5;
const CMD_SETHARDWARE: u8 = 6;
const CMD_RETURN: u8 = 0xFF;

pub struct TncFrameBuffer {
    data: [u8; MAX_KISS_FRAME_SIZE],
    current_len: usize,
}

impl fmt::Display for TncFrameBuffer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:X?}", &self.data[0..self.current_len])
    }
}

impl TncFrameBuffer {
    fn new() -> Self {
        Self {
            data: [0u8; MAX_KISS_FRAME_SIZE],
            current_len: 0usize,
        }
    }
    fn add_byte(&mut self, _byte: &u8) {
        self.data[self.current_len] = *_byte;
        self.current_len += 1;
    }

    pub fn add_bytes(&mut self, _bytes: &[u8]) {
        for _byte in _bytes {
            self.add_byte(_byte);
        }
    }
}

struct PacketDelimitingBuffer {
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

    fn add_byte(&mut self, _byte: &u8) {
        self.data[self.current_len] = *_byte;
        self.current_len += 1;
    }

    fn add_bytes(&mut self, _bytes: &[u8]) {
        for _byte in _bytes {
            self.add_byte(_byte);
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

    pub fn as_byte_slice(&mut self) -> &[u8] {
        &self.data[0..self.current_len]
    }
}

impl From<&[u8]> for PacketDelimitingBuffer {
    fn from(dataslice: &[u8]) -> Self {
        let mut x = PacketDelimitingBuffer::new();
        x.add_data(dataslice);
        x
    }
}

impl Iterator for PacketDelimitingBuffer {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_len == 0 {
            return None;
        }

        let item = self.data[self.current_len - 1];
        self.current_len -= 1;

        Some(item)
    }
}

#[repr(transparent)]
#[derive(Debug)]
struct TncCommand(u8);

impl Into<u8> for TncCommand {
    fn into(self) -> u8 {
        self.0
    }
}

impl From<u8> for TncCommand {
    fn from(number: u8) -> Self {
        TncCommand(number)
    }
}

impl TncCommand {
    fn command_name(&self) -> &str {
        match self.0 {
            CMD_DATAFRAME => "DATAFRAME",
            CMD_TXDELAY => "TXDELAY",
            CMD_P => "P",
            CMD_SLOTTIME => "SLOTTIME",
            CMD_TXTAIL => "TXTAIL",
            CMD_FULLDUPLEX => "FULLDUPLEX",
            CMD_SETHARDWARE => "SETHARDWARE",
            CMD_RETURN => "RETURN",
            _ => "UNKNOWN",
        }
    }
}

struct TncFrame<T> {
    command: TncCommand,
    data: T,
}

type TncOptionFrame = TncFrame<Option<u8>>;
type TncDataFrame = TncFrame<PacketDelimitingBuffer>;

impl fmt::Display for TncOptionFrame {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Option: {}, Data: 0x{:02X?}", self.command.command_name(), self.data.unwrap_or(0x00))
    }
}

#[derive(Debug)]
pub struct InvalidCommandError;
impl fmt::Display for InvalidCommandError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Unrecognized TNC command")
    }
}

#[derive(Debug)]
pub struct FrameGenerationError;
impl fmt::Display for FrameGenerationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error generating frame")
    }
}

fn generate_option_frame(_type: u8, _data_arr: Option<&[u8]>) -> Result<TncOptionFrame, InvalidCommandError> {
    if _data_arr.is_none() {
        return Ok(TncOptionFrame {command: TncCommand(_type), data: None})
    }
    
    let _data = _data_arr.unwrap();
    
    if _data.len() < 1 {
        unsafe { core::intrinsics::unreachable(); }
    }
    match _type {
        
        CMD_TXDELAY | CMD_P | CMD_SLOTTIME | CMD_TXTAIL | CMD_FULLDUPLEX | CMD_SETHARDWARE => Ok(TncOptionFrame { command: _type.into(), data: Some(_data[0])}),
        CMD_RETURN => Ok(TncOptionFrame{command: CMD_RETURN.into(), data: None}),
        CMD_DATAFRAME => unreachable!(),
        _ => return Err(InvalidCommandError),
    }
}

fn generate_data_frame(_data: &[u8]) -> TncDataFrame {
    TncFrame { command: CMD_DATAFRAME.into(), data: PacketDelimitingBuffer::from(_data) }
}

pub fn generate_frame(_type: u8, _data_arr: &[u8]) -> Result<TncFrameBuffer, FrameGenerationError> {
    let mut framebuffer = TncFrameBuffer::new();
    framebuffer.add_bytes(&[_type]);
    match _type {
        CMD_DATAFRAME => framebuffer.add_bytes(generate_data_frame(_data_arr).data.as_byte_slice()),
        _ => framebuffer.add_byte(&generate_option_frame(_type, Some(_data_arr)).unwrap().data.unwrap()),
    }
    Ok(framebuffer)
}




#[cfg(test)]
mod tests {
    use super::*;
    extern crate libc_print;
    //use libc_print::std_name::println;

    fn delimit_packet(_packet: &[u8]) -> PacketDelimitingBuffer {
        let mut delimited_packet_buffer = PacketDelimitingBuffer::new();
        delimited_packet_buffer.add_data(_packet);
        delimited_packet_buffer
    }
    
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
        assert_eq!(_delimitedpacketdata, EXPECTED_DATA, "packet delimiting went wrong! expected {:X?}, found {}", EXPECTED_DATA, _delimitedpacketbuffer)
    }

    #[test]
    fn test_delimiting_packet_buffer() {
        const DATA: &[u8] = &[0x94, FEND, 0x11, FESC, 0x00, FESC, FESC, FEND, TFEND];
        const EXPECTED_DATA: &[u8] = &[0x94, FESC, TFEND, 0x11, FESC, TFESC, 0x00, FESC, TFESC, FESC, TFESC, FESC, TFEND, TFEND];
        let mut buffer = PacketDelimitingBuffer::new();
        buffer.add_data(DATA);
        let buffer_as_byte_slice = buffer.as_byte_slice();
        assert_eq!(buffer_as_byte_slice, EXPECTED_DATA, "packet buffer delimiting went wrong! expected {:X?}, found {:X?}", EXPECTED_DATA, buffer_as_byte_slice);
    }
}

