//! # Fluent, fluently
//! A simple library providing IO for [Project Fluent](https://projectfluent.org/).
//!
//! Sample usage:
//!
//! ```rust
//! let loc = fluent_fluently::Localiser::try_load("./locale".to_string(), "en-US".to_string()).unwrap();
//! let msg = loc.get_message("hello-world", loc.available_languages.get("it"));
//! println!("{}", msg);
//! ```
//!
//! The [FluentMessage] you obtained this way will automatically fall back on `en-US` if no locale
//! of the requested type was found. Though, if you want, you `bundles` is a [HashMap], so you can
//! certainly check whether a language is available manually if you so wish.

use std::{collections::HashMap, sync::Arc};
use fluent::{bundle::FluentBundle, FluentResource, FluentArgs};
use intl_memoizer::concurrent::IntlLangMemoizer;
use unic_langid::LanguageIdentifier;
use crate::error::Result;

pub mod error;

/// Shorthand type handling the [FluentBundle]'s generic types.
type TypedFluentBundle = FluentBundle<Arc<FluentResource>, IntlLangMemoizer>;

/// The main struct of the program.
/// You can obtain a new instance by calling [`Self::try_load()`].
pub struct Localiser {
	/// A [HashMap] tying each bundle to its language identifier.
	pub bundles: HashMap<String, TypedFluentBundle>,
	/// A [HashMap] tying each *available* language identifier [String] to an actual [LanguageIdentifier].
	pub available_languages: HashMap<String, LanguageIdentifier>,
	/// The identifier of the default language.
	pub default_language: String
}

impl Localiser {
	/// Tries to create a new [Localiser] instance given a path and the name of the default language.
	/// The path's direct children will only be considered if their names are valid language codes as
	/// defined by [LanguageIdentifier], and if they are either files with the `.ftl` extension or
	/// directories. In the first case they will be read directly and converted in [FluentResource]s,
	/// in the second case the same will be done to their chilren instead.
	/// [FluentResource]s within a same folder will be considered part of a same [FluentBundle],
	/// forming a single localisation for all intents and purposes.
	pub fn try_load(path: &str, default_language: &str) -> Result<Self> {
		let mut bundles = HashMap::new();
		let mut available_languages = HashMap::new();
		let paths = std::fs::read_dir(path)?
			.filter_map(|res| res.ok())
			.map(|dir_entry| dir_entry.path())
			.filter_map(|path| {
				if path.extension().map_or(false, |ext| ext == "ftl") || path.is_dir() {
					Some(path)
				} else {
					None
				}
			}).collect::<Vec<_>>();

		// validate default
		let default_language = default_language.parse::<LanguageIdentifier>()?.to_string();

		for path in paths {
			// validate filename as language code
			let language_code = path.file_stem()
				.and_then(|f| f.to_str())
				.map(|f| f.parse::<LanguageIdentifier>())
				.and_then(|id| match id {
					Ok(id) => Some(id),
					Err(_) => None
				});

			if language_code.is_none() {
				continue;
			}

			let language_code = language_code.unwrap();

			let mut bundle: TypedFluentBundle = fluent::bundle::FluentBundle::new_concurrent(vec![language_code.clone()]);
			if path.is_dir() { //is a directory
				for res in Self::path_to_resources(&path)? {
					bundle.add_resource(res)?;
				}
			} else { //is a single file
				bundle.add_resource(Self::file_to_resource(&path)?)?;
			}

			bundles.insert(language_code.to_string(), bundle);
			available_languages.insert(language_code.to_string(), language_code);
		}

		Ok(Self {
			bundles,
			available_languages,
			default_language
		})
	}

	/// Reads all files in a certain folder and all of its subfolders that have the `.ftl`
	/// extension, parses them into [FluentResource]s and returns them in a [Vec]. 
	fn path_to_resources(path: &std::path::PathBuf) -> Result<Vec<Arc<FluentResource>>> {
		let mut res = Vec::new();
		for entry in walkdir::WalkDir::new(path).follow_links(true).into_iter().filter_map(|e| e.ok()) {
			let entry_path = entry.path().to_path_buf();
			let entry_extension = entry_path.extension();
			if entry_extension.is_none() || entry_extension.unwrap() != "ftl" {
				continue;
			}

			res.push(Self::file_to_resource(&entry_path)?);
		}
		Ok(res)
	}

	/// Reads the file at the given path, and tries to parse it into a [FluentResource].
	fn file_to_resource(path: &std::path::PathBuf) -> Result<Arc<FluentResource>> {
		Ok(Arc::new(FluentResource::try_new(std::fs::read_to_string(path)?)?))
	}

	/// Extracts a message from the requested bundle, or from the default one if absent. 
	pub fn get_message(&self, key: &str, language: &str, args: Option<&FluentArgs>) -> Result<String> {
		let bundle = self.bundles.get(language)
			.or_else(|| self.bundles.get(&self.default_language))
			.ok_or(error::Error::GenericError("Failed to get default bundle! This is not supposed to happen!".to_string()))?;

		let pattern = bundle.get_message(key)
			.and_then(|msg| msg.value())
			.ok_or(error::Error::MissingMessageError(format!("No such message {} for language {}!", key, language)))?;

		let mut err = Vec::new();
		let res = bundle.format_pattern(pattern, args, &mut err);
		if err.is_empty() {
			Ok(res.to_string())
		} else {
			Err(error::Error::FluentError(err))
		}
	}
}
