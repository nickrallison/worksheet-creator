use rand::prelude::*;
use serde::Deserialize;
use std::fs::File;
use std::io::Write;
use std::path::Path;

#[derive(Deserialize)]
struct Variable {
    #[serde(rename = "type")]
    var_type: String,
    range: (f64, f64),
    round: Option<i32>,
}

#[derive(Deserialize)]
struct Intermediate {
    formula: String,
    dependencies: Vec<String>,
    round: Option<i32>,
}

#[derive(Deserialize)]
struct Question {
    question: String,
    variables: std::collections::HashMap<String, Variable>,
    intermediates: std::collections::HashMap<String, Intermediate>,
    answer: String,
    solution: String,
}

#[derive(Deserialize)]
struct Questions {
    questions: Vec<Question>,
}

fn main() {
    let seed_value: u64 = 42; // Example seed value
    let columns: usize = 2; // Example column value
    let num_problems: usize = 1; // Example number of problems

    let mut rng = StdRng::seed_from_u64(seed_value);

    let file_path = "questions/impulse.json";
    let file = File::open(file_path).expect("Unable to open file");
    let questions: Questions = serde_json::from_reader(file).expect("Unable to parse JSON");

    let mut tex_content = String::new();
    tex_content.push_str("\\documentclass{article}\n\\usepackage{amsmath}\n\\begin{document}\n");
    // push seed value to tex_content
    tex_content.push_str(&format!("Seed value: {}\n", seed_value));
    
    for _ in 0..num_problems {
        for question in &questions.questions {
            let mut vars = std::collections::HashMap::new();
            for (var_name, var) in &question.variables {
                let value = match var.var_type.as_str() {
                    "int" => rng.gen_range(var.range.0 as i64..=var.range.1 as i64) as f64,
                    "float" => rng.gen_range(var.range.0..=var.range.1),
                    _ => panic!("Unknown variable type"),
                };
                let value = match var.round {
                    Some(round) => (value * 10_f64.powi(round as i32)).round() / 10_f64.powi(round as i32),
                    None => value,
                };
                vars.insert(var_name.clone(), value);
            }

            let mut intermediates = std::collections::HashMap::new();
            for (inter_name, inter) in &question.intermediates {
                let mut formula = inter.formula.clone();
                for dep in &inter.dependencies {
                    let value = vars.get(dep).unwrap();
                    formula = formula.replace(dep, &value.to_string());
                }
                let value: f64 = meval::eval_str(&formula).unwrap();
                let value = match inter.round {
                    Some(round) => (value * 10_f64.powi(round as i32)).round() / 10_f64.powi(round as i32),
                    None => value,
                };
                intermediates.insert(inter_name.clone(), value);
            }

            let mut question_text = question.question.clone();
            for (var_name, value) in &vars {
                question_text = question_text.replace(&format!("[{}]", var_name), &value.to_string());
            }
            for (inter_name, value) in &intermediates {
                question_text = question_text.replace(&format!("[{}]", inter_name), &value.to_string());
            }

            let mut answer_text = question.answer.clone();
            for (var_name, value) in &vars {
                answer_text = answer_text.replace(&format!("[{}]", var_name), &value.to_string());
            }
            for (inter_name, value) in &intermediates {
                answer_text = answer_text.replace(&format!("[{}]", inter_name), &value.to_string());
            }

            let mut solution_text = question.solution.clone();
            for (var_name, value) in &vars {
                solution_text = solution_text.replace(&format!("[{}]", var_name), &value.to_string());
            }
            for (inter_name, value) in &intermediates {
                solution_text = solution_text.replace(&format!("[{}]", inter_name), &value.to_string());
            }

            tex_content.push_str(&format!("\\section*{{Question}}\n{}\n", question_text));
            tex_content.push_str(&format!("\\section*{{Answer}}\n{}\n", answer_text));
            tex_content.push_str(&format!("\\section*{{Solution}}\n{}\n", solution_text));
        }
    }

    tex_content.push_str("\\end{document}\n");

    let output_path = Path::new("output.tex");
    let mut output_file = File::create(&output_path).expect("Unable to create file");
    output_file.write_all(tex_content.as_bytes()).expect("Unable to write data");
}