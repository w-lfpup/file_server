# file_server

An `http` file server written in rust using [tokio](https://tokio.rs/) and
[hyper](https://hyper.rs/).

Includes support for:
- http 1.1 / 2
- boxed responses (send large files frame by frame)
- `head` requests
- `range` requests
- encoded requests

## How to use

### Install

Bash the following commands:

```sh
git clone https://github.com/herebythere/file_server
cargo install --path file_server/file_server
```

### Run

Bash the following command:

```sh
file_server
```

This will start `file_server` with it's default configuration in the `cwd`.

Now files can be requested from the `cwd` at `localhost:3000`:

```sh
curl localhost:3000
```

Alternatively, bash the following command to serve files based on a an example configuration:

```sh
file_server file_server.example.json
```

Open a browser and visit `http://localhost:4000`.

### Configuration

A valid [JSON configuration file](./file_server.example.json) matches the following schema.

```JSON
{
    "directory": "./demo",
    "host_and_port": "127.0.0.1:4000",
    "content_encodings": ["gzip", "deflate", "br", "zstd"],
    "filepath_404": "./demo/404.html"
}
```

Filepaths can be relative or absolute. Relative paths are "relative from" the filepath of the JSON configuration.

The `content_encodings` and `filepath_404` properties are optional.

### Accept-Encoding

When an `accept-encoding` header is found in a request, `file_server` will return a corresponding `zip`-ed version of file if available.

So if a request has the following header:

```
Accept-Encoding: gzip;
```

And the source file has a correspponding gziped file: 

```sh
./www/index.html		# source file
./www/index.html.gz		# gzipped file
```

Then `file_server` will send the encoded file, if available. Otherwise, it serves the source file.

### No dynamic encoding support

`File_server` does not encode or zip files ever.

This program serves static files. Just zip them up now to save memory resources.

### Range requests

`File_server` supports single range requests.

Multipart ranges are not currently supported because multipart ranges are a memory hog and difficult to
deliver efficiently without potentially maxing out ram resources.

## Licence

BSD 3-Clause License
