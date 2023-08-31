use crate::parameters::*;
use serde::{Serialize, Deserialize};
// http://www.aprs.org/doc/APRS101.PDF

pub const FLAG      : &'static  u8  = &APRS_FLAG      ;
pub const DST_ADDR  : &'static [u8] = &APRS_DST_ADDR  ;
pub const SRC_ADDR  : &'static [u8] = &APRS_SRC_ADDR  ;
pub const PATH      : &'static [u8] = &APRS_PATH      ;
pub const CTRL_FIELD: &'static  u8  = &APRS_CTRL_FIELD;
pub const PRTCL_ID  : &'static  u8  = &APRS_PRTCL_ID  ; 



#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
struct AX25InformationField {
    data_type: u8,
    data: &'static [u8],
    data_extension: [u8; 7],
}

#[derive(Debug, Copy, Clone)]
struct AX25Block {
    information_field: [u8; 256],
    frame_check_sequence: [u8; 2],
}

impl AX25Block {
    pub fn to_frame(&self) -> [u8; UI_FRAME_MAX] {
        // TODO: there has to be a better way to do this
        let mut _frame: [u8; UI_FRAME_MAX] = [0u8; UI_FRAME_MAX];
        let mut current_lower_bound: usize = 0;
        let frame_contents: [&[u8]; 7] = [
            &[*FLAG],
            SRC_ADDR,
            PATH,
            &[*CTRL_FIELD],
            &[*PRTCL_ID],
            &self.information_field,
            &self.frame_check_sequence,
        ];
        let mut i: usize = 0;
        let mut section: &[u8];
        while i < frame_contents.len() {
            section = frame_contents[i];
            _frame[current_lower_bound..section.len()].copy_from_slice(section);
            current_lower_bound += section.len();
            debug_assert!(current_lower_bound <= UI_FRAME_MAX);
            i += 1;
            if i > frame_contents.len() {
                unreachable!();
            }
        }
        _frame
    }
}

/// We shouldn't have to rely on an external crate for something as simple
/// as a checksum, but CRC16 is a fucking mess. Genuinely miserable.
/// 
/// https://www.reddit.com/r/amateurradio/comments/8o3hlk/aprs_crcfcs_bytes/
const fn build_fcs(_frame: &[u8]) -> [u8; 2] {
    use crc::{Crc, NoTable, CRC_16_IBM_3740};
    const X25: Crc<NoTable<u16>> = Crc::<NoTable<u16>>::new(&CRC_16_IBM_3740);
    X25.checksum(_frame).to_be_bytes()
}

pub fn build_aprs_data(_latitude: f32, _longitude: f32) -> [u8; UI_FRAME_MAX] {
    // todo!();
    // let _mic_e_data: ;
    let mut current_ui_frame: AX25Block = AX25Block { information_field: [0u8; 256], frame_check_sequence: [0u8; 2] };
    current_ui_frame.frame_check_sequence = build_fcs(&current_ui_frame.to_frame());
    current_ui_frame.to_frame()
}

pub fn build_mic_e_data(_latitude: f32, _longitude: f32) -> [u8; 7] {
    todo!();
}
