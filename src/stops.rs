use csv::ReaderBuilder;
use ordermap::OrderMap;

const STOP_CSV: &str = include_str!("sbhacks26_stops.csv");

#[derive(Debug, Clone)]
pub struct Stop {
    pub short_name: String,
    pub long_name: String,
    // 0 at san diego and end at sfo in seconds
    pub loc: f64,
}

// slow, fast
pub fn read_stops() -> (OrderMap<String, Stop>, OrderMap<String, Stop>) {
    let mut rdr = ReaderBuilder::new()
        .delimiter(b',')
        .from_reader(STOP_CSV.as_bytes());

    let mut slow_stops: OrderMap<String, Stop> = rdr.records()
        .into_iter()
        .map(|r| {
            let r = r.unwrap();
            (
                r[0].to_string(),
                Stop {
                    short_name: r[0].to_string(),
                    long_name: r[1].to_string(),
                    loc: r[2].parse().unwrap(),
                },
            )
        })
        .collect();

    let mut rdr = ReaderBuilder::new()
        .delimiter(b',')
        .from_reader(STOP_CSV.as_bytes());
    let mut fast_stops: OrderMap<String, Stop> = rdr.records().into_iter()
        .map(|r| {
            let r = r.unwrap();
            (
                r[0].to_string(),
                Stop {
                    short_name: r[0].to_string(),
                    long_name: r[1].to_string(),
                    loc: r[3].parse().unwrap(),
                },
            )
        })
        .collect();
    slow_stops.reverse();
    fast_stops.reverse();
    (slow_stops, fast_stops)
}

pub fn get_stop(stops: &OrderMap<String, Stop>, time_in_seconds: f64) -> (Stop, Stop) {
    let mut it = stops.iter();
    let mut prev = it.next().unwrap().1.clone();
    for (_, v) in stops {
        if v.loc > time_in_seconds {
            return (prev.clone(), v.clone());
        }
        prev = v.clone();
    }
    panic!("no stop found");
}

