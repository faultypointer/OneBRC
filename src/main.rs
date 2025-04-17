use rayon::prelude::*;

use std::{
    collections::BTreeMap,
    fs::File,
    io::{BufRead, BufReader, Read, Seek, SeekFrom},
};

fn main() {
    let mut args = std::env::args();
    args.next();
    let file_path = args
        .next()
        .unwrap_or_else(|| "data/weather_stations_full.csv".into());

    // need to figure out how many chunks to distribute file to
    // so need to calculate the total size in bytes of file
    let file_size = std::fs::metadata(&file_path).unwrap().len();
    // no of threads available
    let num_chunk = rayon::current_num_threads() as u64;
    // approximate size of file
    let chunk_size = file_size / num_chunk;

    let station_data_distributed: Vec<StationDataMap> = (0..num_chunk)
        .into_par_iter()
        .map(|i| {
            let mut file = File::open(&file_path).unwrap();
            // start of each chunk. if its the first chunk start position is 0
            // otherwise we need to find out the position where the previous chunk ended
            // NOTE im sure this doesn't need a call to align if we store the previous value returned by
            // the align for end location. IDK how much impact it has on the performance tho.
            let start = if i == 0 {
                0
            } else {
                align_newline(&mut file, i * chunk_size)
            };

            // if its the last chunk we just read till the end of file otherwise we read till the
            // new line from its current chunk's approximate end location
            let end = if i == num_chunk - 1 {
                file_size
            } else {
                align_newline(&mut file, (i + 1) * chunk_size)
            };

            process_chunk_station_data(file, start, end)
        })
        .collect();

    let mut final_station_data: StationDataMap = StationDataMap::new();
    for map in station_data_distributed {
        for (station, data) in map {
            final_station_data
                .entry(station)
                .and_modify(|existing| {
                    existing.count += data.count;
                    existing.sum += data.sum;
                    existing.min = f64::min(existing.min, data.min);
                    existing.max = f64::max(existing.max, data.max);
                })
                .or_insert(data);
        }
    }

    print!("{{");
    for (station, data) in final_station_data.into_iter() {
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

type StationDataMap = BTreeMap<String, StationData>;

struct StationData {
    count: u32,
    min: f64,
    max: f64,
    sum: f64,
}

// heading into parallelism land with rayon
// need to define a function that takes a reference to a File
// a start and an end location then returns a BTreeMap on that specific part of
// the file betweem start and end location. This function assumes that start is the
// start of a new line(row) and the end of the end of a row (ie newline character)
fn process_chunk_station_data(mut file: File, start: u64, end: u64) -> StationDataMap {
    file.seek(SeekFrom::Start(start)).unwrap();
    let reader = BufReader::new(file);
    // finally using that BTreeMap and it improved a little
    let mut station_data: BTreeMap<String, StationData> = BTreeMap::new();
    let mut total_bytes = start;
    for line in reader.lines() {
        let line = line.unwrap();
        total_bytes += line.len() as u64;

        if total_bytes > end {
            break;
        }
        // When I first tried the split once method, the to_owned or something showed error so I assumed
        // I couldn't use the split_once method so I went to the split with vector and collect which I saw
        // in the flamegraph was taking a large portion. anyway this reduces the time by nearly 22 seconds
        let (station_name, temp) = line.split_once(';').unwrap();
        let station_name = station_name.to_owned();
        let temp: f64 = temp.trim().parse().unwrap();
        if let Some(data) = station_data.get_mut(&station_name) {
            data.count += 1;
            data.min = f64::min(data.min, temp);
            data.max = f64::max(data.max, temp);
            data.sum += temp;
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
    station_data
}

// now I need a function that will return the next closest location in the file where new line
// character appears given a file and a location. need this function to ensure that tbe end argument
// to process_chunk_station_data function is the location of the new line character
fn align_newline(file: &mut File, mut pos: u64) -> u64 {
    file.seek(SeekFrom::Start(pos)).unwrap();
    let mut buf = [0u8; 1];
    while let Ok(_) = file.read_exact(&mut buf) {
        pos += 1;
        if buf[0] == b'\n' {
            break;
        }
    }
    pos
}
