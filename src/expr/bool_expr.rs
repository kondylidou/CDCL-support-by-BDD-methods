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

    pub fn new_var(var: i32) -> Expr {
        Expr::Var(var)
    }
    
    pub fn parse_dimacs_cnf_file(file_path: &str) -> Result<Dimacs, String> {
        let content = std::fs::read_to_string(file_path).map_err(|e| e.to_string())?;
        let mut var_map: std::collections::HashMap<i32, Expr> = std::collections::HashMap::new();
        // this hashmap contains a variable and the arities of the clauses where
        // this variable is appearing.
        let mut var_clause_arities: std::collections::HashMap<i32, Vec<usize>> = std::collections::HashMap::new();
        let mut expressions: Vec<Vec<Expr>> = Vec::new();
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
                    let mut expr = Vec::new();
                    let clause: Vec<i32> = tokens
                        .iter()
                        .take_while(|&&token| token != "0")
                        .map(|&token| token.parse().expect("Error parsing literal"))
                        .collect();

                    for lit in &clause {
                        // add the clause arity to each variable appearing in this clause
                        if let Some(arities) = var_clause_arities.get_mut(&lit) {
                            arities.push(clause.len());
                        } else {
                            var_clause_arities.insert(*lit, vec![clause.len()]);
                        }
                        expr.push(Expr::parse_lit(*lit, &mut var_map));
                    }
                    expressions.push(expr);
                    
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

    /// If a clause consists of only one literal (positive or
    /// negative), this clause is called a unit clause. We fix the
    /// valuation of an atom occurring in a unit clause to the
    /// value indicated by the sign of the literal.
    pub fn is_unit(clause: Vec<Expr>) -> bool {
        if clause.len() == 1 {
            return true;
        }
        false
    }

    /// This method is used in unit propagation.
    /// It makes sure that the clause from which the variable
    /// will be removed is not unit and removes the given
    /// variable using the above function to remove a variable
    /// from a given Boolean Expression.
    pub fn remove_var(clause: &mut Vec<Expr>, var_name: i32) -> bool {
        if let Some(index) = Expr::find_var_idx(clause, var_name) {
            clause.remove(index);
            return true;
        } 
        false
    }

    pub fn find_var_idx(clause: &mut Vec<Expr>, var_name: i32) -> Option<usize> {
        clause.iter().position(|e| e.contains_var(var_name))
    }

    /// Negate an expression.
    pub fn negate(&self) -> Expr {
        match self {
            Const(value) => {
                if *value {
                    Expr::Const(false)
                } else {
                    Expr::Const(true)
                }
            }
            Var(name) => Expr::Not(Box::new(Expr::Var(*name))),
            Not(inner) => inner.negate(),
        }
    }

    pub fn contains_var(&self, var: i32) -> bool {
        match self {
            Expr::Var(name) => name.eq(&var),
            Expr::Not(inner) => inner.contains_var(var),
            _ => false,
        }
    }

    pub fn contains_neg_var(&self, var: i32) -> bool {
        match self {
            Expr::Not(inner) => inner.contains_pos_var(var),
            _ => false,
        }
    }

    pub fn contains_pos_var(&self, var: i32) -> bool {
        match self {
            Expr::Var(name) => name.eq(&var),
            _ => false,
        }
    }

    pub fn clause_contains_var(clause: &[Expr], index: i32) -> bool {
        clause.iter().any(|expr| expr.contains_var(index))
    }

    pub fn clause_contains_pos_var(clause: &[Expr], index: i32) -> bool {
        clause.iter().any(|expr| expr.contains_pos_var(index))
    }

    pub fn solve(clause: Vec<Expr>, assignment: &std::collections::HashMap<i32, bool>) -> bool {
        let mut assigned_clause = Vec::new();
        for expr in clause {
            assigned_clause.push(expr.set_vars(assignment));
        }
        assigned_clause.iter().fold(false, |acc, opt| {
            acc || opt.unwrap_or(false)
        })
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
    pub fn resolve(&self, other: &Expr) -> Vec<i32> {
        let mut vars = Vec::new();
        let vars1 = self.to_vars_with_polarities();
        let mut vars2 = other.to_vars_with_polarities();
        for (pol, var) in vars1 {
            if !vars2.contains(&(!pol, var)) {
                if pol {
                    if !vars.contains(&var) {
                        vars.push(var);
                    }
                } else {
                    if !vars.contains(&-var) {
                        vars.push(-var);
                    }
                };
            } else {
                vars2.retain(|(pol2, var2)| (*pol2, *var2) != (!pol, var));
            }
        }
        for (pol, var) in vars2 {
            if pol {
                if !vars.contains(&var) {
                    vars.push(var);
                }
            } else {
                if !vars.contains(&-var) {
                    vars.push(-var);
                }
            };
        }
        vars
    }

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

    use crate::expr::bool_expr::Expr;

    #[test]
    fn test_is_unit() {
        let clause1 = vec![Expr::Const(true)];
        let clause2 = vec![Expr::Var(1), Expr::Var(2)];
        assert_eq!(Expr::is_unit(clause1), true);
        assert_eq!(Expr::is_unit(clause2), false);
    }

    #[test]
    fn test_remove_var() {
        let mut clause = vec![Expr::Var(1), Expr::Var(2), Expr::Var(3)];
        assert_eq!(Expr::remove_var(&mut clause, 2), true);
        assert_eq!(clause, vec![Expr::Var(1), Expr::Var(3)]);
    }

    #[test]
    fn test_find_var_idx() {
        let clause = vec![Expr::Var(1), Expr::Var(2), Expr::Var(3)];
        assert_eq!(Expr::find_var_idx(&mut clause.clone(), 2), Some(1));
        assert_eq!(Expr::find_var_idx(&mut clause.clone(), 4), None);
    }

    // Add more tests for other methods and functionality

    #[test]
    fn test_contains_var() {
        let expr = Expr::Not(Box::new(Expr::Var(5)));
        assert_eq!(expr.contains_var(5), true);
        assert_eq!(expr.contains_var(10), false);
    }

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
    fn test_find_var_idx_found() {
        let mut clause = vec![Expr::Var(1), Expr::Var(2), Expr::Var(3)];
        let index = Expr::find_var_idx(&mut clause, 2);
        assert_eq!(index, Some(1));
    }

    #[test]
    fn test_find_var_idx_not_found() {
        let mut clause = vec![Expr::Var(1), Expr::Var(2), Expr::Var(3)];
        let index = Expr::find_var_idx(&mut clause, 4);
        assert_eq!(index, None);
    }

    #[test]
    fn test_solve() {
        let clause = vec![Expr::Var(1)];
        let mut assignment = std::collections::HashMap::new();
        assignment.insert(1, true);

        let result = Expr::solve(clause, &assignment);
        assert_eq!(result, true);
    }

    #[test]
    fn test_parse_dimacs_cnf_file() {
        let result = Expr::parse_dimacs_cnf_file("/home/user/Desktop/PhD/CDCL-support-by-BDD-methods/tests/test1.cnf");
        assert!(result.is_ok());
        let dimacs = result.unwrap();
        assert_eq!(dimacs.nb_v, 50);
        assert_eq!(dimacs.nb_c, 80);
        //-83 16 65 0
        //188 1 171 0
        //23 132 -59 0

        let expressions = vec![vec![Expr::Not(Box::new(Expr::Var(83))), Expr::Var(16), Expr::Var(65)], 
        vec![Expr::Var(188), Expr::Var(1), Expr::Var(171)], vec![Expr::Var(23), Expr::Var(132), Expr::Not(Box::new(Expr::Var(59)))]];
        assert_eq!(dimacs.expressions, expressions);
    }

    #[test]
    fn test_contains_pos_var_positive() {
        let expr = Expr::Var(2);
        assert_eq!(expr.contains_pos_var(2), true);
    }

    #[test]
    fn test_contains_pos_var_negative() {
        let expr = Expr::Not(Box::new(Expr::Var(3)));
        assert_eq!(expr.contains_pos_var(3), false);
    }

    #[test]
    fn test_contains_pos_var_not_found() {
        let expr = Expr::Var(4);
        assert_eq!(expr.contains_pos_var(5), false);
    }

    #[test]
    fn test_contains_neg_var_positive() {
        let expr = Expr::Not(Box::new(Expr::Var(6)));
        assert_eq!(expr.contains_neg_var(6), true);
    }

    #[test]
    fn test_contains_neg_var_negative() {
        let expr = Expr::Var(7);
        assert_eq!(expr.contains_neg_var(7), false);
    }

    #[test]
    fn test_contains_neg_var_not_found() {
        let expr = Expr::Not(Box::new(Expr::Var(8)));
        assert_eq!(expr.contains_neg_var(9), false);
    }
}
