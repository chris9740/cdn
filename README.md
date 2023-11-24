# rs-cdn

`rs-cdn` is a CDN written in Rust for Harmony.

Resources on `rs-cdn` follow this structured path in the file system: `/{category}/{identifier}/{md5hash}.png`

-   `category`: Denotes the group to which the resource belongs, such as `avatars`.
-   `identifier`: Unique ID corresponding to the resource. For avatars, this is the user id.
-   `md5hash`: The MD5 hash of the image file.

Each identifier directory is intended to be exclusive, hosting a single resource at any given time. Resources can be found inside `./cdn`.

## Authentication

Publishers are authenticated through a digital signature accompanying each upload. This signature attests to the legitimacy of the content being uploaded.

To generate a signature, use the provided script:

```bash
./create_signature.sh <path_to_image>
```

Incorporate the output directly into the `signature` field when making a PUT request.

## Uploading Resources

Uploading a new resource requires a valid `signature` as outlined in the Authentication section.

Example PUT Request:

```bash
curl -X PUT http://localhost:8080/avatars/1234567890 \
 -H 'Content-Type: multipart/form-data' \
 -F "image=@assets/orange.jpg" \
 -F "signature=$(./create_signature.sh assets/orange.jpg)"
```

## Accessing Resources

After a successful upload, the resource is accessible through a URL structured as follows:

```
http://localhost:8080/{resource}/{user_id}/{md5hash}.png
```

### Example:

For the uploaded `orange.jpg` image with the identifier `1234567890`:

[http://localhost:8080/avatars/1234567890/c4c3fc411830d858e827ecb6bf8476a7.png](http://localhost:8080/avatars/1234567890/c4c3fc411830d858e827ecb6bf8476a7.png)

Navigating to the above link in a web browser will display the uploaded image.
