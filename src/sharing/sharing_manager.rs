// Overall, there are three reasons why a clause offered by a
// core solver can get discarded. One is that it was duplicate
// or wrongly considered to be duplicate due to the probabilistic
// nature of Bloom filters. Second is that another core solver was adding
// its clause to the data structure for global export at the same time.
// The last reason is that it did not fit into the fixed size message
// sent to the other MPI processes. Although important learned clauses
// might get lost, we believe that this relaxed approach is still beneficial
// since it allows a simpler and more efficient implementation of clause sharing.

use super::clause_database::ClauseDatabase;
use crate::{add_incoming_clause_to_clauses_vec, GlucoseWrapper};
use anyhow::{anyhow, Result};

pub struct SharingManager {
    glucose: GlucoseWrapper,
    database: ClauseDatabase,
}

impl SharingManager {
    pub fn new(glucose: GlucoseWrapper) -> Self {
        let database = ClauseDatabase::new();
        SharingManager { glucose, database }
    }

    pub fn filter_clause(&mut self, clause: Vec<i32>) -> Result<Vec<i32>> {
        if !self.database.global_filter_contains(&clause) {
            self.database.insert_to_global_filter(&clause);

            if !self.database.local_filter_contains(&clause) {
                self.database.insert_to_local_filter(&clause);
                Ok(clause)
            } else {
                Err(anyhow!("Clause didn't pass the local filter: {:?}", clause))
            }
        } else {
            Err(anyhow!(
                "Clause didn't pass the global filter: {:?}",
                clause
            ))
        }
    }

    pub fn reset_local_filter(&mut self) {
        self.database.reset_local_filter();
    }

    pub fn reset_global_filter(&mut self) {
        self.database.reset_global_filter();
    }

    // TODO handle errors based on glucose
    pub fn send_learned_clauses(&mut self, learned_clauses: Vec<Vec<i32>>) -> Result<()> {
        self.reset_local_filter();
        let solver = self.glucose.solver;

        for clause in learned_clauses {
            if let Ok(filtered_clause) = self.filter_clause(clause) {
                println!("clause sent {:?}", filtered_clause);
                add_incoming_clause_to_clauses_vec(solver, filtered_clause);
            }
        }

        Ok(())
    }
}
