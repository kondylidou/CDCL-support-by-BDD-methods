use crate::bdd_util::{BddNode, BddPointer, BddVar};
use crate::expr::bool_expr;
use anyhow::{anyhow, Result};
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::iter::Map;
use std::ops::Range;

// The Bdd receives the clauses 'Vec<Vec<i32>>'. They can be viewed as a boolean
// expression for example (x1 OR x2) AND (NOT x1 OR x2). Then the INF (if then else normalform)
// needs to be found for this expression so that the Bdd can be constructed.

#[derive(Clone, Debug)]
pub struct Bdd {
    pub nodes: Vec<BddNode>,
    // cache for memoization
    cache: HashMap<(BddPointer, BddPointer), BddPointer>,
}

impl Bdd {
    /// Create a new empty Bdd. The terminal pointers are
    /// inserted into the vector of nodes.
    pub fn new() -> Bdd {
        let mut nodes = Vec::new();
        // Maximum number as pointer as in the apply method always the smaller var is
        // selected and we want to replace these nodes.
        let max_ptr = BddVar::new(i32::MAX, 0.0);
        nodes.push(BddNode::mk_zero(max_ptr));
        nodes.push(BddNode::mk_one(max_ptr));
        Bdd {
            nodes,
            cache: HashMap::new(),
        }
    }

    fn new_with_capacity(cap: usize) -> Bdd {
        let mut nodes = Vec::with_capacity(cap);
        // Maximum number as pointer as in the apply method always the smaller var is
        // selected and we want to replace these nodes.
        let max_ptr = BddVar::new(i32::MAX, 0.0);
        nodes.push(BddNode::mk_zero(max_ptr));
        nodes.push(BddNode::mk_one(max_ptr));
        Bdd {
            nodes,
            cache: HashMap::new(),
        }
    }

    fn is_full(&self) -> bool {
        self.nodes.capacity().eq(&self.nodes.len())
    }

    fn is_empty(&self) -> bool {
        self.size().eq(&2)
    }

    /// Get the variable of a specific pointer in the Bdd.
    pub fn var_of_ptr(&self, ptr: BddPointer) -> BddVar {
        self.nodes[ptr.to_index()].var
    }

    // Get a pointer to the BDD node with the specified variable
    fn ptr_of_node_with_var_name(&self, var_name: i32) -> Option<BddPointer> {
        for ptr in self.indices() {
            if self.var_of_ptr(ptr).name.eq(&var_name) {
                return Some(ptr);
            }
        }
        None
    }

    /// Insert a node into the vector of nodes of the Bdd.
    fn push_node(&mut self, node: BddNode) {
        self.nodes.push(node);
    }

    /// Create a new Bdd from a variable and connect it to terminal pointers 0 and 1.
    pub fn new_var(var: BddVar) -> Bdd {
        let mut bdd = Bdd::new();
        bdd.push_node(BddNode::mk_node(
            var,
            BddPointer::new_zero(),
            BddPointer::new_one(),
        ));
        bdd
    }

    /// Create a new Bdd from a boolean value.
    pub fn new_value(var: BddVar, value: &bool) -> Bdd {
        if *value {
            Bdd::new_true(var)
        } else {
            Bdd::new_false(var)
        }
    }

    /// Create a new Bdd for the false formula.
    fn new_false(var: BddVar) -> Bdd {
        let mut bdd = Bdd::new();
        bdd.push_node(BddNode::mk_zero(var));
        bdd
    }

    /// Create a new Bdd for the true formula.
    fn new_true(var: BddVar) -> Bdd {
        let mut bdd = Bdd::new();
        bdd.push_node(BddNode::mk_zero(var));
        bdd.push_node(BddNode::mk_one(var));
        bdd
    }

    /// Create a new Bdd for a negated variable.
    fn new_not_var(var: BddVar) -> Bdd {
        let mut bdd = Bdd::new();
        bdd.push_node(BddNode::mk_node(
            var,
            BddPointer::new_one(),
            BddPointer::new_zero(),
        ));
        bdd
    }

