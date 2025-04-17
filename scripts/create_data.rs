use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::Path;

fn main() {
    // The file created will have the following structure:
    // Each line will contain a city name and a temperature separated by a semicolon.
    // Format: CityName;Temperature
    // Example: Zaragoza;26.0

    let cities = [
        "Zaragoza",
        "Madrid",
        "Graus",
        "Torrelabad",
        "Las Vegas",
        "New York",
        "Paris",
        "London",
    ];

    let temperatures = [
        -5.0, 0.0, 5.0, 10.0, 15.0, 20.0, 25.0, 30.0, 35.0, 40.0, -4.5, 1.0, 6.5, 11.0, 16.0, 21.5,
        26.0, 31.0, 36.0, 41.5,
    ];

    let dir_path = Path::new("./data");
    if !dir_path.exists() {
        fs::create_dir_all(dir_path).unwrap();
    }

    let file = File::create(dir_path.join("weather_stations_small.csv")).unwrap();
    let mut writer = BufWriter::new(file);

    let num_entries = 50_000_000;
    for i in 0..num_entries {
        let city = cities[i % cities.len()];
        let temperature = temperatures[i % temperatures.len()];
        writeln!(writer, "{};{:.1}", city, temperature).unwrap();
    }
}
