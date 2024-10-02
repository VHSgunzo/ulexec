// #![windows_subsystem = "windows"]

use std::env;
use std::fs::read;
use std::path::PathBuf;
use std::process::exit;
use std::io::{self, Read};

use clap::Parser;
use reqwest::blocking::{Client, RequestBuilder};


#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Load the binary file from URL
    #[arg(short, long)]
    url: Option<String>,
    /// Use the POST method instead of GET
    #[arg(short, long)]
    post: bool,
    /// Load the binary file from stdin
    #[arg(short, long)]
    stdin: bool,
    /// Command line arguments for execution
    #[arg(value_parser)]
    exec_args: Vec<String>,
}


fn main() {
    let mut args: Args;

    let _is_child: bool;
    #[cfg(target_os = "windows")]
    {
        fn get_env_var(env_var: &str) -> String {
            let mut ret = "".to_string();
            if let Ok(res) = env::var(env_var) { ret = res };
            return ret;
        }

        _is_child = get_env_var("ULEXEC_CHILD") == "1";
        if _is_child {
            args = Args{
                url: Some("".to_string()),
                post: false,
                stdin: true,
                exec_args: env::args().skip(1).collect(),
            }
        } else {
            args = Args::parse();
        }
    }

    #[cfg(target_os = "linux")]
    { args = Args::parse() }

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
        eprintln!("Specify the path to the binary file!");
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

            exit(child.wait().unwrap().code().unwrap())
        } else {
            unsafe { memexec::memexec_exe(&exec_file).unwrap() }
        }
    }

    #[cfg(target_os = "linux")]
    {
        use std::ffi::CString;
        use std::thread::spawn;
        use std::os::fd::AsRawFd;
        use nix::sys::wait::waitpid;
        use goblin::elf::{Elf, program_header};
        use memfd_exec::{Stdio, MemFdExecutable};
        use nix::unistd::{write, close, fork, ForkResult};
        use nix::sys::memfd::{memfd_create, MemFdCreateFlag};

        fn is_pie(bytes: &Vec<u8>) -> bool {
            let elf = Elf::parse(&bytes).unwrap();
            elf.program_headers.iter()
                .find(|h| h.p_type == program_header::PT_LOAD)
                .unwrap()
            .p_vaddr == 0
        }

        if !is_pie(&exec_file) {
            exit(MemFdExecutable::new("exec", &exec_file)
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
                CString::new("exec").unwrap().as_c_str(),
                MemFdCreateFlag::MFD_CLOEXEC,
            ).unwrap();
            let memfd_raw = memfd.as_raw_fd();

            if file_path.to_str().unwrap().is_empty() && !exec_file.is_empty() {
                write(memfd, &exec_file).unwrap();
                file_path = PathBuf::from(
                    format!("/proc/self/fd/{}", memfd_raw.to_string())
                )
            }
            drop(exec_file);

            let file_cstr = CString::new(
                file_path.to_str().unwrap()
            ).unwrap();
            args_cstrs.insert(0, file_cstr);

            match unsafe { fork() } {
                Ok(ForkResult::Parent { child: pid }) => {
                    spawn(move || {
                        waitpid(pid, None)
                    });
                    userland_execve::exec(
                        &file_path,
                        &args_cstrs,
                        &envs,
                    )
                }
                Ok(ForkResult::Child) => {
                    close(memfd_raw).unwrap()
                }
                Err(_) => {
                    eprintln!("Fork error!");
                    exit(1)
                }
            }
        }
    }
}
