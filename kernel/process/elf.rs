use crate::prelude::*;
use core::{mem::size_of, slice::from_raw_parts};
pub use goblin::elf64::program_header::ProgramHeader;
use goblin::{
    elf::{
        header::ET_DYN, program_header::{PT_INTERP, PT_LOAD}
    },
    elf64::header::{Header, ELFMAG, EM_X86_64, ET_EXEC},
};
use kerla_api::arch::PAGE_SIZE;
use kerla_runtime::address::UserVAddr;

/// A parsed ELF object.
pub struct Elf<'a> {
    header: &'a Header,
    program_headers: &'a [ProgramHeader],
}

impl<'a> Elf<'a> {
    /// Parses a ELF header.
    pub fn parse(buf: &'a [u8]) -> Result<Elf<'a>> {
        if buf.len() < size_of::<Header>() {
            debug_warn!("ELF header buffer is too short");
            return Err(Errno::ENOEXEC.into());
        }

        let header: &Header = unsafe { &*(buf.as_ptr() as *const Header) };
        if &header.e_ident[..4] != ELFMAG {
            debug_warn!("invalid ELF magic");
            return Err(Errno::ENOEXEC.into());
        }

        if header.e_machine != EM_X86_64 {
            debug_warn!("invalid ELF e_machine");
            return Err(Errno::ENOEXEC.into());
        }

        if header.e_type != ET_EXEC && header.e_type != ET_DYN {
            debug_warn!("ELF is not executable or dynamic: {:?}", header.e_type);
            return Err(Errno::ENOEXEC.into());
        }

        let program_headers = unsafe {
            from_raw_parts(
                &buf[header.e_phoff as usize] as *const _ as *const ProgramHeader,
                header.e_phnum as usize,
            )
        };

        Ok(Elf {
            header,
            program_headers,
        })
    }

    /// The ELF header.
    pub fn header(&self) -> &Header {
        self.header
    }

    /// Program headers.
    pub fn program_headers(&self) -> &[ProgramHeader] {
        self.program_headers
    }

    pub fn phdr_vaddr(&self) -> Result<UserVAddr> {
        let phoff = self.header().e_phoff;

        for hdr in self.program_headers() {
            let p_offset = hdr.p_offset;
            if phoff >= p_offset && phoff < p_offset.checked_add(hdr.p_filesz).unwrap_or(0) {
                return UserVAddr::new_nonnull((phoff - p_offset + hdr.p_vaddr) as usize)
                    .map_err(Into::into);
            }
        }
        Err(Errno::ENOEXEC.into())
    }

    /// Returns the ELF interpreter defined in the program headers, or None.
    /// TODO: Elf needs to be a better abstraction where it holds the buf, so we
    /// don't need to pass it in again like this.
    pub fn interpreter(&self, buf: &'a [u8]) -> Option<&'a [u8]> {
        for phdr in self.program_headers() {
            if phdr.p_type == PT_INTERP {
                let start = phdr.p_offset as usize;
                let end = start.checked_add(phdr.p_filesz as usize)?;
                // TODO: The interpreter can be offset past the first page (for
                // example, patchelf --set-interpreter), while we currently only
                // load the first page into the kernel.
                return buf.get(start..end);
            }
        }
        None
    }

    /// Returns the overall alignment of the program to be loaded based on the
    /// alignment of the PT_LOAD segments.  The minimum alignment we can honor
    /// is a page size.
    pub fn max_align(&self) -> u64 {
        let mut align = PAGE_SIZE as u64;
        for phdr in self.program_headers() {
            if phdr.p_type != PT_LOAD {
                continue;
            }
            align = align.max(phdr.p_align);
        }
        align
    }
}