    /*
    /// This method creates a `Bdd` by merging two `Bdd`s based on the concept of resolution.
    pub fn resolve(
        &mut self,
        other: &Bdd,
        ordering: &std::collections::HashMap<i32, usize>,
    ) -> Bdd {
        self.apply_resolution(other, ordering)
    }
    */

    /// This method creates a `Bdd` corresponding to the $\phi \land \psi$ formula, where $\phi$ and $\psi$
    /// are the two given `Bdd`s.
    pub fn and(&mut self, other: &Bdd, ordering: &std::collections::HashMap<i32, usize>) -> Bdd {
        self.apply(other, bool_expr::and, ordering)
    }

    /// This method creates a `Bdd` corresponding to the $\phi \lor \psi$ formula, where $\phi$ and $\psi$
    /// are the two given `Bdd`s.
    pub fn or(&mut self, other: &Bdd, ordering: &std::collections::HashMap<i32, usize>) -> Bdd {
        self.apply(other, bool_expr::or, ordering)
    }

    /// Negate a Bdd.
    pub fn negate(&mut self) -> Bdd {
        if self.is_true() {
            Bdd::new_false(BddVar::new(i32::MAX, 0.0))
        } else if self.is_false() {
            Bdd::new_true(BddVar::new(i32::MAX, 0.0))
        } else {
            let mut nodes = self.nodes.clone();
            for node in nodes.iter_mut().skip(2) {
                // skip terminals
                node.high.flip_if_terminal();
                node.low.flip_if_terminal();
            }
            Bdd {
                nodes,
                cache: HashMap::new(),
            }
        }
    }

    /// The number of nodes in a Bdd.
    pub fn size(&self) -> usize {
        self.nodes.len()
    }

    /// True if a Bdd is exactly the true formula.
    fn is_true(&self) -> bool {
        self.nodes.len() == 2
    }

    /// True if a Bdd is exactly the false formula.
    fn is_false(&self) -> bool {
        self.nodes.len() == 1
    }

    /// Get the pointer of the root node of the Bdd.
    pub fn root_pointer(&self) -> BddPointer {
        if self.is_false() {
            BddPointer::new_zero()
        } else if self.is_true() {
            BddPointer::new_one()
        } else {
            BddPointer::new(self.nodes.len() - 1)
        }
    }

    pub fn indices(&self) -> Map<Range<usize>, fn(usize) -> BddPointer> {
        (0..self.size()).map(BddPointer::new)
    }

    pub fn low_node_ptr(&self, ptr: BddPointer) -> BddPointer {
        self.nodes[ptr.to_index()].low
    }

    fn replace_low(&mut self, ptr: BddPointer, new_ptr: BddPointer) {
        self.nodes[ptr.to_index()].low = new_ptr
    }

    pub fn high_node_ptr(&self, ptr: BddPointer) -> BddPointer {
        self.nodes[ptr.to_index()].high
    }

    fn replace_high(&mut self, ptr: BddPointer, new_ptr: BddPointer) {
        self.nodes[ptr.to_index()].high = new_ptr
    }

    fn delete_node(&mut self, to_delete: BddPointer, node_path: Vec<(BddPointer, bool)>) {
        self.nodes.remove(to_delete.to_index());
        // the path until the node to delete was reached
        for (node, assign) in node_path.into_iter().skip(1) {
            // skip the first one as it was already assigned
            if assign {
                // if true then decrement the high nodes
                self.replace_high(
                    node,
                    BddPointer {
                        index: (self.high_node_ptr(node).index - 1),
                    },
                );
            } else {
                // if false then decrement the low nodes
                self.replace_low(
                    node,
                    BddPointer {
                        index: (self.low_node_ptr(node).index - 1),
                    },
                );
            }
        }
    }

