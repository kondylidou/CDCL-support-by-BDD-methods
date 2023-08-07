use std::collections::HashMap;
use std::fs;

use super::bool_expr::Expr;



fn main() {
    let file_path = "path/to/your/dimacs.cnf";
    match parse_dimacs_cnf_file(file_path) {
        Ok(expressions) => println!("Parsed expressions: {:?}", expressions),
        Err(err) => eprintln!("Error: {}", err),
    }
}

#[cfg(test)]
mod tests {

    #[cfg(test)]
    mod tests {

        #[test]
        pub fn test_parser() {
            // Use relative file paths for testing
            let filename: &str = "tests/test1.cnf";

            let mut vars = Vec::new();
            let mut clauses = Vec::new();

            vars.push(83);
            vars.push(16);
            vars.push(65);
            vars.push(188);
            vars.push(1);
            vars.push(171);
            vars.push(23);
            vars.push(132);
            vars.push(59);

            //-83 16 65 0
            // 188 1 171 0
            // 23 132 -59 0

            clauses.push(vec![-83, 16, 65]);
            clauses.push(vec![188, 1, 171]);
            clauses.push(vec![23, 132, -59]);

            // Parse the CNF formula into a boolean expression
            let expressions = parse_cnf_dimacs_file(filename).unwrap();

            // Assert the boolean expressions against the expected values
            assert_eq!(expressions, vec![
                Expr::Or(
                    Box::new(Expr::Or(
                        Box::new(Expr::Not(Box::new(Expr::Variable(83)))),
                        Box::new(Expr::And(
                            Box::new(Expr::Variable(16)),
                            Box::new(Expr::Variable(65))
                        ))
                    )),
                    Box::new(Expr::And(
                        Box::new(Expr::Variable(188)),
                        Box::new(Expr::And(
                            Box::new(Expr::Variable(1)),
                            Box::new(Expr::Variable(171))
                        ))
                    ))
                ),
                Expr::And(
                    Box::new(Expr::Variable(23)),
                    Box::new(Expr::Or(
                        Box::new(Expr::Variable(132)),
                        Box::new(Expr::Not(Box::new(Expr::Variable(59))))
                    ))
                )
            ]);
        }
    } 
}   