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

    // Function to choose a variable for elimination based on the minimum width heuristic
    fn choose_variable_to_eliminate(&self) -> i32 {
        let mut min_width = usize::MAX;
        let mut selected_var = i32::MAX;

        for clause in &self.0 {
            for lit in &clause.literals {
                let width = self.0.iter().filter(|&c| c.clause_contains_var(lit.get_var_name())).count();
                if width < min_width {
                    min_width = width;
                    selected_var = lit.get_var_name();
                }
            }
        }

        selected_var
    }
    

    // Method to choose a variable for elimination based on some heuristic
    fn choose_variable_to_eliminate_highest_frequency(&self) -> i32 {
        // Example: Choose the variable with the highest frequency in the bucket
        let mut var_frequencies: std::collections::HashMap<i32, usize> = std::collections::HashMap::new();

        for clause in &self.0 {
            for literal in &clause.literals {
                if let Expr::Var(var) = literal {
                    *var_frequencies.entry(*var).or_insert(0) += 1;
                }
            }
        }

        // Find the variable with the highest frequency
        let (chosen_var, _) = var_frequencies
            .into_iter()
            .max_by_key(|&(_, frequency)| frequency)
            .unwrap_or((i32::MAX, 0));

        chosen_var
    }
    
    // Function to eliminate a variable from the bucket
    fn eliminate_variable(&mut self, var_to_eliminate: i32) {
        // Step 1: Find constraints containing the variable to eliminate
        let constraints_to_eliminate: Vec<Clause> = self.0.iter()
            .filter(|clause| clause.clause_contains_var(var_to_eliminate))
            .cloned()
            .collect();

        // Step 2: Solve for the variable and substitute its value
        let mut substitution: HashMap<i32, bool> = HashMap::new();
        for clause in &constraints_to_eliminate {
            let var_value = clause.solve(&substitution); // Implement clause solving
            substitution.insert(var_to_eliminate, var_value);
        }

        // Step 3: Substitute the variable's value into other constraints
        for clause in &mut self.0 {
            if !clause.clause_contains_var(var_to_eliminate) {
                clause.substitute_variable(&substitution);
            }
        }

        // Step 4: Remove constraints that are fully determined or satisfied
        self.0.retain(|clause| !clause.is_determined());
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

type Buckets = Vec<Bucket>;
type Clauses = Vec<Clause>;

// Function to group clauses into buckets based on interacting variables
fn group_clauses_into_buckets_interactions(clauses: Clauses) -> Buckets {
    let interactions: HashMap<i32, Vec<i32>> = find_interacting_variables(&clauses);
  
    let mut buckets: Buckets = Vec::new();

    for clause in clauses {
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
fn find_interacting_variables(clauses: &Clauses) -> HashMap<i32, Vec<i32>> {
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
fn group_clauses_into_buckets_implications(clauses: Clauses) -> Buckets {
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
    let mut buckets: Buckets = Vec::new();
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

// Function to perform full bucket elimination
fn bucket_elimination(buckets: &mut Buckets) -> Result<(), String> {
    while let Some(mut bucket) = buckets.pop() {
        let var_to_eliminate = bucket.choose_variable_to_eliminate(); // Variable selection heuristic
        if var_to_eliminate == i32::MAX {
            return Err("No more variables to eliminate".to_string());
        }
        bucket.eliminate_variable(var_to_eliminate);
        if bucket.0.is_empty() {
            return Err("Unsatisfiable constraint".to_string());
        }
        buckets.push(bucket);
    }
    Result::Ok(())

    /* 
    if buckets.len() == 1 {
        let solution = buckets[0].solve(); // Implement final solution extraction
        Ok(solution)
    } else {
        Err("Unexpected error occurred".to_string())
    }
    */
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_choose_variable_to_eliminate() {
        let clause1 = Clause {
            literals: vec![Expr::Var(1), Expr::Var(2)].into_iter().collect(),
        };
        let clause2 = Clause {
            literals: vec![Expr::Var(2), Expr::Var(3)].into_iter().collect(),
        };
        let bucket = Bucket(vec![clause1.clone(), clause2.clone()]);
    
        let chosen_var = bucket.choose_variable_to_eliminate();
        assert_eq!(chosen_var, 1);
    }    

    #[test]
    fn test_find_interacting_variables() {
        let clauses = vec![
            Clause { literals: HashSet::from_iter(vec![Expr::Var(1), Expr::Var(2)]) },
            Clause { literals: HashSet::from_iter(vec![Expr::Var(2), Expr::Var(3)]) },
            Clause { literals: HashSet::from_iter(vec![Expr::Var(1), Expr::Var(3)]) },
        ];

        let interactions = find_interacting_variables(&clauses);

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
        let buckets = group_clauses_into_buckets_interactions(expressions);
        
        // Test assertions
        assert_eq!(buckets.len(), 2);
        assert_eq!(buckets[0].0.len(), 2);
        assert_eq!(buckets[1].0.len(), 1);
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

        let buckets = group_clauses_into_buckets_implications(clauses);

        assert_eq!(buckets.len(), 2)
    }
}

