use super::bucket::Bucket;
use anyhow::Result;
use std::cmp::Reverse;
use std::collections::{HashMap, HashSet};

use crate::bdd::Bdd;
use crate::bdd_util::BddVar;
use crate::expr::bool_expr::{Clause, Expr};
use crate::parser::Dimacs;
use crate::sharing::sharing_manager::SharingManager;
use crate::variable_ordering::var_ordering_builder::BddVarOrderingBuilder;
use rayon::slice::ParallelSliceMut;

/*
    Variable Ordering: The choice of variable ordering can significantly impact the performance of BDDs.
    By carefully selecting the variable ordering based on heuristics like most-constrained variable or variable interaction,
    you can reduce the BDD's size and improve efficiency.

    Bucket Clustering: Group variables into buckets based on their interactions.
    Variables that frequently appear together in the same clauses or have strong dependencies should be placed in the same bucket.
    Then, apply bucket elimination to each bucket separately.

    Apply Bucket Elimination: In each bucket, perform variable elimination by quantifying out variables that are not essential to the final result.
    This reduces the complexity of the BDD and can lead to significant efficiency gains.

    Dynamic Reordering: Apply dynamic variable reordering periodically during BDD construction.
    After performing bucket elimination on each bucket, reevaluate the variable ordering to find an optimal arrangement that reduces the overall BDD size.

    Dynamic Reordering during Construction: Instead of waiting until the entire BDD is constructed, you can perform variable 
    reordering during the construction process. At certain intervals or based on specific triggers (e.g., node count, memory usage, conflict count), 
    you evaluate whether variable reordering might improve the efficiency of the ongoing construction.

    Reordering Heuristics: Just like before, you use heuristics to identify potential variable orders that could improve the construction process. 
    These heuristics consider factors such as node count, path length, memory usage, or computational time.

    Partial Node Reconstruction: When you decide to apply variable reordering, you don't need to reconstruct the entire BDD. 
    Instead, you adjust the order of variable selections for the remaining nodes that are yet to be constructed. 
    This involves making decisions about which variables to choose at each step of the BDD construction process based on the new variable order.

    Caching: Implement memoization to cache intermediate BDD results.
    This avoids redundant computations during BDD construction and can significantly speed up the process.

    Garbage Collection: Periodically remove unused nodes and apply garbage collection to the BDD to keep it compact and efficient.

    Advanced Variable Ordering: Consider advanced variable ordering techniques, such as Sifting and Symmetry Breaking, to fine-tune the variable arrangement and improve BDD performance.

    Reduce Clause Complexity: Before creating the BDD, preprocess the CNF clauses to simplify them. This can involve clause subsumption, clause resolution, or other techniques to reduce the overall complexity of the problem.

    Multi-Terminal BDDs: For problems with multiple output functions, consider using Multi-Terminal BDDs (MTBDDs) to share common substructures and improve efficiency.

    Parallelization: Apply parallel processing techniques to speed up BDD construction and manipulation, especially during bucket elimination and dynamic reordering steps.
*/

#[derive(Debug, Clone)]
pub struct BddVarOrdering {
    pub variables: Vec<BddVar>,
    pub expressions: Vec<Clause>,
    pub ordering: std::collections::HashMap<i32, usize>,
}
type Buckets = Vec<Bucket>;

impl BddVarOrdering {
    /// Create a new `BddVarOrdering` with the given named variables.
    pub fn new(dimacs: Dimacs) -> BddVarOrdering {
        let mut builder = BddVarOrderingBuilder::new();
        builder.make(dimacs)
    }

    // Function to update the ordering HashMap based on the new variable order
    fn update_ordering_based_on_new_variable_order(&mut self) -> Vec<BddVar> {
        // Rebuild the ordering HashMap based on the new variable order
        let mut new_ordering = HashMap::new();
        // Identify variables where the variable order has changed
        let mut affected_variables: Vec<BddVar> = Vec::new();
        
        for (new_pos, var) in self.variables.iter().enumerate() {
            // TODO handle unwrap
            let old_pos = self.ordering.get(&var.name).unwrap();
            if *old_pos != new_pos {
                affected_variables.push(*var);
            }
            new_ordering.insert(var.name, new_pos);
        }
        self.ordering = new_ordering;

        affected_variables
    }