    fn replace_node(&mut self, to_delete: BddPointer, replacement: BddPointer) {
        self.nodes.remove(to_delete.to_index());
        for ptr in self.indices() {
            if self.low_node_ptr(ptr).eq(&to_delete) {
                self.replace_low(ptr, replacement);
            } else if self.high_node_ptr(ptr).eq(&to_delete) {
                self.replace_high(ptr, replacement);
            }
        }
    }

    /* TODO to fix
    /// Convert this `Bdd` to a `BooleanExpression`.
    pub fn to_clause_vector(&self) -> Vec<Clause> {
        if self.is_false() {
            return vec![Clause { literals: HashSet::from_iter(vec![Expr::Const(false)]) }];
        }
        if self.is_true() {
            return vec![Clause { literals: HashSet::from_iter(vec![Expr::Const(true)]) }];
        }

        let mut res: Vec<Clause> = Vec::with_capacity(self.nodes.len());
        for node in 2..self.nodes.len() { // skip terminals
            // skip terminals
            let bdd_var = self.nodes[node].var;
            let var_name = bdd_var.name;

            let low = self.nodes[node].low;
            let high = self.nodes[node].high;

            let clause = if low.is_terminal() && high.is_terminal() {
                // variable
                if low.is_zero() && high.is_one() {
                    Expr::Var(var_name)
                } else if high.is_zero() && low.is_one() {
                    Expr::Not(Box::new(Expr::Var(var_name)))
                } else {
                    panic!("Invalid node {:?} in bdd {:?}.", self.0[node], self.0);
                }
            } else if low.is_terminal() {
                if low.is_zero() {
                    // a & high
                    Expr::And(
                        Box::new(Expr::Var(var_name)),
                        Box::new(res[high.0 as usize].clone()),
                    )
                } else {
                    // !a | high
                    Expr::Or(
                        Box::new(Expr::Not(Box::new(Expr::Var(var_name)))),
                        Box::new(res[high.0 as usize].clone()),
                    )
                }
            } else if high.is_terminal() {
                if high.is_zero() {
                    // !a & low
                    Expr::And(
                        Box::new(Expr::Not(Box::new(Expr::Var(var_name)))),
                        Box::new(res[low.0 as usize].clone()),
                    )
                } else {
                    // a | low
                    Expr::Or(
                        Box::new(Expr::Var(var_name)),
                        Box::new(res[low.0 as usize].clone()),
                    )
                }
            } else {
                // (a | high) & (!a | low)
                Expr::And(
                    Box::new(Expr::Or(
                        Box::new(Expr::Var(var_name)),
                        Box::new(res[high.0 as usize].clone()),
                    )),
                    Box::new(Expr::Or(
                        Box::new(Expr::Not(Box::new(Expr::Var(var_name)))),
                        Box::new(res[low.0 as usize].clone()),
                    )),
                )
            };
            res.push(expr);
        }

        res.last().unwrap().clone()
    }
    */

