use ntapi::ntioapi::IO_STATUS_BLOCK;
use win_platform::{NtError, NtStatus};

use crate::event::Event;


pub fn default_pipe_name() -> heapless::String<30> {
    let pid = unsafe { ntapi::ntpsapi::NtCurrentProcessId() } as u32;
    heapless::format!(30; r"\??\pipe\{pid}").unwrap()
}

pub fn finish(
    initial: NtStatus,
    event: &Event,
    status_block: &IO_STATUS_BLOCK,
) -> Result<NtStatus, NtError> {
    if !initial.is_pending() {
        return Ok(initial);
    }
    event.wait()?;
    Ok(NtStatus::from_raw(unsafe { status_block.u.Status }))
}
