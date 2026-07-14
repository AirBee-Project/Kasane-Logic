use std::collections::VecDeque;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use std::thread;

// --- ANSI カラーコード定義 ---
const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const BLUE: &str = "\x1b[34m";
const MAGENTA: &str = "\x1b[35m";
const CYAN: &str = "\x1b[36m";
const BG_RED: &str = "\x1b[41m";
const BOLD_WHITE: &str = "\x1b[1;97m";

fn get_label_color(label: &str) -> &'static str {
    if label.starts_with("test") || label.starts_with("doctest") {
        GREEN
    } else if label.starts_with("clippy") {
        YELLOW
    } else if label.starts_with("build") {
        BLUE
    } else if label.starts_with("doc") {
        MAGENTA
    } else {
        CYAN
    }
}

struct Task {
    label: String,
    args: Vec<String>,
    target_dir: String,
    is_doc: bool,
}

fn main() {
    println!("🚀 Starting local CI verification ({CYAN}{BOLD}Godspeed Parallel Mode{RESET})...");

    // ツール検知
    let use_sccache = has_sccache();
    let use_nextest = has_nextest();

    print!("🛠️  Toolchain Auto-Detect: ");
    if use_sccache {
        print!("{GREEN}[sccache: ON]{RESET} ");
    } else {
        print!("{YELLOW}[sccache: OFF]{RESET} ");
    }
    if use_nextest {
        println!("{GREEN}[nextest: ON]{RESET}");
    } else {
        println!("{YELLOW}[nextest: OFF]{RESET}");
    }
    println!();

    // 1. fmt 同期実行
    let fmt_status = Command::new("cargo")
        .args(["fmt", "--all", "--", "--check"])
        .status()
        .expect("Failed to execute cargo fmt");
    if !fmt_status.success() {
        println!("{RED}❌ Formatting check failed. Pipeline aborted.{RESET}");
        std::process::exit(1);
    }
    println!("{GREEN}✨ Formatting check passed. Preparing atomic tasks...{RESET}\n");

    let std_features_full = vec![
        "std",
        "std,rayon",
        "std,random,temporal_id",
        "std,rayon,random,temporal_id",
        "std,persist",
        "std,rayon,random,temporal_id,persist",
    ];
    let std_features_subset = vec![
        "std",
        "std,rayon,random,temporal_id",
        "std,rayon,random,temporal_id,persist",
    ];
    let no_std_features = vec!["", "temporal_id"];

    let mut tasks = Vec::new();
    let mut id = 0;

    // --- 通常のテスト (std) ---
    for f in &std_features_full {
        id += 1;
        let mut args = if use_nextest {
            vec!["nextest".to_string(), "run".to_string()]
        } else {
            vec!["test".to_string()]
        };
        args.extend(vec![
            "--locked".to_string(),
            "--lib".to_string(),
            "--bins".to_string(),
            "--tests".to_string(),
            "--no-default-features".to_string(),
            "--features".to_string(),
            f.to_string(),
        ]);
        tasks.push(Task {
            label: format!("test:{}", f),
            args,
            target_dir: format!("target/ci-t{}", id),
            is_doc: false,
        });
    }

    // --- Ignoredテスト ---
    for f in &std_features_subset {
        id += 1;
        let mut args = if use_nextest {
            vec!["nextest".to_string(), "run".to_string()]
        } else {
            vec!["test".to_string()]
        };
        args.extend(vec![
            "--locked".to_string(),
            "--lib".to_string(),
            "--bins".to_string(),
            "--tests".to_string(),
            "--no-default-features".to_string(),
            "--features".to_string(),
            f.to_string(),
        ]);
        if use_nextest {
            // 修正箇所: "ignored" ではなく "only" を指定
            args.extend(vec!["--run-ignored".to_string(), "only".to_string()]);
        } else {
            args.extend(vec!["--".to_string(), "--ignored".to_string()]);
        }
        tasks.push(Task {
            label: format!("test-ignored:{}", f),
            args,
            target_dir: format!("target/ci-t{}", id),
            is_doc: false,
        });
    }

    // --- Doctest (Nextest非対応のため標準 cargo test) ---
    for f in &std_features_subset {
        id += 1;
        tasks.push(Task {
            label: format!("doctest:{}", f),
            args: vec![
                "test".to_string(),
                "--locked".to_string(),
                "--doc".to_string(),
                "--no-default-features".to_string(),
                "--features".to_string(),
                f.to_string(),
            ],
            target_dir: format!("target/ci-t{}", id),
            is_doc: false,
        });
    }

    // --- Clippy (std) ---
    for f in &std_features_full {
        id += 1;
        tasks.push(Task {
            label: format!("clippy:{}", f),
            args: vec![
                "clippy".to_string(),
                "--locked".to_string(),
                "--lib".to_string(),
                "--bins".to_string(),
                "--tests".to_string(),
                "--no-default-features".to_string(),
                "--features".to_string(),
                f.to_string(),
                "--".to_string(),
                "-D".to_string(),
                "warnings".to_string(),
            ],
            target_dir: format!("target/ci-t{}", id),
            is_doc: false,
        });
    }

    // --- Build & Clippy (no_std) ---
    for f in &no_std_features {
        let display_f = if f.is_empty() { "none" } else { f };
        id += 1;
        let mut build_args = vec![
            "build".to_string(),
            "--locked".to_string(),
            "--lib".to_string(),
            "--no-default-features".to_string(),
        ];
        if !f.is_empty() {
            build_args.extend(vec!["--features".to_string(), f.to_string()]);
        }
        tasks.push(Task {
            label: format!("build-nostd:{}", display_f),
            args: build_args,
            target_dir: format!("target/ci-t{}", id),
            is_doc: false,
        });

        id += 1;
        let mut clippy_args = vec![
            "clippy".to_string(),
            "--locked".to_string(),
            "--lib".to_string(),
            "--no-default-features".to_string(),
        ];
        if !f.is_empty() {
            clippy_args.extend(vec!["--features".to_string(), f.to_string()]);
        }
        clippy_args.extend(vec![
            "--".to_string(),
            "-D".to_string(),
            "warnings".to_string(),
        ]);
        tasks.push(Task {
            label: format!("clippy-nostd:{}", display_f),
            args: clippy_args,
            target_dir: format!("target/ci-t{}", id),
            is_doc: false,
        });
    }

    // --- Rustdoc (std & no_std) ---
    for f in &std_features_full {
        id += 1;
        tasks.push(Task {
            label: format!("doc:{}", f),
            args: vec![
                "doc".to_string(),
                "--locked".to_string(),
                "--no-deps".to_string(),
                "--no-default-features".to_string(),
                "--features".to_string(),
                f.to_string(),
            ],
            target_dir: format!("target/ci-t{}", id),
            is_doc: true,
        });
    }
    for f in &no_std_features {
        id += 1;
        let mut doc_args = vec![
            "doc".to_string(),
            "--locked".to_string(),
            "--no-deps".to_string(),
            "--no-default-features".to_string(),
        ];
        if !f.is_empty() {
            doc_args.extend(vec!["--features".to_string(), f.to_string()]);
        }
        tasks.push(Task {
            label: format!("doc-nostd:{}", if f.is_empty() { "none" } else { f }),
            args: doc_args,
            target_dir: format!("target/ci-t{}", id),
            is_doc: true,
        });
    }

    let max_label_len = tasks.iter().map(|t| t.label.len()).max().unwrap_or(20);
    let total_tasks = tasks.len();

    let queue = Arc::new(Mutex::new(VecDeque::from(tasks)));
    let has_error = Arc::new(AtomicBool::new(false));
    let failed_tasks = Arc::new(Mutex::new(Vec::<(String, Vec<String>)>::new()));

    let concurrency = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(8);
    println!(
        "🔥 Concurrency level: {CYAN}{BOLD}{}{RESET} (Launching worker threads...)\n",
        concurrency
    );

    let mut workers = vec![];

    for _ in 0..concurrency {
        let queue = Arc::clone(&queue);
        let has_error = Arc::clone(&has_error);
        let failed_tasks = Arc::clone(&failed_tasks);

        workers.push(thread::spawn(move || {
            loop {
                let task = {
                    let mut lock = queue.lock().unwrap();
                    if let Some(t) = lock.pop_front() {
                        t
                    } else {
                        break;
                    }
                };

                let color = get_label_color(&task.label);
                let padded_prefix = format!(
                    "{color}{BOLD}[{:<width$}]{RESET} |",
                    task.label,
                    width = max_label_len
                );

                let mut cmd = Command::new("cargo");
                cmd.args(&task.args);
                cmd.env("CARGO_TARGET_DIR", &task.target_dir);
                cmd.env("CARGO_TERM_COLOR", "always");

                if use_sccache {
                    cmd.env("RUSTC_WRAPPER", "sccache");
                }
                if task.is_doc {
                    cmd.env("RUSTDOCFLAGS", "-D warnings");
                }

                #[cfg(target_os = "linux")]
                {
                    if has_mold_installed() {
                        cmd.env("RUSTFLAGS", "-C link-arg=-fuse-ld=mold");
                    }
                }

                cmd.stdout(Stdio::piped());
                cmd.stderr(Stdio::piped());

                let mut child = match cmd.spawn() {
                    Ok(c) => c,
                    Err(e) => {
                        println!("{} {RED}❌ Failed to spawn: {}{RESET}", padded_prefix, e);
                        has_error.store(true, Ordering::SeqCst);
                        continue;
                    }
                };

                let local_logs = Arc::new(Mutex::new(Vec::<String>::new()));
                let stdout = child.stdout.take().unwrap();
                let stderr = child.stderr.take().unwrap();

                let p_out = padded_prefix.clone();
                let l_out = Arc::clone(&local_logs);
                let t_out = thread::spawn(move || {
                    let reader = BufReader::new(stdout);
                    for line in reader.lines().map_while(Result::ok) {
                        let clean_line = line.replace('\r', "");
                        println!("{} {}", p_out, clean_line);
                        l_out.lock().unwrap().push(clean_line);
                    }
                });

                let p_err = padded_prefix.clone();
                let l_err = Arc::clone(&local_logs);
                let t_err = thread::spawn(move || {
                    let reader = BufReader::new(stderr);
                    for line in reader.lines().map_while(Result::ok) {
                        let clean_line = line.replace('\r', "");
                        println!("{} {}", p_err, clean_line);
                        l_err.lock().unwrap().push(clean_line);
                    }
                });

                t_out.join().unwrap();
                t_err.join().unwrap();

                let status = child.wait().unwrap();
                if !status.success() {
                    println!("{} {RED}❌ Task failed: {}{RESET}", padded_prefix, status);
                    has_error.store(true, Ordering::SeqCst);
                    let logs = local_logs.lock().unwrap().clone();
                    failed_tasks.lock().unwrap().push((task.label, logs));
                } else {
                    println!(
                        "{} {GREEN}✅ Task finished successfully.{RESET}",
                        padded_prefix
                    );
                }
            }
        }));
    }

    for worker in workers {
        worker.join().unwrap();
    }

    // --- 📊 最終サマリー出力セクション ---
    let failed_list = failed_tasks.lock().unwrap();
    if !failed_list.is_empty() {
        println!("\n\n");
        println!(
            "{RED}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{RESET}"
        );
        println!(
            "{BG_RED}{BOLD_WHITE}                       🚨 CI PIPELINE FAILED 🚨                        {RESET}"
        );
        println!(
            "{BG_RED}{BOLD_WHITE}                   ({}/{total_tasks} tasks failed)                     {RESET}",
            failed_list.len()
        );
        println!(
            "{RED}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{RESET}\n"
        );

        for (label, logs) in failed_list.iter() {
            println!(
                "{RED}▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼▼{RESET}"
            );
            println!("{BG_RED}{BOLD_WHITE} ❌ FAILED TASK: {:<52} {RESET}", label);
            println!(
                "{RED}----------------------------------------------------------------------{RESET}"
            );
            if logs.is_empty() {
                println!("(No output log captured)");
            } else {
                let tail_count = 100;
                let skip_entries = logs.len().saturating_sub(tail_count);
                if skip_entries > 0 {
                    println!(
                        "{YELLOW}... (truncated {} lines of early logs) ...{RESET}",
                        skip_entries
                    );
                }
                for line in logs.iter().skip(skip_entries) {
                    println!("{}", line);
                }
            }
            println!(
                "{RED}▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲{RESET}\n"
            );
        }
        println!(
            "{RED}{BOLD}❌ Execution finished with errors. Scroll up slightly to see the exact cause.{RESET}"
        );
        std::process::exit(1);
    } else {
        println!(
            "\n{GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{RESET}"
        );
        println!(
            "{GREEN}{BOLD}  ✅ [SUCCESS] All {} CI tasks passed successfully in ultra-parallel!{RESET}",
            total_tasks
        );
        println!(
            "{GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━{RESET}"
        );
    }
}

// ツール検知用ヘルパー関数
fn has_sccache() -> bool {
    Command::new("sccache")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn has_nextest() -> bool {
    Command::new("cargo")
        .args(["nextest", "--version"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

#[cfg(target_os = "linux")]
fn has_mold_installed() -> bool {
    Command::new("mold")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}
