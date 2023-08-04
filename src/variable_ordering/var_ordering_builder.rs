use crate::bdd_util::BddVar;
use crate::expr::bool_expr::Expr;
use crate::parser::parse::Dimacs;
use crate::variable_ordering::var_ordering::BddVarOrdering;
use std::cmp::Ordering;
use std::cmp::Ordering::*;

use super::bucket::Bucket;

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
    pub fn make_variables(
        &mut self,
        names: Vec<i32>,
        vars_scores: &std::collections::HashMap<i32, f64>,
    ) -> Vec<BddVar> {
        // TODO handle unwrap here
        names
            .iter()
            .map(|name| self.make_variable(*name, *vars_scores.get(name).unwrap()))
            .collect()
    }

    /// Convert this builder to an actual variable ordering.
    /// The variables are sorted in decreasing order according to the score,
    /// so that higher-scoring variables
    /// (that is, variables that appear in many mostly short clauses)
    /// correspond to layers nearer the top of the BDD.
    pub fn make(&mut self, mut dimacs: Dimacs) -> BddVarOrdering {
        let variables = self.make_variables(dimacs.vars, &dimacs.vars_scores);
        let formula = Expr::parse_clauses(&mut dimacs.clauses, &variables);
        let mut expressions = formula.clone();

        let mut ordering: std::collections::HashMap<i32, usize> = std::collections::HashMap::new();
        let mut buckets: std::collections::HashMap<usize, Bucket> =
            std::collections::HashMap::new();

        let mut v: Vec<_> = dimacs.vars_scores.iter().collect();
        // v is a sorted vector in decreasing order according to the scores
        v.sort_by(|x, y| BddVarOrderingBuilder::var_dec_cmp(&x.1, &y.1));

        let mut idx = v.len();
        ordering.insert(i32::MAX, idx);
        for (var, _) in v.into_iter().rev() {
            idx -= 1;
            ordering.insert(*var, idx);
            // process the buckets in the reverse order of the variable ordering
            buckets.insert(idx, Bucket::create_bucket(*var, &mut expressions));
        }

        BddVarOrdering {
            variables,
            formula,
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
    use std::collections::HashMap;

    use super::*;
    use crate::{expr::bool_expr::Expr, parser::parse::parse_dimacs, preprocessing};

    #[test]
    fn variable_scores() {
        let dimacs = parse_dimacs("tests/test3.cnf");
        let vars_scores = dimacs.vars_scores;
        // score for 1:
        // number of clauses containing the var: 6
        // average arity of those clauses: (5+2+2+2+2+2) / 5 = 2,5
        // score = 6/2.5 = 2,4

        assert_eq!(*vars_scores.get(&1).unwrap(), 2.4 as f64);
        assert!(vars_scores.get(&1).unwrap() > vars_scores.get(&5).unwrap());
    }

    #[test]
    fn variable_ordering() {
        let dimacs = parse_dimacs("tests/test3.cnf");
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
        let dimacs = parse_dimacs("tests/test3.cnf");
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

    #[test]
    pub fn test_unit_propagation1() {
        let mut clauses = Vec::new();
        let mut clause1 = vec![-1, 2, 3, 4];
        let clause2 = vec![3, 6, 1];
        let clause3 = vec![7, -1, -9];
        let clause4 = vec![1];

        clauses.push(clause1.clone());
        clauses.push(clause2.clone());
        clauses.push(clause3.clone());
        clauses.push(clause4.clone());

        let mut builder = BddVarOrderingBuilder::new();
        let mut scores = HashMap::new();
        let mut i: f64 = 0.0;

        for clause in &clauses {
            for var in clause {
                scores.insert(*var, i);
                i += 1.1;
            }
        }
        clause1.extend(clause2.iter());
        clause1.extend(clause3.iter());
        clause1.extend(clause4.iter());
        clause1.sort();
        clause1.dedup();

        let vars = builder.make_variables(clause1, &scores);

        let mut clauses_set = Expr::parse_clauses(&mut clauses, &vars);

        preprocessing::unit_propagation(&mut clauses_set, vec![Expr::Var(1)]);

        let cla1 = Expr::Or(
            Box::new(Expr::Var(2)),
            Box::new(Expr::Or(Box::new(Expr::Var(3)), Box::new(Expr::Var(4)))),
        );
        let cla3 = Expr::Or(
            Box::new(Expr::Var(7)),
            Box::new(Expr::Not(Box::new(Expr::Var(9)))),
        );
        let cla4 = Expr::Var(1);

        let mut res = Vec::new();
        res.push(cla1);
        res.push(cla3);
        res.push(cla4);

        assert_eq!(clauses_set, res);
    }

    #[test]
    pub fn test_unit_propagation2() {
        let mut clauses = Vec::new();
        let mut clause1 = vec![1, 2, 3, 4];
        let clause2 = vec![3, 6, -1];
        let clause3 = vec![7, 1, -9];
        let clause4 = vec![-1];

        clauses.push(clause1.clone());
        clauses.push(clause2.clone());
        clauses.push(clause3.clone());
        clauses.push(clause4.clone());

        let mut builder = BddVarOrderingBuilder::new();
        let mut scores = HashMap::new();
        let mut i: f64 = 0.0;

        for clause in &clauses {
            for var in clause {
                scores.insert(*var, i);
                i += 1.1;
            }
        }
        clause1.extend(clause2.iter());
        clause1.extend(clause3.iter());
        clause1.extend(clause4.iter());
        clause1.sort();
        clause1.dedup();

        let vars = builder.make_variables(clause1, &scores);

        let mut clauses_set = Expr::parse_clauses(&mut clauses, &vars);

        preprocessing::unit_propagation(&mut clauses_set, vec![Expr::Not(Box::new(Expr::Var(1)))]);

        let cla1 = Expr::Or(
            Box::new(Expr::Var(2)),
            Box::new(Expr::Or(Box::new(Expr::Var(3)), Box::new(Expr::Var(4)))),
        );
        let cla3 = Expr::Or(
            Box::new(Expr::Var(7)),
            Box::new(Expr::Not(Box::new(Expr::Var(9)))),
        );
        let cla4 = Expr::Not(Box::new(Expr::Var(1)));

        let mut res = Vec::new();
        res.push(cla1);
        res.push(cla3);
        res.push(cla4);

        assert_eq!(clauses_set, res);
    }
}
