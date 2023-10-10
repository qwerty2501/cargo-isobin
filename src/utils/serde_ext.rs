use std::fmt::Display;

use super::*;
use io_ext::path_to_string;
use std::path::Path;
use tokio::fs::File;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use utils::fs_ext;

#[allow(dead_code)]
pub struct Json;

impl Json {
    #[allow(dead_code)]
    pub async fn save_to_file<T: serde::Serialize>(
        value: &T,
        path: impl AsRef<Path>,
    ) -> Result<()> {
        write_str_for_serialize(&(Self::serialize_string(value, path.as_ref())?), path).await
    }

    #[allow(dead_code)]
    pub async fn parse_from_file<T: serde::de::DeserializeOwned>(
        path: impl AsRef<Path>,
    ) -> Result<T> {
        let s = read_string_for_deserialize(path.as_ref()).await?;
        Self::deserialize_str(&s, path)
    }

    #[allow(dead_code)]
    pub async fn parse_or_default_if_not_found<T: serde::de::DeserializeOwned + Default>(
        path: impl AsRef<Path>,
    ) -> Result<T> {
        default_if_not_found(Self::parse_from_file(path).await)
    }

    fn serialize_string<T: serde::Serialize>(value: &T, path: impl AsRef<Path>) -> Result<String> {
        serde_json::to_string(value)
            .map_err(|e| SerdeExtError::new_serialize(e.into(), path_to_string(path)).into())
    }
    fn deserialize_str<T: serde::de::DeserializeOwned>(
        s: &str,
        path: impl AsRef<Path>,
    ) -> Result<T> {
        serde_json::from_str(s).map_err(|e| convert_deserialize_json_error(e, path, s).into())
    }
}

pub struct Yaml;

impl Yaml {
    #[allow(dead_code)]
    pub async fn save_to_file<T: serde::Serialize>(
        value: &T,
        path: impl AsRef<Path>,
    ) -> Result<()> {
        write_str_for_serialize(&(Self::serialize_string(value, path.as_ref())?), path).await
    }

    #[allow(dead_code)]
    pub async fn parse_from_file<T: serde::de::DeserializeOwned>(
        path: impl AsRef<Path>,
    ) -> Result<T> {
        let s = read_string_for_deserialize(path.as_ref()).await?;
        Self::deserialize_str(&s, path)
    }

    #[allow(dead_code)]
    pub async fn parse_or_default_if_not_found<T: serde::de::DeserializeOwned + Default>(
        path: impl AsRef<Path>,
    ) -> Result<T> {
        default_if_not_found(Self::parse_from_file(path).await)
    }
    fn serialize_string<T: serde::Serialize>(value: &T, path: impl AsRef<Path>) -> Result<String> {
        serde_yaml::to_string(value)
            .map_err(|e| SerdeExtError::new_serialize(e.into(), path_to_string(path)).into())
    }
    pub fn deserialize_str<T: serde::de::DeserializeOwned>(
        s: &str,
        path: impl AsRef<Path>,
    ) -> Result<T> {
        serde_yaml::from_str(s).map_err(|e| convert_deserialize_yaml_error(e, path, s).into())
    }
}

pub struct Toml;

impl Toml {
    #[allow(dead_code)]
    pub async fn save_to_file<T: serde::Serialize>(
        value: &T,
        path: impl AsRef<Path>,
    ) -> Result<()> {
        write_str_for_serialize(&(Self::serialize_string(value, path.as_ref())?), path).await
    }

    #[allow(dead_code)]
    pub async fn parse_from_file<T: serde::de::DeserializeOwned>(
        path: impl AsRef<Path>,
    ) -> Result<T> {
        let s = read_string_for_deserialize(path.as_ref()).await?;
        Self::deserialize_str(&s, path)
    }

    #[allow(dead_code)]
    pub async fn parse_or_default_if_not_found<T: serde::de::DeserializeOwned + Default>(
        path: impl AsRef<Path>,
    ) -> Result<T> {
        default_if_not_found(Self::parse_from_file(path).await)
    }

    fn serialize_string<T: serde::Serialize>(value: &T, path: impl AsRef<Path>) -> Result<String> {
        toml::to_string(value)
            .map_err(|e| SerdeExtError::new_serialize(e.into(), path_to_string(path)).into())
    }
    fn deserialize_str<T: serde::de::DeserializeOwned>(
        s: &str,
        path: impl AsRef<Path>,
    ) -> Result<T> {
        toml::from_str(s).map_err(|e| convert_deserialize_toml_error(e, path, s).into())
    }
}

