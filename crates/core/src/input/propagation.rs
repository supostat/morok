/// What an element answers to an event routed to it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Propagation {
    /// The element did not use the event: it goes on to the element around it.
    Continue,
    /// The element used the event: nothing else hears of it.
    Stop,
}
