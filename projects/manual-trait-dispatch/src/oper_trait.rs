use toolkit::{U8CStackString, rand::Rng};

#[repr(C)]
#[derive(Debug)]
pub enum CustomResult<T, E> {
    Ok(T),
    Err(E),
}

#[repr(C)]
pub struct OperationArgs<'a> {
    pub rng: Rng,
    pub value: &'a mut [u8],
    pub name: U8CStackString::<10>
}

#[repr(C)]
#[derive(Debug)]
pub enum OperationResult {
    Ignore,
    Generate(OperationData),
    Mutate
}

#[repr(C)]
#[derive(Debug)]
pub struct OperationData {
    pub value: [u8; 10]
}

pub trait Operation {
    fn execute(&self, args: &mut OperationArgs) -> CustomResult<OperationResult, u32>;
}

impl Operation for fn(&mut OperationArgs) -> CustomResult<OperationResult, u32> {
    #[inline(never)]
    fn execute(&self, args: &mut OperationArgs) -> CustomResult<OperationResult, u32> {
  
        self(args)
    }
}