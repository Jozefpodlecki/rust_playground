use abi_stable::{std_types::{ROption, RString}, StableAbi};

#[non_exhaustive]
#[repr(u32)]
#[derive(StableAbi, Debug, Clone, PartialEq)]
#[sabi(kind(WithNonExhaustive(
    size = [usize;32],
    traits(Send, Sync, Debug, Clone, PartialEq),
    assert_nonexhaustive = Command,
)))]
pub enum Command {
    Insert {
        id: u64,
        name: RString,
        value: RString,
    },
    Update {
        id: u64,
        name: ROption<RString>
    },
    Delete {
        id: u64,
    }
}