use super::bucket::Bucket;
use super::var_ordering_builder::Dimacs;
use crate::bdd_util::BddVar;
use crate::expr::bool_expr::Expr;
use crate::variable_ordering::var_ordering_builder::BddVarOrderingBuilder;
//use itertools::Itertools;

#[derive(Debug, Clone)]
pub struct BddVarOrdering {
    pub variables: Vec<BddVar>,
    pub formula: Vec<Vec<Expr>>,
    pub ordering: std::collections::HashMap<i32, usize>,
    pub buckets: std::collections::HashMap<usize, Bucket>,
}

impl BddVarOrdering {
    /// Create a new `BddVarOrdering` with the given named variables.
    pub fn new(dimacs: Dimacs) -> BddVarOrdering {
        let mut builder = BddVarOrderingBuilder::new();
        builder.make(dimacs)
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
    use std::time::Instant;

    use crate::{expr::bool_expr::Expr, variable_ordering::var_ordering::BddVarOrdering};

    #[test]
    pub fn bucket_elimination_bench() {
        let path: &str = "/home/user/Desktop/PhD/CDCL-support-by-BDD-methods/benchmarks/tests/4dfe7816c2c198f8fd0b328d1e9672c9-crafted_n10_d6_c3_num18.cnf";

        let start = Instant::now();
        // create the Dimacs instance
        let expressions = Expr::parse_dimacs_cnf_file(path).unwrap();
        println!(
            "Time elapsed to parse the CNF formula : {:?}",
            start.elapsed()
        );

        let start = Instant::now();
        // build the variable ordering
        let var_ordering = BddVarOrdering::new(expressions);
        println!(
            "Time elapsed to create the variable ordering : {:?}",
            start.elapsed()
        );

        let start = Instant::now();
        //let potential_learnt_clauses = var_ordering.directional_resolution();

        //println!("{}", unit_clauses.len());
        //println!("{}", var_ordering.formula.len());
        // /println!("{}", potential_learnt_clauses.len());

        println!(
            "Time elapsed for directional resolution : {:?}",
            start.elapsed()
        );

        let start = Instant::now();
        // as directional resolution returns many unit clauses do
        // unit propagation
        //preprocessing::unit_propagation(&mut var_ordering.formula, unit_clauses);
        //println!("{}", var_ordering.formula.len());
        println!("Time elapsed for unit propagation : {:?}", start.elapsed());

        //let start = Instant::now();
        //let bdd = Bdd::build(potential_learnt_clauses, &var_ordering.variables,
        //    &var_ordering.ordering, 0);
        //println!("{:?}", bdd);
        //println!("Time elapsed for building the Bdd : {:?}", start.elapsed());
    }
}
