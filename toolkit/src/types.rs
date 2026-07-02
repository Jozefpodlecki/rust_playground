use winapi::shared::ntdef::PVOID;
use winapi::shared::ntdef::{LIST_ENTRY};
use winapi::um::winnt::{RTL_CRITICAL_SECTION, RTL_RUN_ONCE};

use crate::println;

#[repr(C)]
#[derive(Clone, Copy)]
pub union HEAP_UNPACKED_ENTRY_UNION {
    pub SizeFlagsTag: HEAP_SIZE_FLAGS_TAG,
    pub SubSegment: HEAP_SUB_SEGMENT,
    pub CompactHeader: u64,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct HEAP_SIZE_FLAGS_TAG {
    pub Size: u16,
    pub Flags: u8,
    pub SmallTagIndex: u8,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct HEAP_SUB_SEGMENT {
    pub SubSegmentCode: u32,
    pub PreviousSize: u16,
    pub SegmentOffset: u8,
    pub UnusedBytes: u8,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct HEAP_UNPACKED_ENTRY {
    pub PreviousBlockPrivateData: PVOID,
    pub u: HEAP_UNPACKED_ENTRY_UNION,
}

impl core::fmt::Display for HEAP_UNPACKED_ENTRY {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "    PreviousBlockPrivateData: {:p}", self.PreviousBlockPrivateData)?;
        unsafe {
            // Try to interpret as Size/Flags/SmallTagIndex first
            let size = self.u.SizeFlagsTag.Size;
            let flags = self.u.SizeFlagsTag.Flags;
            let tag = self.u.SizeFlagsTag.SmallTagIndex;
            
            if size != 0 || flags != 0 || tag != 0 {
                writeln!(f, "    Size:                    {}", size)?;
                writeln!(f, "    Flags:                   0x{:02X}", flags)?;
                writeln!(f, "    SmallTagIndex:           0x{:02X}", tag)?;
            } else {
                // Try as SubSegment
                let code = self.u.SubSegment.SubSegmentCode;
                let prev_size = self.u.SubSegment.PreviousSize;
                let offset = self.u.SubSegment.SegmentOffset;
                let unused = self.u.SubSegment.UnusedBytes;
                if code != 0 || prev_size != 0 || offset != 0 || unused != 0 {
                    writeln!(f, "    SubSegmentCode:         0x{:08X}", code)?;
                    writeln!(f, "    PreviousSize:           {}", prev_size)?;
                    writeln!(f, "    SegmentOffset:          0x{:02X}", offset)?;
                    writeln!(f, "    UnusedBytes:            0x{:02X}", unused)?;
                } else {
                    writeln!(f, "    CompactHeader:          0x{:016X}", self.u.CompactHeader)?;
                }
            }
        }
        Ok(())
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union HEAP_EXTENDED_ENTRY_UNION {
    pub FunctionContext: HEAP_FUNCTION_CONTEXT,
    pub InterceptorValue: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct HEAP_FUNCTION_CONTEXT {
    pub FunctionIndex: u16,
    pub ContextValue: u16,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct HEAP_EXTENDED_ENTRY {
    pub Reserved: PVOID,
    pub u: HEAP_EXTENDED_ENTRY_UNION,
    pub UnusedBytesLength: u16,
    pub EntryOffset: u8,
    pub ExtendedBlockSignature: u8,
}

impl core::fmt::Display for HEAP_EXTENDED_ENTRY {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "    Reserved:              {:p}", self.Reserved)?;
        unsafe {
            if self.u.InterceptorValue != 0 {
                writeln!(f, "    InterceptorValue:       0x{:08X}", self.u.InterceptorValue)?;
            } else {
                writeln!(f, "    FunctionIndex:          {}", self.u.FunctionContext.FunctionIndex)?;
                writeln!(f, "    ContextValue:           {}", self.u.FunctionContext.ContextValue)?;
            }
        }
        writeln!(f, "    UnusedBytesLength:     {}", self.UnusedBytesLength)?;
        writeln!(f, "    EntryOffset:           0x{:02X}", self.EntryOffset)?;
        writeln!(f, "    ExtendedBlockSignature: 0x{:02X}", self.ExtendedBlockSignature)?;
        Ok(())
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union HEAP_ENTRY_UNION {
    pub UnpackedEntry: HEAP_UNPACKED_ENTRY,
    pub ExtendedEntry: HEAP_EXTENDED_ENTRY,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct HEAP_ENTRY {
    pub u: HEAP_ENTRY_UNION,
}

impl core::fmt::Display for HEAP_ENTRY {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "  HEAP_ENTRY @ {:p}", self)?;
        unsafe {
            // Try to determine which variant this is
            // If ExtendedBlockSignature is valid, it's an extended entry
            if self.u.ExtendedEntry.ExtendedBlockSignature != 0 {
                writeln!(f, "  (Extended Entry)")?;
                write!(f, "{}", self.u.ExtendedEntry)?;
            } else {
                writeln!(f, "  (Unpacked Entry)")?;
                write!(f, "{}", self.u.UnpackedEntry)?;
            }
        }
        Ok(())
    }
}

// The rest of the structures remain the same...
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ERESOURCE {
    pub SystemResourcesList: LIST_ENTRY,
    pub OwnerTable: *mut OWNER_ENTRY,
    pub ActiveCount: i16,
    pub Flag: u16,
    pub ReservedLowFlags: u8,
    pub WaiterPriority: u8,
    pub SharedWaiters: PVOID,
    pub ExclusiveWaiters: PVOID,
    pub OwnerEntry: OWNER_ENTRY,
    pub ActiveEntries: u32,
    pub ContentionCount: u32,
    pub NumberOfSharedWaiters: u32,
    pub NumberOfExclusiveWaiters: u32,
    pub MiscFlags: i8,
    pub Reserved1: [u8; 3],
    pub ResourceTimeoutCount: u32,
    pub Address: PVOID,
    pub CreatorBackTraceIndex: u64,
    pub SpinLock: u64,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct OWNER_ENTRY {
    pub OwnerThread: u64,
    pub OwnerCount: u64,
    pub TableSize: u64,
    pub OwnerEntry: [u8; 0x20],
}

#[repr(C)]
pub struct HEAP_TAG_ENTRY {
    pub Allocs: u32,
    pub Frees: u32,
    pub Size: u64,
    pub TagIndex: u16,
    pub CreatorBackTraceIndex: u16,
    pub TagName: [u16; 24],
}

#[repr(C)]
pub struct HEAP_PSEUDO_TAG_ENTRY {
    pub Allocs: u32,
    pub Frees: u32,
    pub Size: u64,
}

#[repr(C)]
pub struct HEAP_LOCK {
    pub Lock: HEAP_LOCK_UNION,
}

#[repr(C)]
pub union HEAP_LOCK_UNION {
    pub CriticalSection: RTL_CRITICAL_SECTION,
    pub Resource: ERESOURCE,
}

#[repr(C)]
pub struct RTLP_HEAP_COMMIT_LIMIT_DATA {
    pub CommitLimitBytes: u64,
    pub CommitLimitFailureCode: u64,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct HEAP_SEGMENT {
    pub Entry: HEAP_ENTRY,
    pub SegmentSignature: u32,
    pub SegmentFlags: u32,
    pub SegmentListEntry: LIST_ENTRY,
    pub Heap: *mut HEAP,
    pub BaseAddress: PVOID,
    pub NumberOfPages: u32,
    pub FirstEntry: *mut HEAP_ENTRY,
    pub LastValidEntry: *mut HEAP_ENTRY,
    pub NumberOfUnCommittedPages: u32,
    pub NumberOfUnCommittedRanges: u32,
    pub SegmentAllocatorBackTraceIndex: u16,
    pub Reserved: u16,
    pub UCRSegmentList: LIST_ENTRY,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct HEAP_ANONYMOUS {
    pub Entry: HEAP_ENTRY,
    pub SegmentSignature: u32,
    pub SegmentFlags: u32,
    pub SegmentListEntry: LIST_ENTRY,
    pub Heap: *mut HEAP,
    pub BaseAddress: PVOID,
    pub NumberOfPages: u32,
    pub FirstEntry: *mut HEAP_ENTRY,
    pub LastValidEntry: *mut HEAP_ENTRY,
    pub NumberOfUnCommittedPages: u32,
    pub NumberOfUnCommittedRanges: u32,
    pub SegmentAllocatorBackTraceIndex: u16,
    pub Reserved: u16,
    pub UCRSegmentList: LIST_ENTRY,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union HEAP_UNION {
    pub Segment: HEAP_SEGMENT,
    pub Anonymous: HEAP_ANONYMOUS,
}

#[repr(C)]
pub struct HEAP_COUNTERS {
    pub TotalMemoryReserved: u64,
    pub TotalMemoryCommitted: u64,
    pub TotalMemoryLargeUCR: u64,
    pub TotalSizeInVirtualBlocks: u64,
    pub TotalSegments: u32,
    pub TotalUCRs: u32,
    pub CommittOps: u32,
    pub DeCommitOps: u32,
    pub LockAcquires: u32,
    pub LockCollisions: u32,
    pub CommitRate: u32,
    pub DecommittRate: u32,
    pub CommitFailures: u32,
    pub InBlockCommitFailures: u32,
    pub PollIntervalCounter: u32,
    pub DecommitsSinceLastCheck: u32,
    pub HeapPollInterval: u32,
    pub AllocAndFreeOps: u32,
    pub AllocationIndicesActive: u32,
    pub InBlockDeccommits: u32,
    pub InBlockDeccomitSize: u64,
    pub HighWatermarkSize: u64,
    pub LastPolledSize: u64,
}

#[repr(C)]
pub struct HEAP_TUNING_PARAMETERS {
    pub CommittThresholdShift: u32,
    pub MaxPreCommittThreshold: u64,
}

#[repr(C)]
pub struct HEAP {
    pub u: HEAP_UNION,
    pub Flags: u32,
    pub ForceFlags: u32,
    pub CompatibilityFlags: u32,
    pub EncodeFlagMask: u32,
    pub Encoding: HEAP_ENTRY,
    pub Interceptor: u32,
    pub VirtualMemoryThreshold: u32,
    pub Signature: u32,
    pub SegmentReserve: u64,
    pub SegmentCommit: u64,
    pub DeCommitFreeBlockThreshold: u64,
    pub DeCommitTotalFreeThreshold: u64,
    pub TotalFreeSize: u64,
    pub MaximumAllocationSize: u64,
    pub ProcessHeapsListIndex: u16,
    pub HeaderValidateLength: u16,
    pub HeaderValidateCopy: PVOID,
    pub NextAvailableTagIndex: u16,
    pub MaximumTagIndex: u16,
    pub TagEntries: *mut HEAP_TAG_ENTRY,
    pub UCRList: LIST_ENTRY,
    pub AlignRound: u64,
    pub AlignMask: u64,
    pub VirtualAllocdBlocks: LIST_ENTRY,
    pub SegmentList: LIST_ENTRY,
    pub AllocatorBackTraceIndex: u16,
    pub NonDedicatedListLength: u32,
    pub BlocksIndex: PVOID,
    pub UCRIndex: PVOID,
    pub PseudoTagEntries: *mut HEAP_PSEUDO_TAG_ENTRY,
    pub FreeLists: LIST_ENTRY,
    pub LockVariable: *mut HEAP_LOCK,
    pub CommitRoutine: Option<unsafe extern "C" fn(PVOID, *mut PVOID, *mut u64) -> i32>,
    pub StackTraceInitVar: RTL_RUN_ONCE,
    pub CommitLimitData: RTLP_HEAP_COMMIT_LIMIT_DATA,
    pub UserContext: PVOID,
    pub Spare: u64,
    pub FrontEndHeap: PVOID,
    pub FrontHeapLockCount: u16,
    pub FrontEndHeapType: u8,
    pub RequestedFrontEndHeapType: u8,
    pub FrontEndHeapUsageData: *mut u16,
    pub FrontEndHeapMaximumIndex: u16,
    pub FrontEndHeapStatusBitmap: [u8; 129],
    pub ReadOnly: u8,
    pub Counters: HEAP_COUNTERS,
    pub TuningParameters: HEAP_TUNING_PARAMETERS,
}

impl core::fmt::Display for HEAP {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "HEAP @ {:p}", self)?;
        writeln!(f)?;
        
        unsafe {
            // ========== SIGNATURES (Validation) ==========
            writeln!(f, "  [Signatures]")?;
            let seg_sig = self.u.Anonymous.SegmentSignature;
            writeln!(f, "    SegmentSignature:       0x{:08X} {}", seg_sig,
                if seg_sig == 0xFFEEFFEE || seg_sig == 0xEEFFEEFF { "✓" } else { "✗" })?;
            writeln!(f, "    Signature:              0x{:08X} {}", self.Signature,
                if self.Signature == 0xEEFFEEFF { "✓" } else { "✗" })?;
            writeln!(f)?;
            
            // ========== SEGMENT INFORMATION ==========
            writeln!(f, "  [Segment]")?;
            writeln!(f, "    BaseAddress:            {:p}", self.u.Anonymous.BaseAddress)?;
            writeln!(f, "    NumberOfPages:          {} (0x{:X})", 
                self.u.Anonymous.NumberOfPages, self.u.Anonymous.NumberOfPages)?;
            writeln!(f, "    NumberOfUnCommittedPages: {}", self.u.Anonymous.NumberOfUnCommittedPages)?;
            writeln!(f, "    NumberOfUnCommittedRanges: {}", self.u.Anonymous.NumberOfUnCommittedRanges)?;
            writeln!(f, "    FirstEntry:             {:p}", self.u.Anonymous.FirstEntry)?;
            writeln!(f, "    LastValidEntry:         {:p}", self.u.Anonymous.LastValidEntry)?;
            writeln!(f)?;
            
            // ========== HEAP FLAGS (Anti-Debugging) ==========
            writeln!(f, "  [Flags]")?;
            writeln!(f, "    Flags:                  0x{:08X} {}", self.Flags,
                if self.Flags == 0x2 { "(HEAP_GROWABLE ✓)" } 
                else if self.Flags == 0x40000062 { "(DEBUGGED!)" } 
                else { "(OTHER)" })?;
            writeln!(f, "    ForceFlags:             0x{:08X} {}", self.ForceFlags,
                if self.ForceFlags == 0 { "(normal ✓)" } 
                else if self.ForceFlags == 0x40000060 { "(DEBUGGED!)" } 
                else { "(modified)" })?;
            
            // Anti-debugging detection
            let debugged = self.ForceFlags != 0 || (self.Flags & 0x2) == 0;
            writeln!(f, "    IsDebugged:             {}", if debugged { "⚠ YES" } else { "✓ NO" })?;
            writeln!(f)?;
            
            // ========== MEMORY STATISTICS ==========
            writeln!(f, "  [Memory]")?;
            writeln!(f, "    TotalFreeSize:          {} bytes", self.TotalFreeSize)?;
            writeln!(f, "    MaximumAllocationSize:  {}", self.MaximumAllocationSize)?;
            writeln!(f, "    VirtualMemoryThreshold: {}", self.VirtualMemoryThreshold)?;
            writeln!(f, "    SegmentReserve:         {}", self.SegmentReserve)?;
            writeln!(f, "    SegmentCommit:          {}", self.SegmentCommit)?;
            writeln!(f, "    DeCommitFreeBlockThreshold: {}", self.DeCommitFreeBlockThreshold)?;
            writeln!(f, "    DeCommitTotalFreeThreshold: {}", self.DeCommitTotalFreeThreshold)?;
            writeln!(f)?;
            
            // ========== FRONT END (LFH) ==========
            writeln!(f, "  [FrontEnd]")?;
            writeln!(f, "    FrontEndHeap:           {:p}", self.FrontEndHeap)?;
            writeln!(f, "    FrontEndHeapType:       0x{:02X} {}", self.FrontEndHeapType,
                if self.FrontEndHeapType == 0x2 { "(LFH ✓)" } 
                else if self.FrontEndHeapType == 0x0 { "(Standard)" } 
                else { "(Unknown)" })?;
            writeln!(f, "    FrontHeapLockCount:     {}", self.FrontHeapLockCount)?;
            writeln!(f)?;
            
            // ========== TAGS ==========
            writeln!(f, "  [Tags]")?;
            writeln!(f, "    TagEntries:             {:p}", self.TagEntries)?;
            writeln!(f, "    NextAvailableTagIndex:  {}", self.NextAvailableTagIndex)?;
            writeln!(f, "    MaximumTagIndex:        {}", self.MaximumTagIndex)?;
            writeln!(f, "    PseudoTagEntries:       {:p}", self.PseudoTagEntries)?;
            writeln!(f)?;
            
            // ========== LISTS ==========
            writeln!(f, "  [Lists]")?;
            writeln!(f, "    FreeLists:              Flink={:p}, Blink={:p}", 
                self.FreeLists.Flink, self.FreeLists.Blink)?;
            writeln!(f, "    SegmentList:            Flink={:p}, Blink={:p}", 
                self.SegmentList.Flink, self.SegmentList.Blink)?;
            writeln!(f, "    UCRList:                Flink={:p}, Blink={:p}", 
                self.UCRList.Flink, self.UCRList.Blink)?;
            writeln!(f, "    LockVariable:           {:p}", self.LockVariable)?;
            writeln!(f)?;
            
            // ========== COUNTERS (Summary) ==========
            writeln!(f, "  [Counters]")?;
            writeln!(f, "    TotalMemoryReserved:    {}", self.Counters.TotalMemoryReserved)?;
            writeln!(f, "    TotalMemoryCommitted:   {}", self.Counters.TotalMemoryCommitted)?;
            writeln!(f, "    TotalSegments:          {}", self.Counters.TotalSegments)?;
            writeln!(f, "    LockAcquires:           {}", self.Counters.LockAcquires)?;
            writeln!(f, "    LockCollisions:         {}", self.Counters.LockCollisions)?;
            writeln!(f, "    CommitFailures:         {}", self.Counters.CommitFailures)?;
            writeln!(f)?;
            
            // ========== VALIDATION SUMMARY ==========
            let valid = (seg_sig == 0xFFEEFFEE || seg_sig == 0xEEFFEEFF) && self.Signature == 0xEEFFEEFF;
            writeln!(f, "  [Status]")?;
            writeln!(f, "    Structure Valid:        {}", if valid { "✓ YES" } else { "✗ NO" })?;
            
            // Heap type detection
            if seg_sig == 0xFFEEFFEE && self.Signature == 0xEEFFEEFF {
                writeln!(f, "    Heap Type:              NT Heap (Standard)")?;
            } else if seg_sig == 0xDDEEDDEE {
                writeln!(f, "    Heap Type:              Segment Heap")?;
            } else {
                writeln!(f, "    Heap Type:              Unknown")?;
            }
        }
        
        Ok(())
    }
}

#[repr(C)]
pub struct RTL_HEAP_MEMORY_LIMIT_DATA {
    pub CommitLimit: usize,
    pub CommitLimitData: [u8; 0x20],
}