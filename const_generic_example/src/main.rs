use std::collections::HashMap;

#[derive(Debug)]
enum Role<const ACCESS_LEVEL: u8> {
    Guest,
    User,
    Admin,
}

impl<const ACCESS_LEVEL: u8> Role<ACCESS_LEVEL> {
    fn can_access(&self, required_level: u8) -> bool {
        ACCESS_LEVEL >= required_level
    }

    fn describe(&self) {
        println!("{:?} (Access Level: {})", self, ACCESS_LEVEL);
    }
}

fn main() {
    let guest = Role::<1>::Guest;
    let user = Role::<3>::User;
    let admin = Role::<5>::Admin;

    guest.describe();
    user.describe();
    admin.describe();

    println!("Can Guest access level 2? {}", guest.can_access(2));
    println!("Can Admin access level 2? {}", admin.can_access(2));
}
