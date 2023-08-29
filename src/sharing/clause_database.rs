// Overall, there are three reasons why a clause offered by a
// core solver can get discarded. One is that it was duplicate
// or wrongly considered to be duplicate due to the probabilistic
// nature of Bloom filters. Second is that another core solver was adding
// its clause to the data structure for global export at the same time.
// The last reason is that it did not fit into the fixed size message
// sent to the other MPI processes. Although important learned clauses
// might get lost, we believe that this relaxed approach is still beneficial
// since it allows a simpler and more efficient implementation of clause sharing.

use bloom_filters::{BloomFilter, ClassicBloomFilter, DefaultBuildHashKernels};
use rand::random;
use std::collections::hash_map::RandomState;

pub struct ClauseDatabase {
    global_filter: ClassicBloomFilter<DefaultBuildHashKernels<RandomState>>,
    local_filter: ClassicBloomFilter<DefaultBuildHashKernels<RandomState>>,
}

impl ClauseDatabase {
    pub fn new() -> ClauseDatabase {
        ClauseDatabase {
            global_filter: ClassicBloomFilter::new(
                100,
                0.03,
                DefaultBuildHashKernels::new(random(), RandomState::new()),
            ),
            local_filter: ClassicBloomFilter::new(
                100,
                0.03,
                DefaultBuildHashKernels::new(random(), RandomState::new()),
            ),
        }
    }

    pub fn insert_to_local_filter(&mut self, clause: &Vec<i32>) {
        clause.iter().for_each(|i| self.local_filter.insert(i));
    }

    pub fn insert_to_global_filter(&mut self, clause: &Vec<i32>) {
        clause.iter().for_each(|i| self.global_filter.insert(i));
    }

    pub fn local_filter_contains(&self, clause: &Vec<i32>) -> bool {
        clause.iter().all(|i| self.local_filter.contains(i))
    }

    pub fn global_filter_contains(&self, clause: &Vec<i32>) -> bool {
        clause.iter().all(|i| self.global_filter.contains(i))
    }

    pub fn reset_global_filter(&mut self) {
        self.global_filter.reset();
    }

    pub fn reset_local_filter(&mut self) {
        self.local_filter.reset();
    }
}
