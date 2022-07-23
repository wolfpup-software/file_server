# file-server

It serves files. That's it.

An `http` file server written in rust using [tokio](https://tokio.rs/) and [hyper](https://hyper.rs/).

## Create a config

A JSON configuration file is required to run `hbt_file-server` or create a `hbt_file-server` container.

Configuration files are expected to use use the following schema:

```
{
  "host": <string>,
  "port": <number>,
  "directory": <string>,
  "filepath_403": <string>,
  "filepath_404": <string>,
  "filepath_500": <string>
}
```

Change the `host` property to serve from a specific host.

Change the `port` property to serve from a different port.

Change the `directory` property to target a alternative directory.

Each `directory` must include a `filepath` for the following status codes:
- `403` at `filepath_403`
- `404` at `filepath_404`
- `500` at `filepath_500`

All `filepaths` must be a descendant of `directory`.

An example of a valid configuration can be found at `file-server/v0.1/resources/file-server.json.example`

```json
{
  "host": "127.0.0.1",
  "port": 3000,
  "directory": "./docs",
  "filepath_403": "./docs/403.html",
  "filepath_404": "./docs/404.html",
  "filepath_500": "./docs/500.html"
}
```

## Install file-server

Execute the following to install `htb_file-server`.

```
git clone https://github.com/herebythere/file-server
cargo install --path file-server/v0.1/app
```

## Run file-server

The `hbt_file-server` application accepts one argument:
- A valid `hbt_file-server` JSON configuration file

The following psuedo-script shows the argument schema:
```
hbt_file-server <path_to_configuration_file>
```

Execute the following to run a `hbt_file-server` demo hosting the repositories `/docs` directory.
```
hbt_file-server file-server/v0.1/resources/file-server.json.example
```

Open a browser and visit `http://localhost:3000`.

## File-server containers

A utility script is provided to build containers with `podman`.

The containers are built from the same configuration files as the `hbt_file-server` application.

#### Install required software

Install `podman` and `podman-compose`:

```
dnf install podman podman-compose
```

#### Create container scripts

The container script requires two arguments:
- A destination directory for the generated files
- A valid `hbt_file-server` JSON configuration file

The following psuedo cli command shows the argument schema:
```
bash build_container_files.sh <destination directory> <json config filepath>
```

Run the following shell commands to create a container based on `file-server.json.example`:
```
mkdir file-server/ctnr/
bash file-server/v0.1/build_container_files.sh \
  file-server/ctnr \
  file-server/v0.1/resources/file-server.json.example
```

#### Deploy container

The demo `podman` container files will be located in this repository at:
`file-server/cntr`.

To build the container:
```
podman-compose -f file-server/ctnr/file-server.podman-compose.yml build
```

To up the container:
```
podman-compose -f file-server/ctnr/file-server.podman-compose.yml up -d
```

To down the container:
```
podman-compose -f file-server/ctnr/file-server.podman-compose.yml down
```

Replace `file-server/ctnr/file-server.podman-compose.yml` with 
a different filepath to target an alternative container. 

## Licence

BSD 3-Clause License
