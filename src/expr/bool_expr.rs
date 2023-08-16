use std::collections::HashSet;
use std::hash::{Hash, Hasher};

use crate::{
    bdd::Bdd, bdd_util::BddVar, expr::bool_expr::Expr::*, variable_ordering::var_ordering_builder::{Dimacs, self}
};

/// Recursive implementation of boolean expression.
#[derive(Clone, Debug, Eq, Hash)]
pub enum Expr {
    Const(bool),
    Var(i32),
    Not(Box<Expr>),
}

impl Expr {

    pub fn get_var_name(&self) -> i32 {
        match self {
            Expr::Var(name) => *name,
            Expr::Not(inner) => inner.get_var_name(),
            _ => i32::MAX,
        }
    }
    
    pub fn parse_dimacs_cnf_file(file_path: &str) -> Result<Dimacs, String> {
        let content = std::fs::read_to_string(file_path).map_err(|e| e.to_string())?;
        let mut var_map: std::collections::HashMap<i32, Expr> = std::collections::HashMap::new();
        // this hashmap contains a variable and the arities of the clauses where
        // this variable is appearing.
        let mut var_clause_arities: std::collections::HashMap<i32, Vec<usize>> = std::collections::HashMap::new();
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
                },
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
                        literals.insert(Expr::parse_lit(*lit, &mut var_map));
                    }
                    expressions.push(Clause { literals });
                    
                }
            }
        }
        let vars_scores = var_ordering_builder::calculate_scores(var_clause_arities);

        let dimacs = Dimacs {
            nb_v,
            nb_c,
            var_map,
            vars_scores,
            expressions
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

    /// Negate an expression.
    fn negate(&self) -> Expr {
        match self {
            Const(value) => {
                if *value {
                    Expr::Const(false)
                } else {
                    Expr::Const(true)
                }
            }
            Var(name) => Expr::Not(Box::new(Expr::Var(*name))),
            Not(inner) => Expr::Var(inner.get_var_name()),
        }
    }

    fn contains_var(&self, var: i32) -> bool {
        match self {
            Expr::Var(name) => name.eq(&var),
            Expr::Not(inner) => inner.contains_var(var),
            _ => false,
        }
    }

    fn contains_neg_var(&self, var: i32) -> bool {
        match self {
            Expr::Not(inner) => inner.contains_pos_var(var),
            _ => false,
        }
    }

    fn contains_pos_var(&self, var: i32) -> bool {
        match self {
            Expr::Var(name) => name.eq(&var),
            _ => false,
        }
    }

    fn set_vars(&self, assignment: &std::collections::HashMap<i32, bool>) -> Option<bool> {
        match self {
            Const(val) => Some(*val),
            Var(name) => {
                if let Some(valuet) = assignment.get(&name) {
                    Some(*valuet)
                } else if let Some(valuef) = assignment.get(&-name) {
                    Some(*valuef)
                } else {
                    None
                }
            }
            Not(inner) => match inner.set_vars(assignment) {
                Some(val) => {
                    if val {
                        Some(false)
                    } else {
                        Some(true)
                    }
                }
                None => None,
            }
        }
    }

    /// This method constructs a Robdd from a given boolean expression.
    ///
    /// *Panics*:
    /// - Variable wasn't found in variable vector.
    pub fn to_bdd(&self, variables: &Vec<BddVar>, ordering: &std::collections::HashMap<i32, usize>) -> Bdd {
        // The construction of a Bdd from a boolean expression proceeds
        // as in the construction of the if-then-else Normalform. An ordering
        // of the variables is fixed. Using the shannon expansion a node
        // for the expression is constructed by a call to mk (checks if
        // the exact same node exists in the node cache). The nodes for
        // each sub-expression are constructed by recursion.
        match self {
            Const(value) => Bdd::new_value(BddVar::new(i32::MAX, 0.0), value),
            Var(name) => {
                if let Some(pos) = variables.iter().position(|i| i.name.eq(name)) {
                    Bdd::new_var(variables[pos])
                } else {
                    panic!("Variable {} doesn't exists.", name);
                }
            }
            Not(inner) => inner.to_bdd(variables, ordering).negate()
        }
    }

    /*
    // This method helps the process of bucket elimination.
    // It finds the the highest order variable in the clause
    // to be able to sort it afterwards in the correct bucket
    // indexing this variable.
    pub fn find_highest_order(
        &self,
        ordering: &std::collections::HashMap<i32, usize>,
    ) -> (bool, usize) {
        let (mut acc_pol, mut acc_idx) = (false, 0);
        for (pol, var) in self.to_vars_with_polarities() {
            let order = *ordering.get(&var).unwrap();
            if order > acc_idx {
                acc_idx = order;
                acc_pol = pol;
            }
        }
        (acc_pol, acc_idx)
    }*/
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Const(value) => write!(f, "{}", value),
            Var(name) => write!(f, "{}", name),
            Not(inner) => write!(f, "!{}", inner)
        }
    }
}

