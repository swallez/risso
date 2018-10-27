# Risso - a comment server for static websites

Risso is a Rust port of [Isso](https://posativ.org/isso/), a self-hosted comment server.
The name, obviously, is a combination of "**r**ust" and "isso" (which is itself an acronym).

This is my playground to learn Rust and experiment web application development with
the Rust ecosystem.

**Risso is not yet functional**. It compiles successfully, but is still very much a **work in progress**.

It is composed of several sub-projects:
- `risso_api` is the heart of the system, providing the APIs as a set of functions,
  independent of the web environment. This separation allows to easily experiment with
  various web frameworks or even with [serverless](https://github.com/srijs/rust-aws-lambda) front-ends.

- `risso_actix` exposes `risso_api` as an http service using [actix-web](https://actix.rs/).

- `risso_admin` is empty for now, and is meant to be the admin front-end to moderate
  comments. To go full Rust, it will be target WebAssembly using [Yew](https://github.com/DenisKolodin/yew),
  a React-inspired front-end framework.

## Components & features

Risso is the aggregation of many great crates from the Rust ecosystem. Rust comes with
"batteries excluded", with a great but minimal standard library. Finding good
batteries is key to be productive. Risso uses, among others:
- web framework: actix
- data validation: validator
- database access / ORM: diesel
- thread pools: tokio-threadpool
- structured logs: slogs
- metrics: prometheus
- date calculations: chrono
- futures for asynchronous programming (until it's built into the standard lib)
- serialization/deserialization to about any format: serde
- handling configurations: config
- error handling: failure
- markdown parsing & rendering: pulldown-cmark
- html parsing & sanitization: html5ever & ammonia
- smtp client: lettre

## License

This project is licensed under the [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0).
