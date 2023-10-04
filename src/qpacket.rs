use core::mem;

use crate::parameters::*;

const TOTAL_DATA_BLOCKS: usize = 16;
const FEC_BYTES: usize = 19;

type BlockLabelType = u8;
const FIRST_BLOCK_LABEL: BlockLabelType = 128;

const BLOCK_CFG_STACK: BlockCfgStack = [
    BlockCfg {
        block_type: BlockType::I32, // latitude
        do_transmit_label: true,
    },
    BlockCfg {
        block_type: BlockType::I32, // longitude
        do_transmit_label: true,
    },
    BlockCfg {
        block_type: BlockType::I32, // altitude
        do_transmit_label: true,
    },
    BlockCfg {
        block_type: BlockType::I16, // voltage
        do_transmit_label: true,
    },
    BlockCfg {
        block_type: BlockType::I16, // temperature
        do_transmit_label: true,
    },
    BlockCfg {
        block_type: BlockType::I16, // temperature
        do_transmit_label: true,
    },
    BlockCfg {
        block_type: BlockType::I16, // temperature
        do_transmit_label: true,
    },
    BlockCfg {
        block_type: BlockType::I32, // altitude
        do_transmit_label: true,
    },
    BlockCfg {
        block_type: BlockType::I32, // altitude
        do_transmit_label: true,
    },
    BlockCfg {
        block_type: BlockType::I32, // altitude
        do_transmit_label: true,
    },
    BlockCfg {
        block_type: BlockType::I32, // altitude
        do_transmit_label: true,
    },
    BlockCfg {
        block_type: BlockType::I32, // altitude
        do_transmit_label: true,
    },
    BlockCfg {
        block_type: BlockType::I32, // altitude
        do_transmit_label: true,
    },
    BlockCfg {
        block_type: BlockType::I32, // altitude
        do_transmit_label: true,
    },
    BlockCfg {
        block_type: BlockType::I32, // altitude
        do_transmit_label: true,
    },
    BlockCfg {
        block_type: BlockType::I32, // altitude
        do_transmit_label: true,
    },
];

pub struct BlockCfg {
    block_type: BlockType,
    do_transmit_label: bool,
}

type BlockCfgStack = [BlockCfg; TOTAL_DATA_BLOCKS];

#[derive(Clone, Copy)]
pub struct BlockIdent {
    block_type: BlockType,
    label: BlockLabelType,
    do_transmit_label: bool,
    position: (usize, usize),
}

impl BlockIdent {
    const BLANK: Self = Self::new();

    const fn new() -> Self {
        Self {
            block_type: BlockType::NONE,
            label: 0 as BlockLabelType,
            do_transmit_label: false,
            position: (0usize, 0usize + BlockType::NONE.len()),
        }
    }

    pub const fn data_len(&self) -> usize {
        self.block_type.len()
    }

    pub const fn total_len(&self) -> usize {
        debug_assert!(self.position.1 >= self.position.0);
        self.position.1 - self.position.0
    }

    pub const fn len(&self) -> usize {
        self.total_len()
    }
}

type BlockIdentStack = [BlockIdent; TOTAL_DATA_BLOCKS];

const fn cfg_stack_to_ident_stack(cfg_stack: BlockCfgStack) -> BlockIdentStack {
    // a block is its label, its data, and the following delimiter
    let mut output: BlockIdentStack = [BlockIdent::BLANK; TOTAL_DATA_BLOCKS];
    let mut i: usize = 0;
    let mut current_packet_position: usize = START_HEADER_DATA.len() + BLOCK_DELIMITER_SIZE; // bytes, accounting for START_HEADER_DATA and its delimiter

    while i < output.len() {
        if i > BlockLabelType::MAX as usize || i >= cfg_stack.len() {
            unreachable!();
        }

        let _block_type = cfg_stack[i].block_type;
        let _do_transmit_label = cfg_stack[i].do_transmit_label;
        let _block_label: BlockLabelType = FIRST_BLOCK_LABEL + i as BlockLabelType;
        let _block_size: usize = _block_type.len() + {if _do_transmit_label {mem::size_of::<BlockLabelType>()} else {0}} + BLOCK_DELIMITER_SIZE;
        let end_position = current_packet_position + _block_size - 1;
        output[i] = BlockIdent {
            block_type: cfg_stack[i].block_type,
            label: _block_label,
            do_transmit_label: _do_transmit_label,
            position: (current_packet_position, end_position),
        };
        current_packet_position = end_position + 1;
        i += 1;
    }
    output
}