    /// This method merges two Bdds
    fn apply<T>(
        &mut self,
        other: &Bdd,
        op: T,
        ordering: &std::collections::HashMap<i32, usize>,
    ) -> Bdd
    where
        T: Fn(Option<bool>, Option<bool>) -> Option<bool>,
    {
        let mut bdd = Bdd::new();

        // In order to ensure that the Obdd being constructed is reduced,
        // it is necessary to determine from a triple (i,l,h) whether there
        // exists a node u with var(u) = i, low(u) = l and high(u) = h.
        // For this purpose we assume the presence of a table H:(i,l,h) -> u
        // mapping triples (i,h,l) of variables indices i and nodes l and h to u.
        // The table H is the "inverse" of the table T of variable nodes u.
        // T(u) = (i,l,h) if and only if H(i,l,h) = u.

        // We keep track of a nodes_map so that there are no duplicates
        let mut nodes_map: std::collections::HashMap<BddNode, BddPointer> =
            std::collections::HashMap::with_capacity(std::cmp::max(self.size(), other.size()));
        nodes_map.insert(
            BddNode::mk_zero(BddVar::new(i32::MAX, 0.0)),
            BddPointer::new_zero(),
        );
        nodes_map.insert(
            BddNode::mk_one(BddVar::new(i32::MAX, 0.0)),
            BddPointer::new_one(),
        );

        // Task is a pair of pointers into the `left` and `right` BDDs.
        #[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
        struct Task {
            left: BddPointer,
            right: BddPointer,
        }

        // We keep track of the tasks currently on stack so that we build the bdd from down to the top
        let mut stack: Vec<Task> = Vec::with_capacity(std::cmp::max(self.size(), other.size()));

        stack.push(Task {
            left: self.root_pointer(),
            right: other.root_pointer(),
        });

        // We keep track of the tasks already completed, so that we can access the pointers
        let mut finished_tasks: std::collections::HashMap<Task, BddPointer> =
            std::collections::HashMap::with_capacity(std::cmp::max(self.size(), other.size()));

        while let Some(current) = stack.last() {
            if finished_tasks.contains_key(current) {
                stack.pop();
            } else {
                let cached_result = self.cache.get(&(current.left, current.right)).cloned();
                if let Some(result) = cached_result {
                    finished_tasks.insert(*current, result);
                    stack.pop();
                } else {
                    let (lft, rgt) = (current.left, current.right);
                    // find the lowest variable of the two nodes
                    let (l_var, r_var) = (self.var_of_ptr(lft), other.var_of_ptr(rgt));

                    // The min variable is now the one with the higher score, so
                    // the smallest index in the mapping
                    let l_var_index = ordering.get(&l_var.name).unwrap_or(&usize::MAX);
                    let r_var_index = ordering.get(&r_var.name).unwrap_or(&usize::MAX);
                    let min_var = if l_var_index < r_var_index {
                        l_var
                    } else {
                        r_var
                    };

                    // If the nodes have the same index the two low branches are paired
                    // and apply recursively computed on them. Similarly for the high branches.
                    // If they have different indices we proceed by pairing the node
                    // with lowest index with the low- and high- branches of the other.
                    let (l_low, l_high) = if l_var.eq(&min_var) {
                        (self.low_node_ptr(lft), self.high_node_ptr(lft))
                    } else {
                        (lft, lft)
                    };
                    let (r_low, r_high) = if l_var == r_var || r_var.eq(&min_var) {
                        (other.low_node_ptr(rgt), other.high_node_ptr(rgt))
                    } else {
                        (rgt, rgt)
                    };

                    // Two tasks which correspond to the two recursive sub-problems we need to solve.
                    let sub_left = Task {
                        left: l_low,
                        right: r_low,
                    };
                    let sub_right = Task {
                        left: l_high,
                        right: r_high,
                    };

                    let new_low: Option<BddPointer> = op(l_low.as_bool(), r_low.as_bool())
                        .map(BddPointer::from_bool)
                        .or(finished_tasks.get(&sub_left).cloned());

                    let new_high: Option<BddPointer> = op(l_high.as_bool(), r_high.as_bool())
                        .map(BddPointer::from_bool)
                        .or(finished_tasks.get(&sub_right).cloned());

                    if let (Some(new_low), Some(new_high)) = (new_low, new_high) {
                        if new_low == new_high {
                            finished_tasks.insert(*current, new_low);
                        } else {
                            let node = BddNode::mk_node(min_var, new_low, new_high);
                            if let Some(idx) = nodes_map.get(&node) {
                                // Node already exists, just make it a result of this computation.
                                finished_tasks.insert(*current, *idx);
                            } else {
                                // Node does not exist, it needs to be pushed to result.
                                bdd.push_node(node);
                                nodes_map.insert(node, bdd.root_pointer());
                                finished_tasks.insert(*current, bdd.root_pointer());
                            }
                        }
                        // Cache the result
                        self.cache
                            .insert((current.left, current.right), bdd.root_pointer());
                        finished_tasks.insert(*current, bdd.root_pointer());

                        // If both values are computed, mark this task as resolved.
                        stack.pop();
                    } else {
                        // add the subtasks to stack
                        if new_low.is_none() {
                            stack.push(sub_left);
                        }
                        if new_high.is_none() {
                            stack.push(sub_right);
                        }
                    }
                }
            }
        }
        bdd
    }

    fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn find_terminal_nodes_conflicts(&self) -> Vec<(bool, BddPointer)> {
        let mut terminal_nodes: Vec<(bool, BddPointer)> = Vec::new();

        for ptr in self.indices() {
            if ptr.is_terminal() {
                continue;
            }
            if self.low_node_ptr(ptr) == BddPointer::new_zero() {
                terminal_nodes.push((false, ptr));
            }
            if self.high_node_ptr(ptr) == BddPointer::new_zero() {
                terminal_nodes.push((true, ptr));
            }
        }

        terminal_nodes
    }

    /// Check if the Bdd is satisfiable and if its the case return
    /// the satisfiable assignment in a vector of bool.
    fn solve(&self, variables: &Vec<BddVar>) -> Result<HashMap<i32, bool>> {
        // If the Bdd is false return None.
        if self.is_false() {
            return Err(anyhow!("The problem is not solvable!"));
        }
        // Initialise the final assignment with a capacity of the total number of variables.
        let mut assignment: HashMap<i32, bool> = HashMap::with_capacity(variables.len() as usize);
        let mut acc = BddPointer::new_one();

        // Search the Bdd backwards starting from the one pointer.
        for ptr in self.indices() {
            if ptr.is_terminal() {
                // skip the terminal nodes
                continue;
            }
            if self.low_node_ptr(ptr) == acc {
                // push front as we go backwards and assign the variables
                // from the last to the first.
                let var = self.var_of_ptr(ptr).name;
                assignment.insert(var, false);
                acc = ptr;
            }
            if self.high_node_ptr(ptr) == acc {
                let var = self.var_of_ptr(ptr).name;
                assignment.insert(var, true);
                // save the new pointer in the accumulator.
                acc = ptr;
            }
        }

        Ok(assignment)
    }

    fn count_edges(&self, pointer: BddPointer, visited: &mut HashSet<usize>) -> usize {
        if visited.contains(&pointer.to_index()) {
            return 0; // Already visited, return zero edges
        }
        visited.insert(pointer.to_index());

        let node = &self.nodes[pointer.to_index()];
        let mut edges = 0;

        // Count outgoing edges to high and low nodes
        edges += self.count_edges(node.low, visited);
        edges += self.count_edges(node.high, visited);

        edges + 1 // Add one for the current edge
    }

    // Calculate the score for the current variable order using the NEC heuristic
    pub fn calculate_nec_score(&self, ordering: &HashMap<i32, usize>) -> f64 {
        let mut nec_score = 0.0;
        let mut visited = HashSet::new();

        for (index, node) in self.nodes.iter().enumerate() {
            let edges = self.count_edges(BddPointer::new(index), &mut visited);
            let var = node.var;

            if let Some(&order) = ordering.get(&var.name) {
                nec_score += (order as f64) * (edges as f64);
            }
        }

        nec_score
    }

     // Perform sifting variable reordering using the NEC scoring metric
     fn sift_variables_nec(&mut self, ordering: &mut HashMap<i32, usize>, variables: &Vec<BddVar>) {
        // Calculate NEC current score
        let mut current_score = self.calculate_nec_score(&ordering);

        // Create a Vec of keys for iteration
        let keys: Vec<i32> = ordering
            .keys()
            .filter(|&var| !var.eq(&i32::MAX))
            .cloned()
            .collect();

        for (i, &var_i) in keys.iter().enumerate() {
            for (_, &var_j) in keys.iter().enumerate().skip(i + 1) {
                // Clone the ordering to make modifications
                let mut new_ordering = ordering.clone();

                // Swap variable positions
                new_ordering.insert(var_i, *ordering.get(&var_j).unwrap());
                new_ordering.insert(var_j, *ordering.get(&var_i).unwrap());

                // Calculate the score for the current variable order using the NEC heuristic
                let score = self.calculate_nec_score(&new_ordering);

                if score < current_score {
                    current_score = score;
                    // Update the original ordering with the modified new_ordering
                    ordering.insert(var_i, *ordering.get(&var_j).unwrap());
                    ordering.insert(var_j, *ordering.get(&var_i).unwrap());
                    self.partial_reorder_bdd(&variables, &ordering); // Reorder the BDD nodes
                }
            }
        }
    }


