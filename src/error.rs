use std::result::Result as StdResult;

use fluent::FluentResource;

pub type Result<T> = StdResult<T, Error>;

pub enum Error {
	IoError(String)
}

impl From<std::io::Error> for Error {
	fn from(value: std::io::Error) -> Self {
		todo!()
	}
}

impl From<(FluentResource, Vec<fluent_syntax::parser::ParserError>)> for Error {
	fn from(value: (FluentResource, Vec<fluent_syntax::parser::ParserError>)) -> Self {
		todo!()
	}
}

impl From<Vec<fluent::FluentError>> for Error {
	fn from(value: Vec<fluent::FluentError>) -> Self {
		todo!()
	}
}

impl From<unic_langid::LanguageIdentifierError> for Error {
	fn from(value: unic_langid::LanguageIdentifierError) -> Self {
		todo!()
	}
}
