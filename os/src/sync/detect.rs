use alloc::vec;
use alloc::vec::Vec;

#[derive(Default)]
/// detect deadlock
pub struct DeadlockDetector {
    resource_kinds: usize,
    thread_num: usize,
    available: Vec<usize>,
    /// allocation[i][j] means how many resource j are allocated to thread i
    allocation: Vec<Vec<usize>>,
    need: Vec<Vec<usize>>, 
}

impl DeadlockDetector {
    /// ensure detector has at least resource_kinds and thread_num
    fn resize(&mut self, resoure_kinds: usize, thread_num: usize) {
        if resoure_kinds > self.resource_kinds {
            self.available.resize(resoure_kinds, 0);
            self.allocation
                .iter_mut()
                .for_each(|al| al.resize(resoure_kinds, 0));
            self.need
                .iter_mut()
                .for_each(|al| al.resize(resoure_kinds, 0));
            self.resource_kinds = resoure_kinds;
        }
        while self.allocation.len() < thread_num {
            self.allocation.push(vec![0; resoure_kinds]);
            self.need.push(vec![0; resoure_kinds]);
            self.thread_num += 1;
        }
    }

    /// update available resource
    pub fn update_available(&mut self, resource_kind: usize, count: usize) {
        self.resize(resource_kind + 1, self.thread_num);
        self.available[resource_kind] = count;
    }

    /// allocate resource
    pub fn allocate(&mut self, thread: usize, resource: usize, count: usize) {
        self.resize(resource + 1, thread + 1);
        assert!(self.available[resource] >= count);
        self.available[resource] -= count;
        self.allocation[thread][resource] += count;
        assert!(self.need[thread][resource] >= count);
        self.need[thread][resource] -= count;
    }

    /// release resource
    pub fn deallocate(&mut self, thread: usize, resource: usize, count: usize) {
        self.resize(resource + 1, thread + 1);
        assert!(self.allocation[thread][resource] >= count);
        self.available[resource] += count;
        self.allocation[thread][resource] -= count;
        
    }

    /// update need and check safety.
    pub fn try_allocate(&mut self, thread: usize, resource: usize, count: usize) -> bool {
        assert!(resource < self.resource_kinds);
        self.resize(self.resource_kinds, thread + 1);
        self.need[thread][resource] += count;
        let mut work = self.available.clone();
        let mut finish = vec![false; self.thread_num];
        loop {
            let mut step = false;
            for i in 0..self.thread_num {
                if finish[i] {
                    continue;
                }
                if self.need[i].iter().enumerate().all(|(id, num)| *num <= work[id]) {
                    finish[i] = true;
                    self.allocation[i].iter().enumerate().for_each(|(id, num)| work[id] += *num);
                    step = true;
                }
            }
            if !step {
                break;
            }
        }
        finish.iter().all(|f| *f == true)
    }
}
