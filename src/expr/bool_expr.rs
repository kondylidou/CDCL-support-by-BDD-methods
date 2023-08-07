use std::{cmp::Ordering, ops::Not};

use crate::{
    bdd::Bdd, bdd_util::BddVar, expr::bool_expr::Expr::*, variable_ordering::var_ordering,
};

/// Recursive implementation of boolean expression.
#[derive(Clone, Debug, Eq, Hash, Ord)]
pub enum Expr {
    Const(bool),
    Var(i32),
    Not(Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    And(Box<Expr>, Box<Expr>),
}

impl Expr {

    pub fn new_var(var: i32) -> Expr {
        Expr::Var(var)
    }
    
    fn parse_dimacs_cnf_file(file_path: &str) -> Result<Vec<Expr>, String> {
        let content = std::fs::read_to_string(file_path).map_err(|e| e.to_string())?;
        let mut var_map: std::collections::HashMap<i32, Expr> = std::collections::HashMap::new();
        // this hashmap contains a variable and the arities of the clauses where
        // this variable is appearing.
        let mut var_clause_arities: std::collections::HashMap<i32, Vec<usize>> = std::collections::HashMap::new();
        let mut expressions: Vec<Expr> = Vec::new();
    
        for line in content.lines() {
            let tokens: Vec<&str> = line.trim().split_whitespace().collect();
            if tokens.is_empty() {
                continue; // Skip empty lines
            }
    
            match tokens[0] {
                "c" => continue, // Skip comments
                "p" => continue, // Skip the problem line, if any
                _ => {
                    let clause: Vec<i32> = tokens
                        .iter()
                        .take_while(|&&token| token != "0")
                        .map(|&token| token.parse().expect("Error parsing literal"))
                        .collect();
    
                    expressions.push(Expr::parse_clause(clause, &mut var_map, &mut var_clause_arities));
                }
            }
        }
    
        Ok(expressions)
    }
    
     /// This method parses a clause to a boolean expression.
    fn parse_clause(clause: Vec<i32>, var_map: &mut std::collections::HashMap<i32, Expr>, var_clause_arities: &mut std::collections::HashMap<i32, Vec<usize>>) -> Expr {
        if clause.len() == 1 {
            return Expr::parse_lit(clause[0], 1, var_map, var_clause_arities);
        }
        Expr::Or(
            Box::new(Expr::parse_lit(clause[0], clause.len(), var_map, var_clause_arities)),
            Box::new(Expr::parse_clause(clause[1..].to_vec(), var_map, var_clause_arities)),
        )
    }
    
    fn parse_lit(lit: i32, clause_len: usize, var_map: &mut std::collections::HashMap<i32, Expr>, var_clause_arities: &mut std::collections::HashMap<i32, Vec<usize>>) -> Expr {
        // add the clause arity to each variable appearing in this clause
        if let Some(arities) = var_clause_arities.get(&lit) {
            arities.push(clause_len);
        } else {
            var_clause_arities.insert(lit, vec![clause_len]);
        }
    
        let var_expr = var_map
            .entry(lit.abs())
            .or_insert_with(|| Expr::Var(lit.abs()));
    
        if lit < 0 {
            Expr::Not(Box::new(var_expr.clone()))
        } else {
            var_expr.clone()
        }
    }

    pub fn get_right(&self) -> Option<&Box<Expr>> {
        match self {
            Or(_, r) => Some(r),
            And(_, r) => Some(r),
            _ => None,
        }
    }

    pub fn get_left(&self) -> Option<&Box<Expr>> {
        match self {
            Or(l, _) => Some(l),
            And(l, _) => Some(l),
            _ => None,
        }
    }

    /// If a clause consists of only one literal (positive or
    /// negative), this clause is called a unit clause. We fix the
    /// valuation of an atom occurring in a unit clause to the
    /// value indicated by the sign of the literal.
    pub fn is_unit(&self) -> bool {
        match self {
            Var(_) => true,
            Not(inner) => inner.is_unit(),
            _ => false,
        }
    }

