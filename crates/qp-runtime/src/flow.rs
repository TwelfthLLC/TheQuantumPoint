use super::value::RuntimeValue;

pub(crate) enum Flow {
    Next,
    Return(Option<RuntimeValue>),
    Break,
    Continue,
}
