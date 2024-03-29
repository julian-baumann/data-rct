fn main() {
    #[cfg(feature = "sync")]
    uniffi::generate_scaffolding("./src/data_rct_sync.udl").unwrap();
    #[cfg(not(feature = "sync"))]
    uniffi::generate_scaffolding("./src/data_rct.udl").unwrap();
}