    /// This method assumes that the clause provided is
    /// indeed a unit clause and returns its variable.
    pub fn get_var_from_unit_clause(&self) -> Option<i32> {
        assert!(self.is_unit());
        match self {
            Var(v) => Some(*v),
            Not(inner) => inner.get_var_from_unit_clause(),
            _ => None,
        }
    }

    /// Size of a boolean expression.
    pub fn size(&self) -> usize {
        match self {
            Const(_) => 1,
            Var(_) => 1,
            Not(inner) => inner.size(),
            And(l, r) => l.size() + r.size(),
            Or(l, r) => l.size() + r.size(),
        }
    }

    pub fn remove_var(&self, var: i32) -> Option<Expr> {
        match self {
            Const(val) => Some(Const(*val)),
            Var(v) => {
                if !var.eq(v) {
                    return Some(Expr::Var(*v));
                }
                None
            }
            Expr::Not(inner) => {
                if let Some(inner_v) = inner.remove_var(var) {
                    return Some(Expr::Not(Box::new(inner_v)));
                }
                None
            }
            Or(l, r) => match (l.remove_var(var), r.remove_var(var)) {
                (None, None) => None,
                (None, Some(r_v)) => Some(r_v),
                (Some(l_v), None) => Some(l_v),
                (Some(l_v), Some(r_v)) => Some(Expr::Or(Box::new(l_v), Box::new(r_v))),
            },
            And(l, r) => match (l.remove_var(var), r.remove_var(var)) {
                (None, None) => None,
                (None, Some(r_v)) => Some(r_v),
                (Some(l_v), None) => Some(l_v),
                (Some(l_v), Some(r_v)) => Some(Expr::And(Box::new(l_v), Box::new(r_v))),
            },
        }
    }

    /// This method is used in unit propagation.
    /// It makes sure that the clause from which the variable
    /// will be removed is not unit and removes the given
    /// variable using the above function to remove a variable
    /// from a given Boolean Expression.
    pub fn remove_var_on_non_unit_clauses(&self, var: i32) -> Expr {
        assert!(!self.is_unit());
        self.remove_var(var).unwrap()
    }

