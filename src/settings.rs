use config::{Config as Conf, ConfigError, File};
use humantime::parse_duration;
use serde::{de::Error as DeserError, Deserialize, Deserializer};

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
            val @ _ => Err(DeserError::custom(format!(
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
pub struct Interval {
    pub name: String,
    pub duration_secs: u64,
}

// convert interval strings into u64 seconds interval periods
fn deserialize_intervals<'de, D>(de: D) -> Result<Vec<Interval>, D::Error>
where
    D: Deserializer<'de>,
{
    let v: Vec<String> = Vec::<String>::deserialize(de)?;

    v.into_iter()
        .map(|s| {
            let duration_std = parse_duration(&s).map_err(|e| {
                DeserError::custom(format!(
                    "Error parsing interval duration: {:?} ({:?})",
                    &s, &e
                ))
            })?;

            Ok(Interval {
                duration_secs: duration_std.as_secs() * 60,
                name: s,
            })
        })
        .collect()
}

#[derive(Debug, Deserialize)]
pub struct Downsampler {
    pub measurement_template: String,
    pub query_template: String,
    pub x_field_index: usize,
    pub y_field_index: usize,
    pub fields: Vec<Field>,
    #[serde(deserialize_with = "deserialize_intervals")]
    pub intervals: Vec<Interval>,
}

#[derive(Debug, Deserialize)]
pub struct Splitter {
    pub measurement_template: String,
    pub query_template: String,
    pub fields: Vec<Field>,
}

#[derive(Debug, Deserialize)]
pub struct Listener {
    pub redis_url: String,
    pub poll_sleep_ms: u64,
    pub measurement_template: String,
    pub query_template: String,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub influxdb: InfluxDB,
    pub vars: Vars,
    pub downsampler: Downsampler,
    pub splitter: Splitter,
    pub listen: Listener,
}

pub fn config_from_file(filename: &str) -> Result<Config, ConfigError> {
    let mut settings = Conf::default();
    settings.merge(File::with_name(filename)).unwrap();

    // Print out our settings (as a HashMap)
    println!("{:?}", settings.clone().try_into::<Config>().unwrap());

    settings.try_into()
}
