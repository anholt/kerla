use crate::prelude::*;
use core::{mem::size_of, slice::from_raw_parts};
use goblin::{elf::header::ET_DYN, elf64::header::{Header, ELFMAG, EM_X86_64, ET_EXEC}};
pub use goblin::elf64::program_header::ProgramHeader;

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
}
