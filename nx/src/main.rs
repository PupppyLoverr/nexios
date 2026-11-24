// SPDX-License-Identifier: GPL-2.0-only
//! nx — the Nexios driver.
//!
//! One entry point for the hosted-personality lifecycle, replacing the ~30
//! scripts of the upstream bring-up. Today each subcommand wraps the
//! corresponding upstream script; over time the implementations move in-tree.
//!
//! Config: /etc/nexios.conf (KEY=VALUE) plus environment variables; the
//! environment wins. Same knobs as upstream's xnulinux.conf: CPUS, RAM_MB,
//! DISK, GPU, plus repo-specific paths.

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::{exit, Command, ExitStatus};

const CONFIG_FILES: &[&str] = &["/etc/nexios.conf", "/etc/xnulinux.conf"];

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let Some(cmd) = args.first() else {
        usage(2);
    };
    let cfg = Config::load();
    let root = repo_root();
    let status = match cmd.as_str() {
        // Once per host: toolchain + first XNU build.
        "build" => run(&root, "scripts/vm-xnu-build.sh", &args[1..], &cfg),
        // Apply patches, rebuild kernel + kexts, link the kernel collection.
        "rebuild" => run(&root, "scripts/vm-xnu-rebuild.sh", &args[1..], &cfg),
        // On the Mac: turn an Apple-VZ macOS VM disk into the hosted disk.
        "disk" => run(&root, "scripts/mk-macvm-disk.sh", &args[1..], &cfg),
        // Build fki.ko + xnu.ko for the running kernel and boot the hosted
        // system (network + GPU bridge per config).
        "boot" => run(&root, "scripts/xnulinux-boot.sh", &args[1..], &cfg),
        // Serial console into the hosted system.
        "console" => run(&root, "scripts/xnu-console.sh", &args[1..], &cfg),
        // Dump the hosted framebuffer to a PNG (path = argv or stdout name).
        "screenshot" => run(
            &root,
            "tools/hosted/fb2png.py",
            &args[1..],
            &cfg,
        ),
        // Install the systemd service (CPUS/memory/disk/GPU from config).
        "install-service" => run(&root, "scripts/install-service.sh", &args[1..], &cfg),
        "test" => {
            eprintln!("nx test: no expected-results suite yet — see docs/production.md §4");
            exit(2);
        }
        "-h" | "--help" | "help" => usage(0),
        other => {
            eprintln!("nx: unknown command '{other}'");
            usage(2);
        }
    };
    exit(status.code().unwrap_or(1));
}

fn usage(code: i32) -> ! {
    eprintln!(
        "nx — Nexios driver\n\
         \n\
         usage: nx <command> [args...]\n\
         \n\
         build            toolchain + first XNU build (once, on the arm64 host)\n\
         rebuild          apply patches, rebuild kernel + kexts, link collection\n\
         disk             on a Mac: make the hosted system disk from your macOS VM\n\
         boot             build modules for the running kernel, boot the hosted OS\n\
         console          serial console of the hosted system\n\
         screenshot       dump the hosted framebuffer to PNG\n\
         install-service  install/refresh the nx systemd service\n\
         test             expected-results suite (planned)\n\
         \n\
         config: {} (KEY=VALUE), env overrides",
        CONFIG_FILES.join(", ")
    );
    exit(code);
}

struct Config(Vec<(String, String)>);

impl Config {
    fn load() -> Self {
        let mut kv = Vec::new();
        for path in CONFIG_FILES {
            if let Ok(text) = fs::read_to_string(path) {
                for line in text.lines() {
                    let line = line.trim();
                    if line.is_empty() || line.starts_with('#') {
                        continue;
                    }
                    if let Some((k, v)) = line.split_once('=') {
                        kv.push((k.trim().to_string(), v.trim().to_string()));
                    }
                }
            }
        }
        Self(kv)
    }

    fn apply(&self, cmd: &mut Command) {
        // Config file values first; the process environment still wins.
        for (k, v) in &self.0 {
            if env::var_os(k).is_none() {
                cmd.env(k, v);
            }
        }
    }
}

/// Repo root = two levels up from the compiled binary's manifest dir,
/// falling back to the current directory's ancestor containing scripts/.
fn repo_root() -> PathBuf {
    let mut dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    loop {
        if dir.join("scripts/xnulinux-boot.sh").exists() {
            return dir;
        }
        if !dir.pop() {
            return PathBuf::from(".");
        }
    }
}

fn run(root: &PathBuf, script: &str, args: &[String], cfg: &Config) -> ExitStatus {
    let path = root.join(script);
    let mut c = Command::new(&path);
    c.args(args).current_dir(root);
    cfg.apply(&mut c);
    match c.status() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("nx: {script}: {e}");
            exit(127);
        }
    }
}
