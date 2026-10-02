use win_platform::{STATUS_ABANDONED, STATUS_SUCCESS, STATUS_TIMEOUT, rng::Rng};
use winapi::shared::ntstatus::{DBG_TERMINATE_PROCESS, STATUS_CANCELLED, STATUS_CONTROL_C_EXIT};
use crate::types::{DebugEvent, DebugEventKind, DebugVerdict, DebugVerdictKind};

pub fn verdict(event: DebugEvent, rng: &mut Rng) -> DebugVerdict {
    let kind = match event.kind {
        DebugEventKind::ThreadCreated => match rng.random_range(0..20) {
            0 => DebugVerdictKind::TerminateThread { exit_code: exit_code(rng) },
            _ => DebugVerdictKind::Continue,
        },
        DebugEventKind::Breakpoint => match rng.random_range(0..50) {
            0 => DebugVerdictKind::TerminateThread { exit_code: exit_code(rng) },
            1..=2 => DebugVerdictKind::Return,
            _ => DebugVerdictKind::Continue,
        },
        DebugEventKind::ProcessExited => DebugVerdictKind::TerminateProcess {
            exit_code: exit_code(rng),
        },
        DebugEventKind::ThreadExited => DebugVerdictKind::TerminateThread {
            exit_code: exit_code(rng),
        },
    };
    DebugVerdict { tid: event.tid, kind }
}

pub fn exit_code(rng: &mut Rng) -> i32 {
    match rng.random_range(0..100) {
        0..=69 => STATUS_SUCCESS,
        70..=84 => STATUS_ABANDONED,
        85..=92 => STATUS_CANCELLED,
        93..=96 => STATUS_CONTROL_C_EXIT,
        97..=98 => DBG_TERMINATE_PROCESS,
        _ => STATUS_TIMEOUT,
    }
}