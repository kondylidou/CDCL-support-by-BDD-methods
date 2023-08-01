use crate::{bdd_util::BddVar, expr::bool_expr::Expr};
use itertools::Itertools;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Bucket {
    pub index: i32,
    pub pos_occ: Vec<Expr>,
    pub neg_occ: Vec<Expr>,
}

impl Bucket {
    pub fn get_index(&self) -> i32 {
        self.index
    }

    /// This method takes the vector of clauses and puts each clause in the generated bucket
    /// corresponding to its variable which comes first in the total ordering.
    /// The variable is given and represents the index of the bucket.
    pub fn create_bucket(index: i32, expressions: &mut Vec<Expr>) -> Bucket {
        let mut pos_occ = Vec::new();
        let mut neg_occ = Vec::new();
        let clauses: Vec<Expr> = expressions
            .drain_filter(|e| e.contains_var(index))
            .collect();

        for clause in clauses {
            if clause.contains_pos_var(index) {
                pos_occ.push(clause);
            } else {
                neg_occ.push(clause);
            }
        }
        Bucket {
            index,
            pos_occ,
            neg_occ,
        }
    }

    /// This function processes a bucket. It resolves each pair Var[index] AND -Var[index]
    pub fn process_bucket(&mut self, variables: &Vec<BddVar>) -> Result<(Vec<Expr>, Vec<Expr>), String> {
        let mut clauses_chosen = Vec::new();
        let mut resolved_clauses = Vec::new();

        // no pairs can be built here so we have to move on the next bucket
        if self.pos_occ.is_empty() || self.neg_occ.is_empty() {
            return Err("No pairs can be built!".to_string());
        }

        let (vec_pos, vec_neg): (Vec<Expr>, Vec<Expr>) = self.simple_filter_heuristics(10);
        for (expr1, expr2) in vec_pos.into_iter().cartesian_product(vec_neg.into_iter()) {
            let mut resolved_clause = expr1.resolution(&expr2);
            if !resolved_clause.is_empty() {
                let resolved_expr = Expr::parse_clause(&mut resolved_clause, variables);
                resolved_clauses.push(resolved_expr);
            }
            clauses_chosen.push(expr1);
            clauses_chosen.push(expr2);
            if resolved_clauses.is_empty() {
                return Err("The empty clause was generated in resolution!".to_string());
            }
        }
        resolved_clauses = resolved_clauses.into_iter().unique().collect::<Vec<Expr>>();
        Ok((clauses_chosen, resolved_clauses))
    }

    fn simple_filter_heuristics(&mut self, n: usize) -> (Vec<Expr>, Vec<Expr>) {
        self.pos_occ.sort_by(|a, b| a.size().cmp(&b.size()));
        self.pos_occ.dedup();
        self.neg_occ.sort_by(|a, b| a.size().cmp(&b.size()));
        self.neg_occ.dedup();

        let (pos, neg) = if self.pos_occ.len() < n && self.neg_occ.len() < n {
            (self.pos_occ.to_vec(), self.neg_occ.to_vec())
        } else if self.pos_occ.len() < n && self.neg_occ.len() >= n {
            (self.pos_occ.to_vec(), self.neg_occ[0..n].to_vec())
        } else if self.pos_occ.len() >= n && self.neg_occ.len() < n {
            (self.pos_occ[0..n].to_vec(), self.neg_occ.to_vec())
        } else {
            (self.pos_occ[0..n].to_vec(), self.neg_occ[0..n].to_vec())
        };
        (pos, neg)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        expr::bool_expr::Expr, parser::parse::parse_dimacs,
        variable_ordering::var_ordering::BddVarOrdering,
    };

    #[test]
    pub fn create_buckets() {
        // 83 16 65 0
        // 83 16 -65 0
        // 83 -16 65 0
        // -83 0
        // -16 0
        // -83 0
        let input: &str = "tests/test5.cnf";
        let dimacs = parse_dimacs(input);

        // build the variable ordering
        let var_ordering = BddVarOrdering::new(dimacs);
        println!("{:?}", var_ordering.variables);
        println!("{:?}", var_ordering.ordering);

        for (_, bucket) in var_ordering.buckets {
            if bucket.get_index().eq(&65) {
                assert_eq!(
                    bucket.pos_occ,
                    vec![
                        Expr::Or(
                            Box::new(Expr::Var(83)),
                            Box::new(Expr::Or(Box::new(Expr::Var(16)), Box::new(Expr::Var(65))))
                        ),
                        Expr::Or(
                            Box::new(Expr::Var(83)),
                            Box::new(Expr::Or(
                                Box::new(Expr::Not(Box::new(Expr::Var(16)))),
                                Box::new(Expr::Var(65))
                            ))
                        )
                    ]
                );
                assert_eq!(
                    bucket.neg_occ,
                    vec![Expr::Or(
                        Box::new(Expr::Var(83)),
                        Box::new(Expr::Or(
                            Box::new(Expr::Var(16)),
                            Box::new(Expr::Not(Box::new(Expr::Var(65))))
                        ))
                    )]
                );
            }
            if bucket.get_index().eq(&16) {
                assert_eq!(bucket.neg_occ, vec![Expr::Not(Box::new(Expr::Var(16)))]);
            }
            if bucket.get_index().eq(&83) {
                assert_eq!(
                    bucket.neg_occ,
                    vec![
                        Expr::Not(Box::new(Expr::Var(83))),
                        Expr::Not(Box::new(Expr::Var(83)))
                    ]
                );
            }
        }
    }
}
