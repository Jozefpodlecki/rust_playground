use core::ptr::null_mut;

use log::{error, info};
use win_platform::{
    KUserSharedData, NtError, syscalls::{NtAlertThreadByThreadId, NtTerminateProcess, NtTerminateThread, NtWaitForAlertByThreadId}, types::{NtCurrentProcess, NtCurrentThread},
};

use crate::{
    state::REGISTRY,
    types::{DebugEvent, DebugEventKind, DebugVerdict, DebugVerdictKind},
};

pub fn wait_for_verdict(tid: u32) -> Result<(), NtError> {
    let addr = REGISTRY.ptr(tid).ok_or(NtError::STATUS_NOT_FOUND)?;

    NtWaitForAlertByThreadId(addr as _, null_mut()).ok()?;

    let verdict = REGISTRY.verdict(tid).ok_or(NtError::STATUS_NOT_FOUND)?;
    info!("Received verdict {verdict:?}");

    match verdict.kind {
        DebugVerdictKind::Continue => (),
        DebugVerdictKind::TerminateProcess { exit_code } => {
            info!("Terminating process 0x{exit_code:X}");
            NtTerminateProcess(NtCurrentProcess, exit_code);
        }
        DebugVerdictKind::TerminateThread { exit_code } => {
            info!("Terminating thread 0x{exit_code:X}");
            NtTerminateThread(NtCurrentThread, exit_code);
        }
        DebugVerdictKind::Return => {
            info!("Returning");
        }
        _ => {}
    }

    Ok(())
}

pub fn apply_verdict(verdict: &DebugVerdict) -> Result<(), NtError> {
    let tid = verdict.tid;

    {
        let mut guard = REGISTRY.0.write();
        let entry = unsafe { guard.get_mut(&tid).unwrap_unchecked() };
        entry.updated_on = KUserSharedData::system_time();
        entry.verdict = *verdict;
    }

    NtAlertThreadByThreadId(tid as _).ok()?;


    Ok(())
}