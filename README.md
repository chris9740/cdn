# rs-cdn

`rs-cdn` is a CDN written in Rust for Harmony.

## Authorization

We authenticate the publisher by the `metadata` field. The `metadata` field contains an RSA cipher that serves as a digital signature.
When decrypted, it reveals the hash of the image, which is then compared against the image itself.

Using asymmetric encryption allows us to more easily provision new nodes, since the public key used for decryption is checked into version control.

```json
{
    "hash": "c3efdf88ba62a2675f3ba2f746f243fc"
}
```