    // Function to group clauses into buckets based on variable scores
    fn group_clauses_into_buckets_variable_scores(&self) -> Buckets {
        let mut buckets: HashMap<i32, Bucket> = HashMap::new();

        for clause in &self.expressions {
            // Find the highest-scored variable in the clause
            let highest_scored_var = clause.get_highest_scored_var(&self.ordering);

            if let Some(var) = highest_scored_var {
                // Retrieve or create the bucket associated with the highest-scored variable
                let bucket = buckets.entry(var).or_insert_with(|| Bucket {
                    clauses: Vec::new(),
                    index: var,
                });
                bucket.clauses.push(clause.clone());
            }
        }

        // Convert the HashMap values into a Vec of buckets
        let mut result_buckets: Buckets = buckets.values().cloned().collect();
        result_buckets.sort_by_key(|bucket| self.ordering.get(&bucket.index).unwrap());

        result_buckets
    }

    fn find_interacting_variables(&self) -> HashMap<i32, HashSet<i32>> {
        let mut variable_interactions: HashMap<i32, HashSet<i32>> = HashMap::new();

        for clause in &self.expressions {
            for literal in &clause.literals {
                let var = literal.get_var_name();
                let interacting_vars = variable_interactions.entry(var).or_insert(HashSet::new());

                for other_literal in clause.literals.iter().filter(|&lit| lit != literal) {
                    interacting_vars.insert(other_literal.get_var_name());
                }
            }
        }

        variable_interactions
    }

    fn create_interaction_based_variable_order(&mut self) {
        let variable_interactions: HashMap<i32, HashSet<i32>> = self.find_interacting_variables();

        let mut variable_scores: HashMap<i32, usize> = HashMap::new();

        for (var, interactions) in &variable_interactions {
            variable_scores.insert(*var, interactions.len());
        }

        // TODO probably handle unwrap here
        self.variables
            .par_sort_by_key(|var| Reverse(variable_scores.get(&var.name).unwrap()));
    }

     // Method to group clauses into buckets based on interaction-based ordering
     fn group_clauses_into_buckets_interactions(&self) -> HashMap<i32, Bucket> {
        // Calculate interaction scores for each variable
        let variable_interactions: HashMap<i32, HashSet<i32>> = self.find_interacting_variables();

        let mut variable_scores: HashMap<i32, usize> = HashMap::new();

        for (var, interactions) in &variable_interactions {
            variable_scores.insert(*var, interactions.len());
        }

        // Sort variables by interaction scores
        let mut ordered_variables: Vec<BddVar> = self.variables.clone();
        ordered_variables.par_sort_by_key(|var| Reverse(*variable_scores.get(&var.name).unwrap_or(&0)));

        // Initialize buckets
        let mut buckets: HashMap<i32, Bucket> = HashMap::new();

        // Iterate through clauses and assign them to buckets
        for clause in &self.expressions {
            let mut placed = false;

            // Iterate through ordered variables and find the first variable
            // that appears in the clause to determine the bucket
            for var in &ordered_variables {
                if clause.literals.iter().any(|expr| expr.get_var_name().eq(&var.name)) {
                    let bucket = buckets.entry(var.name).or_insert_with(Bucket{ clauses: todo!(), index: todo!() });
                    bucket.clauses.push(clause.clone());
                    placed = true;
                    break;
                }
            }

            // If the clause couldn't be placed in any bucket, add it to a default bucket
            if !placed {
                let bucket = buckets.entry(0).or_insert_with(Vec::new); // 0 represents the default bucket
                bucket.push(clause.clone());
            }
        }

        buckets
    }
    
