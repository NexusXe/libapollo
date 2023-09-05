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

/// It is HEAVILY advised to write your code to never use this struct.
/// 
/// Instead, just put your data in an array and escape it with TncFrameBuffer::escape_byte
/// as part of your serial transmission loop.
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

    pub const fn escape_byte(_byte: u8) -> [Option<u8>; 2] {
        match _byte {
            FEND => [Some(FESC), Some(TFEND)],
            FESC => [Some(FESC), Some(TFESC)],
            _ => [Some(_byte), None],
        }
    }

    pub const fn escape_bytes<const S: usize>(_bytes: [u8; S]) -> [[Option<u8>; 2]; S] {
        let mut i: usize = 0;
        let mut output_array = [[None; 2]; S];
        while i < S {
            output_array[i] = Self::escape_byte(_bytes[i]);
            i += 1;
        }
        output_array
    }

    pub fn escaping_add_byte(&mut self, _byte: u8) {
        for _byteoption in Self::escape_byte(_byte) {
            if _byteoption.is_some() {
                self.raw_add_byte(_byteoption.unwrap());
            }
        }
    }

    pub fn escaping_add_bytes(&mut self, _bytes: &[u8]) {
        for _byte in _bytes {
            self.escaping_add_byte(*_byte);
        }
    }

    pub fn escaping_add_slices(&mut self, _slices: &[&[u8]]) {
        for _slice in _slices {
            self.escaping_add_bytes(*_slice);
        }
    }

    pub fn escaping_new(_data: &[u8]) -> Self {
        let mut framebuffer = TncFrameBuffer::empty_new();
        framebuffer.escaping_add_bytes(_data);
        framebuffer
    }

    pub fn escaping_new_from_slices(_slices: &[&[u8]]) -> Self {
        let mut framebuffer = TncFrameBuffer::empty_new();
        for _slice in _slices {
            framebuffer.escaping_add_bytes(*_slice);
        }
        framebuffer
    }

    /// Escapes all bytes in buffer
    pub fn escape_all(&mut self) {
        let dest_framebuffer = self.clone();
        // We don't need to zero the rest of the data field since it'll THEORETICALLY never be read
        self.current_len = 0usize;
        for i in 0..dest_framebuffer.current_len {
            self.escaping_add_byte(dest_framebuffer.data[i]);
        }
    }

    const fn convert_escaped_byte(_data: u8) -> u8 {
        match _data {
            TFEND => FEND,
            TFESC => FESC,
            _ => panic!(),
        }
    }

    /// Un-escapes all (potentially delmited) bytes in buffer
    pub fn raw_all(&mut self) {
        let dest_framebuffer = self.clone();
        // We don't need to zero the rest of the data field since it'll THEORETICALLY never be read
        self.current_len = 0usize;
        let mut i = 0;
        while i < dest_framebuffer.current_len {
            match dest_framebuffer.data[i] {
                FESC => {
                    i += 1;
                    self.raw_add_byte(Self::convert_escaped_byte(dest_framebuffer.data[i]))
                },
                _ => self.raw_add_byte(dest_framebuffer.data[i]),
            }
            i += 1;
        }

    }
    /// Checks if the current buffer is fully escaped.
    pub fn is_escaped(&self) -> bool {
        let mut position: usize = 0;
        // const _: () = assert!(u8::MAX as usize >= MAX_KISS_FRAME_SIZE, "Iterator type with max value {} in TncFrameBuffer::is_escaped() is too small to handle buffer size of {}", _iterator::MAX, MAX_KISS_FRAME_SIZE);
        while position < self.current_len {
            if self.data[position] == FESC {
                match self.data[position + 1] {
                    TFESC | TFEND => (),
                    _ => return false,
                }
            } else if ((self.data[position] == FESC) || (self.data[position] == FEND)) && (self.data[position] != (self.current_len - 1) as u8) {
                return false;
            }
            position += 1;
        }
        true
    }
}


