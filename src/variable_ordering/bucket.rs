use crate::expr::bool_expr::Expr;
//use itertools::Itertools;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Bucket {
    pub index: i32,
    pub pos_occ: Vec<Vec<Expr>>,
    pub neg_occ: Vec<Vec<Expr>>,
}

impl Bucket {
    pub fn get_index(&self) -> i32 {
        self.index
    }

    pub fn create_bucket(index: i32, expressions: &mut Vec<Vec<Expr>>) -> Bucket {
        let mut pos_occ = Vec::new();
        let mut neg_occ = Vec::new();
        let mut clauses = Vec::new();

        expressions.retain(|clause| {
            let should_keep = !Expr::clause_contains_var(clause, index);
            if !should_keep {
                clauses.push(clause.clone()); // Collect elements containing the variable
            }
            should_keep
        });

        for clause in clauses {
            if Expr::clause_contains_pos_var(&clause, index) {
                pos_occ.push(clause.clone());
            } else {
                neg_occ.push(clause.clone());
            }
        }
        Bucket {
            index,
            pos_occ,
            neg_occ,
        }
    }

    /*
    pub fn process_bucket(&mut self, _variables: &[BddVar]) -> Result<Vec<Expr>, String> {
        let mut resolved_clauses = Vec::new();

        // No pairs can be built here so we move on to the next bucket
        if self.pos_occ.is_empty() || self.neg_occ.is_empty() {
            return Err("No pairs can be built!".to_string());
        }

        for (expr1, expr2) in self.pos_occ.iter().flat_map(|e1| {
            self.neg_occ
                .iter()
                .map(move |e2| (e1.clone(), e2.clone()))
        }) {
            let mut resolved_clause = expr1.resolve(&expr2);
            if !resolved_clause.is_empty() {
                let resolved_expr = Expr::parse_clause(&mut resolved_clause, _variables);
                resolved_clauses.push(resolved_expr);
            }
        }

        if resolved_clauses.is_empty() {
            return Err("The empty clause was generated in resolution!".to_string());
        }

        resolved_clauses = resolved_clauses.into_iter().unique().collect::<Vec<Expr>>();
        Ok(resolved_clauses)
    } */
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_bucket() {
        let mut expressions: Vec<Vec<Expr>> = vec![
            vec![Expr::Var(1),
            Expr::Not(Box::new(Expr::Var(2)))],
            vec![Expr::Var(3),
            Expr::Var(4),
            Expr::Not(Box::new(Expr::Var(1)))],
        ];

        let bucket = Bucket::create_bucket(1, &mut expressions);
        assert_eq!(bucket.index, 1);
        assert_eq!(bucket.pos_occ.len(), 1);
        assert_eq!(bucket.neg_occ.len(), 1);
    }
}
