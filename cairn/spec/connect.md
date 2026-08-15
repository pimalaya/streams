---
cairn: spec
capability: connect
status: current
---

# Connect

Opening the one `Read + Write` handle every io- protocol crate is handed: a TCP socket, a Unix-domain socket, or a TLS session over TCP. `Stream` is that handle, and the variant it wraps is invisible to the caller above it.

### Requirement: The three transports

`Stream` SHALL open a plain TCP connection, a TCP connection wrapped in implicit TLS, or a Unix-domain socket, and SHALL present all three behind the same `Read + Write` surface.

A Unix-domain stream reports `127.0.0.1` as its host, having none of its own, which is what the TLS layer would need were it ever upgraded.

#### Scenario: Implicit TLS
- GIVEN a host and a port speaking TLS from the first byte
- WHEN the caller connects with the TLS options it wants
- THEN it receives a handle whose reads and writes are already inside the session

### Requirement: The STARTTLS upgrade

A plain TCP stream SHALL upgrade in place to a TLS session, consuming the handle and returning the wrapped one, so a protocol that negotiates TLS mid-session (IMAP, SMTP) keeps its socket.

The upgrade SHALL be refused on a Unix-domain stream and on a stream already wrapped in TLS, neither having a plain socket to hand over.

#### Scenario: Upgrading twice
- GIVEN a stream already wrapped in a TLS session
- WHEN an upgrade is requested again
- THEN it fails, saying the stream is already wrapped

### Requirement: One connect path

Opening a stream SHALL have exactly one form: a `connect_*` method per transport, taking that transport's options. No builder, and no second entry point wrapping the first.

Every path SHALL end in the same private constructor, the one place that builds the handle and arms the socket read deadline its retry strategy implies.

### Requirement: Options per transport

Each connect SHALL take an options struct holding what its transport has and nothing more: a proxy where there is one to route through, TLS settings where there is a session to secure, and the retry strategy everywhere.

Options SHALL be plain structs of public fields with a `Default`, filled in as a struct literal, the shape a consumer already meets in the io- protocol crates.

#### Scenario: A caller with its own proxy configuration
- GIVEN a proxy read from the caller's own configuration file
- WHEN the stream is opened with options carrying that proxy
- THEN the connection is dialled through it rather than through the environment's

#### Scenario: A caller with nothing to say
- GIVEN a caller wanting the ambient proxy and the default strategy
- WHEN it connects with the default options
- THEN it names neither, and gets both
