fn main() {
    // Specify the directory where the lib files are located, not the path to a specific lib file
    println!("cargo:rustc-link-search=native=C:/fmod"); // Path to the directory containing the FMOD library files
    println!("cargo:rustc-link-lib=fmod_vc"); // Name of the library file without the extension
}