    pub fn build(&mut self, sharing_manager: &mut SharingManager) -> Result<()> {
        // Bucket Clustering
        let buckets = self.group_clauses_into_buckets_variable_scores();
        let mut reordering = false;
      
        for bucket in &buckets {
            // TODO handle unwrap here
            //let _ = bucket.bucket_elimination();

            let mut bdd = bucket.clauses[0].to_bdd(&self.variables, &self.ordering);
            let mut n = 1;
            while n < bucket.clauses.len() {
                let temp_bdd = bucket.clauses[n].to_bdd(&self.variables, &self.ordering);
                bdd = bdd.and(&temp_bdd, &self.ordering);

                
                if bdd.nodes.len() > 20 {
                    self.create_interaction_based_variable_order();
                    let affected_vars = self.update_ordering_based_on_new_variable_order();
                    bdd.partial_reorder_bdd(&affected_vars, &self.ordering);
                    println!("reordered");
                    reordering = true;
                    break;
                }
                
                n += 1;
            }
            
            if reordering {
                break;
            } 
            let temp_learnts = bdd.build_learned_clause(&bdd.get_conflict_paths());
            // TODO handle unwrap
            sharing_manager.send_learned_clauses(temp_learnts).unwrap();
        }
        
        if reordering {
            println!("creating");
            //self.reorder_clauses_into_new_buckets(&buckets);
            let new_buckets = self.group_clauses_into_buckets_interactions();
            let mut new_clauses_len = 0;
            for (idx,bucket) in new_buckets {
                println!("{:?}", bucket.clauses.len());
                new_clauses_len += bucket.clauses.len();
            }
            assert_eq!(new_clauses_len, self.expressions.len());
            
            println!("new buckets created");
        }
        Ok(())
    }

    pub fn build_bdd(&self, sharing_manager: &mut SharingManager) -> Bdd {
        let mut bdd = self.expressions[0].to_bdd(&self.variables, &self.ordering);
        let mut n = 1;
        while n < self.expressions.len() {
            let (_, temp_bdd) = rayon::join(
                || {
                    let temp_learnts = bdd.build_learned_clause(&bdd.get_conflict_paths());
                    // TODO handle unwrap
                    sharing_manager.send_learned_clauses(temp_learnts).unwrap();
                },
                || self.expressions[n].to_bdd(&self.variables, &self.ordering),
            );
            bdd = bdd.and(&temp_bdd, &self.ordering);
            n += 1;
        }
        bdd
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        bdd_util::BddVar,
        expr::bool_expr::{Clause, Expr},
        init_glucose_solver,
        parser::{self, Dimacs},
        sharing::sharing_manager::SharingManager,
        variable_ordering::var_ordering::BddVarOrdering,
        GlucoseWrapper,
    };
    use std::{
        collections::{HashMap, HashSet},
        time::Instant,
    };

    fn create_sample_bdd_var_ordering() -> BddVarOrdering {
        // Create a sample BddVarOrdering for testing
        // Replace with your initialization logic
        BddVarOrdering {
            variables: vec![
                BddVar { name: 1 },
                BddVar { name: 2 },
                BddVar { name: 3 },
                BddVar { name: 4 },
                BddVar { name: 5 },
                // ... more variables
            ],
            expressions: vec![
                Clause {
                    literals: HashSet::from_iter(vec![
                        Expr::Var(1),
                        Expr::Not(Box::new(Expr::Var(2))),
                    ]),
                },
                Clause {
                    literals: HashSet::from_iter(vec![Expr::Var(1), Expr::Var(4)]),
                },
                Clause {
                    literals: HashSet::from_iter(vec![
                        Expr::Not(Box::new(Expr::Var(2))),
                        Expr::Var(4),
                        Expr::Var(5),
                    ]),
                },
                Clause {
                    literals: HashSet::from_iter(vec![
                        Expr::Not(Box::new(Expr::Var(1))),
                        Expr::Var(2),
                    ]),
                },
                Clause {
                    literals: HashSet::from_iter(vec![
                        Expr::Var(2),
                        Expr::Not(Box::new(Expr::Var(3))),
                    ]),
                },
                // ... more expressions
            ],
            ordering: HashMap::from_iter(vec![(1, 0), (2, 1), (3, 2), (4, 3), (5, 4)]),
        }
    }

