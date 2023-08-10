use crate::expr::bool_expr;
use crate::{
    bdd_util::{BddNode, BddPointer, BddVar},
    expr::bool_expr::Expr,
};
use std::collections::HashMap;
use std::iter::Map;
use std::ops::Range;

// The Bdd receives the clauses 'Vec<Vec<i32>>'. They can be viewed as a boolean
// expression for example (x1 OR x2) AND (NOT x1 OR x2). Then the INF (if then else normalform)
// needs to be found for this expression so that the Bdd can be constructed.

#[derive(Clone, Debug)]
pub struct Bdd(pub Vec<BddNode>);

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
        Bdd(nodes)
    }

    pub fn new_with_capacity(cap: usize) -> Bdd {
        let mut nodes = Vec::with_capacity(cap);
        // Maximum number as pointer as in the apply method always the smaller var is
        // selected and we want to replace these nodes.
        let max_ptr = BddVar::new(i32::MAX, 0.0);
        nodes.push(BddNode::mk_zero(max_ptr));
        nodes.push(BddNode::mk_one(max_ptr));
        Bdd(nodes)
    }

    pub fn is_full(&self) -> bool {
        self.0.capacity().eq(&self.0.len())
    }

    pub fn is_empty(&self) -> bool {
        self.size().eq(&2)
    }

    /// Get the variable of a specific pointer in the Bdd.
    pub fn var_of_ptr(&self, ptr: BddPointer) -> BddVar {
        self.0[ptr.to_index()].var
    }

    /// Insert a node into the vector of nodes of the Bdd.
    pub fn push_node(&mut self, node: BddNode) {
        self.0.push(node);
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
    pub fn new_false(var: BddVar) -> Bdd {
        let mut bdd = Bdd::new();
        bdd.push_node(BddNode::mk_zero(var));
        bdd
    }

    /// Create a new Bdd for the true formula.
    pub fn new_true(var: BddVar) -> Bdd {
        let mut bdd = Bdd::new();
        bdd.push_node(BddNode::mk_zero(var));
        bdd.push_node(BddNode::mk_one(var));
        bdd
    }

    /// Create a new Bdd for a negated variable.
    pub fn new_not_var(var: BddVar) -> Bdd {
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
    pub fn and(&self, other: &Bdd, ordering: &std::collections::HashMap<i32, usize>) -> Bdd {
        self.apply(other, bool_expr::and, ordering)
    }

    /// This method creates a `Bdd` corresponding to the $\phi \lor \psi$ formula, where $\phi$ and $\psi$
    /// are the two given `Bdd`s.
    pub fn or(&self, other: &Bdd, ordering: &std::collections::HashMap<i32, usize>) -> Bdd {
        self.apply(other, bool_expr::or, ordering)
    }

    /// Negate a Bdd.
    pub fn negate(&mut self) -> Bdd {
        if self.is_true() {
            Bdd::new_false(BddVar::new(i32::MAX, 0.0))
        } else if self.is_false() {
            Bdd::new_true(BddVar::new(i32::MAX, 0.0))
        } else {
            let mut nodes = self.0.clone();
            for node in nodes.iter_mut().skip(2) {
                // skip terminals
                node.high.flip_if_terminal();
                node.low.flip_if_terminal();
            }
            Bdd(nodes)
        }
    }

    /// The number of nodes in a Bdd.
    pub fn size(&self) -> usize {
        self.0.len()
    }

    /// True if a Bdd is exactly the true formula.
    pub fn is_true(&self) -> bool {
        self.0.len() == 2
    }

    /// True if a Bdd is exactly the false formula.
    pub fn is_false(&self) -> bool {
        self.0.len() == 1
    }

    /// Get the pointer of the root node of the Bdd.
    pub fn root_pointer(&self) -> BddPointer {
        if self.is_false() {
            BddPointer::new_zero()
        } else if self.is_true() {
            BddPointer::new_one()
        } else {
            BddPointer::new(self.0.len() - 1)
        }
    }

    pub fn indices(&self) -> Map<Range<usize>, fn(usize) -> BddPointer> {
        (0..self.size()).map(BddPointer::new)
    }

    pub fn low_node_ptr(&self, ptr: BddPointer) -> BddPointer {
        self.0[ptr.to_index()].low
    }

    pub fn replace_low(&mut self, ptr: BddPointer, new_ptr: BddPointer) {
        self.0[ptr.to_index()].low = new_ptr
    }

    pub fn high_node_ptr(&self, ptr: BddPointer) -> BddPointer {
        self.0[ptr.to_index()].high
    }

    pub fn replace_high(&mut self, ptr: BddPointer, new_ptr: BddPointer) {
        self.0[ptr.to_index()].high = new_ptr
    }

    pub fn delete_node(&mut self, to_delete: BddPointer, node_path: Vec<(BddPointer, bool)>) {
        self.0.remove(to_delete.to_index());
        // the path until the node to delete was reached
        for (node, assign) in node_path.into_iter().skip(1) {
            // skip the first one as it was already assigned
            if assign {
                // if true then decrement the high nodes
                self.replace_high(node, BddPointer(self.high_node_ptr(node).0 - 1));
            } else {
                // if false then decrement the low nodes
                self.replace_low(node, BddPointer(self.low_node_ptr(node).0 - 1));
            }
        }
    }

    pub fn replace_node(&mut self, to_delete: BddPointer, replacement: BddPointer) {
        self.0.remove(to_delete.to_index());
        for ptr in self.indices() {
            if self.low_node_ptr(ptr).eq(&to_delete) {
                self.replace_low(ptr, replacement);
            } else if self.high_node_ptr(ptr).eq(&to_delete) {
                self.replace_high(ptr, replacement);
            }
        }
    }


    /*
    /// Convert this `Bdd` to a `BooleanExpression`.
    pub fn to_bool_expr(&self) -> Expr {
        if self.is_false() {
            return Expr::Const(false);
        }
        if self.is_true() {
            return Expr::Const(true);
        }

        let mut res: Vec<Expr> = Vec::with_capacity(self.0.len());
        res.push(Expr::Const(false)); // fake terminals
        res.push(Expr::Const(true)); // never used
        for node in 2..self.0.len() {
            // skip terminals
            let bdd_var = self.0[node].var;
            let var_name = bdd_var.name;

            let low = self.0[node].low;
            let high = self.0[node].high;
            let expr = if low.is_terminal() && high.is_terminal() {
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
    }*/

    /// This method merges two Bdds
    fn apply<T>(&self, other: &Bdd, op: T, ordering: &std::collections::HashMap<i32, usize>) -> Bdd
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
                let (lft, rgt) = (current.left, current.right);
                // find the lowest variable of the two nodes
                let (l_var, r_var) = (self.var_of_ptr(lft), other.var_of_ptr(rgt));

                // The min variable is now the one with the higher score, so
                // the smallest index in the mapping
                let l_var_index = ordering.get(&l_var.name).unwrap();
                let r_var_index = ordering.get(&r_var.name).unwrap();
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
        bdd
    }

    pub fn build(
        expressions: Vec<Expr>,
        variables: &Vec<BddVar>,
        ordering: &std::collections::HashMap<i32, usize>,
        //_clause_database: &mut ClauseDatabase,
        mut rec_depth: usize,
        //_solver_wrapper: GlucoseWrapper
    ) -> Self {
        // here we are investigating 2 new clauses
        rec_depth += 2;
        let mut current_bdd = expressions[0].to_bdd(&variables, &ordering);

        let mut n = 1;
        while n < expressions.len() {
            // clear the global filter every 30 clauses
            if rec_depth % 30 == 0 {
                //clause_database.reset_bloom_filter_global();
            }
            // clear the local filter from former clauses
            //clause_database.reset_bloom_filter_local();

            // send the current learned clauses while building the temp_bdd
            let (_, temp_bdd) = rayon::join(
                || {
                    //current_bdd.send_learned_clauses(true,clause_database,solver_wrapper)
                },
                || expressions[n].to_bdd(&variables, &ordering),
            );

            current_bdd = current_bdd.and(&temp_bdd, &ordering);

            rec_depth += 2;
            n += 1;
        }
        current_bdd
    }

    /// Check if the Bdd is satisfiable and if its the case return
    /// the satisfiable assignment in a vector of bool.
    pub fn solve(&self, variables: &Vec<BddVar>) -> Result<HashMap<i32, bool>, &str> {
        // If the Bdd is false return None.
        if self.is_false() {
            return Err("The problem is not solvable!");
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

    /*
    fn find_resolvents(&self, resolvents: &Vec<BddVar>) -> Vec<BddPointer> {
        let mut pointers: Vec<BddPointer> = Vec::new();
        for ptr in self.indices() {
            if resolvents.contains(&self.var_of_ptr(ptr)) {
                pointers.push(ptr);
            }
        }
        pointers
    }

    /// The following methods remove all nodes with a resolvent
    /// variable and replace all pointers linked to it with 1
    fn remove_resolvents(&mut self, resolvents_idx: Vec<BddPointer>) {
        println!("remove_resolvents {:?}", self);
        let mut to_remove = Vec::new();
        for ptr in resolvents_idx {
            self.prune_resolvents(ptr, &mut to_remove);
            to_remove.push(ptr.to_index());
        }
        to_remove.sort();
        self.remove_nodes(to_remove);
    }

    // TODO
    fn prune_resolvents(&mut self, ptr: BddPointer, to_remove: &mut Vec<usize>) {
        // find the parent nodes and update them
        let mut parents = Vec::new();
        // also the children nodes of the resolvents need to be
        // deleted if they are not children of other nodes
        let children = Vec::new();
        let child_low = self.low_node_ptr(ptr);
        let child_high = self.high_node_ptr(ptr);
        let mut includes_child_low = false;
        let mut includes_child_high = false;

        for (idx, node) in self.0.iter_mut().enumerate() {
            if node.low == ptr {
                if node.high.is_one() {
                    parents.push(idx);
                    to_remove.push(idx);
                }
                node.replace_low(BddPointer::new_one());
            }
            if node.high == ptr {
                if node.low.is_one() {
                    parents.push(idx);
                    to_remove.push(idx);
                }
                node.replace_high(BddPointer::new_one());
            }
            if idx != ptr.to_index() && (node.low == child_low || node.high == child_low) {
                println!("child_low {:?}", child_low);
                println!("includes_child_low {:?}", node.low);
                includes_child_low = true;
            }
            if idx != ptr.to_index() && (node.low == child_high || node.high == child_high) {
                println!("child_high {:?}", child_high);
                println!("includes_child_high {:?}", node.high);
                includes_child_high = true;
            }
        }
        if !includes_child_low {
            //children.push(child_low.to_index());
            to_remove.push(child_low.to_index());
        }
        if !includes_child_high {
            //children.push(child_high.to_index());
            to_remove.push(child_high.to_index());
        }
        println!("parents {:?}", parents);
        for par in parents {
            self.prune_resolvents(BddPointer::new(par), to_remove);
        }
        println!("children {:?}", children);
        // TODO somehow consider children
        for chi in children {
            self.prune_resolvents(BddPointer::new(chi), to_remove);
        }
    }

    fn remove_nodes(&mut self, to_remove: Vec<usize>) {
        println!("remove_nodes {:?}", self);
        println!("to_remove {:?}", to_remove);
        let mut index = 0;
        let mut min_del = self.0.len();

        for rem_idx in to_remove {
            println!("rem_idx {:?}", rem_idx);
            let del_idx = if rem_idx > index {
                rem_idx - index
            } else {
                rem_idx
            };
            println!("del_idx {:?}", del_idx);
            self.0.remove(del_idx);
            if del_idx < min_del {
                min_del = del_idx
            };

            // we reduce by 1 the nodes that their low or high index pointer
            // is greater than the current deletion index
            for ptr in self.indices() {
                if !self.low_node_ptr(ptr).is_terminal()
                    && self.low_node_ptr(ptr).to_index() > del_idx - 1
                {
                    self.replace_low(ptr, BddPointer::new(self.low_node_ptr(ptr).to_index() - 1));
                }
                if !self.high_node_ptr(ptr).is_terminal()
                    && self.high_node_ptr(ptr).to_index() > del_idx - 1
                {
                    self.replace_high(ptr, BddPointer::new(self.high_node_ptr(ptr).to_index() - 1));
                }
            }
            println!("self {:?}", self);
            index += 1;
        }
    }

    /// This method merges two Bdds based on resolution.
    /// It is a variation of the Bdd apply method.
    fn apply_resolution(
        &mut self,
        other: &Bdd,
        ordering: &std::collections::HashMap<i32, usize>,
    ) -> Bdd {
        let mut bdd = Bdd::new();

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
            op: fn(Option<bool>, Option<bool>) -> Option<bool>,
        }

        // There can be multiple resolvent variables
        let mut resolvents = Vec::new();

        // We keep track of the tasks currently on stack so that we build the bdd from down to the top
        let mut stack: Vec<Task> = Vec::with_capacity(std::cmp::max(self.size(), other.size()));

        stack.push(Task {
            left: self.root_pointer(),
            right: other.root_pointer(),
            op: bool_expr::and,
        });

        fn find_key_for_value(
            map: &std::collections::HashMap<BddNode, BddPointer>,
            value: BddPointer,
        ) -> Option<BddNode> {
            map.iter()
                .find_map(|(&key, &val)| if val == value { Some(key) } else { None })
        }

        // We keep track of the tasks already completed, so that we can access the pointers
        let mut finished_tasks: std::collections::HashMap<Task, BddPointer> =
            std::collections::HashMap::with_capacity(std::cmp::max(self.size(), other.size()));

        while let Some(current) = stack.last() {
            // We keep track if we are in active resolution procedure or not
            let mut resolution = false;

            if finished_tasks.contains_key(current) {
                stack.pop();
            } else {
                let (lft, rgt) = (current.left, current.right);
                // Find the lowest variable of the two nodes
                let (l_var, r_var) = (self.var_of_ptr(lft), other.var_of_ptr(rgt));

                // The min variable is now the one with the higher score, so
                // the smallest index in the mapping
                let l_var_index = ordering.get(&l_var.name).unwrap();
                let r_var_index = ordering.get(&r_var.name).unwrap();
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
                let (sub_left, sub_right) =
                // If we spot opposite polarities we are in the resolution process.
                if l_var.eq(&r_var) && (l_low.eq(&r_high) || l_high.eq(&r_low)) {
                    resolution = true;
                    if !resolvents.contains(&min_var) {resolvents.push(min_var)};
                    (Task {
                        left: l_low,
                        right: r_high,
                        op: bool_expr::or
                    },
                    Task{
                        left: l_high,
                        right: r_low,
                        op: bool_expr::or
                    })
                } else {
                    (Task {
                        left: l_low,
                        right: r_low,
                        op: current.op
                    },
                    Task{
                        left: l_high,
                        right: r_high,
                        op: current.op
                    })
                };

                // if in resolution these will be two seperate nodes inserted
                // but because of recursion they should have been already in
                // the nodes_map
                let new_low: Option<BddPointer> =
                    (sub_left.op)(sub_left.left.as_bool(), sub_left.right.as_bool())
                        .map(BddPointer::from_bool)
                        .or(finished_tasks.get(&sub_left).cloned());

                let new_high: Option<BddPointer> =
                    (sub_right.op)(sub_right.left.as_bool(), sub_right.right.as_bool())
                        .map(BddPointer::from_bool)
                        .or(finished_tasks.get(&sub_right).cloned());

                if let (Some(new_low), Some(new_high)) = (new_low, new_high) {
                    if new_low == new_high {
                        finished_tasks.insert(*current, new_low);
                    } else {
                        let node = if resolution {
                            let new: BddPointer =
                                bool_expr::and(new_low.as_bool(), new_high.as_bool())
                                    .map(BddPointer::from_bool)
                                    .unwrap_or(if new_low.is_one() {
                                        new_high
                                    } else if new_high.is_one() {
                                        new_low
                                    } else {
                                        *finished_tasks
                                            .get(&Task {
                                                left: new_low,
                                                right: new_high,
                                                op: bool_expr::and,
                                            })
                                            .unwrap()
                                    });
                            find_key_for_value(&nodes_map, new)
                        } else {
                            Some(BddNode::mk_node(min_var, new_low, new_high))
                        };
                        if let Some(node) = node {
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
                    }
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
        bdd.remove_resolvents(bdd.find_resolvents(&resolvents));
        bdd
    }
    */
}

impl PartialEq for Bdd {
    fn eq(&self, other: &Self) -> bool {
        (self.size() == other.size())
            && (self.0.iter().all(|x| other.0.contains(x)))
            && (other.0.iter().all(|y| self.0.contains(y)))
    }
}

/*
#[cfg(test)]
mod tests {
    use crate::{
        bdd::Bdd,
        bdd_util::{BddNode, BddPointer, BddVar},
        expr::bool_expr::{self, Expr},
        parser::parse::parse_dimacs,
        variable_ordering::var_ordering::BddVarOrdering,
    };

    fn do_vecs_match<T: PartialEq>(a: &Vec<T>, b: &Vec<T>) -> bool {
        let difference: Vec<_> = a.into_iter().filter(|item| !b.contains(item)).collect();
        a.len() == b.len() && difference.is_empty()
    }

    pub fn resolve_pairs(pairs: Vec<(Expr, Expr)>) -> Vec<Expr> {
        let mut new_clauses = Vec::new();
        for (expr1, expr2) in pairs {
            let new_clause = expr1.resolution(&expr2);
            new_clauses.push(Expr::parse_clause(&new_clause));
        }
        new_clauses
    }

    #[test]
    pub fn test_create_bdds_from_file1() {
        let path: &str = "tests/test1.cnf";

        // create the Dimacs instance
        let dimacs = parse_dimacs(path);

        // create the vector of the parsed expressions
        let clause_set = bool_expr::Expr::parse_clauses(&dimacs.clauses);

        // build the variable ordering
        let var_ordering = BddVarOrdering::new(dimacs);

        let mut bdd_vec = Vec::new();

        for expr in clause_set.clone() {
            bdd_vec.push(expr.to_bdd(&var_ordering.variables, &var_ordering.ordering));
        }

        let mut expr_res = Vec::new();

        for bdd in bdd_vec {
            expr_res.push(bdd.to_bool_expr());
        }
    }

    #[test]
    pub fn test_refutation_resolution_rule1() {
        // 83 16 65 0
        // 83 16 -65 0
        let input: &str = "tests/test5.cnf";
        let dimacs = parse_dimacs(input);
        let clause_set = bool_expr::Expr::parse_clauses(&dimacs.clauses);

        // build the variable ordering
        let var_ordering = BddVarOrdering::new(dimacs);

        let mut bdd_vec = Vec::new();

        for expr in clause_set.clone() {
            bdd_vec.push(expr.to_bdd(&var_ordering.variables, &var_ordering.ordering));
        }
        let snd = bdd_vec[1].clone();
        let merged_bdd = bdd_vec[0].resolve(&snd, &var_ordering.ordering);

        let mut ref_res_bdd = Bdd::new();
        let node1 = BddNode::mk_node(
            BddVar::new(16, 0.333),
            BddPointer::new_zero(),
            BddPointer::new_one(),
        );
        let node2 = BddNode::mk_node(
            BddVar::new(83, 0.333),
            BddPointer::new(2),
            BddPointer::new_one(),
        );

        ref_res_bdd.push_node(node1);
        ref_res_bdd.push_node(node2);

        assert_eq!(merged_bdd, ref_res_bdd);
    }

    #[test]
    pub fn test_refutation_resolution_rule2() {
        // 83 16 -65 0
        // 83 -16 65 0
        let input: &str = "tests/test5.cnf";
        let dimacs = parse_dimacs(input);
        let clause_set = bool_expr::Expr::parse_clauses(&dimacs.clauses);

        // build the variable ordering
        let var_ordering = BddVarOrdering::new(dimacs);

        let mut bdd_vec = Vec::new();

        for expr in clause_set.clone() {
            bdd_vec.push(expr.to_bdd(&var_ordering.variables, &var_ordering.ordering));
        }
        let snd = bdd_vec[2].clone();
        let merged_bdd = bdd_vec[1].resolve(&snd, &var_ordering.ordering);

        let mut ref_res_bdd = Bdd::new();
        let node = BddNode::mk_node(
            BddVar::new(83, 0.333),
            BddPointer::new_zero(),
            BddPointer::new_one(),
        );

        ref_res_bdd.push_node(node);

        assert_eq!(merged_bdd, ref_res_bdd);
    }

    #[test]
    pub fn resolution_1_1() {
        //83 0
        //-83 0
        let input: &str = "tests/test6.cnf";
        let dimacs = parse_dimacs(input);
        let clause_set = bool_expr::Expr::parse_clauses(&dimacs.clauses);
        // build the variable ordering
        let var_ordering = BddVarOrdering::new(dimacs);

        let mut bdd_vec = Vec::new();

        for expr in clause_set.clone() {
            bdd_vec.push(expr.to_bdd(&var_ordering.variables, &var_ordering.ordering));
        }
        let snd = bdd_vec[2].clone();
        let merged_bdd = bdd_vec[1].resolve(&snd, &var_ordering.ordering);
        let res_bdd = Bdd::new();
        assert_eq!(merged_bdd, res_bdd);
    }
    #[test]
    pub fn resolution_2_1() {
        //83 16 0
        //-83 0
        let input: &str = "tests/test6.cnf";
        let dimacs = parse_dimacs(input);
        let clause_set = bool_expr::Expr::parse_clauses(&dimacs.clauses);
        // build the variable ordering
        let var_ordering = BddVarOrdering::new(dimacs);

        let mut bdd_vec = Vec::new();

        for expr in clause_set.clone() {
            bdd_vec.push(expr.to_bdd(&var_ordering.variables, &var_ordering.ordering));
        }
        let snd = bdd_vec[1].clone();
        let merged_bdd = bdd_vec[0].resolve(&snd, &var_ordering.ordering);
        let mut res_bdd = Bdd::new();
        let node = BddNode::mk_node(
            BddVar::new(16, 0.5),
            BddPointer::new_zero(),
            BddPointer::new_one(),
        );
        res_bdd.push_node(node);
        assert_eq!(merged_bdd, res_bdd);
    }
    #[test]
    pub fn resolution_1_2() {
        //83 0
        //-83 16 0
        let input: &str = "tests/test6.cnf";
        let dimacs = parse_dimacs(input);
        let clause_set = bool_expr::Expr::parse_clauses(&dimacs.clauses);
        // build the variable ordering
        let var_ordering = BddVarOrdering::new(dimacs);

        let mut bdd_vec = Vec::new();

        for expr in clause_set.clone() {
            bdd_vec.push(expr.to_bdd(&var_ordering.variables, &var_ordering.ordering));
        }
        let snd = bdd_vec[3].clone();
        let merged_bdd = bdd_vec[2].resolve(&snd, &var_ordering.ordering);
        let mut res_bdd = Bdd::new();
        let node = BddNode::mk_node(
            BddVar::new(16, 0.5),
            BddPointer::new_zero(),
            BddPointer::new_one(),
        );
        res_bdd.push_node(node);
        assert_eq!(merged_bdd, res_bdd);
    }
    #[test]
    pub fn resolution_2_2_same_vars() {
        //83 16  0
        //-83 16 0
        //-83 0 (we need this to have the correct variable ordering)
        let input: &str = "tests/test7.cnf";
        let dimacs = parse_dimacs(input);
        let clause_set = bool_expr::Expr::parse_clauses(&dimacs.clauses);
        // build the variable ordering
        let var_ordering = BddVarOrdering::new(dimacs);

        let mut bdd_vec = Vec::new();

        for expr in clause_set.clone() {
            bdd_vec.push(expr.to_bdd(&var_ordering.variables, &var_ordering.ordering));
        }
        let snd = bdd_vec[1].clone();
        let merged_bdd = bdd_vec[0].resolve(&snd, &var_ordering.ordering);
        let mut res_bdd = Bdd::new();
        let node = BddNode::mk_node(
            BddVar::new(16, 0.5),
            BddPointer::new_zero(),
            BddPointer::new_one(),
        );
        res_bdd.push_node(node);
        assert_eq!(merged_bdd, res_bdd);
    }
    #[test]
    pub fn resolution_2_2_double_res() {
        //83 -16  0
        //-83 16 0
        //-83 0 (we need this to have the correct variable ordering)
        let input: &str = "tests/test7.cnf";
        let dimacs = parse_dimacs(input);
        let clause_set = bool_expr::Expr::parse_clauses(&dimacs.clauses);
        // build the variable ordering
        let var_ordering = BddVarOrdering::new(dimacs);

        let mut bdd_vec = Vec::new();

        for expr in clause_set.clone() {
            bdd_vec.push(expr.to_bdd(&var_ordering.variables, &var_ordering.ordering));
        }
        let snd = bdd_vec[2].clone();
        let merged_bdd = bdd_vec[1].resolve(&snd, &var_ordering.ordering);
        let res_bdd = Bdd::new();
        assert_eq!(merged_bdd, res_bdd);
    }
    #[test]
    pub fn resolution_2_2_same_vars_res_on_b() {
        //83 -16  0
        //83 16 0
        let input: &str = "tests/test7.cnf";
        let dimacs = parse_dimacs(input);
        let clause_set = bool_expr::Expr::parse_clauses(&dimacs.clauses);
        // build the variable ordering
        let var_ordering = BddVarOrdering::new(dimacs);

        let mut bdd_vec = Vec::new();

        for expr in clause_set.clone() {
            bdd_vec.push(expr.to_bdd(&var_ordering.variables, &var_ordering.ordering));
        }
        let snd = bdd_vec[3].clone();
        let merged_bdd = bdd_vec[2].resolve(&snd, &var_ordering.ordering);
        let mut res_bdd = Bdd::new();
        let node = BddNode::mk_node(
            BddVar::new(83, 0.5),
            BddPointer::new_zero(),
            BddPointer::new_one(),
        );
        res_bdd.push_node(node);
        assert_eq!(merged_bdd, res_bdd);
    }
    #[test]
    pub fn resolution_2_2_dif_vars() {
        //83 16 0
        //-83 65 0
        //-83 0
        //-83 0
        //16 0
        let input: &str = "tests/test8.cnf";
        let dimacs = parse_dimacs(input);
        let clause_set = bool_expr::Expr::parse_clauses(&dimacs.clauses);
        // build the variable ordering
        let var_ordering = BddVarOrdering::new(dimacs);

        let mut bdd_vec = Vec::new();

        for expr in clause_set.clone() {
            bdd_vec.push(expr.to_bdd(&var_ordering.variables, &var_ordering.ordering));
        }
        let snd = bdd_vec[1].clone();
        let merged_bdd = bdd_vec[0].resolve(&snd, &var_ordering.ordering);
        let mut res_bdd = Bdd::new();
        let node1 = BddNode::mk_node(
            BddVar::new(16, 0.5),
            BddPointer::new(2),
            BddPointer::new_one(),
        );
        let node2 = BddNode::mk_node(
            BddVar::new(65, 0.5),
            BddPointer::new_zero(),
            BddPointer::new_one(),
        );
        res_bdd.push_node(node1);
        res_bdd.push_node(node2);
        assert_eq!(merged_bdd, res_bdd);
    }
    #[test]
    pub fn resolution_2_2_dif_vars_res_on_b() {
        //83 -16 0
        //16 65 0
        let input: &str = "tests/test8.cnf";
        let dimacs = parse_dimacs(input);
        let clause_set = bool_expr::Expr::parse_clauses(&dimacs.clauses);
        // build the variable ordering
        let var_ordering = BddVarOrdering::new(dimacs);
        let mut bdd_vec = Vec::new();

        for expr in clause_set.clone() {
            bdd_vec.push(expr.to_bdd(&var_ordering.variables, &var_ordering.ordering));
        }
        let snd = bdd_vec[3].clone();
        let merged_bdd = bdd_vec[2].resolve(&snd, &var_ordering.ordering);
        let mut res_bdd = Bdd::new();
        let node1 = BddNode::mk_node(
            BddVar::new(83, 0.5),
            BddPointer::new(2),
            BddPointer::new_one(),
        );
        let node2 = BddNode::mk_node(
            BddVar::new(65, 0.5),
            BddPointer::new_zero(),
            BddPointer::new_one(),
        );
        res_bdd.push_node(node2);
        res_bdd.push_node(node1);
        assert_eq!(merged_bdd, res_bdd);
    }
    #[test]
    pub fn resolution_3_3() {
        //83 16 65 0
        //-83 16 65 0
        //-83 0
        //-83 0
        //16 0
        let input: &str = "tests/test9.cnf";
        let dimacs = parse_dimacs(input);
        let clause_set = bool_expr::Expr::parse_clauses(&dimacs.clauses);
        // build the variable ordering
        let var_ordering = BddVarOrdering::new(dimacs);

        let mut bdd_vec = Vec::new();

        for expr in clause_set.clone() {
            bdd_vec.push(expr.to_bdd(&var_ordering.variables, &var_ordering.ordering));
        }
        let snd = bdd_vec[1].clone();
        let merged_bdd = bdd_vec[0].resolve(&snd, &var_ordering.ordering);
        let mut res_bdd = Bdd::new();
        let node1 = BddNode::mk_node(
            BddVar::new(16, 0.5),
            BddPointer::new(2),
            BddPointer::new_one(),
        );
        let node2 = BddNode::mk_node(
            BddVar::new(65, 0.5),
            BddPointer::new_zero(),
            BddPointer::new_one(),
        );
        res_bdd.push_node(node1);
        res_bdd.push_node(node2);
        assert_eq!(merged_bdd, res_bdd);
    }
    #[test]
    pub fn resolution_3_3_dif_vars_res_on_d() {
        // 83 -13 16 0
        // 16 65 13 0
        let input: &str = "tests/test10.cnf";
        let dimacs = parse_dimacs(input);
        let clause_set = bool_expr::Expr::parse_clauses(&dimacs.clauses);
        // build the variable ordering
        let var_ordering = BddVarOrdering::new(dimacs);
        let mut bdd_vec = Vec::new();

        for expr in clause_set.clone() {
            bdd_vec.push(expr.to_bdd(&var_ordering.variables, &var_ordering.ordering));
        }
        let snd = bdd_vec[1].clone();
        let merged_bdd = bdd_vec[0].resolve(&snd, &var_ordering.ordering);
        let mut res_bdd = Bdd::new();
        let node1 = BddNode::mk_node(
            BddVar::new(83, 0.5),
            BddPointer::new(3),
            BddPointer::new_one(),
        );
        let node2 = BddNode::mk_node(
            BddVar::new(16, 0.5),
            BddPointer::new(2),
            BddPointer::new_one(),
        );
        let node3 = BddNode::mk_node(
            BddVar::new(65, 0.5),
            BddPointer::new_zero(),
            BddPointer::new_one(),
        );
        res_bdd.push_node(node3);
        res_bdd.push_node(node2);
        res_bdd.push_node(node1);
        assert_eq!(merged_bdd, res_bdd);
    }

    /*
    #[test]
    pub fn test_resolution_simple() {
        let path: &str = "/home/lid/Desktop/LMU/PhD/CDCL-support-by-BDD-methods/tests/test11.cnf";

        // create the Dimacs instance
        let dimacs = parse_dimacs(path);

        // build the variable ordering
        let var_ordering = BddVarOrdering::new(dimacs);

        let mut clause_set = Vec::new();
        let mut bdd_vec = Vec::new();
        let cloned_buckets= var_ordering.buckets.clone();

        // make the pairs in each bucket
        for (_, mut bucket) in var_ordering.buckets {
            bdd_vec.extend(bucket.process_bucket_bdd(&var_ordering.variables, &var_ordering.ordering));
        }
        for (_, mut bucket) in cloned_buckets {
            let pairs = bucket.make_pairs();
            let res_clauses: Vec<Expr> = resolve_pairs(pairs);
            if !res_clauses.is_empty() {
                clause_set.push(res_clauses);
            }
        }
        let mut res_clauses = Vec::new();
        for bdd in bdd_vec {
            res_clauses.push(bdd.to_bool_expr());
        }
        assert_eq!(clause_set, vec![res_clauses])
    }

    #[test]
    pub fn test_resolution_from_bench() {
        let path: &str = "/home/lid/Desktop/LMU/PhD/CDCL-support-by-BDD-methods/benchmarks/tests/sgen4-unsat-65-1.cnf";

        // create the Dimacs instance
        let dimacs = parse_dimacs(path);

        // build the variable ordering
        let var_ordering = BddVarOrdering::new(dimacs);

        let mut bdd_vec_resolution_functions = Vec::new();
        let mut bdd_vec_resolution_bdds = Vec::new();
        let cloned_buckets= var_ordering.buckets.clone();

        for (_, mut bucket) in cloned_buckets {
            let pairs = bucket.make_pairs();
            let resolution: Vec<Expr> = resolve_pairs(pairs);
            if !resolution.is_empty() {
                let mut bucket_bdds = Vec::new();
                for expr in resolution {
                    bucket_bdds.push(expr.to_bdd(&var_ordering.variables, &var_ordering.ordering));
                }
                if !bucket_bdds.is_empty() {
                    bdd_vec_resolution_functions.push(bucket_bdds);
                }
            }
        }

        for (_, mut bucket) in var_ordering.buckets {
            println!("{:?}", bucket);
            let processed_bucket = bucket.process_bucket_bdd(&var_ordering.variables, &var_ordering.ordering);
            println!("{:?}", processed_bucket);
            if !processed_bucket.is_empty() {
                let mut bucket_bdds = Vec::new();
                for bdd in processed_bucket {
                    bucket_bdds.push(bdd);
                }
                if !bucket_bdds.is_empty() {
                    bdd_vec_resolution_bdds.push(bucket_bdds);
                }
            }
        }

        assert_eq!(bdd_vec_resolution_bdds, bdd_vec_resolution_functions)

    }

    #[test]
    pub fn resolution_specific() {
        let path: &str = "/home/lid/Desktop/LMU/PhD/CDCL-support-by-BDD-methods/benchmarks/tests/sgen4-unsat-65-1.cnf";

        // create the Dimacs instance
        let dimacs = parse_dimacs(path);

        // build the variable ordering
        let var_ordering = BddVarOrdering::new(dimacs);
        let mut bucket = Bucket {
            index: 27,
            clauses: vec![Expr::Or(Box::new(Expr::Var(24)), Box::new(Expr::Or(Box::new(Expr::Not(Box::new(Expr::Var(21)))), Box::new(Expr::Var(27))))), Expr::Or(Box::new(Expr::Not(Box::new(Expr::Var(31)))), Box::new(Expr::Or(Box::new(Expr::Not(Box::new(Expr::Var(27)))), Box::new(Expr::Not(Box::new(Expr::Var(38))))))), Expr::Or(Box::new(Expr::Var(27)), Box::new(Expr::Or(Box::new(Expr::Not(Box::new(Expr::Var(10)))), Box::new(Expr::Var(24))))), Expr::Or(Box::new(Expr::Not(Box::new(Expr::Var(21)))), Box::new(Expr::Or(Box::new(Expr::Var(27)), Box::new(Expr::Not(Box::new(Expr::Var(10)))))))],
        };
        println!("{:?}", bucket);

        let mut cloned_bucket = bucket.clone();

        let processed_bucket = bucket.process_bucket_bdd(&var_ordering.variables, &var_ordering.ordering);

        let pairs = cloned_bucket.make_pairs();
        let resolution: Vec<Expr> = resolve_pairs(pairs);
        let mut bucket_bdds = Vec::new();
        if !resolution.is_empty() {
            for expr in resolution {
                bucket_bdds.push(expr.to_bdd(&var_ordering.variables, &var_ordering.ordering));
            }
        }
        assert_eq!(processed_bucket, bucket_bdds)

    }*/
}
 */
