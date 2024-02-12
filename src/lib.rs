use std::{collections::HashMap, sync::Arc};
use fluent::{bundle::FluentBundle, FluentResource, FluentMessage};
use intl_memoizer::concurrent::IntlLangMemoizer;
use unic_langid::LanguageIdentifier;
use crate::error::Result;

pub mod error;

/// Shorthand type handling the [FluentBundle]'s generic types.
type TypedFluentBundle = FluentBundle<Arc<FluentResource>, IntlLangMemoizer>;

/// The main struct of the program.
/// You can obtain a new instance by calling [Self::try_load()].
pub struct Localiser {
	pub bundles: HashMap<LanguageIdentifier, TypedFluentBundle>,
	pub default_language: LanguageIdentifier
}

impl Localiser {
	/// Tries to create a new [Localiser] instance given a path and the name of the default language.
	/// The path's direct children will only be considered if their names are valid language codes as
	/// defined by [LanguageIdentifier], and if they are either files with the `.ftl` extension or
	/// directories. In the first case they will be read directly and converted in [FluentResource]s,
	/// in the second case the same will be done to their chilren instead.
	/// [FluentResource]s within a same folder will be considered part of a same [FluentBundle],
	/// forming a single localisation for all intents and purposes.
	pub fn try_load(path: String, default_language: String) -> Result<Self> {
		let mut bundles = HashMap::new();
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

		let default_language = default_language.parse::<LanguageIdentifier>()?;

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

			bundles.insert(language_code, bundle);
		}

		Ok(Self {
			bundles,
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
	pub fn get_message(&self, key: String, language: LanguageIdentifier) -> Result<FluentMessage> {
		let bundle = self.bundles.get(&language)
			.or_else(|| self.bundles.get(&self.default_language))
			.ok_or(error::Error::GenericError("Failed to get default bundle! This is not supposed to happen!".to_string()))?;

		bundle.get_message(&key)
			.ok_or(error::Error::MissingMessageError(format!("No such message {} for language {}!", key, language)))
	}

	/// Returns a [HashMap] tying each [LanguageIdentifier] to its [String] equivalent, to simplify retrieval.
	/// Call this as little as possible, as it's rather unoptimised and may scale poorly.
	pub fn available_languages(&self) -> HashMap<String, LanguageIdentifier> {
		let mut res = HashMap::new();
		for lang in self.bundles.keys() {
			res.insert(lang.to_string(), lang.clone());
		}
		res
	}
}
