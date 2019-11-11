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
    Success,
    Failure,
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
            CellType::Wall => CellType::Success,
            CellType::Success => CellType::Failure,
            CellType::Failure => CellType::Empty,
        }
    }
}
