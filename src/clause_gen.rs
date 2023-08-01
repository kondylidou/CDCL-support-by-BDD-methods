use crate::bdd::Bdd;
use crate::bdd_util::BddPointer;
use crate::parallel::clause_database::ClauseDatabase;
use crate::GlucoseWrapper;

impl Bdd {
    /// During the top-down construction of a BDD for a SAT instance, infeasibility
    /// of a state is detected when an unsatisfied clause contains no variable corresponding
    /// to a lower layer of the BDD.
    pub fn send_learned_clauses(
        &self,
        on_going: bool,
        _clause_database: &mut ClauseDatabase,
        _solver_wrapper: GlucoseWrapper,
    ) {
        let zero = BddPointer::new_zero();

        // Search the Bdd backwards starting from the zero pointer
        for ptr in self.indices() {
            if ptr.is_terminal() {
                // skip the terminal nodes
                continue;
            }
            if self.low_node_ptr(ptr) == zero {
                // create a new learned clause for every path starting from the zero pointer
                let mut learned_clause: Vec<i32> = Vec::new();
                let var = self.var_of_ptr(ptr).name;
                learned_clause.push(var);
                // Generate the path after connecting the zero pointer.
                let mut path = Vec::new();
                path.push(ptr);

                // try sending the clause if it's valid
                if let Some(valid_learned_clause) =
                    self.build_learned_clause(learned_clause, path, on_going)
                {
                    // do the actual sharing
                    // clause_database.send(valid_learned_clause, solver_wrapper, stats);
                }
            }
            if self.high_node_ptr(ptr) == zero {
                let mut learned_clause: Vec<i32> = Vec::new();
                let var = self.var_of_ptr(ptr).name;
                learned_clause.push(-var);
                let mut path = Vec::new();
                path.push(ptr);

                if let Some(valid_learned_clause) =
                    self.build_learned_clause(learned_clause, path, on_going)
                {
                    // do the actual sharing
                    //clause_database.send(valid_learned_clause, solver_wrapper, stats);
                }
            }
        }
    }

    /// A BDD is used to capture the relationship between Boolean variables of (a part of) the SAT problem,
    /// in the form of a characteristic function. In such a BDD, each path to a “0” (false) node denotes a conflict.
    /// A learned clause corresponding to this conflict is easily obtained by negating the literals that define the path.
    /// Since a BDD captures all paths to 0, i.e. all possible conflicts, the potential advantage is that multiple learned
    /// clauses can be generated and added to the SAT solver at the same time.
    pub fn build_learned_clause(
        &self,
        mut learned_clause: Vec<i32>,
        mut path: Vec<BddPointer>,
        on_going: bool,
    ) -> Option<Vec<i32>> {
        // The acc is the first pointer in the path in the beginnings
        let mut acc = *path.get(0).unwrap();
        for ptr in self.indices().into_iter().skip(acc.to_index()) {
            if ptr.is_terminal() {
                // skip the terminal nodes
                continue;
            }
            // During the top-down construction of a BDD for a SAT instance, infeasibility
            // of a state is detected when an unsatisfied clause contains no variable corresponding
            // to a lower layer of the BDD. When this occurs, we choose one such
            // clause as a witness of the infeasibility of the corresponding node.
            if self.low_node_ptr(ptr) == acc {
                let var = self.var_of_ptr(ptr).name;
                learned_clause.push(var);
                acc = ptr;
                path.push(ptr);
            }
            if self.high_node_ptr(ptr) == acc {
                let var = self.var_of_ptr(ptr).name;
                learned_clause.push(-var);
                // save the new pointer in the accumulator
                acc = ptr;
                path.push(ptr);
            } else {
                if on_going {
                    // We need the root pointer as we will check if unsatisfied clause contains
                    // no variable corresponding to a lower layer of the BDD.
                    // That will happen if the root pointer is equal to the current pointer
                    // after checking if the current pointer equals the accumulator.
                    // If this is the case we can not use this as a witness clause.
                    if ptr.eq(&self.root_pointer()) && ptr.eq(&acc) {
                        return None;
                    }
                }
            }
        }
        Some(learned_clause)
    }
}
