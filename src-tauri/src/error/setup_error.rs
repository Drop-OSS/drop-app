use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub enum SetupError {
    Context,
}

impl Display for SetupError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SetupError::Context => write!(f, "failed to generate contexts for download"),
        }
    }
}
