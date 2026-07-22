use simplelog::{CombinedLogger, ConfigBuilder, LevelFilter, WriteLogger};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

fn log_dir() -> PathBuf {
    let dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("logs")))
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("logs"));
    let _ = fs::create_dir_all(&dir);
    dir
}

fn today() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

fn rotate_old_log(dir: &PathBuf) {
    let log_path = dir.join("errors_log.log");
    if !log_path.exists() {
        return;
    }
    let Ok(meta) = fs::metadata(&log_path) else { return };
    let Ok(modified) = meta.modified() else { return };
    let modified_date = chrono::DateTime::<chrono::Local>::from(modified)
        .format("%Y-%m-%d")
        .to_string();
    if modified_date != today() {
        let dest = dir.join(format!("errors_log.{modified_date}.log"));
        let _ = fs::rename(&log_path, &dest);
    }
}

struct DailyRotatingFile {
    dir: PathBuf,
    file: fs::File,
    date: String,
}

impl DailyRotatingFile {
    fn new(dir: PathBuf) -> io::Result<Self> {
        rotate_old_log(&dir);
        let path = dir.join("errors_log.log");
        let file = fs::OpenOptions::new().create(true).append(true).open(&path)?;
        Ok(Self { dir, file, date: today() })
    }
}

impl Write for DailyRotatingFile {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let now = today();
        if self.date != now {
            self.file.flush()?;
            let old = self.dir.join(format!("errors_log.{}.log", self.date));
            let current = self.dir.join("errors_log.log");
            let _ = fs::rename(&current, &old);
            self.file = fs::OpenOptions::new().create(true).append(true).open(&current)?;
            self.date = now;
        }
        self.file.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}

pub fn init() {
    let dir = log_dir();
    let writer = match DailyRotatingFile::new(dir) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("Cannot open log file: {e}");
            return;
        }
    };

    let mut builder = ConfigBuilder::new();
    builder.set_time_format_rfc3339();
    let _ = builder.set_time_offset_to_local();
    let config = builder.build();

    let _ = CombinedLogger::init(vec![
        WriteLogger::new(LevelFilter::Error, config, writer),
    ]);
}
