#[derive(Clone)]
pub enum Memory {
    MegaByte(i32),
    #[allow(unused)]
    GigaByte(i32),
}
