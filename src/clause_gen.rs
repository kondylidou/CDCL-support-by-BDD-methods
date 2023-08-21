use std::collections::HashMap;

use crate::bdd::Bdd;
use crate::bdd_util::BddPointer;
use crate::parallel::clause_database::ClauseDatabase;
use crate::GlucoseWrapper;

// Detecting a Conflict: When your SAT solver encounters a contradiction during the search process (
// e.g., a clause evaluates to false), you need to detect the conflict. This conflict represents the point 
// where your solver's current partial assignment of variables is inconsistent.

// Backtrack to the Root of the BDD:
// Starting from the conflicted variable, you backtrack through the BDD nodes to the root. 
//This backtracking process helps you identify the path in the BDD that led to the conflict.

// Generating the Learned Clause:
// As you backtrack through the BDD nodes, you collect the variables corresponding to the path that led to the conflict. 
// These variables are the literals of the learned clause. Depending on the direction you traverse the BDD (low or high), 
// you negate or include the variable in the clause.

// Add the Learned Clause to the Solver:
// Once you have generated the learned clause, you add it to your SAT solver's clause database. 
// This learned clause provides the solver with additional information to avoid the same conflict in subsequent search steps.

// Backtrack to Resolve the Conflict:
// After adding the learned clause, you need to backtrack to a suitable decision level where the learned 
// clause can be applied to resolve the conflict. This typically involves undoing variable assignments and possibly 
// propagating the learned clause to prune conflicting branches in the search tree.

// Resume Solving:
// With the new information from the learned clause and the backtracking process, 
// your SAT solver can resume the search for a solution. The solver might use various heuristics to decide which 
// variable to assign next based on the learned clause and other information.


impl Bdd {

    fn get_conflict_paths(&self) -> Vec<Vec<(bool, BddPointer)>> {
        let mut conflict_paths = Vec::new();
        let terminal_nodes: Vec<(bool, BddPointer)> = self.find_terminal_nodes_conflicts();
    
        for (pol, terminal_node) in terminal_nodes {
            let mut current_path = Vec::new();
            current_path.push((pol, terminal_node));
            self.traverse_bottom_up(terminal_node, &mut current_path, &mut conflict_paths);
        }
        conflict_paths
    }

    
    fn traverse_bottom_up(&self, current_ptr: BddPointer, current_path: &mut Vec<(bool, BddPointer)>, conflict_paths: &mut Vec<Vec<(bool, BddPointer)>>) {
        for ptr in self.indices().into_iter().skip(current_ptr.to_index()) {
            // Traverse the low child
            if self.low_node_ptr(ptr) == current_ptr {
                current_path.push((false, ptr));
                self.traverse_bottom_up(ptr, current_path, conflict_paths);
            }
            // Traverse the high child
            if self.high_node_ptr(ptr) == current_ptr {
                current_path.push((true, ptr));
                self.traverse_bottom_up(ptr, current_path, conflict_paths);
            } 
            if current_ptr.eq(&self.root_pointer()) {
                // Push the current path to conflict_paths when reaching the root
                conflict_paths.push(current_path.clone());
                return;
            }
        }
    }

    pub fn build_learned_clause(&self, conflict_paths: &Vec<Vec<(bool, BddPointer)>>) -> Vec<Vec<i32>> {
        let mut learned_clauses = Vec::new();

        for conflict_path in conflict_paths {
            let mut learned_clause = Vec::new();
            
            for &(pol, ptr) in conflict_path.iter().rev() {
                let var = self.var_of_ptr(ptr);
    
                if pol {
                    learned_clause.push(var.name);
                } else {
                    learned_clause.push(-var.name);
                }
            }
            learned_clauses.push(learned_clause);
        }

        learned_clauses
    }

    /* 

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
    pub fn build_learned_clause(&self, mut learned_clause: Vec<i32>, mut path: Vec<BddPointer>, on_going: bool) -> Option<Vec<i32>> {
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
    */
}

#[cfg(test)]
mod tests {
    use crate::{expr::bool_expr::Expr, variable_ordering::var_ordering::BddVarOrdering};

    use super::*;

    #[test]
    fn test_get_conflict_paths() {
        let path: &str = "/home/user/Desktop/PhD/CDCL-support-by-BDD-methods/tests/test3.cnf";

        let expressions = Expr::parse_dimacs_cnf_file(path).unwrap();
        
        // build the variable ordering
        let var_ordering = BddVarOrdering::new(expressions);

        let bdd = var_ordering.build_bdd();
        
        let mut expected_paths: Vec<Vec<(bool, BddPointer)>> = Vec::new();
        expected_paths.push(vec![(false, BddPointer { index: 2 }), (false, BddPointer { index: 3 }), 
        (false, BddPointer { index: 4 }), (true, BddPointer { index: 7 })]);
        expected_paths.push(vec![(false, BddPointer { index: 5 }), (true, BddPointer { index: 6 }), (false, BddPointer { index: 7 })]);
        expected_paths.push(vec![(false, BddPointer { index: 6 }), (false, BddPointer { index: 7 })]);

        let conflict_paths: Vec<Vec<(bool,BddPointer)>> = bdd.get_conflict_paths();
        assert_eq!(conflict_paths, expected_paths)
    }


    #[test]
    fn test_build_learned_clause() {
        let path: &str = "/home/user/Desktop/PhD/CDCL-support-by-BDD-methods/tests/test1.cnf";

        let expressions = Expr::parse_dimacs_cnf_file(path).unwrap();
        
        // build the variable ordering
        let var_ordering = BddVarOrdering::new(expressions);

        let bdd = var_ordering.build_bdd();

        let conflict_paths = bdd.get_conflict_paths();
        let learned_clauses = bdd.build_learned_clause(&conflict_paths);
        assert_eq!(learned_clauses.len(), conflict_paths.len())
    }

    #[test]
    fn test_build_learned_clause_detail() {
        let path: &str = "/home/user/Desktop/PhD/CDCL-support-by-BDD-methods/tests/test3.cnf";

        let expressions = Expr::parse_dimacs_cnf_file(path).unwrap();
        
        // build the variable ordering
        let var_ordering = BddVarOrdering::new(expressions);

        let bdd = var_ordering.build_bdd();

        let mut learned_clauses = Vec::new();
        learned_clauses.push(vec![1, -2,-3,-4]);
        learned_clauses.push(vec![-1,2,-3]);
        learned_clauses.push(vec![-1,-2]);

        let conflict_paths = bdd.get_conflict_paths();
        let learned_clauses_res = bdd.build_learned_clause(&conflict_paths);
        assert_eq!(learned_clauses_res.len(), conflict_paths.len());
        assert_eq!(learned_clauses, learned_clauses_res)
    }
}

