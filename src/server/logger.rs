use std::io::Write;

pub struct Logger {
    file: std::fs::File,
}

/// Logger is a simple struct that allows for writing to a log file
/// either for debugging or to save session information
impl Logger {
    pub fn build(path: String) -> Logger {
        let file = std::fs::OpenOptions::new()
            .write(true)
            .append(true)
            .open(path);

        match file {
            Ok(x) => Logger { file: x },
            Err(_) => panic!("Error opening log path"),
        }
    }

    pub fn write(&mut self, value: String) {
        self.file
            .write_all(format!("{}\n", value).as_bytes())
            .expect("Couldn't write to logger");
    }
}
