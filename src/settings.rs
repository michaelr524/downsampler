use config::{Config as Conf, ConfigError, File};
use serde::{de::Error as SerdeError, Deserialize, Deserializer};

#[derive(Debug, Deserialize)]
pub enum FieldDataType {
    Float,
    Integer,
    Boolean,
    String,
    //    Unknown(String),
}

impl FieldDataType {
    fn deserialize_with<'de, D>(de: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(de)?;
        let s = s.to_lowercase();

        match s.as_ref() {
            "float" => Ok(FieldDataType::Float),
            "integer" => Ok(FieldDataType::Integer),
            "boolean" => Ok(FieldDataType::Boolean),
            "string" => Ok(FieldDataType::String),
            val @ _ => Err(SerdeError::custom(format!(
                "Unrecognized field data type: {:?}",
                val
            ))),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Field {
    pub name: String,
    #[serde(deserialize_with = "FieldDataType::deserialize_with")]
    pub data_type: FieldDataType,
}

#[derive(Debug, Deserialize)]
pub struct InfluxDB {
    pub url: String,
    pub db: String,
    pub retention_policy: String,
    pub username: String,
    pub pass: String,
}

#[derive(Debug, Deserialize)]
pub struct Vars {
    pub ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct Downsampler {
    pub measurement_template: String,
    pub query_template: String,
    pub x_field_index: usize,
    pub y_field_index: usize,
    pub fields: Vec<Field>,
}

#[derive(Debug, Deserialize)]
pub struct Splitter {
    pub measurement_template: String,
    pub query_template: String,
    pub fields: Vec<Field>,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub influxdb: InfluxDB,
    pub vars: Vars,
    pub downsampler: Downsampler,
    pub splitter: Splitter,
}

pub fn config_from_file(filename: &str) -> Result<Config, ConfigError> {
    let mut settings = Conf::default();
    settings.merge(File::with_name(filename)).unwrap();

    // Print out our settings (as a HashMap)
    println!("{:?}", settings.clone().try_into::<Config>().unwrap());

    settings.try_into()
}
