use super::CellCursorEntry;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum CellType {
    Empty,
    ColouredBlock,
    Arrow,
    ColouredArrow,
    ArrowBlock,
    RotateRight,
    RotateLeft,
    Fence,
    Wall,
    Overlay,
}
impl CellType {
    pub fn increment(self) -> Self {
        match self {
            CellType::Empty => CellType::ColouredBlock,
            CellType::ColouredBlock => CellType::Arrow,
            CellType::Arrow => CellType::ColouredArrow,
            CellType::ColouredArrow => CellType::ArrowBlock,
            CellType::ArrowBlock => CellType::RotateRight,
            CellType::RotateRight => CellType::RotateLeft,
            CellType::RotateLeft => CellType::Fence,
            CellType::Fence => CellType::Wall,
            CellType::Wall => CellType::Overlay,
            CellType::Overlay => CellType::Empty,
        }
    }
    pub fn full_palette() -> Vec<CellCursorEntry<CellType>> {
        vec![
            CellType::Empty.into(),
            CellType::ColouredBlock.into(),
            CellType::Arrow.into(),
            CellType::ColouredArrow.into(),
            CellType::ArrowBlock.into(),
            CellType::RotateRight.into(),
            CellType::RotateLeft.into(),
            CellType::Fence.into(),
            CellType::Wall.into(),
            CellType::Overlay.into(),
        ]
    }
    pub fn is_empty(self) -> bool {
        match self {
            CellType::Empty => true,
            _ => false,
        }
    }
}
