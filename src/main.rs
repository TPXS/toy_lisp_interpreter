use std::collections::HashMap;
use std::num::ParseFloatError;
use std::{fmt, io};

/*
macro_rules! ensure_tonicity {
    ($check_fn:expr) => {{
      |args: &[LispExpression]| -> Result<LispExpression, LispError> {
        let floats = parse_list_of_floats(args)?;
        let first = floats.first().ok_or(LispError::Cause("Requires at least one number".to_string()))?;
        let remaining = &floats[1..];
        fn f (prev: &f64, xs: &[f64]) -> bool {
            match xs.first() {
                Some(x) => $check_fn(prev, x) && f(x, &xs[1..]),
                None => true,
            }
        };
        Ok(LispExpression::Bool(f(first, remaining)))
      }
    }};
}
*/

#[derive(Clone)]
enum LispExpression {
    Symbol(String),
    Number(f64),
    List(Vec<LispExpression>),
    Function(fn(&[LispExpression]) -> Result<LispExpression, LispError>),
}

impl fmt::Display for LispExpression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let str = match self {
            LispExpression::Symbol(s) => s.clone(),
            LispExpression::Number(r) => r.to_string(),
            LispExpression::List(list) => {
                let xs: Vec<String> = list.iter().map(|x| x.to_string()).collect();
                format!("({})", xs.join(","))
            },
            LispExpression::Function(_) => "Function {}".to_string(),
        };

        write!(f, "{}", str)
    }
}

#[derive(Debug)]
enum LispError {
    Cause(String),
    //TODO: add errors
}

#[derive(Clone)]
struct LispEnvironment {
    data: HashMap<String, LispExpression>,
}

fn main() {
    //println!("Hello, world!");

    let environment = &mut as_environment();
    loop {
        println!("LISPulator >");
        let expression = input_lisp_expression();
        match lisp_parse_evaluate(expression, environment) {
            Ok(result) => println!("Result => {}", result),
            Err(e) => match e {
                LispError::Cause(message) => println!("Error! => {}", message),
            },
        }
    }
}

fn input_lisp_expression() -> String {
    let mut expression = String::new();

    io::stdin().read_line(&mut expression).expect("Failed to read line");

    expression
}

fn lisp_parse_evaluate(expression: String, environment: &mut LispEnvironment) -> Result<LispExpression, LispError> {
    let (parsed_expression, _) = lisp_parse(&lisp_tokenize(expression))?;
    let evaluated_expression = evaluate(&parsed_expression, environment)?;

    Ok(evaluated_expression)
}

fn evaluate(expression: &LispExpression, environment: &mut LispEnvironment) -> Result<LispExpression, LispError> {
    match expression {
        LispExpression::Symbol(op) => environment.data.get(op).ok_or(LispError::Cause(format!("unexpected symbol: {}", op))).map(|x| x.clone()),
        LispExpression::Number(_r) => Ok(expression.clone()),
        LispExpression::List(list) => {
            let initial_form = list.first().ok_or(LispError::Cause("List is empty".to_string()))?;
            let argument_forms = &list[1..];
            let initial_evaluation = evaluate(initial_form, environment)?;
            match initial_evaluation {
                LispExpression::Function(f) => {
                    let arguments_evaluation = argument_forms.iter().map(|x| evaluate(x, environment)).collect::<Result<Vec<LispExpression>, LispError>>(); 
                    f(&arguments_evaluation?)
                },
                _ => Err(LispError::Cause("Initial form must be a function".to_string())),
            }
        },
        LispExpression::Function(_) => Err(LispError::Cause("Unexpected Form".to_string())),
    }
}

