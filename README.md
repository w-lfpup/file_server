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

This will start `file_server` with using the `cwd` as a base directory.

Now files can be requested from the `cwd` at `localhost:3000`:

```sh
curl localhost:3000
```

### Configuration

A valid [JSON configuration file](./demo/file_server_config_example.json) matches the following schema.

```JSON
{
    "directory": "./",
    "host_and_port": "127.0.0.1:4000",
    "content_encodings": ["gzip", "deflate", "br", "zstd"],
    "filepath_404": "./404.html"
}
```

Filepaths can be relative or absolute. Relative paths are "relative from" the filepath of the JSON configuration file.

The `content_encodings` and `filepath_404` properties are optional.

#### Run with configuration

Bash the following command to serve files based on a an example configuration:

```sh
file_server demo/demo.example.json
```

Open a browser and visit `http://localhost:3000` and an encoded version of `index.html` will be delivered.

### Accept-Encoding

When a request has an `accept-encoding` header, `file_server` will return a corresponding `zip`-ed version of file if available.

So if a request has the following header:

```
Accept-Encoding: gzip;
```

And the target file has a correspponding gziped file: 

```sh
index.html		# source file
index.html.gz	# gzipped file
```

`File_server` will send the encoded file, if available. Otherwise, it serves the source file.

### No dynamic encoding support

`File_server` does not encode or zip files ever.

This program serves static files. Just zip them up now to save memory resources.

### Range requests

`File_server` supports single range requests.

Bash the following command:

```sh
curl -v -r 0-6 localhost:3000
```

And the first 6 bytes of `index.html` will be delivered.

Multipart ranges are not currently supported.

Multipart ranges are memory hogs and difficult to deliver efficiently without abusing memory resources.

## License

`File_server` is released under the BSD 3-Clause License.
