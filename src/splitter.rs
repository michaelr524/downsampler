use chrono::NaiveDateTime;
use influx::{
    extract_timestamp, get_range, influx_client, json_val_to_influx_val, save_points, Error,
    SeriesResult,
};
use influx_db_client::Point;
use rayon::prelude::*;
use settings::Settings;
use std::process::exit;
use time::Duration;
use utils::time::IntervalIterator;

//#[derive(Fail, Debug)]
//pub enum Error {
//}

pub fn split(settings: &Settings) -> () {
    let client = influx_client();

    pair_names().par_iter()
//        .take(1)
        .for_each(|pair_name| {
            println!("start {}", pair_name);

            let measurement = format!("trades_binance_{pair_name}_raw", pair_name = pair_name);

            for (i, (start, end)) in intervals(settings.start, settings.end, Duration::hours(1))
                .enumerate()
//            .take(1)
                {
                    let series = match get_range(&client, pair_name, start, end) {
                        Ok(series) => series,
                        Err(err) => match err {
                            Error::NoResult => continue,
                            e => {
                                println!("Error: {}", e);
                                exit(-1)
                            }
                        },
                    };

                    let count = series.values.iter().count();

//                    println!("{} - [{} - {}] ({})", i, start, end, count);

                    let points = to_points(series, &measurement).unwrap_or_else(|e| {
                        println!("Error: {}", e);
                        exit(-1)
                    });

//                println!("{:#?}", points);

                    // TODO: handle errors
                    save_points(&client, "glukoz-rp", points);
//                        {
//                        Err(e) => {
//
//                        }
//                        _ => continue
//                    }
                }
            println!("end {}", pair_name);
        });
}

pub fn to_points(series: SeriesResult, measurement: &str) -> Result<Vec<Point>, Error> {
    series
        .values
        .iter()
        .map(|record| {
            let mut point = Point::new(measurement);

            let ts = extract_timestamp(&record)?;
            point.add_timestamp(ts);

            for (val, column_name) in record
                .into_iter()
                .skip(1)
                .zip(series.columns.iter().skip(1))
            {
                let influx_val = json_val_to_influx_val(val)?;
                point.add_field(column_name, influx_val);
            }

            Ok(point)
        })
        .collect()
}

pub fn intervals(start: NaiveDateTime, end: NaiveDateTime, step: Duration) -> IntervalIterator {
    IntervalIterator {
        end,
        cur: start,
        prev: start,
        step,
    }
}

