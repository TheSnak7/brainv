#[cfg(all(windows, target_arch = "aarch64"))]
mod tests {
    use std::{fs, io::Write, process::{Command, Output}, sync::atomic::{AtomicUsize, Ordering}};

    static NEXT_FILE: AtomicUsize = AtomicUsize::new(0);

    fn run_program(program: &str, io: Option<&str>) -> Output {
        let path = std::env::temp_dir().join(format!("brainv-output-{}-{}.bf",
            std::process::id(), NEXT_FILE.fetch_add(1, Ordering::Relaxed),
        ));
        let mut file = fs::OpenOptions::new().write(true).create_new(true).open(&path).unwrap();
        file.write_all(program.as_bytes()).unwrap();
        drop(file);

        let mut command = Command::new(env!("CARGO_BIN_EXE_brainv"));
        command.arg(&path);
        if let Some(mode) = io {
            command.args(["--io", mode]);
        }
        let output = command.output();
        fs::remove_file(&path).unwrap();
        output.unwrap()
    }

    #[test]
    fn flushes_output_on_success() {
        for mode in [None, Some("one-print"), Some("simple")] {
            for count in [1, 200, 201, 401] {
                let program = format!("{}{}", "+".repeat(65), ".".repeat(count));
                let output = run_program(&program, mode);

                assert!(output.status.success());
                assert_eq!(output.stdout, vec![b'A'; count], "Output missing for {:?}", mode);
                assert!(output.stderr.is_empty());
            }
        }
    }

    #[test]
    fn flushes_output_before_bounds_error() {
        for mode in [None, Some("one-print"), Some("simple")] {
            let program = format!("{}{}<.", "+".repeat(65), ".".repeat(201));
            let output = run_program(&program, mode);

            assert_eq!(output.status.code(), Some(1));
            assert_eq!(output.stdout, vec![b'A'; 201], "Output missing for {:?}", mode);
            assert!(String::from_utf8_lossy(&output.stderr).contains("Tape pointer out of bounds"));
        }
    }
}
