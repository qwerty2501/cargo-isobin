use super::*;
use async_std::fs::File;
use async_std::io::{self, ReadExt, WriteExt};
use async_std::path::{Path, PathBuf};
use utils::fs_ext;

#[allow(dead_code)]
pub struct Json;

impl Json {
    #[allow(dead_code)]
    pub async fn save<T: serde::Serialize>(value: &T, path: impl AsRef<Path>) -> Result<()> {
        write_str_for_serialize(&(Self::serialize_string(value, path.as_ref())?), path).await
    }

    #[allow(dead_code)]
    pub async fn parse<T: serde::de::DeserializeOwned>(path: impl AsRef<Path>) -> Result<T> {
        let s = read_string_for_deserialize(path.as_ref()).await?;
        Self::deserialize_str(&s, path)
    }

    fn serialize_string<T: serde::Serialize>(value: &T, path: impl AsRef<Path>) -> Result<String> {
        serde_json::to_string(value)
            .map_err(|e| SerdeExtError::new_serialize(e.into(), path.as_ref().into()))
    }
    fn deserialize_str<T: serde::de::DeserializeOwned>(
        s: &str,
        path: impl AsRef<Path>,
    ) -> Result<T> {
        serde_json::from_str(s).map_err(|e| convert_deserialize_json_error(e, path, s))
    }
}

#[derive(thiserror::Error, Debug, new)]
pub enum SerdeExtError {
    #[error("Not found file\npath:{path:?}")]
    NotFound {
        #[source]
        error: anyhow::Error,
        path: PathBuf,
    },
    #[error("An io error occurred\npath:{path:?}\nerror:{error}")]
    Io {
        #[source]
        error: anyhow::Error,
        path: PathBuf,
    },
    #[error("An deserialize error occurred\npath:{path:?}\nerror:{error}\n{hint}")]
    DeserializeWithHint {
        #[source]
        error: anyhow::Error,
        path: PathBuf,
        hint: String,
    },
    #[error("An deserialize error occurred\npath:{path:?}\nerror:{error}")]
    Deserialize {
        #[source]
        error: anyhow::Error,
        path: PathBuf,
    },
    #[error("An serialize error occurred\npath:{path:?}\nerror:{error}")]
    Serialize {
        #[source]
        error: anyhow::Error,
        path: PathBuf,
    },
}

fn convert_io_error(e: io::Error, path: impl AsRef<Path>) -> SerdeExtError {
    if e.kind() == io::ErrorKind::NotFound {
        SerdeExtError::new_not_found(e.into(), path.as_ref().into())
    } else {
        SerdeExtError::new_io(e.into(), path.as_ref().into())
    }
}

fn convert_deserialize_json_error(
    e: serde_json::Error,
    path: impl AsRef<Path>,
    s: &str,
) -> SerdeExtError {
    if e.is_data() || e.is_syntax() {
        let hint = make_hint_string(s, e.line(), e.column());
        SerdeExtError::new_deserialize_with_hint(e.into(), path.as_ref().into(), hint)
    } else {
        SerdeExtError::new_deserialize(e.into(), path.as_ref().into())
    }
}

fn make_hint_string(s: &str, line: usize, column: usize) -> String {
    let lines = s
        .lines()
        .map(|line| line.chars().collect::<Vec<_>>())
        .collect::<Vec<_>>();
    let hint_target = &lines[line - 3..line];
    let mut new_lines = Vec::from_iter(hint_target);
    let mut hint = vec!['_'; if column > 0 { column - 1 } else { column }];
    hint.push('^');
    hint.push('\n');
    new_lines.push(&hint);

    new_lines
        .iter()
        .map(|line| String::from_iter(line.iter()))
        .collect::<Vec<_>>()
        .join("\n")
}

type Result<T> = std::result::Result<T, SerdeExtError>;

async fn write_str_for_serialize(s: &str, path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    let mut file = fs_ext::open_file_create_if_not_exists(path)
        .await
        .map_err(|e| convert_io_error(e, path))?;
    file.write_all(s.as_bytes())
        .await
        .map_err(|e| convert_io_error(e, path))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest]
    #[case(include_str!("testdata/make_hint_string_works/case1_json/given_json.json"), 
        3,
        3,
        include_str!("testdata/make_hint_string_works/case1_json/expected_hint.txt"),
        )]
    #[case(include_str!("testdata/make_hint_string_works/case1_json/given_json.json"), 
        5,
        17,
        include_str!("testdata/make_hint_string_works/case2_json/expected_hint.txt"),
        )]
    fn make_hint_string_works(
        #[case] s: &str,
        #[case] line: usize,
        #[case] column: usize,
        #[case] expected: &str,
    ) {
        let actual = make_hint_string(s, line, column);
        pretty_assertions::assert_eq!(expected, actual);
    }
}
