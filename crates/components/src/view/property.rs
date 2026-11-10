use morok_reactive::{Memo, Runtime, Signal, Source};

/// What a property of an element is given when the element is built: a value that stays, or
/// a signal or memo the property follows from then on.
#[derive(Clone, Debug, PartialEq)]
pub enum Property<Value> {
    Fixed(Value),
    Bound(Source<Value>),
}

impl<Value: Clone + 'static> Property<Value> {
    /// The value the property has now.
    pub fn get<World>(&self, runtime: &mut Runtime<World>) -> Value {
        match self {
            Self::Fixed(value) => value.clone(),
            Self::Bound(source) => runtime.get(*source),
        }
    }
}

impl<Value> From<Value> for Property<Value> {
    fn from(value: Value) -> Self {
        Self::Fixed(value)
    }
}

impl<Value> From<Signal<Value>> for Property<Value> {
    fn from(signal: Signal<Value>) -> Self {
        Self::Bound(signal.into())
    }
}

impl<Value> From<Memo<Value>> for Property<Value> {
    fn from(memo: Memo<Value>) -> Self {
        Self::Bound(memo.into())
    }
}

impl From<&str> for Property<String> {
    fn from(text: &str) -> Self {
        Self::Fixed(text.to_owned())
    }
}
