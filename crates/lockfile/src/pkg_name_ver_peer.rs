use crate::{ParsePkgNameSuffixError, PkgNameSuffix, PkgVerPeer};
use std::convert::Infallible;

/// Syntax: `{name}@{version}({peers})`
///
/// Example: `react-json-view@1.21.3(@types/react@17.0.49)(react-dom@17.0.2)(react@17.0.2)`
///
/// **NOTE:** The suffix isn't guaranteed to be correct. It is only assumed to be.
pub type PkgNameVerPeer = PkgNameSuffix<PkgVerPeer>;

/// Error when parsing [`PkgNameVerPeer`] from a string.
pub type ParsePkgNameVerPeerError = ParsePkgNameSuffixError<Infallible>;

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn name_peer_ver(name: &str, peer_ver: &str) -> PkgNameVerPeer {
        let peer_ver = peer_ver.to_string().parse().unwrap();
        PkgNameVerPeer::new(name.to_string(), peer_ver)
    }

    #[test]
    fn parse() {
        macro_rules! case {
            ($input:expr => $output:expr) => {{
                let input = $input;
                eprintln!("CASE: {input:?}");
                let received: PkgNameVerPeer = input.parse().unwrap();
                let expected = $output;
                assert_eq!(&received, &expected);
            }};
        }

        case!(
            "react-json-view@1.21.3(@types/react@17.0.49)(react-dom@17.0.2)(react@17.0.2)" => name_peer_ver(
                "react-json-view",
                "1.21.3(@types/react@17.0.49)(react-dom@17.0.2)(react@17.0.2)",
            )
        );
        case!("react-json-view@1.21.3" => name_peer_ver("react-json-view", "1.21.3"));
        case!(
            "@algolia/autocomplete-core@1.9.3(@algolia/client-search@4.18.0)(algoliasearch@4.18.0)(search-insights@2.6.0)" => name_peer_ver(
                "@algolia/autocomplete-core",
                "1.9.3(@algolia/client-search@4.18.0)(algoliasearch@4.18.0)(search-insights@2.6.0)",
            )
        );
        case!("@algolia/autocomplete-core@1.9.3" => name_peer_ver("@algolia/autocomplete-core", "1.9.3"));
    }
}
