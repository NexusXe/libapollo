use crate::parameters::MAX_KISS_FRAME_SIZE;


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

#[derive(Clone, Copy)]
pub struct TncFrameBuffer {
    pub data: [u8; MAX_KISS_FRAME_SIZE],
    pub current_len: usize,
}

impl TncFrameBuffer {
    /// Create new empty FrameBuffer with a zeroed array and current_len of 0
    pub const fn empty_new() -> Self {
        Self { data: [0u8; MAX_KISS_FRAME_SIZE], current_len: 0 }
    }

    pub const fn raw_add_byte(&mut self, _byte: u8) {
        self.data[self.current_len] = _byte;
        self.current_len += 1;
    }

    pub fn raw_add_bytes(&mut self, _bytes: &[u8]) {
        for _byte in _bytes {
            self.raw_add_byte(*_byte);
        }
    }

    pub fn raw_add_slices(&mut self, _slices: &[&[u8]]) {
        for _slice in _slices {
            self.raw_add_bytes(*_slice);
        }
    }

    pub fn raw_new(_data: &[u8]) -> Self {
        let mut framebuffer = TncFrameBuffer::empty_new();
        framebuffer.raw_add_bytes(_data);
        framebuffer
    }

    pub fn raw_new_from_slices(_slices: &[&[u8]]) -> Self {
        let mut framebuffer = TncFrameBuffer::empty_new();
        for _slice in _slices {
            framebuffer.raw_add_bytes(*_slice);
        }
        framebuffer
    }

    pub fn delimit_add_byte(&mut self, _byte: u8) {
        match _byte {
            FEND => self.raw_add_bytes(&[FESC, TFEND]),
            FESC => self.raw_add_bytes(&[FESC, TFESC]),
            _ => self.raw_add_byte(_byte)
    
        }
    }

    pub fn delimit_add_bytes(&mut self, _bytes: &[u8]) {
        for _byte in _bytes {
            self.delimit_add_byte(*_byte);
        }
    }

    pub fn delimit_add_slices(&mut self, _slices: &[&[u8]]) {
        for _slice in _slices {
            self.delimit_add_bytes(*_slice);
        }
    }

    pub fn delimit_new(_data: &[u8]) -> Self {
        let mut framebuffer = TncFrameBuffer::empty_new();
        framebuffer.delimit_add_bytes(_data);
        framebuffer
    }

    pub fn delimit_new_from_slices(_slices: &[&[u8]]) -> Self {
        let mut framebuffer = TncFrameBuffer::empty_new();
        for _slice in _slices {
            framebuffer.delimit_add_bytes(*_slice);
        }
        framebuffer
    }

    /// Delimits all bytes in buffer
    pub fn delimit_all(&mut self) {
        let dest_framebuffer = self.clone();
        // We don't need to zero the rest of the data field since it'll THEORETICALLY never be read
        self.current_len = 0usize;
        for i in 0..dest_framebuffer.current_len {
            self.delimit_add_byte(dest_framebuffer.data[i]);
        }
    }

    const fn convert_escaped_byte(_data: u8) -> u8 {
        match _data {
            TFEND => FEND,
            TFESC => FESC,
            _ => panic!(),
        }
    }

    /// Un-delimits all (potentially delmited) bytes in buffer
    pub fn raw_all(&mut self) {
        let dest_framebuffer = self.clone();
        // We don't need to zero the rest of the data field since it'll THEORETICALLY never be read
        self.current_len = 0usize;
        
        for mut i in 0..dest_framebuffer.current_len {
            match dest_framebuffer.data[i] {
                FESC => {
                    i += 1;
                    self.raw_add_byte(Self::convert_escaped_byte(dest_framebuffer.data[i]))
                },

                _ => self.raw_add_byte(dest_framebuffer.data[i]),
            }
        }

    }
}

pub mod tnc_frame_encoder {
    use core::fmt;
    use super::TncFrameBuffer; 

    #[derive(Debug)]
    pub struct InvalidEscapedByteError;

    impl fmt::Display for TncFrameBuffer {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{:X?}", &self.data[0..self.current_len])
        }
    }

    /// TODO: throw error when trying to use data longer than one byte with option frames
    /// TODO: throw error when supplied no data
    pub fn make_tnc_frame(_data: &[&[u8]]) -> TncFrameBuffer {
        TncFrameBuffer::delimit_new_from_slices(_data)
    }
}

pub mod tnc_frame_decoder {
    use core::fmt;
    use super::{CMD_DATAFRAME, CMD_TXDELAY, CMD_P, CMD_SLOTTIME, CMD_TXTAIL, CMD_FULLDUPLEX, CMD_SETHARDWARE, CMD_RETURN};
    use super::TncFrameBuffer;

    const POSSIBLE_COMMANDS: &[u8] = &[CMD_DATAFRAME, CMD_TXDELAY, CMD_P, CMD_SLOTTIME, CMD_TXTAIL, CMD_FULLDUPLEX, CMD_SETHARDWARE, CMD_RETURN];

    #[derive(Debug)]
    pub struct InvalidTncCommandError;

