use super::bucket::Bucket;
use std::cmp::Reverse;
use std::collections::{HashMap, HashSet};

use crate::bdd::Bdd;
use crate::bdd_util::BddVar;
use crate::expr::bool_expr::Clause;
use crate::parser::Dimacs;
use crate::sharing::clause_database::ClauseDatabase;
use crate::variable_ordering::var_ordering_builder::BddVarOrderingBuilder;
use rayon::slice::ParallelSliceMut;

#[repr(C)]
pub struct BddVarOrdering {
    pub variables: Vec<BddVar>,
    pub expressions: Vec<Clause>,
    pub ordering: HashMap<i32, usize>,
}

impl BddVarOrdering {
    /// Create a new `BddVarOrdering` with the given named variables.
    pub fn new(dimacs: Dimacs) -> BddVarOrdering {
        let mut builder = BddVarOrderingBuilder::new();
        builder.make(dimacs)
    }

    // Function to update the ordering HashMap based on the new variable order
    fn update_ordering(&mut self) -> Vec<BddVar> {
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
    pub fn group_clauses_into_buckets(&mut self) -> Vec<Bucket> {
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
        let mut result_buckets: Vec<Bucket> = buckets.values().cloned().collect();
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

    fn create_interaction_based_ordering(&mut self) -> Vec<BddVar> {
        let variable_interactions: HashMap<i32, HashSet<i32>> = self.find_interacting_variables();

        let mut variable_scores: HashMap<i32, usize> = HashMap::new();

        for (var, interactions) in &variable_interactions {
            variable_scores.insert(*var, interactions.len());
        }

        // TODO probably handle unwrap here
        self.variables
            .par_sort_by_key(|var| Reverse(variable_scores.get(&var.name).unwrap()));

        self.update_ordering()
    }

     // Method to group clauses into buckets based on interaction-based ordering
     fn group_clauses_into_buckets_interactions(&mut self, expressions: &Vec<Clause>) -> Vec<Bucket> {
        let variable_interactions = self.find_interacting_variables();
        let mut buckets: Vec<Bucket> = Vec::new();

        for clause in expressions {
            let mut placed = false;

            for bucket in buckets.iter_mut() {
                if clause.literals.iter().any(|expr| {
                    variable_interactions[&expr.get_var_name()]
                        .is_subset(&bucket.vars())
                }) {
                    bucket.clauses.push(clause.clone());
                    placed = true;
                    break;
                }
            }
            if !placed {
                buckets.push(Bucket {
                    clauses: vec![clause.clone()],
                    index: clause.get_highest_scored_var(&self.ordering).unwrap(),
                });
            }
        }

        buckets
    }
    
    pub fn build(&mut self, buckets: &mut Vec<Bucket>, clause_database: &mut ClauseDatabase, filtered_clauses: &mut Vec<Vec<i32>>) {
        let threshold = 30;
        let mut reordered = false;

        while let Some(bucket) = buckets.first() {
            // build the initial bdd from the first clause in the bucket
            let mut bdd = bucket.clauses[0].to_bdd(&self.variables, &self.ordering);
            
            let mut n = 1;
            while n < bucket.clauses.len() {
                //println!("{:?}", bucket.clone());
                // build the final bdd from the rest of the buckets clauses by applying 
                // the and operation between the temp bdds. 
                let temp_bdd = bucket.clauses[n].to_bdd(&self.variables, &self.ordering);
                bdd = bdd.and(&temp_bdd, &self.ordering);

                // If the BDD is becoming too big, it means that we have reached the large buckets.
                // At this point, we will reorder the variables based on interactions and
                // subdivide the bucket into smaller buckets.
                if bdd.size() > threshold {
                    if !reordered {
                        // if the variable ordering hasn't been updated yet, we update it
                        // based on interactions and partially roerdered the already partially
                        // constructed bdd based on the new variable ordering. We only rearrange 
                        // the bdd nodes based on the affected variables.
                        let affected_vars = self.create_interaction_based_ordering();
                        bdd.partial_reorder_bdd(&affected_vars, &self.ordering);
                        reordered = true;
                    }
                    // we will also split the large bucket into smaller ones and add them to the buckets set.
                    let new_buckets = self.group_clauses_into_buckets_interactions(&bucket.clauses);
                    buckets.extend(new_buckets);
                    break;
                }
                n += 1;
            }
            // get the temp learnt clauses from the bucket
            let temp_learnts = bdd.build_learned_clause(&bdd.get_conflict_paths());
            filtered_clauses.extend(clause_database.get_filtered_clauses(temp_learnts));
            println!("{:?}", filtered_clauses.len());
            buckets.remove(0);
        }
    }    

    pub fn build_bdd(&self) -> Bdd {
        let mut bdd = self.expressions[0].to_bdd(&self.variables, &self.ordering);
        let mut n = 1;
        while n < self.expressions.len() {
            let (_, temp_bdd) = rayon::join(
                || {
                    let temp_learnts = bdd.build_learned_clause(&bdd.get_conflict_paths());
                    // TODO handle unwrap
                    //sharing_manager.send_learned_clauses(temp_learnts).unwrap();
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
        parser::{self, Dimacs},
        variable_ordering::var_ordering::BddVarOrdering, sharing::clause_database::ClauseDatabase,
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
            vars_scores: HashMap::from_iter(vec![(1,0.0), (3, 0.3), (2, 0.5), (4, 0.5), (5, 0.7)]),
            expressions,
        };
        let mut var_ordering = BddVarOrdering::new(dimacs);
        // Call the function
        let buckets = var_ordering.group_clauses_into_buckets_interactions(&var_ordering.expressions.clone());

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
       
        // build the variable ordering
        let mut var_ordering = BddVarOrdering::new(expressions);
        println!(
            "Time elapsed to create the variable ordering : {:?}",
            start.elapsed()
        );

        // Clause Database
        let mut clause_database = ClauseDatabase::new();
        // Bucket Clustering
        let buckets = var_ordering.group_clauses_into_buckets();
        for bucket in buckets.clone() {
            var_ordering.build(&bucket, &mut clause_database);
        }
    }
}
