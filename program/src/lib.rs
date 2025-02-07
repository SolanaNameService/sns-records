use bonfida_utils::declare_id_with_central_state;

#[doc(hidden)]
pub mod entrypoint;
#[doc(hidden)]
pub mod error;
/// Program instructions and their CPI-compatible bindings
pub mod instruction;
/// Describes the different data structres that the program uses to encode state
pub mod state;

pub mod utils;

#[doc(hidden)]
pub(crate) mod processor;

#[allow(missing_docs)]
pub mod cpi;

#[cfg(not(feature = "devnet"))]
declare_id_with_central_state!("HP3D4D1ZCmohQGFVms2SS4LCANgJyksBf5s1F77FuFjZ");

#[cfg(feature = "devnet")]
declare_id_with_central_state!("Ga872GkshNeNMDag7m1Bn54dN3NiHksfqnN2pH6A1H9F");

#[cfg(not(feature = "no-entrypoint"))]
solana_security_txt::security_txt! {
    name: env!("CARGO_PKG_NAME"),
    project_url: "http://bonfida.org",
    contacts: "email:security@bonfida.com,link:https://twitter.com/bonfida",
    policy: "https://immunefi.com/bounty/bonfida",
    preferred_languages: "en"
}