    impl fmt::Display for InvalidTncCommandError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "Unrecognized TNC command")
        }
    }

    struct TncCommandType(u8);
    
    impl From<u8> for TncCommandType {
        fn from(num: u8) -> Self {
            assert!(POSSIBLE_COMMANDS.contains(&num));
            Self(num)
        }
    }

    impl Into<u8> for TncCommandType {

        fn into(self) -> u8 {
            assert!(POSSIBLE_COMMANDS.contains(&self.0));
            self.0
        }
    }

    pub fn decode_tnc_frame(_frame: &[u8]) -> Result<(u8, TncFrameBuffer), InvalidTncCommandError> {
        if _frame.len() == 0 {return Err(InvalidTncCommandError)};
        let mut _tncframe = TncFrameBuffer::raw_new( &_frame[..1] ); // Man
        _tncframe.delimit_all();
        Ok((_frame[0], _tncframe))
    }
}

// pub mod oldtnc {
//     use super::{FEND, FESC, TFEND, TFESC};
//     use super::{CMD_DATAFRAME, CMD_TXDELAY, CMD_P, CMD_SLOTTIME, CMD_TXTAIL, CMD_FULLDUPLEX, CMD_SETHARDWARE, CMD_RETURN};
//     use super::MAX_KISS_FRAME_SIZE;
//     use super::fmt;


//     pub struct TncFrameBuffer {
//         pub(crate) data: [u8; MAX_KISS_FRAME_SIZE],
//         pub(crate) current_len: usize,
//     }

//     impl fmt::Display for TncFrameBuffer {
//         fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//             write!(f, "{:X?}", &self.data[0..self.current_len])
//         }
//     }

//     impl TncFrameBuffer {
//         pub(crate) fn new() -> Self {
//             Self {
//                 data: [0u8; MAX_KISS_FRAME_SIZE],
//                 current_len: 0usize,
//             }
//         }
//         pub(crate) fn add_byte(&mut self, _byte: &u8) {
//             self.data[self.current_len] = *_byte;
//             self.current_len += 1;
//         }

//         pub fn add_bytes(&mut self, _bytes: &[u8]) {
//             for _byte in _bytes {
//                 self.add_byte(_byte);
//             }
//         }
//     }

//     pub(crate) struct PacketDelimitingBuffer {
//         pub(crate) data: [u8; MAX_KISS_FRAME_SIZE],
//         pub(crate) current_len: usize,
//     }

//     impl fmt::Display for PacketDelimitingBuffer {
//         fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//             write!(f, "{:X?}", &self.data[0..self.current_len])
//         }
//     }

//     impl PacketDelimitingBuffer {
//         pub(crate) fn new() -> Self {
//             Self {
//                 data: [0u8; MAX_KISS_FRAME_SIZE],
//                 current_len: 0usize,
//             }
//         }

//         pub(crate) fn add_byte(&mut self, _byte: &u8) {
//             self.data[self.current_len] = *_byte;
//             self.current_len += 1;
//         }

//         pub(crate) fn add_bytes(&mut self, _bytes: &[u8]) {
//             for _byte in _bytes {
//                 self.add_byte(_byte);
//             }
//         }

//         pub fn add_data(&mut self, _data: &[u8]) {
//             for _byte in _data {
//                 match *_byte {
//                     FEND => self.add_bytes(&[FESC, TFEND]),
//                     FESC => self.add_bytes(&[FESC, TFESC]),
//                     _ => self.add_bytes(&[*_byte])
                
//                 }
//             }
//         }

//         pub fn as_byte_slice(&mut self) -> &[u8] {
//             &self.data[0..self.current_len]
//         }
//     }

//     impl From<&[u8]> for PacketDelimitingBuffer {
//         fn from(dataslice: &[u8]) -> Self {
//             let mut x = PacketDelimitingBuffer::new();
//             x.add_data(dataslice);
//             x
//         }
//     }

//     impl Iterator for PacketDelimitingBuffer {
//         type Item = u8;

//         fn next(&mut self) -> Option<Self::Item> {
//             if self.current_len == 0 {
//                 return None;
//             }

//             let item = self.data[self.current_len - 1];
//             self.current_len -= 1;

//             Some(item)
//         }
//     }

//     #[repr(transparent)]
//     #[derive(Debug)]
//     pub(crate) struct TncCommand(u8);

//     impl Into<u8> for TncCommand {
//         fn into(self) -> u8 {
//             self.0
//         }
//     }

//     impl From<u8> for TncCommand {
//         fn from(number: u8) -> Self {
//             TncCommand(number)
//         }
//     }

//     impl TncCommand {
//         pub(crate) fn command_name(&self) -> &str {
//             match self.0 {
//                 CMD_DATAFRAME => "DATAFRAME",
//                 CMD_TXDELAY => "TXDELAY",
//                 CMD_P => "P",
//                 CMD_SLOTTIME => "SLOTTIME",
//                 CMD_TXTAIL => "TXTAIL",
//                 CMD_FULLDUPLEX => "FULLDUPLEX",
//                 CMD_SETHARDWARE => "SETHARDWARE",
//                 CMD_RETURN => "RETURN",
//                 _ => "UNKNOWN",
//             }
//         }
//     }

