pub struct IDManager {
    next_id: u16,
    used_ids: Vec<u16>,
}

impl IDManager {
    pub fn new(size: usize) -> IDManager {
        IDManager {
            next_id: 1,
            used_ids: Vec::with_capacity(size),
        }
    }

    pub fn take_id(&mut self) -> u16 {
        if self.used_ids.len() == self.used_ids.capacity() {
            panic!("All ids are in use");
        }
        while self.used_ids.contains(&self.next_id) {
            self.next_id = self.next_id % self.used_ids.capacity() as u16 + 1
        }
        self.used_ids.push(self.next_id);
        self.next_id
    }

    pub fn free_id(&mut self, id: u16) {
        if let Some(index) = self.used_ids.iter().position(|used_id| *used_id == id) {
            self.used_ids.remove(index);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::managers::idmanager::IDManager;
    #[test]
    fn test_take_when_available() {
        let mut sut = IDManager::new(4);
        let next_id = sut.take_id();
        assert_eq!(next_id, 1_u16);
    }

    #[test]
    #[should_panic]
    fn test_take_panics_when_all_ids_in_use() {
        let mgr_size = 3;
        let mut sut = IDManager::new(mgr_size);
        for _ in 0..mgr_size + 1 {
            let _ = sut.take_id();
        }
    }

    #[test]
    fn test_free_frees_already_used_id() {
        let mgr_size = 3;
        let mut sut = IDManager::new(mgr_size);
        // take all ids
        let id_to_be_freed = sut.take_id();
        let _ = sut.take_id();
        let _ = sut.take_id();
        // free the one that was stored in id_to_be_freed
        sut.free_id(id_to_be_freed);
        let reused_id = sut.take_id();
        // check if the freed value can be reused
        assert_eq!(reused_id, id_to_be_freed);
    }
}
