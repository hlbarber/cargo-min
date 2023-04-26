use std::str::FromStr;

use thiserror::Error;
use toml_edit::{Document, Formatted, Item, Value};

#[derive(Debug, PartialEq)]
struct Dependency<'a> {
    name: String,
    version: Version<'a>,
}

#[derive(Debug, PartialEq)]
struct Version<'a> {
    value: &'a mut Formatted<String>,
    version: semver::Version,
    changed: bool,
}

impl<'a> Version<'a> {
    fn new(value: &'a mut Formatted<String>) -> Result<Self, semver::Error> {
        Ok(Self {
            version: semver::Version::from_str(value.value())?,
            value,
            changed: false,
        })
    }
}

impl Version<'_> {
    fn get(&self) -> &semver::Version {
        &self.version
    }

    fn set(&mut self, version: semver::Version) {
        if version != self.version {
            self.changed = true;
            self.version = version;
        }
    }
}

impl Drop for Version<'_> {
    fn drop(&mut self) {
        if self.changed {
            *self.value = Formatted::new(self.version.to_string());
        }
    }
}

enum DependencyType {
    Standard,
    Dev,
}

impl DependencyType {
    const STANDARD: &str = "dependencies";
    const DEV: &str = "dev-dependencies";
}

#[derive(Debug, Error)]
enum FetchDependenciesError {
    #[error("missing dependency group")]
    MissingDependencyItem,
    #[error("unexpected dependencies type \"{ty}\"")]
    UnexpectedDependenciesItem { ty: &'static str },
    #[error("unexpected dependency type \"{ty}\" for \"{key}\"")]
    UnexpectedDependencyItem { key: String, ty: &'static str },
    #[error("unexpected version type \"{ty}\" for \"{key}\"")]
    UnexpectedVersionType { key: String, ty: &'static str },
    #[error("failed to parse dependency \"{key}\"")]
    SemverParse {
        key: String,
        #[source]
        error: semver::Error,
    },
    #[error("missing version key in dependency \"{key}\"")]
    MissingVersionKey { key: String },
}

fn value_to_type(value: &Value) -> &'static str {
    match value {
        Value::String(_) => "String",
        Value::Integer(_) => "Integer",
        Value::Float(_) => "Float",
        Value::Boolean(_) => "Boolean",
        Value::Datetime(_) => "Datetime",
        Value::Array(_) => "Array",
        Value::InlineTable(_) => "InlineTable",
    }
}

fn item_to_type(item: &Item) -> &'static str {
    match item {
        Item::None => "None",
        Item::Value(_) => "Value",
        Item::Table(_) => "Table",
        Item::ArrayOfTables(_) => "ArrayOfTables",
    }
}

fn fetch_dependencies(
    document: &mut Document,
    ty: DependencyType,
) -> Result<Vec<Dependency<'_>>, FetchDependenciesError> {
    use FetchDependenciesError::*;

    let dependency_group = match ty {
        DependencyType::Standard => DependencyType::STANDARD,
        DependencyType::Dev => DependencyType::DEV,
    };

    let dependencies = document
        .get_mut(dependency_group)
        .ok_or(MissingDependencyItem)?;

    let dependencies = match dependencies {
        Item::Table(table) => table,
        other => {
            return Err(UnexpectedDependenciesItem {
                ty: item_to_type(other),
            })
        }
    };

    let depedencies: Result<Vec<Dependency>, _> = dependencies
        .iter_mut()
        .map(move |(key, item)| {
            let key = key.get().to_string();
            match item {
                Item::Value(value) => match value {
                    Value::String(value) => Ok(Dependency {
                        version: match Version::new(value) {
                            Ok(ok) => ok,
                            Err(error) => return Err(SemverParse { key, error }),
                        },
                        name: key,
                    }),
                    Value::InlineTable(table) => {
                        let Some(value) = table.get_mut("version") else {
                            return Err(MissingVersionKey { key })
                        };
                        match value {
                            Value::String(value) => Ok(Dependency {
                                version: match Version::new(value) {
                                    Ok(ok) => ok,
                                    Err(error) => return Err(SemverParse { key, error }),
                                },
                                name: key,
                            }),
                            other => Err(UnexpectedVersionType {
                                key,
                                ty: value_to_type(other),
                            }),
                        }
                    }
                    other => Err(UnexpectedVersionType {
                        key,
                        ty: value_to_type(other),
                    }),
                },
                Item::Table(_) => todo!("support dependency tables"),
                other => Err(UnexpectedDependencyItem {
                    key,
                    ty: item_to_type(other),
                }),
            }
        })
        .collect();
    depedencies
}

// fn parse_file(file: &mut File) -> Document {
//     let mut raw = String::new();
//     file.read_to_string(&mut raw)
// }

#[cfg(test)]
mod tests {
    use super::*;

    const ROOT_TOML: &str = include_str!("../Cargo.toml");

    #[test]
    fn check_dependencies() {
        let mut document = ROOT_TOML.parse().unwrap();
        let expected_depedencies = [
            ("clap", semver::Version::new(4, 2, 4)),
            ("semver", semver::Version::new(1, 0, 17)),
            ("thiserror", semver::Version::new(1, 0, 40)),
            ("toml_edit", semver::Version::new(0, 19, 8)),
        ];
        let actual_dependencies: Vec<_> =
            fetch_dependencies(&mut document, DependencyType::Standard).unwrap();
        let zipped = expected_depedencies.iter().zip(actual_dependencies);
        for ((expected_name, expected_version), Dependency { name, version }) in zipped {
            pretty_assertions::assert_eq!(
                (*expected_name, expected_version),
                (name.as_str(), &version.version)
            );
        }
    }
}
