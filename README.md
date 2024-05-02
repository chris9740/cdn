# rs-cdn

`rs-cdn` is a CDN written in Rust for Harmony.

Resources on `rs-cdn` follow this structured path in the file system:
`/{category}/{identifier}/{sha1hash}.png`

-   `category`: Denotes the group to which the resource belongs, such as `avatars`.
-   `identifier`: Unique ID corresponding to the resource. For avatars, this is the user id.
-   `sha1hash`: The SHA1 hash of the image file.

Each identifier directory is intended to be exclusive, hosting a single resource at any given time.
Resources can be found inside `./cdn`.

## Running the Service

### Prerequisites

 - [Docker Engine](https://docs.docker.com/engine/install/)

 - [Compose V2](https://docs.docker.com/compose/install/linux/)

 - [Cross-rs](https://github.com/cross-rs/cross#installation)

### Building

Before you can start the application, you need to build it first.

You can run `make build` to create a debug build, and `make release` to create a release build.

### Starting the Server

We use Compose v2 for this project. As such, we use `docker compose` instead of `docker-compose`.

#### > Debug mode

To start the application in development mode, run `make build && docker compose up`.

#### > Release mode

You can also run in release mode, using `make release && docker compose -f compose.yaml -f compose.release.yaml up`.

This will give you a more realistic experience in regards to speed of image processing, as debug mode
is significantly slower for large files (`> 1s` in debug, vs `< 50ms` in release).

## Authentication

Publishers are authenticated through a digital signature accompanying each upload. This signature
attests to the legitimacy of the content being uploaded.

To generate a signature for development, use the provided script:

```bash
./create_signature.sh <path_to_image>
```

Incorporate the output directly into the `signature` field when making a PUOS request.

## Uploading Resources

Uploading a new resource requires a valid `signature` as outlined in the Authentication section.

Example POST Request:

```bash
curl -X POST http://localhost:8080/avatars/1234567890 \
 -H 'Content-Type: multipart/form-data' \
 -F "image=@assets/orange.jpg" \
 -F "signature=$(./create_signature.sh assets/orange.jpg)"
```

## Accessing Resources

After a successful upload, the resource is accessible through a URL structured as follows:

```
http://localhost:8080/{category}/{user_id}/{sha1hash}.png
```

### Example:

For the uploaded `orange.jpg` image with the identifier `1234567890`:

[http://localhost:8080/avatars/1234567890/b4d3499823b249df78507443a2fa6ec90933e3c4.png](http://localhost:8080/avatars/1234567890/b4d3499823b249df78507443a2fa6ec90933e3c4.png)

Navigating to the above link in a web browser will display the uploaded image.
