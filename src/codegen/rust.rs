use super::*;

pub fn generate(_tree: &Pkg, _o: &GenOpts) -> Vec<OutFile> {
    vec![]
}

#[allow(unused_variables)]
pub fn parse(src: &str, rel: &[String], file: &str) -> Parsed {
    Parsed { units: vec![], errors: false }
}
