fn main() {
    uniffi::generate_scaffolding("src/ddk_ffi.udl").unwrap();
}