pub fn pair_names() -> Vec<&'static str> {
    vec![
        "QSPBNB", "HOTBTC", "POEBTC", "XMRETH", "PHXBTC", "STORMBTC", "SCBNB", "ONTETH", "AEBNB",
        "KMDBTC", "STEEMBTC", "NCASHBTC", "XLMETH", "NXSBNB", "STEEMETH", "OSTBNB", "ZILBTC",
        "OSTETH", "LSKBNB", "POWRBTC", "DATAETH", "RLCBTC", "CVCETH", "VENUSDT", "AMBETH",
        "WABIBTC", "VETBTC", "TNTETH", "WINGSETH", "NPXSBTC", "THETAETH", "NCASHBNB", "PIVXETH",
        "DASHETH", "MFTBTC", "MDABTC", "PPTBTC", "REQBTC", "TUSDUSDT", "XRPBNB", "RDNBTC",
        "XLMUSDT", "TRIGETH", "RDNBNB", "NEBLBTC", "LSKETH", "MANABTC", "QLCETH", "STRATBTC",
        "MDAETH", "AGIETH", "DLTETH", "POEETH", "WANETH", "VIBETH", "IOTXBTC", "KMDETH", "SNTBTC",
        "VIBEETH", "IOSTBTC", "SUBBTC", "BCPTETH", "IOTABTC", "ARNETH", "ENGETH", "OMGETH",
        "XLMBTC", "NULSUSDT", "STORMBNB", "CHATBTC", "ASTBTC", "TUSDBNB", "GVTETH", "MFTBNB",
        "DOCKETH", "RCNBTC", "PIVXBTC", "KEYETH", "CMTBNB", "SKYETH", "ZENETH", "REPBNB",
        "ARDRETH", "BLZBNB", "LOOMBNB", "ENJBTC", "NXSETH", "XVGETH", "SCETH", "BLZETH", "BCPTBTC",
        "MTHETH", "ETCUSDT", "GNTETH", "IOTABNB", "ELFBTC", "LUNBTC", "DENTETH", "BCCBTC",
        "SALTBTC", "SYSBTC", "EOSBNB", "TRXUSDT", "YOYOBNB", "MCOBNB", "TNTBTC", "BQXBTC",
        "VENBNB", "QKCETH", "REPBTC", "BRDBNB", "GRSETH", "ZECETH", "QTUMUSDT", "ENJBNB",
        "AIONBTC", "VIAETH", "VIBBTC", "XLMBNB", "RLCETH", "PHXBNB", "ONTBNB", "ZRXETH", "TNBBTC",
        "ARDRBNB", "NPXSETH", "POWRETH", "CLOAKBTC", "CLOAKETH", "XEMBTC", "NASETH", "ICXUSDT",
        "TUSDETH", "STORJBTC", "NEBLETH", "ZILETH", "RCNETH", "TRXBNB", "PAXUSDT", "QSPETH",
        "STRATETH", "ARDRBTC", "ADXBNB", "QTUMBNB", "SNMBTC", "BCCUSDT", "BCNBTC", "POLYBTC",
        "ZECBTC", "ADXETH", "DGDBTC", "LTCBTC", "WTCETH", "QTUMBTC", "SNGLSBTC", "PAXBNB",
        "XMRBTC", "THETABNB", "VIABTC", "STEEMBNB", "ZENBNB", "BTCUSDT", "WABIBNB", "BATETH",
        "VIABNB", "VETUSDT", "EVXETH", "CNDBTC", "BCCBNB", "EOSBTC", "ICXETH", "LUNETH", "LTCUSDT",
        "BCDBTC", "INSETH", "ETHUSDT", "MCOBTC", "GXSETH", "CNDBNB", "NANOETH", "AEBTC", "CVCBTC",
        "ONTUSDT", "ICXBTC", "NEOBTC", "SKYBTC", "NAVBTC", "ADABTC", "LTCETH", "TRXBTC", "VETBNB",
        "BATBTC", "GTOBNB", "NEOETH", "WPRBTC", "SKYBNB", "MODETH", "IOTAETH", "XEMBNB", "OMGBTC",
        "DNTETH", "NULSBNB", "PPTETH", "RPXBTC", "KNCBTC", "IOTXETH", "WTCBNB", "TRIGBTC",
        "WAVESBTC", "WAVESBNB", "FUNBTC", "BNBUSDT", "RCNBNB", "TNBETH", "BNTETH", "XZCBNB",
        "ETHBTC", "GNTBTC", "CMTETH", "POLYBNB", "ONTBTC", "XRPUSDT", "REPETH", "AGIBTC", "WTCBTC",
        "BTSETH", "BTGETH", "MODBTC", "POWRBNB", "RPXBNB", "SYSBNB", "EDOETH", "GTOETH", "KNCETH",
        "CVCBNB", "ZRXBTC", "SYSETH", "MANAETH", "WAVESETH", "APPCBTC", "CMTBTC", "RDNETH",
        "CNDETH", "OAXBTC", "FUNETH", "SNTETH", "ETCETH", "WANBTC", "AGIBNB", "GOBTC", "XZCBTC",
        "ICNETH", "RLCBNB", "NANOBNB", "QKCBTC", "LENDETH", "YOYOETH", "NULSBTC", "PAXBTC",
        "POAETH", "QSPBTC", "THETABTC", "GXSBTC", "NEBLBNB", "LOOMBTC", "MFTETH", "BTSBNB",
        "LRCETH", "EOSETH", "GOBNB", "NXSBTC", "ARKBTC", "HSRBTC", "WANBNB", "LRCBTC", "ADAETH",
        "BCNBNB", "NULSETH", "BNBETH", "FUELBTC", "OAXETH", "XVGBTC", "CDTETH", "TRIGBNB",
        "LINKBTC", "BQXETH", "MTHBTC", "ADAUSDT", "WINGSBTC", "YOYOBTC", "PAXETH", "DLTBNB",
        "LTCBNB", "DATABTC", "BRDETH", "NAVETH", "MTLBTC", "ZILBNB", "RVNBNB", "LSKBTC", "TRXETH",
        "REQETH", "HCBTC", "ARKETH", "POABNB", "LENDBTC", "ELFETH", "QTUMETH", "BCCETH", "BTSBTC",
        "QLCBNB", "STORJETH", "NANOBTC", "NASBTC", "NASBNB", "CHATETH", "AMBBTC", "HCETH",
        "ICNBTC", "VIBEBTC", "MCOETH", "EOSUSDT", "INSBTC", "BCDETH", "BRDBTC", "BLZBTC", "GTOBTC",
        "BNTBTC", "AMBBNB", "KEYBTC", "AEETH", "ENJETH", "PIVXBNB", "XZCETH", "RVNBTC", "NEOBNB",
        "SALTETH", "ETCBNB", "ARNBTC", "VENETH", "OSTBTC", "SNGLSETH", "ETCBTC", "BCPTBNB",
        "SUBETH", "ADXBTC", "WABIETH", "CDTBTC", "WPRETH", "GVTBTC", "GRSBTC", "LOOMETH", "VENBTC",
        "ADABNB", "LINKETH", "BCNETH", "NCASHETH", "ICXBNB", "EDOBTC", "HSRETH", "POABTC",
        "ZENBTC", "NEOUSDT", "BNBBTC", "DGDETH", "SCBTC", "XEMETH", "TUSDBTC", "APPCBNB",
        "AIONETH", "RPXETH", "PHXETH", "GNTBNB", "MTLETH", "FUELETH", "HOTETH", "EVXBTC",
        "DASHBTC", "XRPETH", "GASBTC", "ENGBTC", "DOCKBTC", "XRPBTC", "STORMETH", "DLTBTC",
        "DENTBTC", "BATBNB", "NAVBNB", "IOSTETH", "AIONBNB", "SNMETH", "DNTBTC", "APPCETH",
        "QLCBTC", "ASTETH", "BTGBTC", "VETETH", "IOTAUSDT",
    ]
}
