use std::fs::OpenOptions;
use std::io::Write;
use std::sync::OnceLock;

fn log_path() -> Option<&'static str> {
    static LOG_PATH: OnceLock<Option<String>> = OnceLock::new();
    LOG_PATH
        .get_or_init(|| {
            std::env::var("FLOEM_SHOWCASE_IME_STATE_LOG_FILE")
                .ok()
                .filter(|value| !value.is_empty())
        })
        .as_deref()
}

pub(crate) fn append_state_log(line: impl AsRef<str>) {
    let Some(path) = log_path() else {
        return;
    };

    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(file, "{}", line.as_ref());
    }
}
