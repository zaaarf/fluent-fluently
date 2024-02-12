# Fluent, fluently
A small Rust library handling loading runtime loading of [Fluent](https://github.com/projectfluent/fluent-rs) localisation. By design, Fluent does not touch the IO part, only providing String parsing. This library takes care of that.

I intentionally kept this as simple as possible to reflect my very basic use case. Check out [fluent-localization](https://github.com/AEnterprise/fluent-localization) for something with more features, namely compile-time validation and localisation struct generation for easier access.
