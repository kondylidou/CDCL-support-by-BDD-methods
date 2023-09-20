use std::collections::HashSet;
use std::fmt;
use std::hash::{Hash, Hasher};

use crate::{bdd::Bdd, bdd_util::BddVar, expr::bool_expr::Expr::*};

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
            },
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
            Const(value) => Bdd::new_value(BddVar::new(i32::MAX), value),
            Var(name) => {
                if let Some(pos) = variables.iter().position(|i| i.name.eq(name)) {
                    Bdd::new_var(variables[pos])
                } else {
                    panic!("Variable {} doesn't exists.", name);
                }
            }
            Not(inner) => inner.to_bdd(variables, ordering).negate(),
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Const(value) => write!(f, "{}", value),
            Expr::Var(name) => write!(f, "{}", name),
            Expr::Not(inner) => write!(f, "!{}", inner),
        }
        .map_err(|_| fmt::Error) // Convert anyhow::Error to fmt::Error
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
#[repr(C)]
pub struct Clause {
    pub literals: HashSet<Expr>,
}

impl Clause {
    pub fn is_empty(&self) -> bool {
        self.literals.is_empty()
    }

    pub fn size(&self) -> usize {
        self.literals.len()
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

    // This method helps the process of bucket elimination.
    // It finds the the highest order variable in the clause
    // to be able to sort it afterwards in the correct bucket
    // indexing this variable.
    // Method to get the highest variable index within the bucket
    pub fn get_highest_scored_var(&self, ordering: &std::collections::HashMap<i32, usize>) -> Option<i32> {
        self
        .literals
        .iter()
        .map(|lit| lit.get_var_name())
        .max_by_key(|var| ordering.get(var).unwrap())
    }

    pub fn to_bdd(
        &self,
        variables: &Vec<BddVar>,
        ordering: &std::collections::HashMap<i32, usize>,
    ) -> Bdd {
        let mut bdd = self
            .literals
            .iter()
            .next()
            .unwrap()
            .to_bdd(&variables, &ordering);

        for expr in self.literals.iter().skip(1) {
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
        self.literals
            .iter()
            .any(|expr| expr.contains_pos_var(index))
    }

    pub fn solve(&self, assignment: &std::collections::HashMap<i32, bool>) -> bool {
        let mut assigned_clause = Vec::new();
        for expr in &self.literals {
            assigned_clause.push(expr.set_vars(assignment));
        }
        assigned_clause
            .iter()
            .fold(false, |acc, opt| acc || opt.unwrap_or(false))
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

    use crate::bdd_util::{BddNode, BddPointer};

    use super::*;

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

        let mut res = Bdd::new();
        res.nodes.push(BddNode::mk_node(
            BddVar { name: 2 },
            BddPointer::new_one(),
            BddPointer::new_zero(),
        ));
        assert_eq!(bdd, res)
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
        let assignment: std::collections::HashMap<i32, bool> =
            [(1, true), (2, false), (3, true)].iter().cloned().collect();

        let clause = Clause {
            literals: HashSet::from_iter(vec![
                Expr::Var(1),
                Expr::Not(Box::new(Expr::Var(2))),
                Expr::Var(3),
            ]),
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
        let expected_literals =
            HashSet::from_iter(vec![Expr::Var(1), Expr::Var(2), Expr::Var(3), Expr::Var(4)]);
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

        let expected_literals = HashSet::from_iter(vec![Expr::Var(1), Expr::Var(2), Expr::Var(3)]);
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
