# file_server

A simple static `http` file server written in rust using [tokio](https://tokio.rs/) and
[hyper](https://hyper.rs/).

## How to use

### Install

Run the following commands:

```sh
git clone https://github.com/herebythere/file_server
cargo install --path file_server
```

### Run 

Run the following command to serve files at the `cwd` at `localhost:3000`:

```sh
file_server localhost:3000
```

Open a browser and visit `http://localhost:3000`.

### Implementation Details

#### Content-Encoding

A common expectation of file servers is to serve encoded files when requested.

`File_server` expects encoded files to exist alongside their unencoded counterparts.

If a request has a `content-encoding` header:

```
Content-Encoding: gzip;
```

`File-server` will serve the `gzip`-ed version of a requested file if available.

Otherwise it will serve the default file.

Encoded files will not be served if their unencoded counterpart does not exist.

## Licence

BSD 3-Clause License
