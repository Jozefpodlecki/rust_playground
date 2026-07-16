use bitflags::bitflags;

bitflags! {
    #[repr(transparent)]
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct HeapFlags: u32 {
        const NONE               = 0x00000000;
        const NO_SERIALIZE       = 0x00000001;
        const GROWABLE           = 0x00000002;
        const GENERATE_EXCEPTIONS = 0x00000004;
        const ZERO_MEMORY        = 0x00000008;
        const REALLOC_IN_PLACE   = 0x00000010;
        const TAIL_CHECKING      = 0x00000020;
        const FREE_CHECKING      = 0x00000040;
        const DISABLE_COALESCE   = 0x00000080;
        const ALIGN_16           = 0x00000200;
        const ALIGN_8            = 0x00000400;
        const ALIGN_4            = 0x00000800;
        const ALIGN_2            = 0x00001000;
        const ALIGN_1            = 0x00002000;
        const COMMIT_ADD         = 0x00010000;
        const COMMIT_GROW        = 0x00020000;
        const COMMIT_DEALLOC     = 0x00040000;
        const COMMIT                = 0x00080000;
        const DEALLOC             = 0x00100000;
        const LOCK_INTERNAL       = 0x00200000;
        const STICKY_STARTUP     = 0x01000000;
        const CACHE_ALIGN        = 0x02000000;
        const ALIGN_MASK         = 0x00003C00;
    }
}

impl Default for HeapFlags {
    fn default() -> Self {
        HeapFlags::NONE
    }
}