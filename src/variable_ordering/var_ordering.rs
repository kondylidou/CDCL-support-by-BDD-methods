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

    // Function to group clauses into buckets based on variable scores
    fn group_clauses_into_buckets_variable_scores(&self) -> Buckets {
        let mut buckets: HashMap<i32, Bucket> = HashMap::new();

        for clause in &self.expressions {
            // Find the highest-scored variable in the clause
            let highest_scored_var = clause
                .literals
                .iter()
                .map(|lit| lit.get_var_name())
                .max_by_key(|var| self.ordering.get(var).unwrap());

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

    fn reorder_based_on_interactions(&mut self, processed_clauses: Vec<usize>) -> Vec<Bucket> {
        // Retain elements that are not at positions to remove
        for idx in processed_clauses {
            self.expressions.remove(idx);
        }
        println!("removed");
        self.create_interaction_based_ordering();
        println!("ordering");
        self.group_clauses_into_buckets_interactions()
    }

    fn group_clauses_into_buckets_interactions(&self) -> Vec<Bucket> {
        let variable_interactions = self.find_interacting_variables();
        let mut variable_buckets: HashMap<usize, HashSet<i32>> = HashMap::new();

        let mut buckets: Vec<Bucket> = Vec::new();

        for clause in &self.expressions {
            let mut placed = false;

            for (bucket_idx, bucket) in buckets.iter_mut().enumerate() {
                if clause.literals.iter().any(|expr| {
                    variable_interactions[&expr.get_var_name()]
                        .is_subset(&variable_buckets[&bucket_idx])
                }) {
                    bucket.clauses.push(clause.clone());
                    placed = true;
                    break;
                }
            }

            if !placed {
                buckets.push(Bucket {
                    clauses: vec![clause.clone()],
                    index: (buckets.len()) as i32,
                });
                variable_buckets.insert(
                    buckets.len() - 1,
                    clause
                        .literals
                        .iter()
                        .map(|expr| expr.get_var_name())
                        .collect(),
                );
            }
        }

        buckets
    }

    fn create_interaction_based_ordering(&mut self) -> HashMap<i32, HashSet<i32>> {
        let variable_interactions = self.find_interacting_variables();

        let mut variable_scores: HashMap<i32, usize> = HashMap::new();

        for (var, interactions) in &variable_interactions {
            variable_scores.insert(*var, interactions.len());
        }

        // TODO probably handle unwrap here
        self.variables
            .par_sort_by_key(|var| Reverse(variable_scores.get(&var.name).unwrap()));

        let mut new_ordering = HashMap::new();
        for (index, var) in self.variables.iter().enumerate() {
            new_ordering.insert(var.name, index);
        }
        self.ordering = new_ordering;

        variable_interactions
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

    // Method to group clauses into buckets based on implications
    fn group_clauses_into_buckets_implications(&self) -> Buckets {
        // Construct a hashmap to track the implications of each variable
        let mut implications: HashMap<i32, HashSet<i32>> = HashMap::new();

        let mut n = 0;
        // Populate the implications hashmap based on the clauses
        for clause in &self.expressions {
            for literal in &clause.literals {
                if let Expr::Not(inner) = literal {
                    if let Expr::Var(var) = &**inner {
                        implications
                            .entry(*var)
                            .or_insert_with(HashSet::new)
                            .extend(clause.literals.iter().filter_map(|lit| {
                                if let Expr::Var(v) = lit {
                                    Some(*v)
                                } else {
                                    None
                                }
                            }));
                    }
                } else if let Expr::Var(var) = literal {
                    implications
                        .entry(*var)
                        .or_insert_with(HashSet::new)
                        .extend(clause.literals.iter().filter_map(|lit| {
                            if let Expr::Not(inner) = lit {
                                if let Expr::Var(v) = &**inner {
                                    Some(*v)
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        }));
                }
            }
        }

        // Group clauses into buckets based on implications
        let mut buckets: Buckets = Vec::new();
        for clause in &self.expressions {
            let mut placed = false;

            for bucket in &mut buckets {
                if clause.literals.iter().any(|expr| {
                    bucket.clauses.iter().any(|c| {
                        c.contains_expr(expr) || c.contains_expr(&Expr::Not(Box::new(expr.clone())))
                    })
                }) {
                    bucket.clauses.push(clause.clone());
                    placed = true;
                    break;
                }
            }

            if !placed {
                buckets.push(Bucket {
                    clauses: vec![clause.clone()],
                    index: n,
                });
                n += 1;
            }
        }
        buckets
    }

    // Function to reorder variables based on frequency
    fn reorder_variables_by_frequency(&mut self) {
        // Create a HashMap to store variable frequencies
        let mut variable_frequencies: HashMap<i32, usize> = HashMap::new();

        // Calculate variable frequencies based on expressions (clauses)
        for clause in &self.expressions {
            for literal in &clause.literals {
                if let Expr::Var(var) = literal {
                    *variable_frequencies.entry(*var).or_insert(0) += 1;
                }
            }
        }

        // Sort variables by frequency in descending order
        self.variables
            .sort_by_key(|var| variable_frequencies.get(&var.name).cloned().unwrap_or(0));

        // Update the ordering HashMap based on the new variable order
        self.update_ordering_based_on_new_variable_order();
    }

    // Function to update the ordering HashMap based on the new variable order
    fn update_ordering_based_on_new_variable_order(&mut self) {
        // Rebuild the ordering HashMap based on the new variable order
        self.ordering.clear();
        for (index, variable) in self.variables.iter().enumerate() {
            self.ordering.insert(variable.name, index);
        }
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

    pub fn build(&mut self, sharing_manager: &mut SharingManager) -> Result<()> {
        let mut clauses_processed = Vec::new();
        let mut clauses_num_processed = 0;
        // Bucket Clustering
        let buckets = self.group_clauses_into_buckets_variable_scores();
        let variable_scores_ordering = true;
        // TODO find the right order
        for mut bucket in buckets {
            // the initial ordering doesn't help at this point so we go and reorder
            /*
            if bucket.clauses.len() > 50 {
                assert_eq!(clauses_num_processed, clauses_processed.len());
                println!("PASSED!");
                self.reorder_based_on_interactions(clauses_processed.clone());
                println!("NEW BUCKETS CREATED!");
            }*/
            let positions: Vec<usize> = bucket
                .clauses
                .iter()
                .filter_map(|clause| self.expressions.iter().position(|c| c.eq(clause)))
                .collect();

            clauses_processed.extend(positions);
            clauses_num_processed += bucket.clauses.len();
            // Bucket Elimination
            // TODO handle unwrap
            if variable_scores_ordering {
                let _ = bucket.bucket_elimination();
            }
            // After performing bucket elimination on each bucket,
            // reevaluate the variable ordering to find an optimal
            // arrangement that reduces the overall BDD size.
            // TODO

            let mut bdd = bucket.clauses[0].to_bdd(&self.variables, &self.ordering);

            let mut n = 1;
            while n < bucket.clauses.len() {
                let (_, temp_bdd) = rayon::join(
                    || {
                        let temp_learnts = bdd.build_learned_clause(&bdd.get_conflict_paths());
                        // TODO handle unwrap
                        sharing_manager.send_learned_clauses(temp_learnts).unwrap();
                    },
                    ||
                    //  TODO Dynamic Reordering
                    bucket.clauses[n].to_bdd(&self.variables, &self.ordering),
                );

                bdd = bdd.and(&temp_bdd, &self.ordering);
                n += 1;
            }
        }
        Ok(())
    }

    /*
    /// This method represents the method of directional resolution
    /// or bucket elimination for CNF formulas.
    /// The buckets are processed in the reversed order of the
    /// variable ordering and the resoved clauses are then stored
    /// in the lower buckets depending on their highest variable.
    pub fn directional_resolution(&mut self) -> Vec<Expr> {
        // We need a vector to store potential strong learnt clauses
        let mut potential_learnt_clauses: Vec<Vec<Expr>> = Vec::new();
        //let mut unit_clauses: Vec<Expr> = Vec::new();
        // We need to process buckets in the reverse order of the variable ordering
        let mut idx = self.buckets.len();
        while idx > 0 {
            idx -= 1;
            // unwrap was not handled here as there is no possibility this will give back None
            let current_bucket = self.buckets.get_mut(&idx).unwrap();

            // bucket contains a unit clause, perform only unit resolution.
            if current_bucket.pos_occ.len().eq(&1) {
                potential_learnt_clauses.push(current_bucket.pos_occ[0].clone());
            }
            if current_bucket.neg_occ.len().eq(&1) {
                potential_learnt_clauses.push(current_bucket.neg_occ[0].clone());
            }
            match current_bucket.process_bucket(&self.variables) {
                Ok(current_clauses) => {
                    for expr in current_clauses {
                        /*
                        if idx <= self.buckets.len() / 2 && expr.is_unit() {
                            unit_clauses.push(expr.clone());
                        }
                        */

                        if idx.eq(&0) {
                            potential_learnt_clauses.push(expr.clone());
                        }

                        let (pol, order) = expr.find_highest_order(&self.ordering);
                        // we insert the resolvents in the appropriate lower buckets
                        if let Some(lower_bucket) = self.buckets.get_mut(&order) {
                            if pol {
                                lower_bucket.pos_occ.push(expr);
                            } else {
                                lower_bucket.neg_occ.push(expr);
                            }
                        }
                    }
                }
                // if the empty clause is generated the theory is not satisfiable
                // WARNING: as we process a sample and not the whole bucket we cannot
                // know for sure that if this returns an error then the whole
                // formula is unsatisfiable. So we will comment this and hold
                // it for future implementation.
                Err(_) => {
                    //return Err("The formula is unsatisfiable");
                }
            }
        }


        potential_learnt_clauses = potential_learnt_clauses
            .into_iter()
            .unique()
            .collect::<Vec<Expr>>();
        /*
        unit_clauses = unit_clauses.into_iter().unique().collect::<Vec<Expr>>();
        clauses_to_delete = clauses_to_delete
            .into_iter()
            .unique()
            .collect::<Vec<Expr>>();
        self.formula
            .drain_filter(|e| clauses_to_delete.contains(&e));
        */
        potential_learnt_clauses
    }*/
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
        bdd_var_ordering.create_interaction_based_ordering();
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
    fn test_group_clauses_into_buckets_implications() {
        let clauses = vec![
            Clause {
                literals: vec![Expr::Var(1), Expr::Var(2)].into_iter().collect(),
            },
            Clause {
                literals: vec![
                    Expr::Not(Box::new(Expr::Var(2))),
                    Expr::Var(3),
                    Expr::Not(Box::new(Expr::Var(4))),
                ]
                .into_iter()
                .collect(),
            },
            Clause {
                literals: vec![Expr::Var(4), Expr::Var(5)].into_iter().collect(),
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
        let buckets = var_ordering.group_clauses_into_buckets_implications();

        assert_eq!(buckets.len(), 2)
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
