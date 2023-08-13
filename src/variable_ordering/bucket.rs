use std::collections::{HashSet, HashMap};

use crate::expr::bool_expr::{Clause, Expr};

/*
    Clause Structure: Variables that frequently appear together within the same clause tend to interact. 
    Clauses often capture specific relationships or conditions involving multiple variables.

    Logical Dependencies: Variables that are logically dependent on each other have strong interactions. 
    If the presence or absence of one variable directly affects the interpretation of another, they have dependencies.

    Implications: Variables that imply each other's assignments interact. 
    If the assignment of one variable forces the assignment of another, they have a strong dependency.

    Mutual Exclusivity: Variables that are mutually exclusive often interact. 
    If only one of the variables can be assigned true in a clause, they are dependent.

    Grouping and Patterns: Variables that group together due to shared properties or patterns interact. 
    Clusters of variables with similar roles often have dependencies.

    Symmetry Breaking: Variables introduced to break symmetry in the problem often interact. 
    They may play a crucial role in ensuring unique solutions.

    Constraints: Variables that participate in shared constraints or logical conditions interact. 
    Variables that are part of the same logical rule or constraint are dependent.
*/

#[derive(Clone, Debug)]
pub struct Bucket(Vec<Clause>);

impl Bucket {
    // Function to group clauses into buckets based on interacting variables
    fn group_clauses_into_buckets_interactions(expressions: Vec<Clause>) -> Vec<Bucket> {
        let interactions: HashMap<i32, Vec<i32>> = Bucket::find_interacting_variables(&expressions);
      
        let mut buckets: Vec<Bucket> = Vec::new();
    
        for clause in expressions {
            let mut placed = false;

            for bucket in &mut buckets {
                if clause.literals.iter().any(|expr| {
                    interactions[&expr.get_var_name()]
                        .iter()
                        .any(|var| bucket.0.iter().any(|clause| clause.clause_contains_var(*var)))
                }) {
                    bucket.0.push(clause.clone());
                    placed = true;
                    break;
                }
            }

            if !placed {
                buckets.push(Bucket(vec![clause]));
            }
        }
    
        buckets
    }

    // Function to find frequently interacting variables based on clause structure
    fn find_interacting_variables(clauses: &Vec<Clause>) -> HashMap<i32, Vec<i32>> {
        let mut variable_interactions: HashMap<i32, Vec<i32>> = HashMap::new();

        for clause in clauses {
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
    fn group_clauses_into_buckets_implications(clauses: Vec<Clause>) -> Vec<Bucket> {
        // Construct a hashmap to track the implications of each variable
        let mut implications: HashMap<i32, HashSet<i32>> = HashMap::new();

        // Populate the implications hashmap based on the clauses
        for clause in &clauses {
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
                }
                else if let Expr::Var(var) = literal {
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
        let mut buckets: Vec<Bucket> = Vec::new();
        for clause in clauses {
            let mut placed = false;

            for bucket in &mut buckets {
                if clause
                    .literals
                    .iter()
                    .any(|expr| bucket.0.iter().any(|c| c.contains_expr(expr) || c.contains_expr(&Expr::Not(Box::new(expr.clone())))))
                {
                    bucket.0.push(clause.clone());
                    placed = true;
                    break;
                }
            }

            if !placed {
                buckets.push(Bucket(vec![clause]));
            }
        }
        buckets
    }

    /*
    pub fn process_bucket(&self) -> Result<Vec<Clause>, String> {
        let mut resolved_clauses: Vec<Clause> = Vec::new();

        let (pos_occ, neg_occ): (Vec<Clause>, Vec<Clause>) = self.clauses.clone().into_iter().partition(|clause| 
            clause.clause_contains_pos_var(self.index));

        // No pairs can be built here so we move on to the next bucket
        if pos_occ.is_empty() || neg_occ.is_empty() {
            return Err("No pairs can be built!".to_string());
        }

        for (expr1, expr2) in pos_occ.iter().flat_map(|e1| {
            neg_occ
                .iter()
                .map(move |e2| (e1, e2))
        }) {
            let resolved_clause = expr1.resolve(expr2);
            if !resolved_clause.is_empty() {
                resolved_clauses.push(resolved_clause);
            }
        }

        if resolved_clauses.is_empty() {
            return Err("The empty clause was generated in resolution!".to_string());
        }

        resolved_clauses = resolved_clauses.into_iter().unique().collect::<Vec<Clause>>();
        Ok(resolved_clauses)
    } 
    */
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::expr::bool_expr::Expr;

    use super::*;

    #[test]
    fn test_find_interacting_variables() {
        let clauses = vec![
            Clause { literals: HashSet::from_iter(vec![Expr::Var(1), Expr::Var(2)]) },
            Clause { literals: HashSet::from_iter(vec![Expr::Var(2), Expr::Var(3)]) },
            Clause { literals: HashSet::from_iter(vec![Expr::Var(1), Expr::Var(3)]) },
        ];

        let interactions = Bucket::find_interacting_variables(&clauses);

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

        // Call the function
        let buckets = Bucket::group_clauses_into_buckets_interactions(expressions);
        
        // Test assertions
        assert_eq!(buckets.len(), 2);
        assert_eq!(buckets[0].0.len(), 2);
        assert_eq!(buckets[1].0.len(), 1);

        // You can add more specific assertions based on your implementation and data structures
        // For example, check that clauses with interacting variables are in the same bucket
        // and that non-interacting clauses are in different buckets.
    }

    #[test]
    fn test_group_clauses_into_buckets_implications() {
        let clauses = vec![
            Clause {
                literals: vec![
                    Expr::Var(1),
                    Expr::Var(2),
                ].into_iter().collect()
            },
            Clause {
                literals: vec![
                    Expr::Not(Box::new(Expr::Var(2))),
                    Expr::Var(3),
                    Expr::Not(Box::new(Expr::Var(4))),
                ].into_iter().collect()
            },
            Clause {
                literals: vec![
                    Expr::Var(4),
                    Expr::Var(5),
                ].into_iter().collect()
            },
        ];

        let buckets = Bucket::group_clauses_into_buckets_implications(clauses);

        assert_eq!(buckets.len(), 2)
    }

}