fn as_environment() -> LispEnvironment { //add, subtract, multiply, divide, //TODO: add other functions
    let mut data: HashMap<String, LispExpression> = HashMap::new();
    data.insert(
        "+".to_string(),
        LispExpression::Function(
            |arguments: &[LispExpression]| -> Result<LispExpression, LispError> {
                let sum = parse_list_of_floats(arguments)?.iter().fold(0.0, |sum, r| sum + r);
                Ok(LispExpression::Number(sum))
            }
        )
    );
    data.insert(
        "-".to_string(), 
        LispExpression::Function(
          |arguments: &[LispExpression]| -> Result<LispExpression, LispError> {
            let floats = parse_list_of_floats(arguments)?;
            let first = *floats.first().ok_or(LispError::Cause("Requires at least one number".to_string()))?;
            let sum_of_remaining = floats[1..].iter().fold(0.0, |sum, a| sum + a);
            
            Ok(LispExpression::Number(first - sum_of_remaining))
          }
        )
    );
    /*
    data.insert(
        "=".to_string(), 
        LispExpression::Function(ensure_tonicity!(|a, b| a == b))
    );
    data.insert(
        ">".to_string(), 
        LispExpression::Function(ensure_tonicity!(|a, b| a > b))
    );
    data.insert(
        ">=".to_string(), 
        LispExpression::Function(ensure_tonicity!(|a, b| a >= b))
    );
    data.insert(
        "<".to_string(), 
        LispExpression::Function(ensure_tonicity!(|a, b| a < b))
    );
    data.insert(
        "<=".to_string(), 
        LispExpression::Function(ensure_tonicity!(|a, b| a <= b))
    );
    */
    LispEnvironment {data}
} 

    /*
    data.insert(
        "*".to_string(),
        LispExpression::Function(
            |arguments: &[LispExpression]| -> Result<LispExpression, LispError> {
                let product = parse_list_of_floats(arguments)?.iter().fold(1.0, |product, r| product * r);
                Ok(LispExpression::Number(product))
            }
        )
    );
    data.insert(
        "/".to_string(),
        LispExpression::Function(
            |arguments: &[LispExpression]| -> Result<LispExpression, LispError> {
                let quotient = parse_list_of_floats(arguments)?.iter().fold(1.0, |quotient, r| quotient / r);
                Ok(LispExpression::Number(quotient))
            }
        )
    ); */



fn parse_list_of_floats(arguments: &[LispExpression]) -> Result<Vec<f64>, LispError> {
    arguments
      .iter()
      .map(|x| parse_single_float(x))
      .collect()
  }
  
  fn parse_single_float(expression: &LispExpression) -> Result<f64, LispError> {
    match expression {
      LispExpression::Number(num) => Ok(*num),
      _ => Err(LispError::Cause("expected a number".to_string())),
    }
  }

fn lisp_tokenize(expression: String) -> Vec<String> {
    expression
        .replace("(", " ( ")
        .replace(")", " ) ")
        .split_whitespace()
        .map(|x| x.to_string())
        .collect()
}

fn read_sequentially<'a>(tokens: &'a [String]) -> Result<(LispExpression, &'a [String]), LispError> {
    let mut res: Vec<LispExpression> = Vec::new();
    let mut xs = tokens;
    loop {
        let (next_token, remaining) = xs. split_first().ok_or(LispError::Cause("No closing `)`".to_string()))?;
        if next_token == ")" {
            return Ok((LispExpression::List(res), remaining))
        }
        let (expression, new_xs) = lisp_parse(&xs)?;
        res.push(expression);
        xs = new_xs;

    }
}

fn parse_atomic(token: &str) -> LispExpression {
    let potential_float: Result<f64, ParseFloatError> = token.parse();
    match potential_float {
        Ok(r) => LispExpression::Number(r),
        Err(_) => LispExpression::Symbol(token.to_string().clone())
    }
}

fn lisp_parse<'a>(tokens: &'a [String]) -> Result<(LispExpression, &'a [String]), LispError> {
    let (token, remaining) = tokens.split_first().ok_or(LispError::Cause("Unable to get token".to_string()))?;
    match &token[..] {
        "(" => read_sequentially(remaining),
        ")" => Err(LispError::Cause("Unexpected `)`".to_string())),
        _ => Ok((parse_atomic(token), remaining)),
    }
}