//     pub(crate) struct TncFrame<T> {
//         pub(crate) command: TncCommand,
//         pub(crate) data: T,
//     }

//     pub(crate) type TncOptionFrame = TncFrame<Option<u8>>;

//     pub(crate) type TncDataFrame = TncFrame<PacketDelimitingBuffer>;

//     impl fmt::Display for TncOptionFrame {
//         fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//             write!(f, "Option: {}, Data: 0x{:02X?}", self.command.command_name(), self.data.unwrap_or(0x00))
//         }
//     }

//     #[derive(Debug)]
//     pub struct InvalidCommandError;

//     impl fmt::Display for InvalidCommandError {
//         fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//             write!(f, "Unrecognized TNC command")
//         }
//     }

//     #[derive(Debug)]
//     pub struct FrameGenerationError;

//     impl fmt::Display for FrameGenerationError {
//         fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//             write!(f, "Error generating frame")
//         }
//     }

//     pub(crate) fn generate_option_frame(_type: u8, _data_arr: Option<&[u8]>) -> Result<TncOptionFrame, InvalidCommandError> {
//         if _data_arr.is_none() {
//             return Ok(TncOptionFrame {command: TncCommand(_type), data: None})
//         }
    
//         let _data = _data_arr.unwrap();
    
//         if _data.len() < 1 {
//             unsafe { core::intrinsics::unreachable(); }
//         }
//         match _type {
        
//             CMD_TXDELAY | CMD_P | CMD_SLOTTIME | CMD_TXTAIL | CMD_FULLDUPLEX | CMD_SETHARDWARE => Ok(TncOptionFrame { command: _type.into(), data: Some(_data[0])}),
//             CMD_RETURN => Ok(TncOptionFrame{command: CMD_RETURN.into(), data: None}),
//             CMD_DATAFRAME => unreachable!(),
//             _ => return Err(InvalidCommandError),
//         }
//     }

//     pub(crate) fn generate_data_frame(_data: &[u8]) -> TncDataFrame {
//         TncFrame { command: CMD_DATAFRAME.into(), data: PacketDelimitingBuffer::from(_data) }
//     }

//     pub fn generate_frame(_type: u8, _data_arr: &[u8]) -> Result<TncFrameBuffer, FrameGenerationError> {
//         let mut framebuffer = TncFrameBuffer::new();
//         framebuffer.add_bytes(&[_type]);
//         match _type {
//             CMD_DATAFRAME => framebuffer.add_bytes(generate_data_frame(_data_arr).data.as_byte_slice()),
//             _ => framebuffer.add_byte(&generate_option_frame(_type, Some(_data_arr)).unwrap().data.unwrap()),
//         }
//         Ok(framebuffer)
//     }

//     #[cfg(test)]
//     pub(crate) mod tests {
//         use super::*;
//         extern crate libc_print;
//         //use libc_print::std_name::println;

//         fn delimit_packet(_packet: &[u8]) -> PacketDelimitingBuffer {
//             let mut delimited_packet_buffer = PacketDelimitingBuffer::new();
//             delimited_packet_buffer.add_data(_packet);
//             delimited_packet_buffer
//         }
    
//         #[test]
//         fn test_delimit_packet() {
//             let _delimitedpacketbuffer = delimit_packet(&[0x94, FEND, 0x11, FESC]);
//             let _delimitedpacketdata = &_delimitedpacketbuffer.data[0.._delimitedpacketbuffer.current_len];
//             // for i in _delimitedpacketdata {
//             //     match *i {
//             //         FEND => println!("FEND"),
//             //         FESC => println!("FESC"),
//             //         TFEND => println!("TFEND"),
//             //         TFESC => println!("TFESC"),
//             //         _ => println!("0x{:02X}", i),
//             //     }
//             // }
//             const EXPECTED_DATA: &[u8] = &[0x94, FESC, TFEND, 0x11, FESC, TFESC];
//             assert_eq!(_delimitedpacketdata, EXPECTED_DATA, "packet delimiting went wrong! expected {:X?}, found {}", EXPECTED_DATA, _delimitedpacketbuffer)
//         }

//         #[test]
//         fn test_delimiting_packet_buffer() {
//             const DATA: &[u8] = &[0x94, FEND, 0x11, FESC, 0x00, FESC, FESC, FEND, TFEND];
//             const EXPECTED_DATA: &[u8] = &[0x94, FESC, TFEND, 0x11, FESC, TFESC, 0x00, FESC, TFESC, FESC, TFESC, FESC, TFEND, TFEND];
//             let mut buffer = PacketDelimitingBuffer::new();
//             buffer.add_data(DATA);
//             let buffer_as_byte_slice = buffer.as_byte_slice();
//             assert_eq!(buffer_as_byte_slice, EXPECTED_DATA, "packet buffer delimiting went wrong! expected {:X?}, found {:X?}", EXPECTED_DATA, buffer_as_byte_slice);
//         }
//     }
// }

