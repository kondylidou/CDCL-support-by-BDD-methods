use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;
use hashbrown::HashMap as FastHashMap;
use nom::branch::alt;
use nom::{
    bytes::complete::tag,
    character::complete::{digit1, line_ending, space0, space1},
    combinator::{map, map_res},
    multi::many0,
    sequence::{delimited, separated_pair},
    IResult,
};

// Define the BooleanExpression enum to represent the different types of boolean expressions
#[derive(Debug)]
enum BooleanExpression {
    Variable(i32),
    Not(Box<BooleanExpression>),
    And(Box<BooleanExpression>, Box<BooleanExpression>),
    Or(Box<BooleanExpression>, Box<BooleanExpression>),
}

// Helper function to create a variable expression
fn var(variable_id: i32) -> BooleanExpression {
    BooleanExpression::Variable(variable_id)
}

// Helper function to create a NOT expression
fn not(expr: BooleanExpression) -> BooleanExpression {
    BooleanExpression::Not(Box::new(expr))
}

// Helper function to create an AND expression
fn and(left: BooleanExpression, right: BooleanExpression) -> BooleanExpression {
    BooleanExpression::And(Box::new(left), Box::new(right))
}

// Helper function to create an OR expression
fn or(left: BooleanExpression, right: BooleanExpression) -> BooleanExpression {
    BooleanExpression::Or(Box::new(left), Box::new(right))
}

// Parse an integer and convert it to i32
fn parse_i32(input: &str) -> IResult<&str, i32> {
    map_res(digit1, |s: &str| s.parse::<i32>())(input)
}

// Parse a DIMACS variable and convert it to i32
fn parse_dimacs_variable(input: &str) -> IResult<&str, i32> {
    let (input, _) = tag(" ")(input)?;
    let (input, var) = parse_i32(input)?;
    Ok((input, var))
}

// Parse a variable name
fn variable(input: &str, var_map: FastHashMap<i32, i32>) -> IResult<&str, BooleanExpression> {
    let (input, v) = parse_dimacs_variable(input)?;
    let var_id = v.abs();
    let mapped_var = var_map.get(&var_id).copied().unwrap_or(var_id);
    Ok((input, var(mapped_var)))
}

// Parse a NOT expression
fn not_expr(input: &str, var_map: FastHashMap<i32, i32>) -> IResult<&str, BooleanExpression> {
    let (input, _) = tag("-")(input)?;
    let (input, expr) = expression(input, var_map)?;
    Ok((input, not(expr)))
}

// Parse an AND expression
fn and_expr(input: &str, var_map: FastHashMap<i32, i32>) -> IResult<&str, BooleanExpression> {
    let (input, _) = tag(" ")(input)?;
    let (input, left) = expression(input, var_map)?;
    let (input, _) = space1(input)?;
    let (input, right) = expression(input, var_map)?;
    Ok((input, and(left, right)))
}

// Parse an OR expression
fn or_expr(input: &str, var_map: FastHashMap<i32, i32>) -> IResult<&str, BooleanExpression> {
    let (input, _) = tag(" ")(input)?;
    let (input, left) = expression(input, var_map)?;
    let (input, _) = space1(input)?;
    let (input, right) = expression(input, var_map)?;
    Ok((input, or(left, right)))
}

// Parse a boolean expression
fn expression(input: &str, var_map: FastHashMap<i32, i32>) -> IResult<&str, BooleanExpression> {
    let var_map = var_map.clone();
    alt((|i| variable(i, var_map), |i| not_expr(i, var_map), |i| and_expr(i, var_map), |i| or_expr(i, var_map)))(input)
}

// Parse multiple lines of boolean expressions
fn parse_cnf(input: &str) -> IResult<&str, Vec<BooleanExpression>> {
    let mut var_map: FastHashMap<i32, i32> = FastHashMap::new();
    let mut expr_list: Vec<BooleanExpression> = Vec::new();
    let (mut input, _) = tag("p cnf")(input)?;

    loop {
        let (i, _) = space1(input)?;
        let (i, var_count) = parse_i32(i)?;
        let (i, clause_count) = parse_i32(i)?;

        for var_id in 1..=var_count {
            var_map.insert(var_id, var_id);
            var_map.insert(-var_id, -var_id);
        }

        for _ in 0..clause_count {
            let (i, _) = space1(i)?;
            let (i, clause) = expression(i, var_map)?;
            expr_list.push(clause);
        }

        let (i, _) = space0(i)?;
        if i.is_empty() {
            break;
        }

        input = i;
    }

    Ok((input, expr_list))
}

// Parse a CNF DIMACS file and return the boolean expressions
fn parse_cnf_dimacs_file(filename: &str) -> Result<Vec<BooleanExpression>, std::io::Error> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    let mut input = String::new();

    for line in reader.lines() {
        input.push_str(&line?);
        input.push('\n');
    }

    let (_, expressions) = parse_cnf(&input).unwrap();
    Ok(expressions)
}


#[cfg(test)]
mod tests {
    use super::parse_cnf_dimacs_file;

    #[test]
    pub fn test_parser() {
        let filename: &str = "/home/lkondylidou/Desktop/PhD/CDCL-support-by-BDD-methods/tests/test1.cnf";

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

        // Print the boolean expressions
        for expr in expressions {
            println!("{:?}", expr);
        }
    }
}
