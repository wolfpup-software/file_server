# file_server

`http` file server written in rust using [tokio](https://tokio.rs/) and
[hyper](https://hyper.rs/).

## Install

Run the following commands:

```
git clone https://github.com/herebythere/file_server
cargo install --path file_server
```

## Run

Run the following command to serve files at the `cwd` at `localhost:3000`:

```sh
file_server localhost:3000
```

Open a browser and visit `http://localhost:3000`.

## Licence

BSD 3-Clause License
