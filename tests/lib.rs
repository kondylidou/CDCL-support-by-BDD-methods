use std::time::Instant;

use bdd_sat_solver::{
    get_glucose_solution, get_glucose_solution_no_malloc, init_glucose_solver,
    parse_dimacs_and_add_clause_to_glucose, run_glucose,
};

#[test]
pub fn test_solver_get_solution_1() {
    let solver = init_glucose_solver();
    let nb_v = parse_dimacs_and_add_clause_to_glucose("tests/test4.cnf".to_string(), solver);
    let ret = run_glucose(solver);
    match ret {
        0 => {
            let _sol = get_glucose_solution(solver, nb_v);
        }
        _ => println!("Solution assertion failed."),
    }
}

#[test]
pub fn test_solver_get_solution_2() {
    let solver = init_glucose_solver();
    let start = Instant::now();
    let nb_v = parse_dimacs_and_add_clause_to_glucose(
        "/home/lkondylidou/Desktop/PhD/CDCL-support-by-BDD-methods/benchmarks/tests/0b1041a1e55af6f3d2c63462a7400bd2-fermat-907547022132073.cnf".to_string(),
        solver,
    );
    let ret = run_glucose(solver);
    match ret {
        0 => {
            let mut sol = Vec::with_capacity(nb_v);
            get_glucose_solution_no_malloc(solver, &mut sol, nb_v);
        }
        _ => println!("Solution assertion failed."),
    }
    println!("Time elapsed is : {:?}", start.elapsed());
}
