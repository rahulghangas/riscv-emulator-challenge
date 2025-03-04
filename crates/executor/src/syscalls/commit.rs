use super::{Syscall, SyscallCode, SyscallContext};

pub(crate) struct CommitSyscall;

impl Syscall for CommitSyscall {
    #[allow(clippy::mut_mut)]
    fn execute(&self, _: &mut SyscallContext, _: SyscallCode, _: u32, _: u32) -> Option<u32> {
        None
    }
}