    /// Reorder the BDD nodes based on the given BddVarOrdering

    /// Reordering variables in a Binary Decision Diagram (BDD) doesn't inherently make the BDD smaller in terms of the number of nodes or its overall size.
    /// Instead, the primary goal of variable reordering is to potentially improve the performance and efficiency of BDD operations,
    /// such as BDD minimization, traversal, and manipulation.
    pub fn partial_reorder_bdd(&mut self, affected_vars: &Vec<BddVar>, new_ordering: &HashMap<i32, usize>) {
        self.clear_cache();
        let mut nodes_map: HashMap<BddPointer, BddPointer> = HashMap::new();
        let mut sorted_nodes: Vec<_> = self.nodes.iter().enumerate().skip(2).collect();

        // Sort nodes based on the new variable order
        sorted_nodes.sort_by(|(_, node1), (_, node2)| {
            let var1_index = new_ordering.get(&node1.var.name).unwrap();
            let var2_index = new_ordering.get(&node2.var.name).unwrap();
            var2_index.cmp(var1_index)
        });

        // Update the BDD nodes' pointers based on the new mapping
        let mut new_nodes = Vec::with_capacity(self.nodes.len());
        new_nodes.push(BddNode::mk_zero(BddVar { name: i32::MAX }));
        new_nodes.push(BddNode::mk_one(BddVar { name: i32::MAX }));

        for (new_index, (old_index, &node)) in sorted_nodes.iter().enumerate() {
            let old_pointer = BddPointer::new(*old_index);
            if affected_vars.contains(&self.var_of_ptr(old_pointer)) {
                let new_pointer = BddPointer::new(new_index + 2); // because we skipped the terminals

                let new_low = self.low_node_ptr(new_pointer);
                let new_high = self.high_node_ptr(new_pointer);
                let new_node = BddNode::mk_node(node.var, new_low, new_high);

                new_nodes.push(new_node);
                nodes_map.insert(old_pointer, new_pointer);
            } else {
                new_nodes.push(node);
                nodes_map.insert(old_pointer, old_pointer);
            }
        }
        // Update the BDD nodes with the new ordering
        self.nodes = new_nodes;
    }

    

    /*
    /// Randomly choose clauses from the set of clauses and check if the found assignment satisfies them.
    pub fn check_sat(
        &self,
        variables: &Vec<BddVar>,
        clauses_set: &Vec<Vec<i32>>,
        clauses_count: usize,
    ) -> Result<bool, &'static str> {
        let assignment = self.solve(variables);
        match assignment {
            Ok(mut sat) => {
                // If variables are not set its because a non canonical bdd is formed.
                // These variables appear in two clause once not negated and once negated.
                // By resolution they are deleted from both clauses as they are always true.
                for var in variables {
                    if !sat.contains_key(&var.name) && !sat.contains_key(&-var.name) {
                        // it is not important what polarity these variables have
                        sat.insert(var.name, false);
                    }
                }
                let amount;
                if clauses_count > 1 {
                    let mut rng = rand::thread_rng();
                    amount = rng.gen_range(1..clauses_count);
                } else {
                    amount = 1;
                }
                let mut sample_clauses: Vec<_> = clauses_set
                    .choose_multiple(&mut rand::thread_rng(), amount)
                    .cloned()
                    .collect();
                let sample_vec_expr =
                    bool_expr::Expr::parse_clauses(&mut sample_clauses, variables);
                for sample_expr in sample_vec_expr {
                    match sample_expr.set_vars_and_solve(&sat) {
                        Some(value) => {
                            if !value {
                                return Err("The assignment is false!");
                            }
                        }
                        None => return Err("Not sufficient information in the assignment!"),
                    }
                }
                Ok(true)
            }
            Err(err) => panic!("{}", err),
        }
    }
    */
}

