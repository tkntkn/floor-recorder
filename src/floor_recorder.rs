use std::{
    cell::RefCell,
    fs::{create_dir_all, File},
    io::{BufWriter, Write},
    ops::DerefMut,
    path::Path,
    u128,
};

pub struct FloorRecorder<'a> {
    record_path: &'a Path,
    writer: RefCell<Option<BufWriter<File>>>,
}

impl FloorRecorder<'_> {
    pub fn new(record_path: &Path) -> FloorRecorder {
        FloorRecorder {
            record_path,
            writer: RefCell::new(None),
        }
    }

    pub fn begin(&self, first_epoch: u128) {
        let filename = format!("./{}.dat", first_epoch);
        let path = self.record_path.join(Path::new(&filename));
        create_dir_all(path.parent().unwrap()).unwrap();

        let writer = BufWriter::new(File::create(path).unwrap());
        self.writer.replace(Some(writer));
    }

    pub fn record(&self, data: String) {
        if let Some(writer) = self.writer.borrow_mut().deref_mut() {
            writer.write_all(data.as_bytes()).unwrap();
            writer.write("\n".as_bytes()).unwrap();
            writer.flush().unwrap();
        }
    }

    pub fn finish(&self) {
        // TODO: zip
    }
}
