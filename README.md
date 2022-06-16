# File-server

A file-server written in rust.

# Usage

create a configuration file like the one below:

```
{
  dir: ./files/to/serve,
  port: 3000,
  filepath_403: ./file/to/serve/403.html,
  filepath_404: ./file/to/serve/404.html,
  filepath_500: ./file/to/serve/500.html,
}
```
