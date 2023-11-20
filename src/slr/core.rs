pub enum SLRInstruction {
    Reduce(usize),
    Shift(usize),
    Goto(usize),
}
