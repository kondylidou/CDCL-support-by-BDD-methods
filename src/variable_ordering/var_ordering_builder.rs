use crate::bdd_util::BddVar;
use crate::expr::bool_expr::Expr;
use crate::parser::Dimacs;
use crate::variable_ordering::var_ordering::BddVarOrdering;

#[derive(Clone, Debug)]
pub struct BddVarOrderingBuilder {
    var_names: Vec<i32>,
    var_names_set: std::collections::HashSet<i32>,
}

impl BddVarOrderingBuilder {
    /// Create a new builder without any variables.
    pub fn new() -> BddVarOrderingBuilder {
        BddVarOrderingBuilder {
            var_names: Vec::new(),
            var_names_set: std::collections::HashSet::new(),
        }
    }

    /// Create a new variable with the given `name`. Returns a `BddVar`+
    /// instance that can be later used to create and query actual BDDs.
    ///
    /// *Panics*:
    ///  - Each variable name has to be unique.
    pub fn make_variable(&mut self, name: i32) -> BddVar {
        if self.var_names_set.contains(&name) {
            panic!("BDD variable {} already exists.", name);
        }
        self.var_names_set.insert(name);
        self.var_names.push(name);
        BddVar { name }
    }

    /// Similar to `make_variable`, but allows creating multiple variables at the same time.
    pub fn make_variables(&mut self, var_map: std::collections::HashMap<i32, Expr>) -> Vec<BddVar> {
        let mut variables = Vec::new();
        for (var_name, _) in var_map {
            variables.push(self.make_variable(var_name));
        }
        variables
    }

    /// Convert this builder to an actual variable ordering.
    /// The variables are sorted in decreasing order according to the score,
    /// so that higher-scoring variables
    /// (that is, variables that appear in many mostly short clauses)
    /// correspond to layers nearer the top of the BDD.
    pub fn make(&mut self, dimacs: Dimacs) -> BddVarOrdering {
        let variables = self.make_variables(dimacs.var_map);

        let mut ordering: std::collections::HashMap<i32, usize> = std::collections::HashMap::new();

        let mut v: Vec<_> = dimacs.vars_scores.iter().collect();
        // v is a sorted vector in decreasing order according to the scores
        v.sort_by(|x, y| BddVarOrderingBuilder::var_dec_cmp(&x.1, &y.1));

        let mut idx = v.len();
        ordering.insert(i32::MAX, idx);
        for (var, _) in v.into_iter().rev() {
            idx -= 1;
            ordering.insert(*var, idx);
        }

        BddVarOrdering {
            variables,
            expressions: dimacs.expressions,
            ordering,
        }
    }

    fn var_dec_cmp(x: &f64, y: &f64) -> std::cmp::Ordering {
        if x.eq(&y) {
            std::cmp::Ordering::Equal
        } else if x < y {
            std::cmp::Ordering::Greater
        } else {
            std::cmp::Ordering::Less
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        init_glucose_solver, parser, sharing::sharing_manager::SharingManager,
        variable_ordering::var_ordering::BddVarOrdering, GlucoseWrapper,
    };

    #[test]
    fn variable_ordering() {
        let dimacs = parser::parse_dimacs_cnf_file("tests/test3.cnf").unwrap();
        // build the solver
        let solver = init_glucose_solver();
        let glucose = GlucoseWrapper::new(solver);
        // build the sharing manager
        let mut sharing_manager = SharingManager::new(glucose);
        // build the variable ordering
        let var_ordering = BddVarOrdering::new(dimacs);
        println!("{:?}", var_ordering.ordering);

        let mut var_index_mapping: std::collections::HashMap<i32, usize> =
            std::collections::HashMap::new();
        var_index_mapping.insert(1, 0);
        var_index_mapping.insert(2, 1);
        var_index_mapping.insert(3, 2);
        var_index_mapping.insert(4, 3);
        var_index_mapping.insert(5, 4);
        var_index_mapping.insert(i32::MAX, 5);

        let bdd = var_ordering.build_bdd(&mut sharing_manager);
        println!("{:?}", bdd);

        assert_eq!(var_index_mapping, var_ordering.ordering);
    }
}
