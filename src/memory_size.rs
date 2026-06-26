#[derive(Clone)]
pub enum Memory {
    MegaByte(u32),
    #[allow(unused)]
    GigaByte(u32),
}
