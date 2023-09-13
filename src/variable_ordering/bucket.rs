use crate::expr::bool_expr::{Clause, Expr};
use anyhow::{anyhow, Ok, Result};
use itertools::Itertools;
use std::collections::{HashMap, HashSet};

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

#[derive(Clone, Debug, PartialEq)]
#[repr(C)]
pub struct Bucket {
    pub clauses: Vec<Clause>,
    pub index: i32,
}

impl Bucket {

    pub fn vars(&self) -> HashSet<i32> {
        let mut vars = HashSet::new();

        for clause in &self.clauses {
            for expr in &clause.literals {
                vars.insert(expr.get_var_name());
            }
        }

        vars
    }
    // Function to choose a variable for elimination based on the minimum width heuristic
    pub fn choose_variable_to_eliminate(&self) -> i32 {
        let mut min_width = usize::MAX;
        let mut selected_var = i32::MAX;

        for clause in &self.clauses {
            for lit in &clause.literals {
                let width = self
                    .clauses
                    .iter()
                    .filter(|&c| c.clause_contains_var(lit.get_var_name()))
                    .count();
                if width < min_width {
                    min_width = width;
                    selected_var = lit.get_var_name();
                }
            }
        }

        selected_var
    }

    // Method to choose a variable for elimination based on some heuristic
    pub fn choose_variable_to_eliminate_highest_frequency(&self) -> i32 {
        // Example: Choose the variable with the highest frequency in the bucket
        let mut var_frequencies: std::collections::HashMap<i32, usize> =
            std::collections::HashMap::new();

        for clause in &self.clauses {
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
    pub fn eliminate_variable(&mut self, var_to_eliminate: i32) {
        // Step 1: Find constraints containing the variable to eliminate
        let constraints_to_eliminate: Vec<Clause> = self
            .clauses
            .iter()
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
        for clause in &mut self.clauses {
            if !clause.clause_contains_var(var_to_eliminate) {
                clause.substitute_variable(&substitution);
            }
        }

        // Step 4: Remove constraints that are fully determined or satisfied
        self.clauses.retain(|clause| !clause.is_determined());
    }

    /// This function processes a bucket. It resolves each pair Var[index] AND -Var[index]
    pub fn bucket_elimination(&mut self) -> Result<()> {
        let mut resolved_clauses: Vec<Clause> = Vec::new();

        let (pos_occ, neg_occ): (Vec<Clause>, Vec<Clause>) = self
            .clauses
            .clone()
            .into_iter()
            .partition(|clause| clause.clause_contains_pos_var(self.index));

        // No pairs can be built here so we move on to the next bucket
        if pos_occ.is_empty() || neg_occ.is_empty() {
            return Err(anyhow!("No pairs can be built!".to_string()));
        }

        for (expr1, expr2) in pos_occ
            .iter()
            .flat_map(|e1| neg_occ.iter().map(move |e2| (e1, e2)))
        {
            let resolved_clause = expr1.resolve(expr2);
            if !resolved_clause.is_empty() {
                resolved_clauses.push(resolved_clause);
            }
        }

        if resolved_clauses.is_empty() {
            return Err(anyhow!(
                "The empty clause was generated in resolution!".to_string()
            ));
        }

        resolved_clauses = resolved_clauses
            .into_iter()
            .unique()
            .collect::<Vec<Clause>>();

        self.clauses = resolved_clauses;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_choose_variable_to_eliminate() {
        let clause1 = Clause {
            literals: vec![Expr::Var(1), Expr::Var(2)].into_iter().collect(),
        };
        let clause2 = Clause {
            literals: vec![Expr::Var(2), Expr::Var(3)].into_iter().collect(),
        };
        let bucket = Bucket {
            clauses: vec![clause1.clone(), clause2.clone()],
            index: 2,
        };

        let chosen_var = bucket.choose_variable_to_eliminate();
        assert_eq!(chosen_var, 1);
    }
}
