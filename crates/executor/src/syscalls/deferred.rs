use super::{Syscall, SyscallCode, SyscallContext};

pub(crate) struct CommitDeferredSyscall;

impl Syscall for CommitDeferredSyscall {
    #[allow(clippy::mut_mut)]
    fn execute(&self, _: &mut SyscallContext, _: SyscallCode, _: u32, _: u32) -> Option<u32> {
        None
    }
}
