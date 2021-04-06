use penguin_utils::alignment::is_aligned;

use crate::fs::opened_file::Fd;
use crate::{
    arch::UserVAddr,
    result::{Errno, Result},
};
use crate::{arch::PAGE_SIZE, ctypes::*, mm::vm::VmAreaType};
use crate::{process::current_process, syscalls::SyscallDispatcher};

impl<'a> SyscallDispatcher<'a> {
    pub fn sys_mmap(
        &mut self,
        addr_hint: UserVAddr,
        len: c_size,
        _prot: MMapProt,
        flags: MMapFlags,
        fd: Fd,
        offset: c_off,
    ) -> Result<isize> {
        // TODO: Respect `prot`.

        if !is_aligned(len as usize, PAGE_SIZE) {
            return Err(Errno::EINVAL.into());
        }

        if !is_aligned(offset as usize, PAGE_SIZE) {
            return Err(Errno::EINVAL.into());
        }

        let area_type = if flags.contains(MMapFlags::MAP_ANONYMOUS) {
            VmAreaType::Anonymous
        } else {
            let file = current_process()
                .opened_files
                .lock()
                .get(fd)?
                .lock()
                .as_file()?
                .clone();

            VmAreaType::File {
                file,
                offset: offset as usize,
                file_size: len as usize,
            }
        };

        // Determine the virtual address space to map.
        let mut vm = current_process().vm();
        let mapped_uaddr = if addr_hint.is_null() {
            vm.alloc_vaddr_range(len as usize)?
        } else if vm.is_free_vaddr_range(addr_hint, len as usize) {
            addr_hint
        } else {
            // [addr_hint, addr_hint + len) is already in use or invalid.
            if flags.contains(MMapFlags::MAP_FIXED) {
                return Err(Errno::EINVAL.into());
            } else {
                vm.alloc_vaddr_range(len as usize)?
            }
        };

        vm.add_vm_area(mapped_uaddr, len as usize, area_type)?;
        Ok(mapped_uaddr.value() as isize)
    }
}