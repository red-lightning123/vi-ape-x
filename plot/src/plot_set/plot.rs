use file_io::{create_file_buf_write, has_data_left, open_file_buf_read};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Serialize, Deserialize)]
pub struct Plot {
    points: Vec<(f64, f64)>,
    current_n: usize,
    current_sum: f64,
    data_per_point: usize,
    output_path: PathBuf,
    fs_name: PathBuf,
}

impl Plot {
    pub fn new(output_path: PathBuf, fs_name: PathBuf, data_per_point: usize) -> Self {
        Self {
            points: vec![],
            current_n: 0,
            current_sum: 0.0,
            data_per_point,
            output_path,
            fs_name,
        }
    }
    pub fn add_datum(&mut self, (x, y): (f64, f64)) {
        self.current_n += 1;
        self.current_sum += y;
        if self.current_n == self.data_per_point {
            let y_average = self.current_sum / (self.data_per_point as f64);
            self.points.push((x, y_average));
            self.current_n = 0;
            self.current_sum = 0.0;
            self.update_plot();
        }
    }
    fn update_plot(&self) {
        // The plot is meant to be used in a data-science context. It
        // is therefore desirable for external tools to be able to
        // analyze the plot as they see fit, possibly producing several
        // images from the same data.
        // As such, we do not export to a visual format like svg or
        // draw to the screen, which would heavily constrain the types
        // of analysis an external tool could do.
        // Instead, it seems better to serialize the plot and let the
        // external viewer decide what it wants to do with the data.
        // As for the chosen serialization format, json lends itself
        // quite naturally. Being simple, readable, and
        // self-documenting, it is an ideal format for basic analysis
        self.export_json();
    }
    fn export_json(&self) {
        fs::create_dir_all(&self.output_path).unwrap();
        let file =
            create_file_buf_write(self.output_path.join(&self.fs_name).with_extension("json"))
                .unwrap();
        serde_json::to_writer(file, self).unwrap();
    }
    pub fn save<P: AsRef<Path>>(&self, path: P) {
        let file = create_file_buf_write(path).unwrap();
        bincode::serialize_into(file, self).unwrap();
    }
    pub fn load<P: AsRef<Path>>(&mut self, path: P) {
        let mut file = open_file_buf_read(path).unwrap();
        *self = bincode::deserialize_from(&mut file).unwrap();
        assert!(
            !has_data_left(file).unwrap(),
            "deserialization of file didn't reach EOF"
        );
    }
    pub fn fs_name(&self) -> &PathBuf {
        &self.fs_name
    }
}
