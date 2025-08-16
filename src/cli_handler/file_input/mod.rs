use std::{env::current_dir, path::PathBuf};

use anyhow::Context;
use inquire::Autocomplete;

use crate::provider::{ChatHistoryItem, FileUploadData};

pub const FILE_INPUT_TRIGGER: &'static str = "#file:";

#[derive(Clone)]
pub struct FileInputHandler {
    cwd: PathBuf,
}

impl FileInputHandler {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            cwd: current_dir().context("Failed to get cwd for File Input Handler.")?,
        })
    }

    pub fn chat_from_file(&self, path: &str) -> anyhow::Result<ChatHistoryItem> {
        let contents = std::fs::read_to_string(path).context("Failed to read file contents.")?;
        Ok(FileUploadData {
            text: contents,
            relative_filepath: path.to_owned(),
        }
        .into())
    }
}

impl Autocomplete for FileInputHandler {
    fn get_suggestions(&mut self, input: &str) -> Result<Vec<String>, inquire::CustomUserError> {
        if !input.starts_with(FILE_INPUT_TRIGGER) {
            Ok(Vec::new())
        } else {
            let path = input.trim_start_matches(FILE_INPUT_TRIGGER);
            let relative_path = self.cwd.join(path);

            let glob_matches = glob::glob(
                format!(
                    "{}{}",
                    relative_path
                        .to_str()
                        .context("Failed to convert path to unicode for file glob autocomplete.")?,
                    "*",
                )
                .as_str(),
            )
            .context("Failed to get glob completions.")?
            .filter_map(|path| path.ok().map(|value| value.into_os_string().into_string()))
            .filter_map(|path_string| path_string.ok())
            .map(|path_string| format!("{FILE_INPUT_TRIGGER}{path_string}"))
            .collect::<Vec<_>>();

            Ok(glob_matches)
        }
    }

    fn get_completion(
        &mut self,
        _input: &str,
        highlighted_suggestion: Option<String>,
    ) -> Result<inquire::autocompletion::Replacement, inquire::CustomUserError> {
        if let Some(suggestion) = highlighted_suggestion {
            let trailing =
                if std::path::Path::new(suggestion.trim_start_matches(FILE_INPUT_TRIGGER)).is_dir()
                {
                    &std::path::MAIN_SEPARATOR.to_string()
                } else {
                    ""
                };
            Ok(Some(format!("{}{}", suggestion, trailing)))
        } else {
            Ok(None)
        }
    }
}