impl PartialEq for Expr {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Const(val1), Const(val2)) => val1.eq(&val2),
            (Var(name1), Var(name2)) => name1.eq(&name2),
            (Not(in1), Not(in2)) => in1.eq(&in2),
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Eq)]
pub struct Clause {
    pub literals: HashSet<Expr>,
}

impl Clause {
    pub fn is_empty(&self) -> bool {
        self.literals.is_empty()
    }

    // Method to check if a clause contains an expression
    pub fn contains_expr(&self, expr: &Expr) -> bool {
        self.literals.contains(expr)
    }

    fn remove(&mut self, literal: &Expr) {
        self.literals.remove(literal);
    }

    // Function to check if the clause is fully determined
    pub fn is_determined(&self) -> bool {
        // Check if all variables in the clause have assignments
        self.literals.iter().all(|literal| {
            if let Expr::Var(var) = literal {
                // Check if the variable has an assignment
                self.has_assignment(*var)
            } else {
                true // Constants are always determined
            }
        })
    }

    // Function to check if a variable has an assignment in the clause
    fn has_assignment(&self, var: i32) -> bool {
        self.literals.iter().any(|literal| {
            if let Expr::Var(name) = literal {
                name == &var || name == &(-var) // Check both variable and negation
            } else {
                false
            }
        })
    }

    pub fn to_bdd(&self, variables: &Vec<BddVar>, ordering: &std::collections::HashMap<i32, usize>) -> Bdd {
        let mut bdd = self.literals.iter().next().unwrap().to_bdd(&variables, &ordering);

        if let Some(expr) = self.literals.iter().next() {
            let temp_bdd = expr.to_bdd(&variables, &ordering);
            bdd = bdd.or(&temp_bdd, &ordering); // OR operation because we are in a clause
        }
        bdd
    }

    // Function to substitute the value of a variable based on a substitution map
    pub fn substitute_variable(&mut self, substitution: &std::collections::HashMap<i32, bool>) {
        let mut new_literals = HashSet::new();
        for literal in &self.literals {
            match literal {
                Expr::Var(var) => {
                    if let Some(&value) = substitution.get(var) {
                        new_literals.insert(if value {
                            Expr::Const(true)
                        } else {
                            Expr::Const(false)
                        });
                    } else {
                        new_literals.insert(literal.clone());
                    }
                }
                Expr::Not(inner) => {
                    if let Expr::Var(var) = &**inner {
                        if let Some(&value) = substitution.get(var) {
                            new_literals.insert(if value {
                                Expr::Const(false)
                            } else {
                                Expr::Const(true)
                            });
                        } else {
                            new_literals.insert(literal.clone());
                        }
                    } else {
                        new_literals.insert(literal.clone());
                    }
                }
                _ => {
                    new_literals.insert(literal.clone());
                }
            }
        }
        self.literals = new_literals;
    }


    /// If a clause consists of only one literal (positive or
    /// negative), this clause is called a unit clause. We fix the
    /// valuation of an atom occurring in a unit clause to the
    /// value indicated by the sign of the literal.
    pub fn is_unit(&self) -> bool {
        if self.literals.len() == 1 {
            return true;
        }
        false
    }

    pub fn clause_contains_var(&self, index: i32) -> bool {
        self.literals.iter().any(|expr| expr.contains_var(index))
    }

    pub fn clause_contains_pos_var(&self, index: i32) -> bool {
        self.literals.iter().any(|expr| expr.contains_pos_var(index))
    }

    pub fn solve(&self, assignment: &std::collections::HashMap<i32, bool>) -> bool {
        let mut assigned_clause = Vec::new();
        for expr in &self.literals {
            assigned_clause.push(expr.set_vars(assignment));
        }
        assigned_clause.iter().fold(false, |acc, opt| {
            acc || opt.unwrap_or(false)
        })
    }

    pub fn resolve(&self, other: &Clause) -> Clause {
        let mut new_literals = HashSet::new();

        for literal in self.literals.iter() {
            if !other.literals.contains(literal) && !other.literals.contains(&literal.negate()) {
                new_literals.insert(literal.clone());
            }
            if other.literals.contains(literal) && !other.literals.contains(&literal.negate()) {
                new_literals.insert(literal.clone());
            }
        }

        for literal in other.literals.iter() {
            if !self.literals.contains(literal) && !self.literals.contains(&literal.negate()) {
                new_literals.insert(literal.clone());
            }
            if self.literals.contains(literal) && !self.literals.contains(&literal.negate()) {
                new_literals.insert(literal.clone());
            }
        }

        Clause {
            literals: new_literals,
        }
    }

}

impl PartialEq for Clause {
    fn eq(&self, other: &Self) -> bool {
        self.literals == other.literals
    }
}

