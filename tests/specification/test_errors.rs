use std::error::Error;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AssertEqError {
    left: String,
    right: String,
}

impl AssertEqError {
    pub fn assert_eq<T: std::fmt::Debug + PartialEq>(left: T, right: T) -> Result<(), Self> {
        if left != right {
            return Err(AssertEqError {
                left: format!("{:?}", left),
                right: format!("{:?}", right),
            });
        }

        Ok(())
    }
}
impl Error for AssertEqError {}
impl std::fmt::Display for AssertEqError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "assert_eq failed: left: {}, right: {}",
            self.left, self.right
        )
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PanicError {
    message: String,
}

impl PanicError {
    pub fn new(message: String) -> Self {
        PanicError { message }
    }
}

impl Error for PanicError {}
impl std::fmt::Display for PanicError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Panic: {}", self.message)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct WasmInterpreterError(wasm::Error);

impl Error for WasmInterpreterError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self.0 {
            wasm::Error::MalformedUtf8String(inner) => Some(inner),
            _ => None,
        }
    }
}

impl From<wasm::Error> for WasmInterpreterError {
    fn from(error: wasm::Error) -> Self {
        WasmInterpreterError(error)
    }
}

impl std::fmt::Display for WasmInterpreterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.0)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct GenericError(String);

impl GenericError {
    pub fn new(message: &str) -> Self {
        GenericError(message.to_string())
    }
}

impl Error for GenericError {}
impl std::fmt::Display for GenericError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
