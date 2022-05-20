# Podbringer

Podbringer is a web service that provides podcasts for services that don't
offer them (anymore). It provides a way to get the RSS feed for your podcast
client and it facilites the downloads of the pods (enclosures).

It currently only supports [Mixcloud](https://mixcloud.com).
Other back-ends might be added in the future.

## Building & running

Using Cargo, it is easy to build and run Podbringer, just run:

```shell
$ cargo run --release
...
   Compiling podbringer v0.1.0 (/path/to/podbringer)
    Finished release [optimized] target(s) in 9m 26s
     Running `/path/to/podbringer/target/release/podbringer`
```

(Note that Rocket listens on `127.0.0.1:8000` by default for debug builds, i.e.
builds when you don't add `--release`.)

### Configuration

For now, you will need to provide Rocket with configuration to tell it with which URL Podbringer is
reachable. Even if you are not using a reverse proxy, in which case you need to provide it with the
proxied URL. You can also use the configuration to configure a different address and/or port.
Just create a `Rocket.toml` file that contains (or copy `Rocket.toml.example`):

```toml
[default]
address = "0.0.0.0"
port = 7062
url = "https://my.domain.tld/podbringer"
```

This will work independent of the type of build. For more about Rocket's
configuration, see: <https://rocket.rs/v0.5-rc/guide/configuration/>.

## License

Podbringer is licensed under the MIT license (see the `LICENSE` file or
<http://opensource.org/licenses/MIT>).
