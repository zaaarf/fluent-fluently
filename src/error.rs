use std::result::Result as StdResult;

use fluent::FluentResource;

pub type Result<T> = StdResult<T, Error>;

/// Simple wrapper around the errors that may occur during the program's execution.
#[derive(Debug)]
pub enum Error {
	/// A generic error - you are not supposed to ever actually encounter this, but it beats
	/// using a wild unwrap in a library.
	GenericError(String),
	/// Wraps a [`std::io::Errror`] that occurred while reading the files.
	IoError(std::io::Error),
	/// Wraps a [`unic_langid::LanguageIdentifierError`] that occurred while parsing a language
	/// identifier.
	LanguageIdentifierError(unic_langid::LanguageIdentifierError),
	/// Wraps any number of [`fluent::FluentError`] that have occurred while parsing.
	FluentError(Vec<fluent::FluentError>),
	/// Happens when you try to get a message that does not actually exist.
	MissingMessageError(String)
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
