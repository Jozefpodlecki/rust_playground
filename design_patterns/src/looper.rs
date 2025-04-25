
pub trait Modifier {
    fn modify<'a>(&self, state: &mut State<'a>);
}

pub trait Modifier1<'a> {
    fn modify(&self, state: &mut State<'a>);
}


pub struct Service;

impl Modifier for Service {
    fn modify<'a>(&self, state: &mut State<'a>) {
        state.value = "a";
    }
}

pub struct State<'a> {
    value: &'a str
}

pub fn looper(data: &mut State) {

    let service = Service;

    loop {
        service.modify(data);
        break;
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        
        let mut data = State { value: "" };
        looper(&mut data);
    }
}