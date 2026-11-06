// Compile-time string obfuscation via obfstr.
// Strings are XOR-encrypted at compile time and decrypted in-place at runtime.
// No plaintext literal appears in the .rdata section of the release binary.
pub use obfstr::obfstr;

/// Decrypt an obfuscated string literal at runtime and return an owned String.
///
/// Usage: `let s = hide!("kernel32.dll");`
#[macro_export]
macro_rules! hide {
    ($s:literal) => {
        obfstr::obfstr!($s).to_owned()
    };
}
