use std::collections::HashMap;

#[derive(Debug)]
struct Payload {
    id: u32
}

trait DataProvider {
    type Data<'a> where Self: 'a;

    fn fetch<'a>(&'a self) -> Self::Data<'a>;
}

struct OwnedProvider {
    data: Vec<Payload>,
}

impl DataProvider for OwnedProvider {
    type Data<'a> = &'a [Payload];

    fn fetch<'a>(&'a self) -> Self::Data<'a> {
        &self.data
    }
}

fn main() {
    let provider = OwnedProvider {
        data: vec![Payload { id: 1 }, Payload { id: 2 }],
    };

    let fetched = provider.fetch();
    println!("{:?}", fetched);
}
