pub struct Cursor<'a> {
    pos: usize,
    data: &'a [u8]
}

impl<'a> Cursor<'a> {
    pub fn new(data: &'a [u8]) -> Cursor<'a> {
        Cursor { pos: 0, data }
    }

    pub fn data(&self) -> &'a [u8] {
        if self.pos >= self.data.len() {
            &[]
        } else {
            &(self.data)[self.pos..]
        }
    }

    pub fn remaining(&self) -> usize {
        if self.pos >= self.data.len() {
            0
        } else {
            self.data.len() - self.pos
        }
    }

    pub fn read(&mut self, len: usize) -> &'a [u8] {
        let res = &(self.data)[self.pos..self.pos + len];
        self.pos += len;
        res
    }

    pub fn advance(&mut self, len: usize) {
        self.pos += len;
    }
}