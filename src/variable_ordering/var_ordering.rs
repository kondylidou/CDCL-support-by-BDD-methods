use std::collections::{HashMap, HashSet};
use super::bucket::Bucket;
use anyhow::Result;
use super::var_ordering_builder::Dimacs;
use crate::bdd::Bdd;
use crate::bdd_util::BddVar;
use crate::expr::bool_expr::{Clause, Expr};
use crate::sharing::sharing_manager::SharingManager;
use crate::variable_ordering::var_ordering_builder::BddVarOrderingBuilder;

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
                let bucket = buckets.entry(var).or_insert_with(|| Bucket { clauses: Vec::new(), index: var });
                bucket.clauses.push(clause.clone());
            }
        }

        // Convert the HashMap values into a Vec of buckets
        let mut result_buckets: Buckets = buckets.values().cloned().collect();
        result_buckets.sort_by_key(|bucket| self.ordering.get(&bucket.index).unwrap());

        result_buckets
    }

    // Function to group clauses into buckets based on interacting variables
    fn group_clauses_into_buckets_interactions(&self) -> Buckets {
        let interactions: HashMap<i32, Vec<i32>> = self.find_interacting_variables();
        let mut buckets: Buckets = Vec::new();

        let mut n = 0;
        for clause in &self.expressions {
            let mut placed = false;

            for bucket in &mut buckets {
                if clause.literals.iter().any(|expr| {
                    interactions[&expr.get_var_name()].iter().any(|var| {
                        bucket
                            .clauses
                            .iter()
                            .any(|clause| clause.clause_contains_var(*var))
                    })
                }) {
                    bucket.clauses.push(clause.clone());
                    placed = true;
                    break;
                }
            }

            if !placed {
                buckets.push(Bucket{clauses: vec![clause.clone()], index: n });
                n+=1;
            }
        }

        buckets
    }

    // Function to find frequently interacting variables based on clause structure
    fn find_interacting_variables(&self) -> HashMap<i32, Vec<i32>> {
        let mut variable_interactions: HashMap<i32, Vec<i32>> = HashMap::new();

        for clause in &self.expressions {
            for literal in &clause.literals {
                let var = literal.get_var_name();
                let interacting_vars = variable_interactions.entry(var).or_insert(Vec::new());

                // Iterate through the rest of the literals in the clause
                for other_literal in clause.literals.iter().filter(|&lit| lit != literal) {
                    let other_var = other_literal.get_var_name();
                    if !interacting_vars.contains(&other_var) {
                        interacting_vars.push(other_var);
                    }
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
                buckets.push(Bucket{ clauses: vec![clause.clone()], index: n });
                n+=1;
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
        self.variables.sort_by_key(|var| {
            variable_frequencies.get(&var.name).cloned().unwrap_or(0)
        });

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
                    || self.expressions[n].to_bdd(&self.variables, &self.ordering)
                );
                bdd = bdd.and(&temp_bdd, &self.ordering);
                n += 1;
            }
            bdd
    }

    pub fn build(&self, sharing_manager: &mut SharingManager) -> Result<()> {
        // Bucket Clustering
        let buckets = self.group_clauses_into_buckets_variable_scores();
        // TODO find the right order 
        for mut bucket in buckets {
            // Bucket Elimination
            let _ = bucket.bucket_elimination();
            println!("----------------------------------------------{:?}", bucket.clauses.len());
            // After performing bucket elimination on each bucket, 
            // reevaluate the variable ordering to find an optimal 
            // arrangement that reduces the overall BDD size.
            // TODO 

            let mut bdd = bucket.clauses[0].to_bdd(&self.variables, &self.ordering);
            let mut n = 1;
            while n < bucket.clauses.len() {
                rayon::join(
                    || {
                        let temp_learnts = bdd.build_learned_clause(&bdd.get_conflict_paths());
                        // TODO handle unwrap
                        println!("===========================================================");
                        println!("{:?}",temp_learnts.len());
                        sharing_manager.send_learned_clauses(temp_learnts).unwrap();
                    },
                    || 
                    //  TODO Dynamic Reordering
                    {}
                    ,
                );
                let temp_bdd = self.expressions[n].to_bdd(&self.variables, &self.ordering);
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
    use std::{time::Instant, collections::{HashSet, HashMap}};
    use crate::{expr::bool_expr::{Expr, Clause}, variable_ordering::{var_ordering::BddVarOrdering, var_ordering_builder::Dimacs}, sharing::sharing_manager::{self, SharingManager}, GlucoseWrapper, init_glucose_solver};

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
        let dimacs = Dimacs { nb_v: 3, nb_c: 3, var_map: HashMap::new(), vars_scores: HashMap::new(), expressions: clauses };
        let var_ordering = BddVarOrdering::new(dimacs);
        let interactions = var_ordering.find_interacting_variables();

        assert_eq!(interactions.len(), 3);
        assert_eq!(interactions.get(&1), Some(&vec![2, 3]));
        assert_eq!(interactions.get(&2), Some(&vec![1, 3]));
        assert_eq!(interactions.get(&3), Some(&vec![2, 1]));
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

        let dimacs = Dimacs { nb_v: 3, nb_c: 3, var_map: HashMap::new(), vars_scores: HashMap::new(), expressions };
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

        let dimacs = Dimacs { nb_v: 3, nb_c: 3, var_map: HashMap::new(), vars_scores: HashMap::new(), expressions: clauses };
        let var_ordering = BddVarOrdering::new(dimacs);
        let buckets = var_ordering.group_clauses_into_buckets_implications();

        assert_eq!(buckets.len(), 2)
    }
    
    #[test]
    pub fn bucket_elimination_bench() {
        let path: &str = "/home/user/Desktop/PhD/CDCL-support-by-BDD-methods/benchmarks/tests/70a8711118d4eaf15674ebc71bfb7c35-sted1_0x0_n438-636.cnf";

        let start = Instant::now();
        // create the Dimacs instance
        let expressions = Expr::parse_dimacs_cnf_file(path).unwrap();
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
        let var_ordering = BddVarOrdering::new(expressions);
        println!(
            "Time elapsed to create the variable ordering : {:?}",
            start.elapsed()
        );

        var_ordering.build(&mut sharing_manager);

        //let start = Instant::now();

        //let first = var_ordering::get_key_by_value(&var_ordering.ordering, &0);
        //let bucket = Bucket::create_bucket(first.unwrap(), &mut var_ordering.formula);
        //println!("First bucket clauses num {:?}", bucket.neg_occ.len() + bucket.pos_occ.len());
        //println!(
        //    "Time elapsed for to create first bucket : {:?}",
        //    start.elapsed()
        //);
    }
}
