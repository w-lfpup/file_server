# File server

## Configuration

A valid [JSON configuration file](./file_server.json) matches the following schema.

```JSON
{
	"directory": "./demo",
	"host_and_port": "127.0.0.1:4000",
	"content_encodings": ["gzip", "deflate", "br", "zstd"],
	"filepath_404": "./demo/404.html"
}
```

The `content_encodings` and `filepath_404` properties are optional.

### Run

Bash the following command to serve files based on a configuration:

```sh
file_server path/to/config.json
```

Open a browser and visit `http://localhost:4000`.

### Accept-Encoding

When an `accept-encoding` header is found in a request, `file_server` will return a corresponding `zip`-ed version of a requested file if available.

So if a request has the following header:

```
Accept-Encoding: gzip;
```

And the target file has a correspponding gziped file: 

```sh
./www/index.html		# target file
./www/index.html.gz		# gzipped file
```

Then `file_server` will send the encoded file, if available. Otherwise, it serves the unencoded file.

### No dynamic encoding support

`File_server` does not encode or zip files ever.

This program serves static files. Just zip them up now it saves on memory resources.

### Range requests

`File_server` supports single range requests.

Multipart ranges are not currently supported because multipart ranges are a memory hog.
