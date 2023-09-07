use crate::parameters::{FIGURES_FRAME_SIZE, FiguresFrameArray};

pub struct FiguresFrame {
    data: FiguresFrameArray,
    pos: usize,
}

pub fn make_figuresframe(data: &[u8]) -> FiguresFrame {
    todo!();
}