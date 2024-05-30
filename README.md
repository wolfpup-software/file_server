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

Run the following command:

```sh
file_server
```

Open a browser and visit `http://localhost:3000`.

### Change Authority and Port

`File_server` accepts an `authority` as an optional argument from the command line:

```sh
file_server 0.0.0.0:7890
```

Open a browser and visit `http://0.0.0.0:7890`.

## Licence

BSD 3-Clause License
