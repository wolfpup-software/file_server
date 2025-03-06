# file_server

An `http` file server written in rust using [tokio](https://tokio.rs/) and
[hyper](https://hyper.rs/).

## How to use

### Install

Bash the following commands:

```sh
git clone https://github.com/herebythere/file_server
cargo install --path file_server
```

### Run

Run the following command:

```sh
file_server
```

This will start `file_server` with it's default configuration in the `cwd`.

Bash the following command to serve files in the `cwd` at `localhost:3000`:

```sh
curl localhost:3000
```

### Configuration

Click [here](./configuration.md) to learn how to configure `file_server`.

## Licence

BSD 3-Clause License
