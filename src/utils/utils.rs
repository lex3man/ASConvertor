use std::{io::Write, path::Path};

use calamine::{open_workbook, DataType, Reader, Xlsx};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use crate::utils::types::{get_point_types, PointType};

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Point {
    num: u16,
    name: String,
    r#type: String,
    odo: u32,
    lat: f32,
    lon: f32,
    max_speed: u16,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Race {
    code: String,
    event_name: String,
    race_name: String,
    sets: Settings,
    types: Vec<PointType>,
    points: Vec<Point>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Settings {
    total: u16,
    max_speed: u16,
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    days: Vec<Day>,
    races: RaceParams,
    point_types: PointTypes,
}

#[derive(Debug, Serialize, Deserialize)]
struct Day {
    code: String,
    points: Vec<Point>,
}

#[derive(Debug, Serialize, Deserialize)]
struct RaceParams {
    info: Info,
    sets: Settings,
}

#[derive(Debug, Serialize, Deserialize)]
struct Info {
    event_name: String,
    race_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct PointTypes {
    types: Vec<PointType>,
}

pub fn convert(file: &str, path: &str, dataset: &str) -> Result<(), String>{
    
    let mut wb: Xlsx<_> = open_workbook(file).map_err(|e: calamine::XlsxError| e.to_string())?;
    let sheets_names = wb.sheet_names();

    let mut point = Point {
        num: 0,
        name: "default".to_string(),
        r#type: "DEF".to_string(),
        odo: 0,
        lat: 0.0,
        lon: 0.0,
        max_speed: 0,
    };

    let point_types = get_point_types(dataset);
    let mut races: Vec<Race> = vec![];

    let sets = Settings {
        total: 0,
        max_speed: 110,
    };

    let filename = Path::new(file)
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("unknown");

    for sheet_name in sheets_names {
        let mut race = Race {
            code: "0000".to_string(),
            event_name: filename.to_string(),
            race_name: sheet_name.clone(),
            sets: sets.clone(),
            types: point_types.clone(),
            points: vec![],
        };
        let range = wb.worksheet_range(&sheet_name).unwrap();

        let mut cords = "".to_string();
        for cell in range.used_cells() {
            if cell.0 > 2 {
                match cell.1 {
                    0 => unsafe {
                        point.num = cell.2.get_float().unwrap().to_int_unchecked::<u16>();
                    },
                    1 => {
                        point.name = cell.2.get_string().unwrap().to_string();
                    }
                    2 => {
                        let t = cell.2.get_string().unwrap().to_string();
                        point.max_speed = 0;
                        if t.starts_with("FZ") {
                            point.r#type = "FZ".to_string();
                            if t.len() > 2 {
                                point.max_speed = t.replace("FZ", "").parse::<u16>().unwrap();
                            }
                        } else if t.starts_with("DZ") {
                            point.r#type = "DZ".to_string();
                            
                        } else {
                            point.r#type = t;
                        }
                        
                    }
                    3 => { cords = cell.2.get_string().unwrap().to_string(); },
                    4 => {
                        let dm_parts: Vec<&str> = cords.split('°').collect();
                        if dm_parts.len() != 2 {
                            return Err("Invalid degrees and minutes format".to_string());
                        }

                        let degrees_str = dm_parts[0];
                        let minutes_str = dm_parts[1].replace(',', ".");

                        let degrees = degrees_str.trim().parse::<f64>().map_err(|_| "Invalid degrees".to_string())?;
                        let minutes = minutes_str.trim().parse::<f64>().map_err(|_| "Invalid minutes".to_string())?;
                        if cell.2.get_string().unwrap().contains("N") {
                            point.lat = (degrees + minutes / 60.0) as f32;
                        } else {
                            point.lat = (-degrees - minutes / 60.0) as f32;
                        }
                    },
                    5 => { cords = cell.2.get_string().unwrap().to_string(); },
                    6 => {
                        let dm_parts: Vec<&str> = cords.split('°').collect();
                        if dm_parts.len() != 2 {
                            return Err("Invalid degrees and minutes format".to_string());
                        }

                        let degrees_str = dm_parts[0];
                        let minutes_str = dm_parts[1].replace(',', ".");

                        let degrees = degrees_str.trim().parse::<f64>().map_err(|_| "Invalid degrees".to_string())?;
                        let minutes = minutes_str.trim().parse::<f64>().map_err(|_| "Invalid minutes".to_string())?;
                        if cell.2.get_string().unwrap().contains("E") {
                            point.lon = (degrees + minutes / 60.0) as f32;
                        } else {
                            point.lon = (-degrees - minutes / 60.0) as f32;
                        }
                    },
                    7 => unsafe {
                        point.odo =
                            (cell.2.get_float().unwrap() * 1000.0).to_int_unchecked::<u32>();
                        if point.max_speed == 0 {
                            point.max_speed = sets.max_speed;
                            for t in race.types.clone() {
                                if t.caption == point.r#type {
                                    point.max_speed = t.max_speed;
                                }
                            }
                        }
                        race.points.push(point.clone());
                        if cell.0 == range.rows().count() - 1 {
                            race.points.push(point.clone());
                            races.push(race.clone())
                        }
                    },
                    // 4 => {
                    //     let coordinates = cell.2.get_string().unwrap().to_string();
                    //     let coords = convert_coordinates(&coordinates).unwrap();
                    //     if coords.len() > 1 {
                    //         let lat = coords[0];
                    //         let lon = coords[1];
                    //         point.lat = lat as f32;
                    //         point.lon = lon as f32;
                    //         point.max_speed = sets.max_speed;
                    //         for t in race.types.clone() {
                    //             if t.caption == point.r#type {
                    //                 point.max_speed = t.max_speed;
                    //             }
                    //         }
                    //         race.points.push(point.clone());
                    //     }
                    //     if cell.0 == range.rows().count() - 1 {
                    //         race.points.push(point.clone());
                    //         races.push(race.clone())
                    //     }
                    // }
                    _ => {}
                }
            }
        }
    }
    let (days, race_params, point_types) = build_config_from_races(races);
    create_file(days, race_params, point_types, path, filename);
    Ok(())
}


// fn convert_coordinates(input: &str) -> Result<Vec<f64>, String> {
//     let parts: Vec<&str> = input.split(';').collect();
//     if parts.len() != 2 {
//         return Err("Invalid input format".to_string());
//     }

//     let latitude = parse_coordinate(parts[0])?;
//     let longitude = parse_coordinate(parts[1])?;

//     Ok(vec![latitude, longitude])
// }


// fn parse_coordinate(value: &str) -> Result<f64, String> {
//     let components: Vec<&str> = value.split_whitespace().collect();
//     if components.len() != 2 {
//         return Err("Invalid coordinate format".to_string());
//     }
//     let degrees_minutes = components[0];
//     let direction = components[1].trim();

//     let dm_parts: Vec<&str> = degrees_minutes.split('°').collect();
//     if dm_parts.len() != 2 {
//         return Err("Invalid degrees and minutes format".to_string());
//     }

//     let degrees_str = dm_parts[0];
//     let minutes_str = dm_parts[1].replace(',', ".");

//     let degrees = degrees_str.parse::<f64>().map_err(|_| "Invalid degrees".to_string())?;
//     let minutes = minutes_str.parse::<f64>().map_err(|_| "Invalid minutes".to_string())?;

//     let sign = match direction {
//         "N" | "E" => 1.0,
//         "S" | "W" => -1.0,
//         _ => return Err("Invalid direction".to_string()),
//     };

//     let decimal = degrees + minutes / 60.0;
//     Ok(sign * decimal)
// }


fn create_file(days: Vec<Day>, race_params: RaceParams, point_types: PointTypes, path: &str, filename: &str) {
    let file_name = format!("{}/config_{}.ini", path, filename);

    let config = Config {
        days,
        races: race_params,
        point_types,
    };

    let toml = toml::to_string_pretty(&config).expect("Failed to serialize to TOML")
        .replace("[races.info]", "[[RACE_PARAMS]]\n[INFO]")
        .replace("[[days.points]]", "[[races.points]]")
        .replace(" = ", "=")
        .replace("[[point_types.types]]", "[[races.types]]");

    if std::path::Path::new(&file_name).exists() {
        std::fs::remove_file(&file_name).expect("Failed to remove existing file");
    }
    let mut file = std::fs::File::create(&file_name).expect("Failed to create file");
    file.write_all(toml.as_bytes()).expect("Failed to write to file");

    println!("Config saved to {}", file_name);
}


fn build_config_from_races(races: Vec<Race>) -> (Vec<Day>, RaceParams, PointTypes) {
    let days: Vec<Day> = races.iter().map(|race| {
        Day {
            code: race.race_name.clone(),
            points: race.points.clone(),
        }
    }).collect();

    let first_race = races.first().expect("No races provided");
    let race_params = RaceParams {
        info: Info {
            event_name: first_race.event_name.clone(),
            race_name: first_race.race_name.clone(),
        },
        sets: first_race.sets.clone(),
    };

    let mut types_set = HashSet::new();
    for race in &races {
        for pt in &race.types {
            types_set.insert(pt.clone());
        }
    }
    let mut types: Vec<PointType> = types_set.into_iter().collect();
    types.sort_by(|a, b| a.caption.cmp(&b.caption));
    let point_types = PointTypes { types };

    (days, race_params, point_types)
}


#[cfg(test)]
mod test {
    
    #[test]
    fn test_convert() -> Result<(), Box<dyn std::error::Error>>{

        use super::*;

        let _ = convert("data_for_config.xlsx", "output", "dataset.ini").map_err(|e| e.to_string())?;
        Ok(())
    }

}
