# ulexec
A tool for loading and executing PE on Windows and ELF on Linux from memory written in Rust

## To get started:
* **Download the latest revision**
```
git clone https://github.com/VHSgunzo/ulexec.git && cd ulexec
```
* **Setup toolchain**
```
rustup default nightly
rustup component add rust-src --toolchain nightly
```
* **Compile the Linux binary (musl required)**
```
rustup target add $(uname -m)-unknown-linux-musl
cargo build --release
./target/$(uname -m)-unknown-linux-musl/release/ulexec ~~help
```
* **Compile the Windows binary (mingw-w64-gcc required)**
```
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu
./target/x86_64-pc-windows-gnu/release/ulexec.exe ~~help
```
* Or take an already precompiled binary file from the [releases](https://github.com/VHSgunzo/ulexec/releases)

## Usage
```
ulexec [OPTIONS] [EXEC ARGS]...

Arguments:
  [EXEC ARGS]...  Command line arguments for execution

Options:
  ~~u,  ~~url <URL>    Load the binary file from URL (env: ULEXEC_URL)
  ~~p,  ~~post         Use the POST method instead of GET (env: ULEXEC_POST)
  ~~f,  ~~file <PATH>  Path to the binary file for exec (env: ULEXEC_FILE)
  ~~s,  ~~stdin        Load the binary file from stdin (env: ULEXEC_STDIN)
  ~~r,  ~~remove       Self remove (env: ULEXEC_REMOVE)
  ~~re, ~~reexec       Reexec fix (env: ULEXEC_REEXEC)
  ~~m,  ~~mfdexec      Force use memfd exec (env: ULEXEC_MFDEXEC)
  ~~n,  ~~name         Set process name or cmdline for memfd exec (env: ULEXEC_NAME)
  ~~v,  ~~version      Print version
  ~~h,  ~~help         Print help
```

## Examples
The tool fully supports static and dynamically compiled Linux executables and Windows PE (portable executable). Simply pass the filename of the binary to `ulexec` and any arguments you want to supply to the binary. The environment will be directly copied over from the environment in which you execute `ulexec`

The file path can be passed directly or with `~~f | ~~file` argument or env var `ULEXEC_FILE`

```
ulexec /bin/ls -lha
```

You can read a binary from `stdin` if you specify `~~s | ~~stdin` argument or env var `ULEXEC_STDIN=1`

```
cat /bin/ls|ulexec ~~s -lha
# or
ulexec ~~s</bin/ls -lha
```

To download a binary into memory and immediately execute it you can use `~~u | ~~url` argument or env var `ULEXEC_URL` or pass the URL directly

```
ulexec http://example.com/bin/ls -lha
```

If the resource (for example https://temp.sh) on which the binary file is located requires using the POST method instead of GET to start downloading, you can specify this with the `~~p | ~~post` argument or env var `ULEXEC_POST=1`

```
ulexec ~~p http://temp.sh/ABCDEF/ls -lha
```

For executable files that need to fork themselves, you can use the `~~re | ~~reexec` argument or env var `ULEXEC_REEXEC=1`

```
ulexec ~~re http://example.com/bin/tun2proxy --unshare --setup --proxy socks5://127.0.0.1:1080 -- /bin/bash
ulexec ~~re http://example.com/bin/gocryptfs /tmp/cryptfs /tmp/mnt
```

To self remove `ulexec` at startup, you can use the `~~r | ~~remove` argument or env var `ULEXEC_REMOVE=1`

```
ulexec ~~r http://example.com/bin/ls -lha
```

You can pass `stdin` data to the executed binary

```
ulexec http://example.com/bin/tar -xzvf-</path/to/local.tar.gz
```

## References
1. [userland-execve](https://crates.io/crates/userland-execve)
2. [memfd-exec](https://github.com/novafacing/memfd-exec)
2. [memexec](https://lib.rs/crates/memexec)
3. [ulexecve](https://github.com/anvilsecure/ulexecve)
