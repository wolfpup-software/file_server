# file_server

`http` file server written in rust using [tokio](https://tokio.rs/) and
[hyper](https://hyper.rs/).

## Create a config

A JSON configuration file is required to run `hbt_file_server` or create a
`file_server` container.

Configuration files are expected to use use the following schema:

```
{
  "host": <string>,
  "port": <number>,
  "directory": <string>
}
```

Change the `host` property to serve from a specific host.

Change the `port` property to serve from a different port.

Change the `directory` property to target an alternative directory.

An example of a valid configuration can be found at
`file_server/v0.1/resources/file_server.json.example`

```json
{
  "host": "127.0.0.1",
  "port": 3000,
  "directory": "./docs"
}
```

## Install file_server

Execute the following to install `htb_file_server`.

```
git clone https://github.com/herebythere/file_server
cargo install --path file_server/v0.1/file_server
```

## Run file_server

The `file_server` application accepts one argument:

- A valid `file_server` JSON configuration file

The following psuedo-script shows the argument schema:

```
file_server <path_to_configuration_file>
```

Execute the following to run a `hbt_file_server` demo hosting the repositories
`/docs` directory.

```
file_server file_server/v0.1/resources/file_server.json.example
```

Open a browser and visit `http://localhost:3000`.

## file_server containers

A utility script is provided to build containers with `podman`.

The containers are built from the same configuration files as the
`file_server` application.

#### Install required software

Install `podman` and `podman-compose`:

```
dnf install podman podman-compose
```

#### Create containers 

The container script requires four arguments:

1. Destination directory for the generated files
2. A valid `file_server` JSON configuration file
3. A podmanfile template
4. A podman-compose template

The following psuedo cli command shows the argument schema:

```
bash build_container_files.sh <destination directory> <json config filepath>
```

Run the following shell commands to create a container based on
`file_server.json.example`:

```
mkdir file_server/ctnr/
bash file_server/v0.1/build_container_files.sh \
  file_server/ctnr \
  file_server/v0.1/resources/file_server.json.example
```

#### SELinux labels

add the `container_file_t` label or an equivalent label to `file_server/cntr` and all descendant files.

#### Deploy container

The demo `podman` container files will be located in this repository at:
`file_server/cntr`.

To build the container:

```
podman-compose -f file_server/ctnr/podman-compose.yml build
```

To up the container:

```
podman-compose -f file_server/ctnr/podman-compose.yml up -d
```

To down the container:

```
podman-compose -f file_server/ctnr/podman-compose.yml down
```

## Licence

BSD 3-Clause License
