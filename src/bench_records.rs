use crate::common::Record;

#[derive(Clone, Copy)]
pub struct LigthRecord {
    pub value: (usize, u128),
}

impl Record<usize> for LigthRecord {
    fn key(&self) -> usize {
        println!("{:?}", self.value);
        self.value.0.clone()
    }
}