    #[test]
    fn test_create_interaction_based_ordering() {
        let mut bdd_var_ordering = create_sample_bdd_var_ordering();
        bdd_var_ordering.create_interaction_based_variable_order();
        bdd_var_ordering.update_ordering_based_on_new_variable_order();
        assert_eq!(*bdd_var_ordering.ordering.get(&2).unwrap(), 0 as usize);
        assert_eq!(*bdd_var_ordering.ordering.get(&4).unwrap(), 1 as usize);
        assert_eq!(*bdd_var_ordering.ordering.get(&3).unwrap(), 4 as usize);
    }

    #[test]
    fn test_find_interacting_variables() {
        let clauses = vec![
            Clause {
                literals: HashSet::from_iter(vec![Expr::Var(1), Expr::Var(2)]),
            },
            Clause {
                literals: HashSet::from_iter(vec![Expr::Var(2), Expr::Var(3)]),
            },
            Clause {
                literals: HashSet::from_iter(vec![Expr::Var(1), Expr::Var(3)]),
            },
        ];
        let dimacs = Dimacs {
            nb_v: 3,
            nb_c: 3,
            var_map: HashMap::new(),
            vars_scores: HashMap::new(),
            expressions: clauses,
        };
        let var_ordering = BddVarOrdering::new(dimacs);
        let interactions = var_ordering.find_interacting_variables();

        assert_eq!(interactions.len(), 3);
        assert_eq!(interactions.get(&1), Some(&HashSet::from_iter(vec![2, 3])));
        assert_eq!(interactions.get(&2), Some(&HashSet::from_iter(vec![1, 3])));
        assert_eq!(interactions.get(&3), Some(&HashSet::from_iter(vec![2, 1])));
    }

    #[test]
    fn test_group_clauses_into_buckets_interactions() {
        // Example clauses
        let clause1 = Clause {
            literals: vec![Expr::Var(1), Expr::Var(2)].into_iter().collect(),
        };
        let clause2 = Clause {
            literals: vec![Expr::Var(2), Expr::Var(3)].into_iter().collect(),
        };
        let clause3 = Clause {
            literals: vec![Expr::Var(4), Expr::Var(5)].into_iter().collect(),
        };

        let expressions = vec![clause1.clone(), clause2.clone(), clause3.clone()];

        let dimacs = Dimacs {
            nb_v: 3,
            nb_c: 3,
            var_map: HashMap::new(),
            vars_scores: HashMap::new(),
            expressions,
        };
        let var_ordering = BddVarOrdering::new(dimacs);
        // Call the function
        let buckets = var_ordering.group_clauses_into_buckets_interactions();

        // Test assertions
        assert_eq!(buckets.len(), 2);
        assert_eq!(buckets[0].clauses.len(), 2);
        assert_eq!(buckets[1].clauses.len(), 1);
    }

    #[test]
    pub fn test_bench() {
        let path: &str = "/home/user/Desktop/PhD/CDCL-support-by-BDD-methods/benchmarks/tests/0b1041a1e55af6f3d2c63462a7400bd2-fermat-907547022132073.cnf";

        let start = Instant::now();
        // create the Dimacs instance
        let expressions = parser::parse_dimacs_cnf_file(path).unwrap();
        println!(
            "Time elapsed to parse the CNF formula : {:?}",
            start.elapsed()
        );

        let start = Instant::now();
        // build the solver
        let solver = init_glucose_solver();
        let glucose = GlucoseWrapper::new(solver);
        // build the sharing manager
        let mut sharing_manager = SharingManager::new(glucose);
        // build the variable ordering
        let mut var_ordering = BddVarOrdering::new(expressions);
        println!(
            "Time elapsed to create the variable ordering : {:?}",
            start.elapsed()
        );

        var_ordering.build(&mut sharing_manager);
    }
}
