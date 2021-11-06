use crate::{
    arch::{arch_prctl, UserVAddr},
    result::Result,
};
use crate::{process::current_process, syscalls::SyscallHandler};

impl<'a> SyscallHandler<'a> {
    pub fn sys_arch_prctl(&mut self, code: i32, uaddr: UserVAddr) -> Result<isize> {
        arch_prctl(current_process(), code, uaddr)?;
        Ok(0)
    }
}
