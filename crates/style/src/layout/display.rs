/// How an element lays out its children, or that it takes no part in layout at all.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Display {
    Block,
    Flex,
    Grid,
    None,
}
