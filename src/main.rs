// #![windows_subsystem = "windows"]

use std::env;
use std::path::PathBuf;
use std::process::exit;
use std::io::{self, Read};
use std::fs::{read, remove_file};

use reqwest::blocking::{Client, RequestBuilder};


#[derive(Debug)]
struct Args {
    url: Option<String>,
    post: bool,
    stdin: bool,
    remove: bool,
    exec_args: Vec<String>,
}

impl Default for Args {
    fn default() -> Self {
        Args {
            url: None,
            post: false,
            stdin: false,
            remove: false,
            exec_args: Vec::new(),
        }
    }
}

fn parse_args() -> Args {
    let env_args: Vec<String> = env::args().skip(1).collect();
    let mut args = Args::default();

    let mut i = 0;
    while i < env_args.len() {
        match env_args[i].as_str() {
            "~~url" | "~~u" => {
                if i + 1 < env_args.len() {
                    args.url = Some(env_args[i + 1].to_string());
                    i += 2;
                } else {
                    eprintln!("error: a value is required for '~~url <URL>' but none was supplied");
                    exit(1);
                }
            }
            "~~post" | "~~p" => {
                args.post = true;
                i += 1;
            }
            "~~stdin" | "~~s" => {
                args.stdin = true;
                i += 1;
            }
            "~~remove" | "~~r" => {
                args.remove = true;
                i += 1;
            }
            "~~version" | "~~v" => {
                println!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
                exit(0);
            }
            "~~help" | "~~h" => {
                print_usage();
                exit(0);
            }
            arg => {
                args.exec_args.push(arg.to_string());
                i += 1;
            }
        }
    }
    args
}

fn print_usage() {
    println!("{}\n", env!("CARGO_PKG_DESCRIPTION"));
    println!("Usage: {} [OPTIONS] [EXEC ARGS]...\n", env!("CARGO_PKG_NAME"));
    println!("Arguments:");
    println!("  [EXEC ARGS]...  Command line arguments for execution\n");
    println!("Options:");
    println!("  ~~u, ~~url <URL>    Load the binary file from URL");
    println!("  ~~p, ~~post         Use the POST method instead of GET");
    println!("  ~~s, ~~stdin        Load the binary file from stdin");
    println!("  ~~r, ~~remove       Self remove");
    println!("  ~~v, ~~version      Print version");
    println!("  ~~h, ~~help         Print help");
}

fn try_self_remove(remove: bool) {
    if remove {
        let _ = remove_file(env::current_exe().unwrap());
    }
}

fn main() {
    let mut args: Args;

    let _is_child: bool;
    #[cfg(target_os = "windows")]
    {
        fn get_env_var(env_var: &str) -> String {
            let mut ret = "".to_string();
            if let Ok(res) = env::var(env_var) { ret = res };
            ret
        }

        _is_child = get_env_var("ULEXEC_CHILD") == "1";
        if _is_child {
            args = Args::default();
            args.stdin = true;
            args.exec_args = env::args().skip(1).collect();
        } else {
            args = parse_args()
        }
    }

    #[cfg(target_os = "linux")]
    {
        args = parse_args();
        try_self_remove(args.remove);
    }

    let mut exec_file: Vec<u8> = Vec::new();
    let mut file_path = PathBuf::new();

    if args.stdin {
        io::stdin().lock().read_to_end(&mut exec_file).unwrap();
    } else if args.url.is_some() {
        let client = Client::builder();

        #[cfg(target_os = "windows")]
        let client = client.use_rustls_tls();

        let client = client
            .danger_accept_invalid_certs(true)
            .timeout(None)
            .build()
            .unwrap();

        let req: RequestBuilder;
        let url = args.url.as_ref().unwrap();
        if args.post {
            req = client.post(url)
        } else {
            req = client.get(url)
        }
        exec_file = req.send().unwrap().bytes().unwrap().to_vec();
        drop(client)
    } else if !args.exec_args.is_empty() {
        file_path = PathBuf::from(args.exec_args.remove(0));
    } else {
        eprintln!("Specify the path to the binary file or see '{} ~~help'",  env!("CARGO_PKG_NAME"));
        exit(1)
    }

    if !file_path.to_str().unwrap().is_empty() && exec_file.is_empty() {
        exec_file = read(&file_path).unwrap()
    }

    #[cfg(target_os = "windows")]
    {
        use std::io::Write;
        use std::process::{Stdio, Command};

        if !_is_child && (!exec_file.is_empty() || args.stdin) {
            env::set_var("ULEXEC_CHILD", "1");
            let mut child = Command::new(env::current_exe().unwrap())
                .args(args.exec_args)
                .envs(env::vars())
                .stdin(Stdio::piped())
                .stdout(io::stdout())
                .stderr(io::stderr())
                .spawn().unwrap();

            let mut exec_stdin = child.stdin.as_ref().unwrap();
            exec_stdin.write_all(&exec_file).unwrap();
            drop(exec_file);

            try_self_remove(args.remove);

            exit(child.wait().unwrap().code().unwrap())
        } else {
            unsafe { memexec::memexec_exe(&exec_file).unwrap() }
        }
    }

    #[cfg(target_os = "linux")]
    {
        use std::time;
        use std::ffi::CString;
        use std::os::fd::AsRawFd;
        use nix::unistd::{write, close};
        use std::thread::{spawn, sleep};
        use goblin::elf::{Elf, program_header};
        use memfd_exec::{Stdio, MemFdExecutable};
        use nix::sys::memfd::{memfd_create, MemFdCreateFlag};


        fn is_pie(bytes: &Vec<u8>) -> bool {
            let elf = Elf::parse(&bytes).unwrap();
            elf.program_headers.iter()
                .find(|h| h.p_type == program_header::PT_LOAD)
                .unwrap()
            .p_vaddr == 0
        }

        let memfd_name = "exec";
        if !is_pie(&exec_file) {
            exit(MemFdExecutable::new(memfd_name, &exec_file)
                .args(args.exec_args)
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .envs(env::vars())
                .status().unwrap().code().unwrap())
        } else {
            let envs: Vec<CString> = env::vars()
                .map(|(key, value)| CString::new(
                    format!("{}={}", key, value)
            ).unwrap()).collect();

            let mut args_cstrs: Vec<CString> = args.exec_args.iter()
                .map(|arg| CString::new(arg.clone()).unwrap()).collect();

            let memfd = &memfd_create(
                CString::new(memfd_name).unwrap().as_c_str(),
                MemFdCreateFlag::MFD_CLOEXEC,
            ).unwrap();
            let memfd_raw = memfd.as_raw_fd();

            if file_path.to_str().unwrap().is_empty() && !exec_file.is_empty() {
                write(memfd, &exec_file).unwrap();
                file_path = PathBuf::from(
                    format!("/proc/self/fd/{}", memfd_raw.to_string())
                );

            }
            drop(exec_file);

            let file_cstr = CString::new(
                file_path.to_str().unwrap()
            ).unwrap();
            args_cstrs.insert(0, file_cstr);

            spawn(move || {
                sleep(time::Duration::from_millis(1));
                close(memfd_raw).unwrap()
            });

            userland_execve::exec(
                &file_path,
                &args_cstrs,
                &envs,
            )
        }
    }
}