impl PartialEq for Bdd {
    fn eq(&self, other: &Self) -> bool {
        (self.size() == other.size())
            && (self.nodes.iter().all(|x| other.nodes.contains(x)))
            && (other.nodes.iter().all(|y| self.nodes.contains(y)))
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    fn create_sample_bdd() -> Bdd {
        let mut bdd = Bdd::new();

        let x1 = BddVar::new(1, 0.0);
        let x2 = BddVar::new(2, 0.0);
        let x3 = BddVar::new(3, 0.0);

        let node3: BddNode =
            BddNode::mk_node(x3.clone(), BddPointer::new_zero(), BddPointer::new_one());
        let node2: BddNode =
            BddNode::mk_node(x2.clone(), BddPointer::new(2), BddPointer::new_one());
        let node4: BddNode =
            BddNode::mk_node(x1.clone(), BddPointer::new(3), BddPointer::new_one());

        bdd.nodes.push(node3);
        bdd.nodes.push(node2);
        bdd.nodes.push(node4);

        bdd
    }

    fn create_sample_bdd_complicated() -> Bdd {
        let mut bdd = Bdd::new();

        let x1 = BddVar::new(1, 0.0);
        let x2 = BddVar::new(2, 0.0);
        let x3 = BddVar::new(3, 0.0);

        let node3: BddNode =
            BddNode::mk_node(x3.clone(), BddPointer::new_zero(), BddPointer::new_one());
        let node2: BddNode =
            BddNode::mk_node(x2.clone(), BddPointer::new(2), BddPointer::new_zero());
        let node4: BddNode = BddNode::mk_node(x1.clone(), BddPointer::new(3), BddPointer::new(2));

        bdd.nodes.push(node3);
        bdd.nodes.push(node2);
        bdd.nodes.push(node4);

        bdd
    }

    #[test]
    fn test_var_of_ptr() {
        let bdd = create_sample_bdd();
        let ptr = BddPointer::new(3);
        let var = bdd.var_of_ptr(ptr);
        assert_eq!(var, BddVar::new(2, 0.0));
    }

    #[test]
    fn test_reorder_variables() {
        let mut bdd = create_sample_bdd();
        let mut ordering = HashMap::with_capacity(4); // Set initial capacity to accommodate indices 0, 1, and 2
        ordering.insert(2, 0);
        ordering.insert(1, 1);
        ordering.insert(3, 2);
        ordering.insert(i32::MAX, 3);

        let variables = vec![BddVar::new(2, 0.0), BddVar::new(1, 0.0), BddVar::new(3, 0.0)];

        bdd.partial_reorder_bdd(&variables, &mut ordering);

        // Assert the correct reordering has occurred
        let var_order = bdd
            .nodes
            .iter()
            .map(|node| node.var.name)
            .collect::<Vec<i32>>();
        assert_eq!(var_order, vec![i32::MAX, i32::MAX, 3, 1, 2]);
    }

    #[test]
    fn test_reorder_variables_complicated() {
        let mut bdd = create_sample_bdd_complicated();
        let mut ordering = HashMap::with_capacity(4); // Set initial capacity to accommodate indices 0, 1, and 2
        ordering.insert(2, 0);
        ordering.insert(1, 1);
        ordering.insert(3, 2);
        ordering.insert(i32::MAX, 3);

        let variables = vec![BddVar::new(2, 0.0), BddVar::new(1, 0.0), BddVar::new(3, 0.0)];

        bdd.partial_reorder_bdd(&variables, &mut ordering);

        // Assert the correct reordering has occurred
        let var_order = bdd
            .nodes
            .iter()
            .map(|node| node.var.name)
            .collect::<Vec<i32>>();
        assert_eq!(var_order, vec![i32::MAX, i32::MAX, 3, 1, 2]);
    }

    #[test]
    fn test_sift_variables_nec() {
        let mut bdd = create_sample_bdd();
        let mut ordering = HashMap::new();
        ordering.insert(2, 0);
        ordering.insert(1, 1);
        ordering.insert(3, 2);
        ordering.insert(i32::MAX, 3);

        let variables = vec![BddVar::new(2, 0.0), BddVar::new(1, 0.0), BddVar::new(3, 0.0)];

        bdd.sift_variables_nec(&mut ordering, &variables);

        // Assert the correct reordering has occurred
        let var_order = bdd
            .nodes
            .iter()
            .map(|node| node.var.name)
            .collect::<Vec<i32>>();
        assert_eq!(var_order, vec![i32::MAX, i32::MAX, 3, 2, 1]);
    }

    #[test]
    fn test_sift_variables_nec_complicated() {
        let mut bdd = create_sample_bdd_complicated();
        let mut ordering = HashMap::new();
        ordering.insert(2, 0);
        ordering.insert(1, 1);
        ordering.insert(3, 2);
        ordering.insert(i32::MAX, 3);

        let variables = vec![BddVar::new(2, 0.0), BddVar::new(1, 0.0), BddVar::new(3, 0.0)];

        bdd.sift_variables_nec(&mut ordering, &variables);

        // Assert the correct reordering has occurred
        let var_order = bdd
            .nodes
            .iter()
            .map(|node| node.var.name)
            .collect::<Vec<i32>>();
        assert_eq!(var_order, vec![i32::MAX, i32::MAX, 3, 2, 1]);
    }

    #[test]
    fn test_reorder_variables_detailed() {
        let mut bdd = create_sample_bdd();
        let mut ordering = HashMap::new();
        ordering.insert(i32::MAX, 0);
        ordering.insert(2, 1);
        ordering.insert(1, 2);
        ordering.insert(3, 3);

        println!("Original BDD: {:?}", bdd);

        let variables = vec![BddVar::new(2, 0.0), BddVar::new(1, 0.0), BddVar::new(3, 0.0)];

        bdd.partial_reorder_bdd(&variables, &mut ordering);


        println!("Variable Ordering: {:?}", ordering);
        println!("Reordered BDD: {:?}", bdd);
        // Assert the correct reordering has occurred
        let var_order = bdd
            .nodes
            .iter()
            .map(|node| node.var.name)
            .collect::<Vec<i32>>();
        assert_eq!(var_order, vec![i32::MAX, i32::MAX, 3, 1, 2]);

        assert_eq!(bdd.nodes[0].low.to_index(), 0);
        assert_eq!(bdd.nodes[0].high.to_index(), 0);
        assert_eq!(bdd.nodes[1].low.to_index(), 1);
        assert_eq!(bdd.nodes[1].high.to_index(), 1);
        assert_eq!(bdd.nodes[2].low.to_index(), 0);
        assert_eq!(bdd.nodes[2].high.to_index(), 1);
        assert_eq!(bdd.nodes[3].low.to_index(), 2);
        assert_eq!(bdd.nodes[3].high.to_index(), 1);
        assert_eq!(bdd.nodes[4].low.to_index(), 3);
        assert_eq!(bdd.nodes[4].high.to_index(), 1);
    }

    /*
    #[test]
    pub fn test_to_clause_vector() {
        let path: &str = "tests/test1.cnf";

        // create the Dimacs instance
        let dimacs = Expr::parse_dimacs_cnf_file(path).unwrap();

        // build the variable ordering
        let var_ordering = BddVarOrdering::new(dimacs);

        let bdd = var_ordering.build_bdd();

        let res = bdd.to_clause_vector();

        assert_eq!(dimacs.expressions, res);
    }*/
}
