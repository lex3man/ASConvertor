use std::{
    fs::File,
    io::{BufRead, BufReader},
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct PointType {
    pub caption: String,
    pub default_rad: u16,
    pub is_open: bool,
    pub ghost: bool,
    pub in_game: bool,
    pub arrow_threshold: u16,
    pub max_speed: u16,
}

pub fn get_point_types(dataset_path: &str) -> Vec<PointType> {
    let mut point_types: Vec<PointType> = vec![];
    if let Ok(file) = File::open(dataset_path) {
        let reader = BufReader::new(file);
        let mut max_off_race: u16 = 110;
        let mut max_on_race: u16 = 80;

        let mut name: String = "".to_string();
        let mut default_rad: u16 = 90;
        let mut is_open: bool = false;
        let mut in_game: bool = false;
        let mut arrow_threshold: u16 = 800;
        let mut max_speed: u16 = 140;

        for line in reader.lines() {
            let line = line.unwrap();
            if line.starts_with("off_race") {
                max_off_race = line.split("=").nth(1).unwrap().parse::<u16>().unwrap();
            } else if line.starts_with("on_race") {
                max_on_race = line.split("=").nth(1).unwrap().parse::<u16>().unwrap();
            } else if line.starts_with("[point_types.") {
                let point_type: Vec<&str> = line.split(".").collect();
                if point_type.len() > 1 {
                    name = point_type[1].replace("]", "")
                }
                if name == "WPV" || name == "DSS" {
                    max_speed = max_off_race;
                } else {
                    max_speed = max_on_race;
                }
            } else if line.starts_with("default_rad") {
                default_rad = line.split("=").nth(1).unwrap().parse::<u16>().unwrap();
            } else if line.starts_with("is_open") {
                is_open = line.split("=").nth(1).unwrap().parse::<bool>().unwrap();
            } else if line.starts_with("in_game") {
                in_game = line.split("=").nth(1).unwrap().parse::<bool>().unwrap();
            } else if line.starts_with("arrow_threshold") {
                arrow_threshold = line.split("=").nth(1).unwrap().parse::<u16>().unwrap();
            } else if line.starts_with("max_speed") {
                max_speed = line.split("=").nth(1).unwrap().parse::<u16>().unwrap();
            } else if line.starts_with("-") {
                let pt = PointType {
                    caption: name,
                    default_rad,
                    is_open,
                    in_game,
                    arrow_threshold,
                    max_speed,
                    ghost: false,
                };
                point_types.push(pt.clone());
                name = "".to_string();
                default_rad = 90;
                is_open = false;
                in_game = false;
                arrow_threshold = 800;
                max_speed = 140;
            }
        }
    } else {
        println!("Error: Could not open dataset file");
        let point_types_captions = [
            "WPV", "WPM", "WPS", "WPE", "DSS", "FZ", "DZ", "WPC", "ASS", "default",
        ];
        for t in point_types_captions {
            let mut pt = PointType {
                caption: t.to_string(),
                default_rad: 90,
                is_open: false,
                ghost: false,
                in_game: true,
                arrow_threshold: 800,
                max_speed: 140,
            };
            match t {
                "WPV" => {
                    pt.default_rad = 200;
                    pt.is_open = true;
                    pt.in_game = false;
                }
                "WPM" => {
                    pt.max_speed = 90;
                    pt.default_rad = 50;
                    pt.arrow_threshold = 800;
                }
                "WPS" => {
                    pt.default_rad = 50;
                    pt.max_speed = 90;
                    pt.arrow_threshold = 1000;
                }
                "WPE" => {
                    pt.max_speed = 90;
                    pt.arrow_threshold = 5000;
                }
                "DSS" => {
                    pt.default_rad = 200;
                    pt.in_game = false;
                    pt.is_open = true;
                }
                "ASS" => {
                    pt.max_speed = 90;
                    pt.arrow_threshold = 1000;
                }
                "FZ" => {
                    pt.max_speed = 40;
                    pt.default_rad = 90;
                    pt.is_open = true;
                }
                "WPC" => {
                    pt.is_open = true;
                    pt.max_speed = 90;
                    pt.ghost = true;
                }
                _ => {}
            }
            point_types.push(pt);
        }
    }

    point_types
}