impl Hash for Clause {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for literal in &self.literals {
            literal.hash(state);
        }
    }
}

/// Partial operator function corresponding to $x \land y$.
pub fn and(l: Option<bool>, r: Option<bool>) -> Option<bool> {
    match (l, r) {
        (Some(true), Some(true)) => Some(true),
        (Some(false), _) => Some(false),
        (_, Some(false)) => Some(false),
        _ => None,
    }
}

/// Partial operator function corresponding to $x \lor y$.
pub fn or(l: Option<bool>, r: Option<bool>) -> Option<bool> {
    match (l, r) {
        (Some(false), Some(false)) => Some(false),
        (Some(true), _) => Some(true),
        (_, Some(true)) => Some(true),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn test_parse_lit_positive() {
        let mut var_map: std::collections::HashMap<i32, Expr> = std::collections::HashMap::new();
        let expr = Expr::parse_lit(1, &mut var_map);
        assert_eq!(expr, Expr::Var(1));
    }

    #[test]
    fn test_parse_lit_negative() {
        let mut var_map: std::collections::HashMap<i32, Expr> = std::collections::HashMap::new();
        let expr = Expr::parse_lit(-2, &mut var_map);
        assert_eq!(expr, Expr::Not(Box::new(Expr::Var(2))));
    }

    #[test]
    fn test_to_bdd() {
        let var1 = BddVar::new(1, 0.5);
        let var2 = BddVar::new(2, 0.3);
        let var3 = BddVar::new(3, 0.8);

        let variables = vec![var1.clone(), var2.clone(), var3.clone()];

        let mut ordering = std::collections::HashMap::new();
        ordering.insert(1, 0);
        ordering.insert(2, 1);
        ordering.insert(3, 2);

        let expr = Expr::Not(Box::new(Expr::Var(2)));

        let bdd = expr.to_bdd(&variables, &ordering);

        // You can add more specific assertions about the BDD structure if needed
        //assert_eq!(bdd.var, var2);
        //assert_eq!(bdd.high, None);
        //assert_eq!(bdd.low, None);
    }

    #[test]
    fn test_is_unit() {
        let clause = Clause {
            literals: HashSet::from_iter(vec![Expr::Var(1)]),
        };
        assert_eq!(clause.is_unit(), true);

        let clause = Clause {
            literals: HashSet::from_iter(vec![Expr::Var(1), Expr::Var(2)]),
        };
        assert_eq!(clause.is_unit(), false);
    }

    #[test]
    fn test_solve() {
        let assignment: std::collections::HashMap<i32, bool> = [(1, true), (2, false), (3, true)]
            .iter()
            .cloned()
            .collect();

        let clause = Clause {
            literals: HashSet::from_iter(vec![Expr::Var(1), Expr::Not(Box::new(Expr::Var(2))), Expr::Var(3)]),
        };

        assert_eq!(clause.solve(&assignment), true);
    }

    #[test]
    fn test_resolve_disjoint_clauses() {
        let literals1 = HashSet::from_iter(vec![Expr::Var(1), Expr::Var(2)]);
        let literals2 = HashSet::from_iter(vec![Expr::Var(3), Expr::Var(4)]);
        let clause1 = Clause {
            literals: literals1,
        };
        let clause2 = Clause {
            literals: literals2,
        };
        let expected_literals = HashSet::from_iter(vec![Expr::Var(1), Expr::Var(2),Expr::Var(3), Expr::Var(4)]);
        let expected_result = Clause {
            literals: expected_literals,
        };

        assert_eq!(clause1.resolve(&clause2), expected_result);
    }

    #[test]
    fn test_resolve_overlap_clauses() {
        let literals1 = HashSet::from_iter(vec![Expr::Var(1), Expr::Var(2)]);
        let literals2 = HashSet::from_iter(vec![Expr::Var(2), Expr::Var(3)]);
        let clause1 = Clause {
            literals: literals1,
        };
        let clause2 = Clause {
            literals: literals2,
        };

        let expected_literals = HashSet::from_iter(vec![Expr::Var(1),Expr::Var(2), Expr::Var(3)]);
        let expected_result = Clause {
            literals: expected_literals,
        };

        assert_eq!(clause1.resolve(&clause2), expected_result);
    }

    #[test]
    fn test_resolve_opposite_literals() {
        let literals1 = HashSet::from_iter(vec![Expr::Var(1), Expr::Var(2)]);
        let literals2 = HashSet::from_iter(vec![Expr::Not(Box::new(Expr::Var(1))), Expr::Var(3)]);
        let clause1 = Clause {
            literals: literals1,
        };
        let clause2 = Clause {
            literals: literals2,
        };

        let expected_literals = HashSet::from_iter(vec![Expr::Var(2), Expr::Var(3)]);
        let expected_result = Clause {
            literals: expected_literals,
        };

        assert_eq!(clause1.resolve(&clause2), expected_result);
    }
}
