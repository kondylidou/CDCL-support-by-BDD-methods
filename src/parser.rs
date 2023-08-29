use crate::expr::bool_expr::{Clause, Expr};
use anyhow::{Context, Result};
use std::collections::HashSet;

#[derive(Clone, Debug)]
pub struct Dimacs {
    pub nb_v: i32,
    pub nb_c: i32,
    pub var_map: std::collections::HashMap<i32, Expr>,
    pub vars_scores: std::collections::HashMap<i32, f64>,
    pub expressions: Vec<Clause>,
}

pub fn parse_dimacs_cnf_file(file_path: &str) -> Result<Dimacs> {
    let content = std::fs::read_to_string(file_path)
        .with_context(|| format!("Error reading file: {}", file_path))?;
    let mut var_map: std::collections::HashMap<i32, Expr> = std::collections::HashMap::new();
    // this hashmap contains a variable and the arities of the clauses where
    // this variable is appearing.
    let mut var_clause_arities: std::collections::HashMap<i32, Vec<usize>> =
        std::collections::HashMap::new();
    let mut expressions: Vec<Clause> = Vec::new();
    let mut nb_v = 0;
    let mut nb_c = 0;

    for line in content.lines() {
        let tokens: Vec<&str> = line.trim().split_whitespace().collect();
        if tokens.is_empty() {
            continue; // Skip empty lines
        }

        match tokens[0] {
            "c" => continue, // Skip comments
            "p" => {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    nb_v = parts[2].parse().unwrap();
                    nb_c = parts[3].parse().unwrap();
                }
            }
            _ => {
                let mut literals = HashSet::new();
                let clause: Vec<i32> = tokens
                    .iter()
                    .take_while(|&&token| token != "0")
                    .map(|&token| token.parse().expect("Error parsing literal"))
                    .collect();

                for lit in &clause {
                    // add the clause arity to each variable appearing in this clause
                    if let Some(arities) = var_clause_arities.get_mut(&lit.abs()) {
                        arities.push(clause.len());
                    } else {
                        var_clause_arities.insert(lit.abs(), vec![clause.len()]);
                    }
                    literals.insert(parse_lit(*lit, &mut var_map));
                }
                expressions.push(Clause { literals });
            }
        }
    }
    let vars_scores = calculate_scores(var_clause_arities);

    let dimacs = Dimacs {
        nb_v,
        nb_c,
        var_map,
        vars_scores,
        expressions,
    };

    Ok(dimacs)
}

fn parse_lit(lit: i32, var_map: &mut std::collections::HashMap<i32, Expr>) -> Expr {
    let var_expr = var_map
        .entry(lit.abs())
        .or_insert_with(|| Expr::Var(lit.abs()));

    if lit < 0 {
        Expr::Not(Box::new(var_expr.clone()))
    } else {
        var_expr.clone()
    }
}

/// For our implementation, we use a simple heuristic to determine the variable ordering:
/// each variable is assigned a score, computed as the quotient between the number of clauses
/// containing the variable and the average arity of those clauses.
pub fn calculate_scores(
    var_clause_arities: std::collections::HashMap<i32, Vec<usize>>,
) -> std::collections::HashMap<i32, f64> {
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

#[cfg(test)]
mod tests {
    use crate::parser;

    use super::*;

    #[test]
    fn test_parse_lit_positive() {
        let mut var_map: std::collections::HashMap<i32, Expr> = std::collections::HashMap::new();
        let expr = parser::parse_lit(1, &mut var_map);
        assert_eq!(expr, Expr::Var(1));
    }

    #[test]
    fn test_parse_lit_negative() {
        let mut var_map: std::collections::HashMap<i32, Expr> = std::collections::HashMap::new();
        let expr = parser::parse_lit(-2, &mut var_map);
        assert_eq!(expr, Expr::Not(Box::new(Expr::Var(2))));
    }

    #[test]
    fn variable_scores() {
        let dimacs = parser::parse_dimacs_cnf_file("tests/test3.cnf").unwrap();
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
}
