use crate::bdd::Bdd;
use crate::bdd_util::BddPointer;

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
    pub fn get_conflict_paths(&self) -> Vec<Vec<(bool, BddPointer)>> {
        let mut conflict_paths = Vec::new();
        let terminal_nodes: Vec<(bool, BddPointer)> = self.find_terminal_nodes_conflicts();

        for (pol, terminal_node) in terminal_nodes {
            let mut current_path = Vec::new();
            current_path.push((pol, terminal_node));
            self.traverse_bottom_up(terminal_node, &mut current_path, &mut conflict_paths);
        }
        conflict_paths
    }

    fn traverse_bottom_up(
        &self,
        current_ptr: BddPointer,
        current_path: &mut Vec<(bool, BddPointer)>,
        conflict_paths: &mut Vec<Vec<(bool, BddPointer)>>,
    ) {
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

    pub fn build_learned_clause(
        &self,
        conflict_paths: &Vec<Vec<(bool, BddPointer)>>,
    ) -> Vec<Vec<i32>> {
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
}

#[cfg(test)]
mod tests {
    use crate::{
        init_glucose_solver, parser, sharing::sharing_manager::SharingManager,
        variable_ordering::var_ordering::BddVarOrdering, GlucoseWrapper,
    };

    use super::*;

    #[test]
    fn test_get_conflict_paths() {
        let path: &str = "/home/user/Desktop/PhD/CDCL-support-by-BDD-methods/tests/test3.cnf";

        let expressions = parser::parse_dimacs_cnf_file(path).unwrap();
        // build the solver
        let solver = init_glucose_solver();
        let glucose = GlucoseWrapper::new(solver);
        // build the sharing manager
        let mut sharing_manager = SharingManager::new(glucose);
        // build the variable ordering
        let var_ordering = BddVarOrdering::new(expressions);

        let bdd = var_ordering.build_bdd(&mut sharing_manager);

        let mut expected_paths: Vec<Vec<(bool, BddPointer)>> = Vec::new();
        expected_paths.push(vec![
            (false, BddPointer { index: 2 }),
            (false, BddPointer { index: 3 }),
            (false, BddPointer { index: 4 }),
            (true, BddPointer { index: 7 }),
        ]);
        expected_paths.push(vec![
            (false, BddPointer { index: 5 }),
            (true, BddPointer { index: 6 }),
            (false, BddPointer { index: 7 }),
        ]);
        expected_paths.push(vec![
            (false, BddPointer { index: 6 }),
            (false, BddPointer { index: 7 }),
        ]);

        let conflict_paths: Vec<Vec<(bool, BddPointer)>> = bdd.get_conflict_paths();
        assert_eq!(conflict_paths, expected_paths)
    }

    #[test]
    fn test_build_learned_clause() {
        let path: &str = "/home/user/Desktop/PhD/CDCL-support-by-BDD-methods/tests/test1.cnf";

        let expressions = parser::parse_dimacs_cnf_file(path).unwrap();
        // build the solver
        let solver = init_glucose_solver();
        let glucose = GlucoseWrapper::new(solver);
        // build the sharing manager
        let mut sharing_manager = SharingManager::new(glucose);
        // build the variable ordering
        let var_ordering = BddVarOrdering::new(expressions);

        let bdd = var_ordering.build_bdd(&mut sharing_manager);

        let conflict_paths = bdd.get_conflict_paths();
        let learned_clauses = bdd.build_learned_clause(&conflict_paths);
        assert_eq!(learned_clauses.len(), conflict_paths.len())
    }

    #[test]
    fn test_build_learned_clause_detail() {
        let path: &str = "/home/user/Desktop/PhD/CDCL-support-by-BDD-methods/tests/test3.cnf";

        let expressions = parser::parse_dimacs_cnf_file(path).unwrap();
        // build the solver
        let solver = init_glucose_solver();
        let glucose = GlucoseWrapper::new(solver);
        // build the sharing manager
        let mut sharing_manager = SharingManager::new(glucose);
        // build the variable ordering
        let var_ordering = BddVarOrdering::new(expressions);

        let bdd = var_ordering.build_bdd(&mut sharing_manager);

        let mut learned_clauses = Vec::new();
        learned_clauses.push(vec![1, -2, -3, -4]);
        learned_clauses.push(vec![-1, 2, -3]);
        learned_clauses.push(vec![-1, -2]);

        let conflict_paths = bdd.get_conflict_paths();
        let learned_clauses_res = bdd.build_learned_clause(&conflict_paths);
        assert_eq!(learned_clauses_res.len(), conflict_paths.len());
        assert_eq!(learned_clauses, learned_clauses_res)
    }
}