pub const BLOCK_IDENT_STACK: BlockIdentStack = cfg_stack_to_ident_stack(BLOCK_CFG_STACK);
pub const QPACKET_DATA_LEN: usize = BLOCK_IDENT_STACK[BLOCK_IDENT_STACK.len() - 1].position.1 - 1;
const START_END_HEADER_SIZE: usize = mem::size_of_val::<_>(&START_END_HEADER);
pub const QPAKCET_BARE_LEN: usize = START_HEADER_DATA.len() + QPACKET_DATA_LEN + START_END_HEADER_SIZE;
pub const QPACKET_FULL_LEN: usize = QPAKCET_BARE_LEN + FEC_BYTES;
pub const BLOCK_SIZE_STACK: [usize; TOTAL_DATA_BLOCKS] = {
    let mut output: [usize; TOTAL_DATA_BLOCKS] = [0usize; TOTAL_DATA_BLOCKS];
    let mut i: usize = 0;

    while i < TOTAL_DATA_BLOCKS {
        output[i] = BLOCK_IDENT_STACK[i].total_len();
        i += 1;
    }

    output
};

pub struct QPacketBlock<'a> {
    identity: BlockIdent,
    data: &'a [u8],
}

impl<'a> QPacketBlock<'a> {
    pub const BLANK: Self = Self {
        identity: BlockIdent::BLANK,
        data: &[u8::MAX],
    };

    pub const fn new(_ident: &'static BlockIdent, _data: &'a [u8]) -> Self {
        Self {
            identity: *_ident,
            data: _data,
        }
    }

    pub const fn len(&self) -> usize {
        self.identity.total_len()
    }
    
    pub const fn as_bytes<const LEN: usize>(&self) -> [u8; LEN] {
        let mut output: [u8; LEN] = [0u8; LEN];
        

        let mut i: usize = 0usize; // output position
        let mut x: usize = 0usize; // data position (can't use an iterator because this is a const fn)

        // first two bytes are always delimiter
        output[i] = BLOCK_DELIMITER.to_be_bytes()[0];
        i += 1;
        output[i] = BLOCK_DELIMITER.to_be_bytes()[1];
        i += 1;

        if self.identity.do_transmit_label {
            output[i] = self.identity.label;
            i += 1;
        }

        while i < self.len() {
            if i >= self.len() || x >= self.data.len() {
                unreachable!()
            }
            output[i] = self.data[x];
            i += 1;
            x += 1;
        }

        output
    }
}

pub type QPacketBlockStack<'a> = [QPacketBlock<'a>; TOTAL_DATA_BLOCKS];

pub const fn construct_bare_packet(_blockstack: BlockIdentStack) -> [u8; QPAKCET_BARE_LEN] {
    let mut output: [u8; QPAKCET_BARE_LEN] = [0u8; QPAKCET_BARE_LEN];
    let mut i: usize = 0;
    let mut x: usize = 0;

    while x < START_HEADER_DATA.len() {
        output[x] = START_HEADER_DATA[x];
        x += 1;
    }

    let mut x: usize = 0;
    while x < BLOCK_DELIMITER_SIZE { // manually insert first delimiter
        output[x + START_HEADER_DATA.len()] = BLOCK_DELIMITER.to_be_bytes()[x];
        x += 1;
    }

    while i < _blockstack.len() {
        let _block = &_blockstack[i];

        let mut left = _block.position.0;


        if _block.do_transmit_label { // first is (maybe) the label,
            output[left] = _block.label;
            left += 1;
        }
        
        while left <= _block.position.1 - BLOCK_DELIMITER_SIZE { // then the data,
            output[left] = 0u8;
            left += 1;
        }

        output[left] = BLOCK_DELIMITER.to_be_bytes()[0]; // and finally the delimiter.
        left += 1;
        output[left] = BLOCK_DELIMITER.to_be_bytes()[1];

        i += 1;
    }

    let mut x: usize = 0;

    while x < END_HEADER_DATA.len() {
        output[output.len() - (END_HEADER_DATA.len() - x)] = END_HEADER_DATA[x];
        x += 1;
    }

    output
}

pub const BARE_QPACKET: [u8; QPAKCET_BARE_LEN] = construct_bare_packet(BLOCK_IDENT_STACK);


#[cfg(test)]
mod tests {
    use super::*;

    fn check_packet_formation<const T: usize>(packet: [u8; T]) {
        let mut _packet_head = [0u8; (START_HEADER_DATA.len() + BLOCK_DELIMITER_SIZE)];
        let mut i: usize = 0;
        _packet_head[0..START_HEADER_DATA.len()].copy_from_slice(&START_HEADER_DATA);
        i += START_HEADER_DATA.len();
        _packet_head[i..i+BLOCK_DELIMITER_SIZE].copy_from_slice(&BLOCK_DELIMITER.to_be_bytes());
        assert_eq!(packet[0..i+BLOCK_DELIMITER_SIZE], _packet_head);

        for block in BLOCK_IDENT_STACK {
            assert_eq!(packet[block.position.0], block.label);
            for x in 0..BLOCK_DELIMITER.to_be_bytes().len() {
                const BLOCK_DELIMITER_BYTES: [u8; BLOCK_DELIMITER_SIZE] = BLOCK_DELIMITER.to_be_bytes();
                assert_eq!(packet[(block.position.1 - (BLOCK_DELIMITER_SIZE - 1)) + x], BLOCK_DELIMITER_BYTES[x]);
            }
            
        }
    }

    #[test]
    pub fn check_blank_packet_formation() {
        check_packet_formation(BARE_QPACKET);
    }
}
