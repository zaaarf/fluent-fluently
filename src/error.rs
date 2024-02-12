use std::result::Result as StdResult;

use fluent::FluentResource;

pub type Result<T> = StdResult<T, Error>;

#[derive(Debug)]
pub enum Error {
	IoError(std::io::Error),
	LanguageIdentifierError(unic_langid::LanguageIdentifierError),
	FluentError(Vec<fluent::FluentError>)
}

impl From<std::io::Error> for Error {
	fn from(err: std::io::Error) -> Self {
		Self::IoError(err)
	}
}

impl From<(FluentResource, Vec<fluent_syntax::parser::ParserError>)> for Error {
	fn from(err: (FluentResource, Vec<fluent_syntax::parser::ParserError>)) -> Self {
		let err = err.1.iter().map(|e| fluent::FluentError::ParserError(e.clone())).collect();
		Self::FluentError(err)
	}
}

impl From<Vec<fluent::FluentError>> for Error {
	fn from(err: Vec<fluent::FluentError>) -> Self {
		Self::FluentError(err)
	}
}

impl From<unic_langid::LanguageIdentifierError> for Error {
	fn from(err: unic_langid::LanguageIdentifierError) -> Self {
		Self::LanguageIdentifierError(err)
	}
}
