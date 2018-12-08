#[macro_use]
extern crate serde_json;
extern crate lambda_http;
extern crate lambda_runtime as runtime;
extern crate utils;
use lambda_http::{lambda, IntoResponse, Request, RequestExt};
use runtime::{error::HandlerError, Context};
use std::error::Error;
use std::io;
use utils::constraints;
use utils::http_helper;

fn main() -> Result<(), Box<dyn Error>> {
    lambda!(output_constraints_wrapper);
    Ok(())
}
fn output_constraints_wrapper(
    event: Request,
    _ctx: Context,
) -> Result<impl IntoResponse, HandlerError> {
    match output_constraints(event) {
        Ok(res) => Ok(http_helper::build_response(200, &res)),
        Err(e) => Ok(http_helper::build_response(
            400,
            &http_helper::construct_error(&e.to_string()),
        )),
    }
}

fn output_constraints(event: Request) -> Result<String, io::Error> {
    let default_model = "";
    let path_parameters = event.path_parameters();
    let model = match path_parameters.get("model") {
        Some(m) => m,
        None => default_model,
    };
    let results = match model {
        "heston" => json!(constraints::get_heston_constraints()).to_string(),
        "cgmy" => json!(constraints::get_cgmy_constraints()).to_string(),
        "merton" => json!(constraints::get_merton_constraints()).to_string(),
        _ => json!(constraints::get_constraints()).to_string(),
    };
    Ok(results)
}
