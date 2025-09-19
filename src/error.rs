use super::*;

#[derive(Debug)]
pub struct Error(pub anyhow::Error);

impl Display for Error {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl<E> From<E> for Error
where
  E: Into<anyhow::Error>,
{
  fn from(err: E) -> Self {
    Self(err.into())
  }
}

impl From<Error> for McpError {
  fn from(val: Error) -> Self {
    McpError {
      code: rmcp::model::ErrorCode(-1),
      message: val.0.to_string().into(),
      data: None,
    }
  }
}
