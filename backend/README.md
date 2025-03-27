# Setup the database

After installing MySql/MariaDB create the database:

```sql
CREATE DATABASE gestao_documental;
```

The backend expects the root user to have a password of "root"

## Tls certificate

For testing purposes, use [mkcert](https://github.com/FiloSottile/mkcert):

```bash
mkdir certs
cd certs
mkcert -install
mkcert -key-file key.pem -cert-file cert.pem 127.0.0.1 localhost
```

## Run development server

To improve rust compilation times, for development purposes, I used the [cranelift codegen backend for rust](https://github.com/rust-lang/rustc_codegen_cranelift), the [mold linker](https://github.com/rui314/mold) and I enabled parallel execution of the compiler's frontend, which for now only works on the nightly toolchain.  
Make sure to have to have installed the cranelift backend, the mold linker and the rust nightly toolchain.

The [cranelift backend doesn't support aws-lc-rs](https://github.com/rust-lang/rustc_codegen_cranelift/issues/1520) for now, so if you want to test https with a development build, the dev script will use the default llvm backend.

```bash
./dev.sh --help
Usage: ./dev.sh [OPTIONS] [-- PROGRAM_ARGS]

Options:
  -h, --help        Show this help message and exit
  --https           Run with HTTPS support (disables cranelift)

Examples:
  ./dev.sh                         # Run with default settings
  ./dev.sh --https                 # Run with HTTPS support
  ./dev.sh --https -- --port=8443  # Run with HTTPS and pass --port=8443
```

## Run production server

```bash
# Usage: ./release.sh [PROGRAM_ARGS]
./release.sh --help
```
