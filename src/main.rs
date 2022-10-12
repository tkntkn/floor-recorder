mod camera_recorder;
mod domain;
mod floor_reader;
mod floor_recorder;

use camera_recorder::{Camera, CameraRecorder};
use floor_reader::TimeSlicingFloorReader;
use floor_recorder::FloorRecorder;

use chrono::Local;
use std::{
    fs::create_dir_all,
    path::Path,
    process,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

fn main() {
    let floor_address = "ws://127.0.0.1:8080/";
    let millis_per_file = 3 * 1000;
    let cameras = vec![Camera::new(0, "Iriun Webcam".to_string())];
    let record_directory = "./dataout";
    let record_name = "test";

    let reader = TimeSlicingFloorReader::new(floor_address, millis_per_file);

    let record_datetime = Local::now().format("%Y%m%d-%H%M%S").to_string();
    let record_path = format!("{}/{}-{}", record_directory, record_datetime, record_name);
    let record_path = Path::new(&record_path);
    create_dir_all(record_path.parent().unwrap()).unwrap();

    let mut camera_recorder = CameraRecorder::new(record_path, cameras);
    let floor_recorder = FloorRecorder::new(record_path);
    let stop = Arc::new(AtomicBool::new(false));
    let stop_ref = stop.clone();
    ctrlc::set_handler(move || {
        if stop_ref.load(Ordering::Relaxed) {
            println!("force exitting");
            process::exit(1);
        }
        stop_ref.store(true, Ordering::Relaxed);
    })
    .unwrap();
    for time_sliced_reader in reader {
        println!("new slice: {}", time_sliced_reader.first_epoch);
        camera_recorder.begin(time_sliced_reader.first_epoch);
        floor_recorder.begin(time_sliced_reader.first_epoch);
        for (index, (data_epoch, data)) in time_sliced_reader.enumerate() {
            if stop.load(Ordering::Relaxed) {
                break;
            }
            camera_recorder.record(String::from(&data));
            floor_recorder.record(String::from(&data));
            if false {
                println!("{} {}", index, data_epoch);
            }
        }
        camera_recorder.finish();
        floor_recorder.finish();
        if stop.load(Ordering::Relaxed) {
            break;
        }
    }
    camera_recorder.wait();
}
