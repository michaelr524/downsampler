# downsampler
Utilities for transforming InfluxDB time series data

## Get up and running
The basics are:

1. Install Rust (to build the project): https://rustup.rs/
1. git clone git@github.com:michaelr524/downsampler.git
1. cd downsampler
1. cargo build --release
1. Edit config.toml to your needs
1. Run like this: target/release/downsampler downsample -s ‘2018-08-13 15:22:06’ -e ‘2018-08-17 19:03:17’

The following operations are currently supported:

#### downsample
Create downsampled series from other series.

#### split
Creates many measurement with a single serie from measurements with multiple series.

#### listen - continuous downsampling
Continuously downsampling the configured series as new data arrives.
You need to have Redis running. 
Redis is used for notifying downsampler about new data. 
Additionally, Redis is used to store a checkpoint so that downsampler knows how to restart from the point where it stopped last time it ran.
The way to send updates to Redis is:
```
HSET downsampler_updates "binance_BTCUSDT" "1537710900000000000"
```
The key will be used in the downsampled series name (check `listen` > `measurement_template` in `config.toml`) and the value is the timestamp in nanoseconds format.
Downsampler will downsample all the data from the previous checkpoint up to the given timestamp.
