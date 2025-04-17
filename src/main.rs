use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
};

struct StationData {
    count: u32,
    min: f64,
    max: f64,
    sum: f64,
}

fn main() {
    let mut args = std::env::args();
    _ = args.next();

    let data_file = match args.next() {
        None => File::open("data/weather_stations_full.csv").expect("Failed to read the data file"),
        Some(path_arg) => File::open(path_arg).expect("Failed to read the data file"),
    };

    let reader = BufReader::new(data_file);
    let mut station_data: HashMap<String, StationData> = HashMap::new();

    for line in reader.lines() {
        let line = line.unwrap().to_string();
        let line_split: Vec<&str> = line.split(';').collect();
        let station_name = line_split[0].to_owned();
        let temp: f64 = line_split[1].trim().parse().unwrap();
        if let Some(data) = station_data.get(line_split[0]) {
            // maybe just updating the previous data would yeild better result than creating a new one
            let new_data = StationData {
                count: data.count + 1,
                min: f64::min(data.min, temp),
                max: f64::max(data.max, temp),
                sum: data.sum + temp,
            };
            station_data.insert(station_name, new_data);
        } else {
            let data = StationData {
                count: 1,
                min: temp,
                max: temp,
                sum: temp,
            };
            station_data.insert(station_name, data);
        }
    }

    // the original problem had way more cities. the script I copied has only 8. so
    // sorting is really not that expensive I think.
    let mut v: Vec<_> = station_data.into_iter().collect();
    v.sort_by(|x, y| x.0.cmp(&y.0));
    print!("{{");
    for (station, data) in v {
        print!(
            "{}={:.1}/{:.1}/{:.1}, ",
            station,
            data.min,
            data.sum / data.count as f64,
            data.max
        );
    }
    print!("}}\n");
}
