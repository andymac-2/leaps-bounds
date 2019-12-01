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
}
