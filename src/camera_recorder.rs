use std::{
    cell::RefCell,
    fs::create_dir_all,
    io::{Read, Write},
    os::{raw::c_ulong, windows::process::CommandExt},
    path::Path,
    process::{Child, Command, Stdio},
    thread,
    time::Duration,
    u128,
};

use regex::Regex;

// https://github.com/rust-lang/rust/blob/master/library/std/src/sys/windows/c.rs#L198-L199
type DWORD = c_ulong;
const DETACHED_PROCESS: DWORD = 0x00000008;
const CREATE_NEW_PROCESS_GROUP: DWORD = 0x00000200;

pub struct Camera {
    number: i32,
    name: String,
}

impl Camera {
    pub fn new(number: i32, name: String) -> Camera {
        Camera { number, name }
    }
}

pub struct CameraRecorder<'a> {
    record_path: &'a Path,
    cameras: Vec<Camera>,
    children: RefCell<Vec<Child>>,
    current_epoch: u128,
}

impl CameraRecorder<'_> {
    pub fn new(record_path: &Path, cameras: Vec<Camera>) -> CameraRecorder {
        CameraRecorder {
            record_path,
            cameras,
            children: RefCell::new(vec![]),
            current_epoch: 0,
        }
    }

    pub fn begin(&mut self, first_epoch: u128) {
        self.current_epoch = first_epoch;
        // self.wait();
        let children = self.cameras.iter().map(|camera| self.create_child(camera));
        self.children.replace(children.collect());
    }

    pub fn record(&self, _data: String) {
        let children = self.children.take();
        let mut new_children = vec![];

        let mut index = 0;
        for mut child in children {
            if let Some(status) = child.try_wait().unwrap() {
                println!("Restart children, status: {}", status);
                new_children.push(self.create_child(self.cameras.get(index).unwrap()));
            } else {
                new_children.push(child);
            }
            index += 1;
        }

        self.children.replace(new_children);
    }

    pub fn finish(&self) {
        for child in self.children.borrow_mut().iter_mut() {
            let stdin = child.stdin.as_mut().unwrap();
            stdin.write_all("q\n".as_bytes()).unwrap();
            stdin.flush().unwrap();
        }
        // self.children.replace(vec![]);
    }

    pub fn wait(&self) {
        for child in self.children.borrow_mut().iter_mut() {
            child.wait().unwrap();
        }
    }

    fn create_child(&self, camera: &Camera) -> Child {
        let replacer = Regex::new(r"[^\x00-\x7F]").unwrap();
        let clean_name = replacer.replace_all(&camera.name, "?");

        let filename = format!(
            "{}-{}-{}.mp4",
            self.current_epoch, clean_name, camera.number
        );
        let path = self.record_path.join(Path::new(&filename));
        create_dir_all(path.parent().unwrap()).unwrap();

        let video_device_number = camera.number.to_string();
        let i = format!("video={}", camera.name);

        let mut child = Command::new("ffmpeg")
            .args(
                #[cfg_attr(rustfmt, rustfmt::skip)]
                [
                    "-hide_banner",
                    "-f", "dshow",
                    "-video_device_number", &video_device_number, // -i videoより先
                    "-i", &i,
                    // "-loglevel", "quiet",
                    path.to_str().unwrap(),
                ],
            )
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP)
            // .process_group(0);
            .spawn()
            .unwrap();

        if false {
            thread::sleep(Duration::from_secs(1));
            let stderr = child.stderr.as_mut().unwrap();
            let mut buf = [0; 1024];
            stderr.read(&mut buf).ok();
            println!("{}", String::from_utf8_lossy(&buf));
        }

        child
    }
}