#[derive(thiserror::Error, Debug, new)]
pub enum SerdeExtError {
    #[error("Not found file\npath:{path}")]
    NotFound {
        #[source]
        error: anyhow::Error,
        path: String,
    },
    #[error("An io error occurred\npath:{path}\nerror:{error}")]
    Io {
        #[source]
        error: anyhow::Error,
        path: String,
    },
    #[error("An deserialize error occurred\npath:{path}\nerror:{error}\n{hint}")]
    DeserializeWithHint {
        #[source]
        error: anyhow::Error,
        path: String,
        hint: ErrorHint,
    },
    #[error("An deserialize error occurred\npath:{path}\nerror:{error}")]
    Deserialize {
        #[source]
        error: anyhow::Error,
        path: String,
    },
    #[error("An serialize error occurred\npath:{path}\nerror:{error}")]
    Serialize {
        #[source]
        error: anyhow::Error,
        path: String,
    },
}

#[derive(PartialEq, new, Getters, Debug)]
pub struct ErrorHint {
    line: usize,
    column: usize,
    source: String,
}

impl ErrorHint {
    pub fn source_summary(&self) -> String {
        let lines = self.source.lines().collect::<Vec<_>>();
        let diff = std::cmp::min(self.line, 2);
        let summary_target = &lines[self.line - diff..=self.line];
        summary_target.join("\n")
    }
    pub fn hint(&self) -> String {
        let mut hint = vec!['_'; self.column];
        hint.push('^');
        String::from_iter(hint.iter())
    }
}

impl Display for ErrorHint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        let source_summary = self.source_summary();
        let hint = self.hint();
        f.write_fmt(format_args!("{source_summary}\n{hint}"))
    }
}

fn convert_io_error(e: io::Error, path: impl AsRef<Path>) -> SerdeExtError {
    if e.kind() == io::ErrorKind::NotFound {
        SerdeExtError::new_not_found(e.into(), path_to_string(path))
    } else {
        SerdeExtError::new_io(e.into(), path_to_string(path))
    }
}

fn convert_deserialize_json_error(
    e: serde_json::Error,
    path: impl AsRef<Path>,
    s: &str,
) -> SerdeExtError {
    if e.is_data() || e.is_syntax() {
        let hint = ErrorHint::new(e.line(), e.column(), s.into());
        SerdeExtError::new_deserialize_with_hint(e.into(), path_to_string(path), hint)
    } else {
        SerdeExtError::new_deserialize(e.into(), path_to_string(path))
    }
}

fn convert_deserialize_yaml_error(
    e: serde_yaml::Error,
    path: impl AsRef<Path>,
    s: &str,
) -> SerdeExtError {
    if let Some(location) = e.location() {
        let hint = ErrorHint::new(location.line(), location.column(), s.into());
        SerdeExtError::new_deserialize_with_hint(e.into(), path_to_string(path), hint)
    } else {
        SerdeExtError::new_deserialize(e.into(), path_to_string(path))
    }
}

fn convert_deserialize_toml_error(
    e: toml::de::Error,
    path: impl AsRef<Path>,
    s: &str,
) -> SerdeExtError {
    if let Some((line, col)) = e.line_col() {
        let hint = ErrorHint::new(line + 1, col + 1, s.into());
        SerdeExtError::new_deserialize_with_hint(e.into(), path_to_string(path), hint)
    } else {
        SerdeExtError::new_deserialize(e.into(), path_to_string(path))
    }
}

async fn write_str_for_serialize(s: &str, path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    let mut file = fs_ext::open_file_create_if_not_exists(path)
        .await
        .map_err(|e| convert_io_error(e, path))?;
    Ok(file
        .write_all(s.as_bytes())
        .await
        .map_err(|e| convert_io_error(e, path))?)
}

async fn read_string_for_deserialize(path: impl AsRef<Path>) -> Result<String> {
    let path = path.as_ref();
    let mut file = File::open(path)
        .await
        .map_err(|e| convert_io_error(e, path))?;
    let mut s = String::new();
    file.read_to_string(&mut s)
        .await
        .map_err(|e| convert_io_error(e, path))?;
    Ok(s)
}

fn default_if_not_found<T: Default>(result: Result<T>) -> Result<T> {
    match result {
        Ok(v) => Ok(v),
        Err(err) => match err.downcast::<SerdeExtError>() {
            Ok(_) => Ok(Default::default()),
            Err(err) => Err(err),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest]
    #[case(include_str!("testdata/make_hint_string_works/case1_json/given_json.json"), 
        2,
        2,
        include_str!("testdata/make_hint_string_works/case1_json/expected_hint.txt"),
        )]
    #[case(include_str!("testdata/make_hint_string_works/case1_json/given_json.json"), 
        4,
        16,
        include_str!("testdata/make_hint_string_works/case2_json/expected_hint.txt"),
        )]
    fn make_hint_string_works(
        #[case] s: &str,
        #[case] line: usize,
        #[case] column: usize,
        #[case] expected: &str,
    ) {
        let actual = ErrorHint::new(line, column, s.into());
        pretty_assertions::assert_eq!(expected, format!("{actual}"));
    }
}
