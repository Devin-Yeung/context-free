#[derive(Clone)]
pub enum SLRInstruction {
    Reduce(usize),
    Shift(usize),
    Goto(usize),
    Empty,
}

impl ToString for SLRInstruction {
    fn to_string(&self) -> String {
        match self {
            SLRInstruction::Reduce(i) => format!("r{}", i),
            SLRInstruction::Shift(i) => format!("s{}", i),
            SLRInstruction::Goto(i) => format!("g{}", i),
            SLRInstruction::Empty => String::new(),
        }
    }
}

impl From<&SLRInstruction> for String {
    fn from(val: &SLRInstruction) -> Self {
        val.to_string()
    }
}
