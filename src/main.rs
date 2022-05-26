#![doc = include_str!("../README.md")]
#![warn(
    clippy::all,
    missing_copy_implementations,
    missing_debug_implementations,
    rust_2018_idioms,
    rustdoc::broken_intra_doc_links,
    trivial_numeric_casts
)]
#![deny(missing_docs)]

/// Sets up and launches Rocket.
#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let rocket = podbringer::setup();
    let _ = rocket.ignite().await?.launch().await?;

    Ok(())
}