    /* 
    /// This method creates a set of already parsed clauses.
    pub fn parse_clauses(clauses: &mut Vec<Vec<i32>>, variables: &Vec<BddVar>) -> Vec<Expr> {
        let mut clause_set: Vec<Expr> = Vec::new();

        let mut n = 0;
        while n < clauses.len() {
            clause_set.push(Expr::parse_clause(&mut clauses[n], variables));
            n += 1;
        }
        clause_set
    }

    /// This method parses a clause to a boolean expression.
    pub fn parse_clause(clause: &mut Vec<i32>, variables: &Vec<BddVar>) -> Expr {
        if clause.len() == 1 {
            return Expr::parse_var(clause[0]);
        }
        Expr::Or(
            Box::new(Expr::parse_var(clause[0])),
            Box::new(Expr::parse_clause(&mut clause[1..].to_vec(), variables)),
        )
    }

    pub fn parse_var(var: i32) -> Expr {
        if var.to_string().chars().nth(0).unwrap() == '-' {
            let val = &var.to_string()[1..];
            Expr::Not(Box::new(Var(val.parse().unwrap())))
        } else {
            Expr::Var(var)
        }
    }*/

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
            And(l, r) => Expr::Or(Box::new(l.negate()), Box::new(r.negate())),
            Or(l, r) => Expr::And(Box::new(l.negate()), Box::new(r.negate())),
        }
    }

    pub fn contains_var(&self, var: i32) -> bool {
        match self {
            Const(_) => false,
            Var(name) => {
                if name.eq(&var) {
                    true
                } else {
                    false
                }
            }
            Not(inner) => inner.contains_var(var),
            And(l, r) => l.contains_var(var) || r.contains_var(var),
            Or(l, r) => l.contains_var(var) || r.contains_var(var),
        }
    }

    pub fn contains_pos_var(&self, var: i32) -> bool {
        match self {
            Const(_) => false,
            Var(name) => {
                if name.eq(&var) {
                    true
                } else {
                    false
                }
            }
            Not(inner) => inner.contains_neg_var(var),
            And(l, r) => l.contains_pos_var(var) || r.contains_pos_var(var),
            Or(l, r) => l.contains_pos_var(var) || r.contains_pos_var(var),
        }
    }

    pub fn contains_neg_var(&self, var: i32) -> bool {
        match self {
            Const(_) => false,
            Var(_) => false,
            Not(inner) => inner.contains_pos_var(var),
            And(l, r) => l.contains_neg_var(var) || r.contains_neg_var(var),
            Or(l, r) => l.contains_neg_var(var) || r.contains_neg_var(var),
        }
    }

    pub fn to_vars_with_polarities(&self) -> Vec<(bool, i32)> {
        match self {
            Var(v) => {
                vec![(true, *v)]
            }
            Not(inner) => {
                let mut vars = Vec::new();
                for (pol, var) in inner.to_vars_with_polarities() {
                    vars.push((!pol, var));
                }
                vars
            }
            And(l, r) => {
                let mut vars_left = l.to_vars_with_polarities();
                let vars_right = r.to_vars_with_polarities();
                vars_left.extend(vars_right);
                vars_left
            }
            Or(l, r) => {
                let mut vars_left = l.to_vars_with_polarities();
                let vars_right = r.to_vars_with_polarities();
                vars_left.extend(vars_right);
                vars_left
            }
            Const(_) => Vec::new(),
        }
    }

    pub fn set_vars_and_solve(
        &self,
        assignment: &std::collections::HashMap<i32, bool>,
    ) -> Option<bool> {
        match self {
            Const(val) => Some(*val),
            Var(name) => {
                if let Some(valuet) = assignment.get(name) {
                    Some(*valuet)
                } else if let Some(valuef) = assignment.get(&-name) {
                    Some(valuef.not())
                } else {
                    None
                }
            }
            Not(inner) => match inner.set_vars_and_solve(assignment) {
                Some(val) => {
                    if val {
                        Some(false)
                    } else {
                        Some(true)
                    }
                }
                None => None,
            },
            And(l, r) => {
                let left = l.set_vars_and_solve(assignment);
                let right = r.set_vars_and_solve(assignment);
                and(left, right)
            }
            Or(l, r) => {
                let left = l.set_vars_and_solve(assignment);
                let right = r.set_vars_and_solve(assignment);
                or(left, right)
            }
        }
    }

    /// This method constructs a Robdd from a given boolean expression.
    ///
    /// *Panics*:
    /// - Variable wasn't found in variable vector.
    pub fn to_bdd(
        &self,
        variables: &Vec<BddVar>,
        ordering: &std::collections::HashMap<i32, usize>,
    ) -> Bdd {
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
            Not(inner) => inner.to_bdd(variables, ordering).negate(),
            And(l, r) => {
                let (left, right) = (l.to_bdd(variables, ordering), r.to_bdd(variables, ordering));
                left.and(&right, ordering)
            }
            Or(l, r) => {
                let (left, right) = (l.to_bdd(variables, ordering), r.to_bdd(variables, ordering));
                left.or(&right, ordering)
            }
        }
    }

    pub fn resolution(&self, other: &Expr) -> Vec<i32> {
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
    }
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Const(value) => write!(f, "{}", value),
            Var(name) => write!(f, "{}", name),
            Not(inner) => write!(f, "!{}", inner),
            And(l, r) => write!(f, "({} & {})", l, r),
            Or(l, r) => write!(f, "({} | {})", l, r),
        }
    }
}

