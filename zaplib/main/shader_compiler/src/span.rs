#[derive(Clone, Copy, Debug, Default, Hash, Eq, Ord, PartialOrd, PartialEq)]
pub struct CodeFragmentId(pub usize);

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialOrd, PartialEq)]
pub struct Span {
    pub code_fragment_id: CodeFragmentId,
    pub start: usize,
    pub end: usize,
}
