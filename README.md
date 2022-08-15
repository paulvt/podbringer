# Podbringer

Podbringer is a web service that provides podcasts for services that don't
offer them (anymore). It provides a way to get the RSS feed for your podcast
client and it facilites the downloads of the pods (enclosures).

It currently only supports [Mixcloud](https://www.mixcloud.com) and
[YouTube](https://www.youtube.com).
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

For now, you will need to provide Rocket with configuration to tell it at which
public URL Podbringer is hosted. This needs to be done even if you are not using a
reverse proxy, in which case you need to provide it with the proxied URL. You
can also use the configuration to configure a different address and/or port.
Just create a `Rocket.toml` file that contains (or copy `Rocket.toml.example`):

```toml
[default]
address = "0.0.0.0"
port = 7062
public_url = "https://my.domain.tld/podbringer"
```

This will work independent of the type of build. For more about Rocket's
configuration, see: <https://rocket.rs/v0.5-rc/guide/configuration/>.

## Usage

Podbringer currently has no front-end or web interface yet that can help you
use it. Until then, you just have to enter the right service-specific RSS feed
URL in your favorite podcast client to start using it.

Given the Mixcloud URL <https://www.mixcloud.com/myfavouriteband/>, the URL you
need to use for Podbringer is comprised of the following parts:

```text
  https://my.domain.tld/podbringer/feed/mixcloud/myfavouriteband
  |------------------------------|     |-------||--------------|
   The Podbringer public URL            Service  User @ service
```

### Feed item limit

To prevent feeds with a very large number of items, any feed that is returned
contains at most 50 items by default. If you want to have more (or less) items,
provide the limit in the URL by setting the `limit` parameter.

For example, to get up until 1000 items the URL becomes:
`https://my.domain.tld/podbringer/feed/mixcloud/myfavouriteband?limit=1000`

## License

Podbringer is licensed under the MIT license (see the `LICENSE` file or
<http://opensource.org/licenses/MIT>).
