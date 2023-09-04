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
