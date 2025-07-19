#[derive(Debug)]
pub enum Error {
  Io(std::io::Error),
  Poison,
  LinkCreate(&'static str),
  JoinHandle,
  FromInt,
}

impl std::fmt::Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match self {
      Self::Io(err) => writeln!(f, "I/O Error: {err}"),
      Self::Poison => writeln!(f, "Poison Error"),
      Self::LinkCreate(err) => writeln!(f, "Link creation error: {err}"),
      Self::JoinHandle => writeln!(f, "Join Handle Error"),
      Self::FromInt => writeln!(f, "Could not convert int to float32"),
    }
  }
}

impl std::error::Error for Error {
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
      match self {
          Error::Io(e) => Some(e),
          Error::Poison => None,
          Error::LinkCreate(_) => None,
          Error::JoinHandle => None,
          Error::FromInt => None,
      }
  }
}

impl From<std::io::Error> for Error { 
  fn from(err: std::io::Error) -> Error {
    Error::Io(err)
  }
}

impl<T> From<std::sync::PoisonError<T>> for Error {
  fn from(_value: std::sync::PoisonError<T>) -> Self {
      Error::Poison
  }
}

impl From<std::num::TryFromIntError> for Error {
  fn from(_err: std::num::TryFromIntError) -> Self {
    Error::FromInt
  }
}