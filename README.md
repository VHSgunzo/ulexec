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
rustup target add x86_64-unknown-linux-musl
cargo build --release
./target/x86_64-unknown-linux-musl/release/ulexec --help
```
* **Compile the Windows binary (mingw-w64-gcc required)**
```
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu
./target/x86_64-pc-windows-gnu/release/ulexec.exe --help
```
* Or take an already precompiled binary file from the [releases](https://github.com/VHSgunzo/ulexec/releases)

## Usage
```
ulexec [OPTIONS] [EXEC_ARGS]...

Arguments:
  [EXEC_ARGS]...  Command line arguments for execution

Options:
  -u, --url <URL>  Load the binary file from URL
  -p, --post       Use the POST method instead of GET
  -s, --stdin      Load the binary file from stdin
  -h, --help       Print help
  -V, --version    Print version
```

## Examples
The tool fully supports static and dynamically compiled Linux executables and Windows PE (portable executable). Simply pass the filename of the binary to `ulexec` and any arguments you want to supply to the binary. The environment will be directly copied over from the environment in which you execute `ulexec`

```
ulexec /bin/ls -- -lha
```

You can have it read a binary from `stdin` if you specify `-s | --stdin` argument

```
cat /bin/ls|ulexec -s -- -lha
# or
ulexec -s</bin/ls -- -lha
# or
ulexec</bin/ls -- -lha
# or
cat /bin/ls|ulexec -- -lha
```

To download a binary into memory and immediately execute it you can use `-u | --url`

```
ulexec -u http://examples.com/bin/ls -- -lha
```

If the resource (for example https://temp.sh) on which the binary file is located requires using the POST method instead of GET to start downloading, you can specify this with the `-p | --post` argument

```
ulexec -p -u http://temp.sh/bqcnS/ls -- -lha
```

## References
1. [userland-execve](https://crates.io/crates/userland-execve)
2. [memfd-exec](https://github.com/novafacing/memfd-exec)
2. [memexec](https://lib.rs/crates/memexec)
3. [ulexecve](https://github.com/anvilsecure/ulexecve)
