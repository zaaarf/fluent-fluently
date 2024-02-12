use std::{collections::HashMap, sync::Arc};
use fluent::{bundle::FluentBundle, FluentResource};
use intl_memoizer::concurrent::IntlLangMemoizer;
use unic_langid::LanguageIdentifier;
use crate::error::Result;

pub mod error;

type TypedFluentBundle = FluentBundle<Arc<FluentResource>, IntlLangMemoizer>;
pub struct Localiser {
	pub bundles: HashMap<LanguageIdentifier, TypedFluentBundle>,
	pub default_language: LanguageIdentifier
}

impl Localiser {
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

		//TODO load default first and call bundle.add_resource_overriding(default_bundle) on others
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

	fn path_to_resources(path: &std::path::PathBuf) -> Result<Vec<Arc<FluentResource>>> {
		let mut res = Vec::new();
		for entry in walkdir::WalkDir::new(path).follow_links(true).into_iter().filter_map(|e| e.ok()) {
			let entry_path = entry.path().to_path_buf();
			res.push(Self::file_to_resource(&entry_path)?);
		}
		Ok(res)
	}

	fn file_to_resource(path: &std::path::PathBuf) -> Result<Arc<FluentResource>> {
		let content = std::fs::read_to_string(path)?;
		Ok(Arc::new(FluentResource::try_new(content)?))
	}
}
