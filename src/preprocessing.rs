use crate::expr::bool_expr::Expr;

/// If a set of clauses contains the unit clause l, the other clauses are simplified
/// by the application of the two following rules:
///
/// 1.every clause (other than the unit clause itself) containing l is removed
/// (the clause is satisfied if l is);
///
/// in every clause that contains ¬ l this literal is deleted
/// ( ¬ l can not contribute to it being satisfied).
pub fn unit_propagation(expressions: Vec<Expr>, unit_clauses: Vec<Expr>) -> Vec<Expr> {
    let mut tmp: Vec<Expr> = expressions.to_vec();
    for unit_clause in unit_clauses {
        let var = unit_clause.get_var_from_unit_clause().unwrap();

        if unit_clause.contains_pos_var(var) {
            // delete all the clauses containing the variable with positive polarity
            tmp.drain_filter(|e| e.contains_pos_var(var) && !e.is_unit());

            // remove the variable with negative polarity from the rest of the clauses
            tmp = tmp
                .iter()
                .map(|e| {
                    if !e.is_unit() {
                        e.remove_var_on_non_unit_clauses(var)
                    } else {
                        e.clone()
                    }
                })
                .collect();

        } else {
            // delete all the clauses containing the variable with positive polarity
            tmp.drain_filter(|e| e.contains_neg_var(var) && !e.is_unit());

            // Remove the negation of the unit literal from all clauses
            tmp.iter().for_each(|expr| if !expr.is_unit() {
                expr.remove_var_on_non_unit_clauses(var);
            });
            
            // remove the variable with positive polarity from the rest of the clauses
            tmp = tmp
                .iter()
                .map(|e| {
                    if !e.is_unit() {
                        e.remove_var_on_non_unit_clauses(var)
                    } else {
                        e.clone()
                    }
                })
                .collect();
        }
        if !tmp.contains(&unit_clause) {
            tmp.push(unit_clause);
        }
    }
    tmp
}