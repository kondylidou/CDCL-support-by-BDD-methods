use crate::bdd_util::BddVar;
use crate::expr::bool_expr::Expr;
use crate::variable_ordering::var_ordering::BddVarOrdering;
use std::cmp::Ordering;
use std::cmp::Ordering::*;

use super::bucket::Bucket;

#[derive(Clone, Debug)]
pub struct Dimacs {
    pub nb_v: i32,
    pub nb_c: i32,
    pub var_map: std::collections::HashMap<i32, Expr>,
    pub vars_scores: std::collections::HashMap<i32, f64>,
    pub expressions: Vec<Vec<Expr>>,
}

/// For our implementation, we use a simple heuristic to determine the variable ordering:
/// each variable is assigned a score, computed as the quotient between the number of clauses
/// containing the variable and the average arity of those clauses.
pub fn calculate_scores(var_clause_arities: std::collections::HashMap<i32, Vec<usize>>) -> std::collections::HashMap<i32, f64> {
    let mut vars_scores = std::collections::HashMap::new();
    for (var, clause_arities) in var_clause_arities {
        // the number of clauses where the variable appears
        let clauses_num = clause_arities.len() as f64;
        // the average arity of those clauses is computed by dividing
        // the sum of the arities with the total number of clauses
        let sum: usize = clause_arities.iter().sum();
        let aver_arity = sum as f64 / clauses_num;
        // the score is computed as the quotient between the number of clauses
        // containing the variable and the average arity of those clauses
        let score = clauses_num / aver_arity;
        vars_scores.insert(var, score);
    }
    vars_scores
}


#[derive(Clone, Debug)]
pub struct BddVarOrderingBuilder {
    var_names: Vec<i32>,
    var_names_set: std::collections::HashSet<i32>,
}

impl BddVarOrderingBuilder {
    /// Create a new builder without any variables.
    pub fn new() -> BddVarOrderingBuilder {
        BddVarOrderingBuilder {
            var_names: Vec::new(),
            var_names_set: std::collections::HashSet::new(),
        }
    }

    /// Create a new variable with the given `name`. Returns a `BddVar`+
    /// instance that can be later used to create and query actual BDDs.
    ///
    /// *Panics*:
    ///  - Each variable name has to be unique.
    pub fn make_variable(&mut self, name: i32, score: f64) -> BddVar {
        if self.var_names_set.contains(&name) {
            panic!("BDD variable {} already exists.", name);
        }
        self.var_names_set.insert(name);
        self.var_names.push(name);
        BddVar { name, score }
    }

    /// Similar to `make_variable`, but allows creating multiple variables at the same time.
    pub fn make_variables(&mut self, var_map: std::collections::HashMap<i32, Expr>, vars_scores: &std::collections::HashMap<i32, f64>) -> Vec<BddVar> {
        let mut variables = Vec::new();
        for (var_name, _) in var_map {
            // TODO handle unwrap here
            variables.push(self.make_variable(var_name, *vars_scores.get(&var_name).unwrap()));
        }
        variables
    }

    /// Convert this builder to an actual variable ordering.
    /// The variables are sorted in decreasing order according to the score,
    /// so that higher-scoring variables
    /// (that is, variables that appear in many mostly short clauses)
    /// correspond to layers nearer the top of the BDD.
    pub fn make(&mut self, mut dimacs: Dimacs) -> BddVarOrdering {
        let variables = self.make_variables(dimacs.var_map, &dimacs.vars_scores);

        let mut ordering: std::collections::HashMap<i32, usize> = std::collections::HashMap::new();
        //let mut buckets: std::collections::HashMap<usize, Bucket> =
        //    std::collections::HashMap::new();

        let mut v: Vec<_> = dimacs.vars_scores.iter().collect();
        // v is a sorted vector in decreasing order according to the scores
        v.sort_by(|x, y| BddVarOrderingBuilder::var_dec_cmp(&x.1, &y.1));

        let mut idx = v.len();
        ordering.insert(i32::MAX, idx);
        for (var, _) in v.into_iter().rev() {
            idx -= 1;
            ordering.insert(*var, idx);
            // process the buckets in the reverse order of the variable ordering
            //buckets.insert(idx, Bucket::create_bucket(*var, &mut dimacs.expressions));
        }

        BddVarOrdering {
            variables,
            formula: dimacs.expressions,
            ordering,
            buckets,
        }
    }

    fn var_dec_cmp(x: &f64, y: &f64) -> Ordering {
        if x.eq(&y) {
            Equal
        } else if x < y {
            Greater
        } else {
            Less
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use crate::{expr::bool_expr::Expr, variable_ordering::{var_ordering::BddVarOrdering, var_ordering_builder::BddVarOrderingBuilder}};


    #[test]
    fn variable_scores() {
        let dimacs = Expr::parse_dimacs_cnf_file("tests/test3.cnf").unwrap();
        let vars_scores = dimacs.vars_scores;
        // score for 1:
        // number of clauses containing the var: 6
        // average arity of those clauses: (5+2+2+2+2+2) / 6 = 2,5
        // score = 6/2.5 = 2,4
        // score for 3:
        // number of clauses containing the var: 6
        // average arity of those clauses: (2+4+2+5+3+4) / 6 = 3,3
        // score = 6/3.3 = 1,81

        assert_eq!(*vars_scores.get(&1).unwrap(), 2.4 as f64);
        assert!(vars_scores.get(&1).unwrap() > vars_scores.get(&5).unwrap());
    }

    #[test]
    fn variable_ordering() {
        let dimacs = Expr::parse_dimacs_cnf_file("tests/test3.cnf").unwrap();
        let var_ordering = BddVarOrdering::new(dimacs);

        let mut var_index_mapping: std::collections::HashMap<i32, usize> =
            std::collections::HashMap::new();
        var_index_mapping.insert(1, 0);
        var_index_mapping.insert(2, 1);
        var_index_mapping.insert(3, 2);
        var_index_mapping.insert(4, 3);
        var_index_mapping.insert(5, 4);
        var_index_mapping.insert(i32::MAX, 5);
 
        assert_eq!(var_index_mapping, var_ordering.ordering);
    }

    #[test]
    fn buckets_ordering() {
        let dimacs = Expr::parse_dimacs_cnf_file("tests/test3.cnf").unwrap();
        let var_ordering = BddVarOrdering::new(dimacs);
        let buckets = var_ordering.buckets;
        assert_eq!(buckets.len() + 1, var_ordering.ordering.len());

        for (var, idx) in var_ordering.ordering {
            if var != i32::MAX {
                let bucket = buckets.get(&idx).unwrap();
                assert_eq!(bucket.get_index(), var);
            }
        }
    }

}
