use std::fs::File;
use std::io::{self, BufRead, Write};
use std::time::Duration;
use chrono::{Utc, NaiveDateTime, DateTime};
use rand::Rng;

#[derive(Debug)]
struct Coordinate {
    latitude: f64,
    longitude: f64,
}

// 좌표와 시작 시간을 파일에서 읽어들이는 함수
fn read_coordinates_and_time_from_file(filename: &str) -> io::Result<(Vec<Coordinate>, DateTime<Utc>)> {
    let mut coordinates = Vec::new();
    let file = File::open(filename)?;
    let reader = io::BufReader::new(file);
    let mut lines = reader.lines();

    // 첫 줄에서 시작 날짜 및 시간 읽기
    let first_line = lines.next().ok_or(io::Error::new(io::ErrorKind::UnexpectedEof, "File is empty"))??;
    let datetime_str = first_line.trim();
    let start_time = NaiveDateTime::parse_from_str(datetime_str, "%Y/%m/%d %H:%M:%S")
        .map(|ndt| ndt.and_utc())
        .expect("Invalid date format");

    // 나머지 좌표 읽기
    for line in lines {
        let line = line?;
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() == 2 {
            let lat = parts[0].trim().parse().unwrap();
            let lon = parts[1].trim().parse().unwrap();
            coordinates.push(Coordinate { latitude: lat, longitude: lon });
        }
    }

    Ok((coordinates, start_time))
}

// 두 좌표 간의 선형 보간을 통해 새로운 좌표를 계산하는 함수
fn calculate_new_position(start: &Coordinate, end: &Coordinate, progress: f64) -> Coordinate {
    let lat = start.latitude + (end.latitude - start.latitude) * progress;
    let lon = start.longitude + (end.longitude - start.longitude) * progress;
    Coordinate { latitude: lat, longitude: lon }
}

// 두 좌표 간의 거리 계산 (단위: 미터)
fn haversine_distance(coord1: &Coordinate, coord2: &Coordinate) -> f64 {
    let earth_radius = 6371000.0; // 지구 반지름 (미터)
    let lat1_rad = coord1.latitude.to_radians();
    let lat2_rad = coord2.latitude.to_radians();
    let delta_lat = (coord2.latitude - coord1.latitude).to_radians();
    let delta_lon = (coord2.longitude - coord1.longitude).to_radians();

    let a = (delta_lat / 2.0).sin().powi(2) + lat1_rad.cos() * lat2_rad.cos() * (delta_lon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().asin();

    earth_radius * c // 거리 (미터)
}

fn main() -> io::Result<()> {
    // 좌표와 시작 날짜 및 시간 읽기
    let (coordinates, start_time) = read_coordinates_and_time_from_file("coordinates.txt")?;
    
    let mut current_time = start_time;

    // gps.data 파일 생성
    let mut output_file = File::create("gps.data")?;

    for i in 0..coordinates.len() - 1 {
        let start_coord = &coordinates[i];
        let end_coord = &coordinates[i + 1];

        // 두 좌표 간의 거리 계산
        let distance = haversine_distance(start_coord, end_coord);
        let max_speed_kmh = 80; // 최대 속도 (km/h)
        let speed_ms = max_speed_kmh as f64 * 1000.0 / 3600.0; // km/h to m/s
        let travel_time = distance / speed_ms; // 이동 시간 (초)

        // 이동할 점의 수 계산 (최대 속도로 이동 시)
        let steps = travel_time.ceil() as u64;

        // 랜덤 속도 생성기
        let mut rng = rand::thread_rng();
        let mut current_speed_kmh: i32 = rng.gen_range(0..=max_speed_kmh); // 초기 속도를 랜덤으로 설정

        for step in 0..steps {
            // 속도 변화 범위 설정 (이전 속도에서 ±10km/h)
            let speed_change = rng.gen_range(-10..=10);
            current_speed_kmh = (current_speed_kmh + speed_change).clamp(0, max_speed_kmh); // 속도를 제한

            let progress = step as f64 / steps as f64; // 0.0 ~ 1.0
            let new_position = calculate_new_position(start_coord, end_coord, progress);

            let output_line = format!(
                "{} {:.6}N {:.6}E {:.3}kmh V\n",
                current_time.format("%Y/%m/%d %H:%M:%S"),
                new_position.latitude,
                new_position.longitude,
                current_speed_kmh
            );

            output_file.write_all(output_line.as_bytes())?;

            // 1초씩 시간 증가
            current_time = current_time + Duration::new(1, 0);
        }
    }

    Ok(())
}


