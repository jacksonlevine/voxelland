fn main() {
    // Specify the directory where the lib files are located, not the path to a specific lib file
    info!("cargo:rustc-link-search=native=C:/fmod"); // Path to the directory containing the FMOD library files
    info!("cargo:rustc-link-lib=fmod_vc"); // Name of the library file without the extension
}