impl PartialEq for Expr {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Const(val1), Const(val2)) => val1.eq(&val2),
            (Var(name1), Var(name2)) => name1.eq(&name2),
            (Not(in1), Not(in2)) => in1.eq(&in2),
            (And(l1, r1), And(l2, r2)) => (l1.eq(&l2)) && (r1.eq(&r2)),
            (Or(l1, r1), Or(l2, r2)) => (l1.eq(&l2)) && (r1.eq(&r2)),
            _ => false,
        }
    }
}

impl PartialOrd for Expr {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Const(v1), Const(v2)) => {
                if v1.eq(&v2) {
                    Some(Ordering::Equal)
                } else if *v1 {
                    Some(Ordering::Greater)
                } else {
                    Some(Ordering::Less)
                }
            }
            (Const(_), _) => Some(Ordering::Less),
            (_, Const(_)) => Some(Ordering::Greater),
            (Var(v1), Var(v2)) => {
                if v1.eq(&v2) {
                    Some(Ordering::Equal)
                } else if v1 < v2 {
                    Some(Ordering::Less)
                } else {
                    Some(Ordering::Greater)
                }
            }
            (Var(_), Expr::Not(_)) => Some(Ordering::Less),
            (Var(_), Or(_, _)) => Some(Ordering::Less),
            (Expr::Not(_), Var(_)) => Some(Ordering::Greater),
            (Expr::Not(in1), Expr::Not(in2)) => in1.partial_cmp(&in2),
            (Expr::Not(_), Or(_, _)) => Some(Ordering::Less),
            (Or(_, _), Var(_)) => Some(Ordering::Greater),
            (Or(_, _), Expr::Not(_)) => Some(Ordering::Greater),
            (Or(l1, r1), Or(l2, r2)) => {
                let l = l1.partial_cmp(&l2);
                let r = r1.partial_cmp(&r2);
                l.partial_cmp(&r)
            }
            (_, And(_, _)) => Some(Ordering::Less),
            (And(_, _), _) => Some(Ordering::Greater),
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
    use std::collections::HashMap;

    use crate::{
        bdd_util::BddVar, expr::bool_expr::Expr,
        variable_ordering::var_ordering_builder::BddVarOrderingBuilder,
    };

    #[test]
    pub fn test_parse_var1() {
        let var1 = -1;
        let var2 = 2;
        let var3 = 3;

        let res_var1 = Expr::Not(Box::new(Expr::Var(1)));
        let res_var2 = Expr::Var(2);
        let res_var3 = Expr::Var(3);

        let parsed_var1 = Expr::parse_var(var1);
        let parsed_var2 = Expr::parse_var(var2);
        let parsed_var3 = Expr::parse_var(var3);

        assert_eq!(parsed_var1, res_var1);
        assert_eq!(parsed_var2, res_var2);
        assert_eq!(parsed_var3, res_var3);
    }

    #[test]
    pub fn test_parse_or1() {
        let mut clause = vec![-1, 2, 3];

        let mut builder = BddVarOrderingBuilder::new();
        let mut scores = HashMap::new();
        let mut i: f64 = 0.0;
        for var in &clause {
            scores.insert(*var, i);
            i += 1.1;
        }
        let vars = builder.make_variables(clause.clone(), &scores);
        let parsed_clause = Expr::parse_clause(&mut clause, &vars);

        let var1 = Expr::Not(Box::new(Expr::Var(1)));
        let var2 = Expr::Var(2);
        let var3 = Expr::Var(3);

        let res_cla = Expr::Or(
            Box::new(var1),
            Box::new(Expr::Or(Box::new(var2), Box::new(var3))),
        );

        assert_eq!(parsed_clause, res_cla);
    }

    #[test]
    pub fn test_parse_boolexpr_to_clauses_set_1() {
        let mut clauses = Vec::new();
        let mut clause1 = vec![-1, 2, 3];
        let clause2 = vec![4, -5, 6];
        let clause3 = vec![7, 8, -9];

        clauses.push(clause1.clone());
        clauses.push(clause2.clone());
        clauses.push(clause3.clone());

        clause1.extend(clause2.iter());
        clause1.extend(clause3.iter());

        let mut builder = BddVarOrderingBuilder::new();
        let mut scores = HashMap::new();
        let mut i: f64 = 0.0;

        for clause in &clauses {
            for var in clause {
                scores.insert(*var, i);
                i += 1.1;
            }
        }
        let vars = builder.make_variables(clause1.clone(), &scores);

        let clauses_set = Expr::parse_clauses(&mut clauses, &vars);

        let var1 = Expr::Not(Box::new(Expr::Var(1)));
        let var2 = Expr::Var(2);
        let var3 = Expr::Var(3);
        let var4 = Expr::Var(4);
        let var5 = Expr::Not(Box::new(Expr::Var(5)));
        let var6 = Expr::Var(6);
        let var7 = Expr::Var(7);
        let var8 = Expr::Var(8);
        let var9 = Expr::Not(Box::new(Expr::Var(9)));

        let cla1 = Expr::Or(
            Box::new(var1),
            Box::new(Expr::Or(Box::new(var2), Box::new(var3))),
        );
        let cla2 = Expr::Or(
            Box::new(var4),
            Box::new(Expr::Or(Box::new(var5), Box::new(var6))),
        );
        let cla3 = Expr::Or(
            Box::new(var7),
            Box::new(Expr::Or(Box::new(var8), Box::new(var9))),
        );

        let res_clauses = vec![cla1, cla2, cla3];

        assert_eq!(clauses_set, res_clauses);
    }

    #[test]
    pub fn test_parse_boolexpr_to_clauses_set_2() {
        let mut clauses: Vec<Vec<i32>> = Vec::new();
        let mut clause1 = vec![-1, 2, 3, 4];
        let clause2 = vec![-5, 6];
        let clause3 = vec![7, 8, -9];
        let clause4 = vec![10, 11, 12, 13];
        let clause5 = vec![14];

        clauses.push(clause1.clone());
        clauses.push(clause2.clone());
        clauses.push(clause3.clone());
        clauses.push(clause4.clone());
        clauses.push(clause5.clone());

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
        clause1.extend(clause5.iter());

        let vars = builder.make_variables(clause1, &scores);

        let clauses_set = Expr::parse_clauses(&mut clauses, &vars);

        let var1 = Expr::Not(Box::new(Expr::Var(1)));
        let var2 = Expr::Var(2);
        let var3 = Expr::Var(3);
        let var4 = Expr::Var(4);
        let var5 = Expr::Not(Box::new(Expr::Var(5)));
        let var6 = Expr::Var(6);
        let var7 = Expr::Var(7);
        let var8 = Expr::Var(8);
        let var9 = Expr::Not(Box::new(Expr::Var(9)));
        let var10 = Expr::Var(10);
        let var11 = Expr::Var(11);
        let var12 = Expr::Var(12);
        let var13 = Expr::Var(13);
        let var14 = Expr::Var(14);

        let cla1 = Expr::Or(
            Box::new(var1),
            Box::new(Expr::Or(
                Box::new(var2),
                Box::new(Expr::Or(Box::new(var3), Box::new(var4))),
            )),
        );
        let cla2 = Expr::Or(Box::new(var5), Box::new(var6));
        let cla3 = Expr::Or(
            Box::new(var7),
            Box::new(Expr::Or(Box::new(var8), Box::new(var9))),
        );
        let cla4 = Expr::Or(
            Box::new(var10),
            Box::new(Expr::Or(
                Box::new(var11),
                Box::new(Expr::Or(Box::new(var12), Box::new(var13))),
            )),
        );
        let cla5 = var14;

        let res_clauses = vec![cla1, cla2, cla3, cla4, cla5];

        assert_eq!(clauses_set, res_clauses);
    }

    #[test]
    pub fn test_parse_boolexpr_3() {
        let mut clauses: Vec<Vec<i32>> = Vec::new();
        let clause1 = vec![-1];

        let mut builder = BddVarOrderingBuilder::new();
        let mut scores = HashMap::new();
        let mut i: f64 = 0.0;

        let mut vars: Vec<BddVar> = Vec::new();
        for clause in &clauses {
            for var in clause {
                scores.insert(*var, i);
                i += 1.1;
            }
            let mut vars_clause = builder.make_variables(clause.clone(), &scores);
            vars.append(&mut vars_clause);
        }

        clauses.push(clause1);

        let clauses_set = Expr::parse_clauses(&mut clauses, &vars);

        let var1 = Expr::Not(Box::new(Expr::Var(1)));

        let cla1 = var1;
        let res_clauses = vec![cla1];

        assert_eq!(clauses_set, res_clauses);
    }

    #[test]
    pub fn remove_variable_from_non_unit_clause() {
        let mut clauses: Vec<Vec<i32>> = Vec::new();
        let mut clause1 = vec![-1, 2, 3, 4];
        let clause2 = vec![3, 6];
        let clause3 = vec![7, -1, -9];

        clauses.push(clause1.clone());
        clauses.push(clause2.clone());
        clauses.push(clause3.clone());

        let mut builder = BddVarOrderingBuilder::new();
        let mut scores = HashMap::new();
        let mut i: f64 = 0.0;

        for clause in &clauses {
            for var in clause {
                scores.insert(*var, i);
                i += 1.1;
            }
        }

        clause1.extend(clause2);
        clause1.extend(clause3);
        clause1.sort();
        clause1.dedup();

        let vars = builder.make_variables(clause1, &scores);
        let clauses_set = Expr::parse_clauses(&mut clauses, &vars);

        let mut rem_clauses_set = Vec::new();

        for expr in clauses_set {
            rem_clauses_set.push(expr.remove_var_on_non_unit_clauses(1));
        }

        let cla1 = Expr::Or(
            Box::new(Expr::Var(2)),
            Box::new(Expr::Or(Box::new(Expr::Var(3)), Box::new(Expr::Var(4)))),
        );
        let cla2 = Expr::Or(Box::new(Expr::Var(3)), Box::new(Expr::Var(6)));
        let cla3 = Expr::Or(
            Box::new(Expr::Var(7)),
            Box::new(Expr::Not(Box::new(Expr::Var(9)))),
        );

        let mut res = Vec::new();
        res.push(cla1);
        res.push(cla2);
        res.push(cla3);

        assert_eq!(rem_clauses_set, res);
    }

    #[test]
    pub fn remove_neg_variable() {
        let mut clauses = Vec::new();
        let mut clause1 = vec![-1, 2, 3, 4];
        let clause2 = vec![3, 6];
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

        clause1.extend(clause2);
        clause1.extend(clause3);
        clause1.extend(clause4);
        clause1.sort();
        clause1.dedup();

        let vars = builder.make_variables(clause1, &scores);
        let clauses_set = Expr::parse_clauses(&mut clauses, &vars);

        let mut rem_clauses_set = Vec::new();

        for expr in clauses_set {
            if !expr.is_unit() {
                if let Some(e) = expr.remove_var(1) {
                    rem_clauses_set.push(e);
                }
            } else {
                rem_clauses_set.push(expr);
            }
        }

        let cla1 = Expr::Or(
            Box::new(Expr::Var(2)),
            Box::new(Expr::Or(Box::new(Expr::Var(3)), Box::new(Expr::Var(4)))),
        );
        let cla2 = Expr::Or(Box::new(Expr::Var(3)), Box::new(Expr::Var(6)));
        let cla3 = Expr::Or(
            Box::new(Expr::Var(7)),
            Box::new(Expr::Not(Box::new(Expr::Var(9)))),
        );
        let cla4 = Expr::Var(1);

        let mut res = Vec::new();
        res.push(cla1);
        res.push(cla2);
        res.push(cla3);
        res.push(cla4);

        assert_eq!(rem_clauses_set, res);
    }

    #[test]
    pub fn remove_pos_variable() {
        let mut clauses = Vec::new();
        let mut clause1 = vec![1, 2, 3, 4];
        let clause2 = vec![3, 6];
        let clause3 = vec![7, -9, 1];
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

        clause1.extend(clause2);
        clause1.extend(clause3);
        clause1.extend(clause4);
        clause1.sort();
        clause1.dedup();

        let vars = builder.make_variables(clause1, &scores);

        let clauses_set = Expr::parse_clauses(&mut clauses, &vars);

        let mut rem_clauses_set = Vec::new();

        for expr in clauses_set {
            if !expr.is_unit() {
                if let Some(e) = expr.remove_var(1) {
                    rem_clauses_set.push(e);
                }
            } else {
                rem_clauses_set.push(expr);
            }
        }

        let cla1 = Expr::Or(
            Box::new(Expr::Var(2)),
            Box::new(Expr::Or(Box::new(Expr::Var(3)), Box::new(Expr::Var(4)))),
        );
        let cla2 = Expr::Or(Box::new(Expr::Var(3)), Box::new(Expr::Var(6)));
        let cla3 = Expr::Or(
            Box::new(Expr::Var(7)),
            Box::new(Expr::Not(Box::new(Expr::Var(9)))),
        );
        let cla4 = Expr::Not(Box::new(Expr::Var(1)));

        let mut res = Vec::new();
        res.push(cla1);
        res.push(cla2);
        res.push(cla3);
        res.push(cla4);

        assert_eq!(rem_clauses_set, res);
    }

    #[test]
    pub fn test_to_variables() {
        let mut clauses = Vec::new();
        let mut clause1 = vec![-1, 2, 3, 4];
        let clause2 = vec![-5, 6];
        let clause3 = vec![7, 8, -9];
        let clause4 = vec![10, 11, 12, 13];
        let clause5 = vec![14];

        clauses.push(clause1.clone());
        clauses.push(clause2.clone());
        clauses.push(clause3.clone());
        clauses.push(clause4.clone());
        clauses.push(clause5.clone());

        clause1.extend(clause2.iter());
        clause1.extend(clause3.iter());
        clause1.extend(clause4.iter());
        clause1.extend(clause5.iter());

        let mut builder = BddVarOrderingBuilder::new();
        let mut scores = HashMap::new();
        let mut i: f64 = 0.0;

        for clause in &clauses {
            for var in clause {
                scores.insert(*var, i);
                i += 1.1;
            }
        }
        let vars = builder.make_variables(clause1.clone(), &scores);

        let clauses_set = Expr::parse_clauses(&mut clauses, &vars);

        let mut vars_set = Vec::new();

        for expr in clauses_set {
            let vars = expr.to_vars_with_polarities();
            vars_set.extend(vars);
        }

        let res = vec![
            (false, 1),
            (true, 2),
            (true, 3),
            (true, 4),
            (false, 5),
            (true, 6),
            (true, 7),
            (true, 8),
            (false, 9),
            (true, 10),
            (true, 11),
            (true, 12),
            (true, 13),
            (true, 14),
        ];

        assert_eq!(res, vars_set);
    }

    #[test]
    pub fn contains_pos_var() {
        let cla1 = Expr::Var(1);
        let cla2 = Expr::Not(Box::new(Expr::Var(1)));
        let cla3 = Expr::Not(Box::new(Expr::Not(Box::new(Expr::Var(1)))));
        let cla4 = Expr::Or(
            Box::new(Expr::Var(1)),
            Box::new(Expr::Or(
                Box::new(Expr::Var(2)),
                Box::new(Expr::Or(Box::new(Expr::Var(3)), Box::new(Expr::Var(4)))),
            )),
        );
        let cla5 = Expr::Not(Box::new(Expr::Or(
            Box::new(Expr::Var(1)),
            Box::new(Expr::Or(
                Box::new(Expr::Var(2)),
                Box::new(Expr::Or(Box::new(Expr::Var(3)), Box::new(Expr::Var(4)))),
            )),
        )));
        let cla6 = Expr::Not(Box::new(Expr::Or(
            Box::new(Expr::Var(1)),
            Box::new(Expr::Var(4)),
        )));
        let cla7 = Expr::Or(
            Box::new(Expr::Not(Box::new(Expr::Var(1)))),
            Box::new(Expr::Var(4)),
        );
        let cla8 = Expr::Or(
            Box::new(Expr::Not(Box::new(Expr::Not(Box::new(Expr::Var(1)))))),
            Box::new(Expr::Var(4)),
        );
        let cla9 = Expr::Not(Box::new(Expr::Or(
            Box::new(Expr::Not(Box::new(Expr::Var(1)))),
            Box::new(Expr::Var(4)),
        )));

        assert!(cla1.contains_pos_var(1));
        assert!(!cla1.contains_pos_var(2));
        assert!(!cla2.contains_pos_var(1));
        assert!(cla3.contains_pos_var(1));
        assert!(cla4.contains_pos_var(1));
        assert!(!cla5.contains_pos_var(1));
        assert!(!cla6.contains_pos_var(1));
        assert!(!cla7.contains_pos_var(1));
        assert!(cla8.contains_pos_var(1));
        assert!(cla9.contains_pos_var(1));
    }

    #[test]
    pub fn contains_neg_var() {
        let cla1 = Expr::Var(1);
        let cla2 = Expr::Not(Box::new(Expr::Var(1)));
        let cla3 = Expr::Not(Box::new(Expr::Not(Box::new(Expr::Var(1)))));
        let cla4 = Expr::Or(
            Box::new(Expr::Var(1)),
            Box::new(Expr::Or(
                Box::new(Expr::Var(2)),
                Box::new(Expr::Or(Box::new(Expr::Var(3)), Box::new(Expr::Var(4)))),
            )),
        );
        let cla5 = Expr::Not(Box::new(Expr::Or(
            Box::new(Expr::Var(1)),
            Box::new(Expr::Or(
                Box::new(Expr::Var(2)),
                Box::new(Expr::Or(Box::new(Expr::Var(3)), Box::new(Expr::Var(4)))),
            )),
        )));
        let cla6 = Expr::Not(Box::new(Expr::Or(
            Box::new(Expr::Var(1)),
            Box::new(Expr::Var(4)),
        )));
        let cla7 = Expr::Or(
            Box::new(Expr::Not(Box::new(Expr::Var(1)))),
            Box::new(Expr::Var(4)),
        );
        let cla8 = Expr::Or(
            Box::new(Expr::Not(Box::new(Expr::Not(Box::new(Expr::Var(1)))))),
            Box::new(Expr::Var(4)),
        );
        let cla9 = Expr::Not(Box::new(Expr::Or(
            Box::new(Expr::Not(Box::new(Expr::Var(1)))),
            Box::new(Expr::Var(4)),
        )));

        assert!(!cla1.contains_neg_var(1));
        assert!(!cla1.contains_neg_var(2));
        assert!(cla2.contains_neg_var(1));
        assert!(!cla3.contains_neg_var(1));
        assert!(!cla4.contains_neg_var(1));
        assert!(cla5.contains_neg_var(1));
        assert!(cla6.contains_neg_var(1));
        assert!(cla7.contains_neg_var(1));
        assert!(!cla8.contains_neg_var(1));
        assert!(!cla9.contains_neg_var(1));
    }
}
