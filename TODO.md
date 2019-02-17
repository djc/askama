Setup:

```rust
// in target crate:
init_askama_i18n!();
```

```rust
// generates
mod __askama_i18n {
    struct _Pointless {}

    const SOURCES: ::askama::shared::i18n::Sources = &[
        ("en-us", EN_US),
        ("es-ar", ES_AR)
    ];

    const FALLBACK_CHAINS: ::askama::shared::i18n::FallbackChains = &[
        &["en-us", "en-uk"],
        &["en-uk", "en-us"],
        &["zh-bj", "zh-sh", "zh-hk"],
    ];

    const EN_US: &'static str = """
    greeting = Hello!
    """;

    const ES_AR: &'static str = """
    greeting = ¡Hola!
    """;

    ::askama::askama_derive::lazy_static! {
        static RESOURCES: ::askama::shared::fluent_bundle::Resources =
        static BUNDLES: ::askama::shared::fluent_bundle::FluentBundles =
            ::askama::shared::fluent_bundle::FluentBundles::new(FLUENT_SOURCES, FALLBACK_CHAINS);
    }

    #[cfg(test)]
    mod tests {
        #[test]
        fn parse() {
            let _parse_all_sources = *BUNDLES;
        }
    }
}
```