impl From<(&[u8], bool)> for TncFrameBuffer {
    fn from(starting_tuple: (&[u8], bool)) -> Self {
        let _data = starting_tuple.0;
        let do_escape = starting_tuple.1;
        if do_escape {
            TncFrameBuffer::escaping_new(_data)
        } else {
            TncFrameBuffer::raw_new(_data)
        }
    }
}

impl From<&[u8]> for TncFrameBuffer {
    /// By default, this cast does not escape. Use `TncFrameBuffer::from((&[u8], true))` to escape during the cast.
    fn from(_data: &[u8]) -> Self {
        Self::from((_data, false))
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
        TncFrameBuffer::escaping_new_from_slices(_data)
    }

    #[cfg(test)]
    mod tests {
        use crate::tnc::*;
        use super::*;

        const _DATA: &[u8] = &[11u8, FEND, 0u8, FESC, FESC, FESC, 124u8, 11u8];
        const _EXPECTED_ESCAPED_DATA: [u8; 12] = [11u8, FESC, TFEND, 0u8, FESC, TFESC, FESC, TFESC, FESC, TFESC, 124u8, 11u8];


        #[test]
        pub fn test_tnc_encode() {
            const _LABEL: &[u8] = &[CMD_DATAFRAME];
            let data_frame = make_tnc_frame(&[_LABEL, _DATA]);
            assert_eq!(data_frame.data[_LABEL.len()..data_frame.current_len], _EXPECTED_ESCAPED_DATA);
        }

        #[test]
        pub fn test_tnc_escape() {
            let data_frame = TncFrameBuffer::raw_new(_DATA);
            let mut cycled_data_frame = data_frame.clone();

            cycled_data_frame.escape_all();
            cycled_data_frame.raw_all();

            assert_eq!(data_frame.data[0..data_frame.current_len], cycled_data_frame.data[0..cycled_data_frame.current_len]);
        }

        #[test]
        pub fn test_tnc_is_escaped() {
            let escaped_buffer: TncFrameBuffer = TncFrameBuffer::raw_new(&_EXPECTED_ESCAPED_DATA);
            let unescaped_buffer: TncFrameBuffer = TncFrameBuffer::raw_new(&[0x11, FEND, 0x00, FEND, 0x41]);
            assert!(escaped_buffer.is_escaped());
            assert!(!unescaped_buffer.is_escaped());

        }
    }
}

pub mod tnc_frame_decoder {
    use core::{fmt, panic};
    use super::{CMD_DATAFRAME, CMD_TXDELAY, CMD_P, CMD_SLOTTIME, CMD_TXTAIL, CMD_FULLDUPLEX, CMD_SETHARDWARE, CMD_RETURN};
    use super::TncFrameBuffer;

    const POSSIBLE_COMMANDS: [u8; 8] = [CMD_DATAFRAME, CMD_TXDELAY, CMD_P, CMD_SLOTTIME, CMD_TXTAIL, CMD_FULLDUPLEX, CMD_SETHARDWARE, CMD_RETURN];

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
            if !POSSIBLE_COMMANDS.contains(&num) {
                panic!("{}: 0x{:02X?}", InvalidTncCommandError, &num);
            } else {
                Self(num)
            }
        }
    }

    impl Into<u8> for TncCommandType {
        fn into(self) -> u8 {
            if !POSSIBLE_COMMANDS.contains(&self.0) {
                panic!("{}: 0x{:02X?}", InvalidTncCommandError, &self.0);
            }
            self.0
        }
    }

    pub fn decode_tnc_frame(_frame: &[u8]) -> Result<(u8, TncFrameBuffer), InvalidTncCommandError> {
        if _frame.len() == 0 {return Err(InvalidTncCommandError)};
        let mut _tncframe = TncFrameBuffer::raw_new( &_frame[..1] ); // Man
        _tncframe.escape_all();
        Ok((_frame[0], _tncframe))
    }
}
