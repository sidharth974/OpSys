use alloc::string::String;
use alloc::vec::Vec;

/// ELF64 magic number.
const ELF_MAGIC: [u8; 4] = [0x7F, b'E', b'L', b'F'];

/// ELF64 header.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Elf64Header {
    pub e_ident: [u8; 16],
    pub e_type: u16,
    pub e_machine: u16,
    pub e_version: u32,
    pub e_entry: u64,
    pub e_phoff: u64,
    pub e_shoff: u64,
    pub e_flags: u32,
    pub e_ehsize: u16,
    pub e_phentsize: u16,
    pub e_phnum: u16,
    pub e_shentsize: u16,
    pub e_shnum: u16,
    pub e_shstrndx: u16,
}

/// ELF64 program header.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Elf64Phdr {
    pub p_type: u32,
    pub p_flags: u32,
    pub p_offset: u64,
    pub p_vaddr: u64,
    pub p_paddr: u64,
    pub p_filesz: u64,
    pub p_memsz: u64,
    pub p_align: u64,
}

/// ELF64 section header.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Elf64Shdr {
    pub sh_name: u32,
    pub sh_type: u32,
    pub sh_flags: u64,
    pub sh_addr: u64,
    pub sh_offset: u64,
    pub sh_size: u64,
    pub sh_link: u32,
    pub sh_info: u32,
    pub sh_addralign: u64,
    pub sh_entsize: u64,
}

const PT_LOAD: u32 = 1;
const ET_EXEC: u16 = 2;
const EM_X86_64: u16 = 62;

/// Parsed ELF information.
#[derive(Debug)]
pub struct ElfInfo {
    pub entry_point: u64,
    pub machine: &'static str,
    pub elf_type: &'static str,
    pub segments: Vec<SegmentInfo>,
    pub total_mem: u64,
}

#[derive(Debug)]
pub struct SegmentInfo {
    pub vaddr: u64,
    pub memsz: u64,
    pub filesz: u64,
    pub flags: u32, // PF_X=1, PF_W=2, PF_R=4
    pub flags_str: String,
}

/// Parse an ELF64 binary and return info about it.
pub fn parse(data: &[u8]) -> Result<ElfInfo, &'static str> {
    if data.len() < 64 {
        return Err("Too small for ELF header");
    }

    if data[0..4] != ELF_MAGIC {
        return Err("Not an ELF file (bad magic)");
    }

    if data[4] != 2 {
        return Err("Not a 64-bit ELF");
    }

    let header: &Elf64Header = unsafe { &*(data.as_ptr() as *const Elf64Header) };

    let machine = match header.e_machine {
        EM_X86_64 => "x86_64",
        0x03 => "i386",
        0xB7 => "aarch64",
        0xF3 => "riscv",
        _ => "unknown",
    };

    let elf_type = match header.e_type {
        ET_EXEC => "executable",
        3 => "shared object",
        1 => "relocatable",
        4 => "core dump",
        _ => "unknown",
    };

    // Parse program headers
    let mut segments = Vec::new();
    let mut total_mem = 0u64;

    for i in 0..header.e_phnum {
        let offset = header.e_phoff as usize + i as usize * header.e_phentsize as usize;
        if offset + core::mem::size_of::<Elf64Phdr>() > data.len() {
            break;
        }
        let phdr: &Elf64Phdr = unsafe { &*(data.as_ptr().add(offset) as *const Elf64Phdr) };

        if phdr.p_type == PT_LOAD {
            let mut flags_str = String::new();
            if phdr.p_flags & 4 != 0 { flags_str.push('R'); }
            if phdr.p_flags & 2 != 0 { flags_str.push('W'); }
            if phdr.p_flags & 1 != 0 { flags_str.push('X'); }

            total_mem += phdr.p_memsz;
            segments.push(SegmentInfo {
                vaddr: phdr.p_vaddr,
                memsz: phdr.p_memsz,
                filesz: phdr.p_filesz,
                flags: phdr.p_flags,
                flags_str,
            });
        }
    }

    Ok(ElfInfo {
        entry_point: header.e_entry,
        machine,
        elf_type,
        segments,
        total_mem,
    })
}

/// Validate if an ELF binary is safe to load on this system.
pub fn validate(info: &ElfInfo) -> Result<(), &'static str> {
    if info.machine != "x86_64" {
        return Err("Wrong architecture (need x86_64)");
    }

    // Check W^X
    for seg in &info.segments {
        if seg.flags & 2 != 0 && seg.flags & 1 != 0 {
            return Err("W^X violation: segment is both writable and executable");
        }
    }

    Ok(())
}
