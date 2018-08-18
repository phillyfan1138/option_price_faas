extern crate fang_oost_option;
extern crate fang_oost;
extern crate rayon;
extern crate black_scholes;
extern crate cf_functions;
extern crate num_complex;
extern crate cf_dist_utils;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

extern crate constraints;

#[cfg(test)]
extern crate rand;
#[cfg(test)]
use rand::{SeedableRng, StdRng};
#[cfg(test)]
use rand::distributions::Uniform;
#[cfg(test)]
use rand::distributions::{Distribution};


use fang_oost_option::option_pricing;
use std::env;
use std::io;
use rayon::prelude::*;
use num_complex::Complex;


const PUT_PRICE:i32=0;
const CALL_PRICE:i32=1;

const PUT_DELTA:i32=2;
const CALL_DELTA:i32=3;

const PUT_GAMMA:i32=4;
const CALL_GAMMA:i32=5;

const PUT_THETA:i32=6;
const CALL_THETA:i32=7;

const DENSITY:i32=8;
const RISK_MEASURES:i32=9;


#[derive(Serialize, Deserialize)]
struct GraphElementIV {
    at_point:f64,
    value:f64,
    iv:f64
}
#[derive(Serialize, Deserialize)]
struct GraphElement {
    at_point:f64,
    value:f64
}
#[derive(Serialize, Deserialize)]
struct RiskMeasures {
    value_at_risk:f64,
    expected_shortfall:f64
}

fn get_jump_diffusion_vol(
    sigma:f64,
    lambda:f64,
    mu_l:f64,
    sig_l:f64,
    maturity:f64
)->f64 {
    ((sigma.powi(2)+lambda*(mu_l.powi(2)+sig_l.powi(2)))*maturity).sqrt()
}

fn print_risk_measures(
    risk_measure:(f64, f64)
) {
    let (expected_shortfall, value_at_risk)=risk_measure;
    let json_value=json!(
        RiskMeasures {
            value_at_risk,
            expected_shortfall
        }
    );
    println!("{}", serde_json::to_string_pretty(&json_value).unwrap())
}

fn print_density(
    x_values:&[f64],
    values:&[f64]
) { //void, prints to stdout
    let json_value=json!(
        x_values.iter().zip(values.iter()).map(|(x_val, val)|{
            GraphElement {
                at_point:*x_val,
                value:*val
            }
        }).collect::<Vec<_>>()
    );
    println!("{}", serde_json::to_string_pretty(&json_value).unwrap())
}
fn print_greeks(
    x_values:&[f64],
    values:&[f64]
) { //void, prints to stdout
    let x_val_crit=x_values.len()-1;
    let json_value=json!(
        x_values.iter().zip(values.iter())
            .enumerate().filter(|(index, _)|index>&0&&index<&x_val_crit)
            .map(|(_, (x_val, val))|{
                GraphElement {
                    at_point:*x_val,
                    value:*val
                }
            }).collect::<Vec<_>>()
    );
    println!("{}", serde_json::to_string_pretty(&json_value).unwrap())
}

fn print_call_prices(
    strikes:&[f64],
    values:&[f64],
    asset:f64,
    rate:f64,
    maturity:f64
) { //void, prints to stdout
    let x_val_crit=values.len()-1;
    let json_call_prices=json!(
        strikes.iter().zip(values.iter())
            .enumerate().filter(|(index, _)|index>&0&&index<&x_val_crit)
            .map(|(_, (strike, price))|{
                let iv=black_scholes::call_iv(*price, asset, *strike, rate, maturity, 0.3);
                GraphElementIV {
                    at_point:*strike,
                    value:*price,
                    iv
                }
            }).collect::<Vec<_>>()
    );
    println!("{}", serde_json::to_string_pretty(&json_call_prices).unwrap())
}

fn adjust_density<T>(
    num_u:usize,
    x_max:f64,
    cf:T    
) 
    where T:Fn(&Complex<f64>)->Complex<f64>+
    std::marker::Sync+std::marker::Send
{
    let num_x=128;
    let x_range=fang_oost::compute_x_range(
        num_x, -x_max, x_max
    );
    let option_range:Vec<f64>=fang_oost::get_density(
        num_u, &x_range, cf
    ).collect();
    print_density(&x_range, &option_range)
}


fn get_vol_from_parameters(
    parameters:&constraints::OptionParameters
)->f64{
    let constraints::OptionParameters {
        sigma, lambda, mu_l, 
        sig_l, maturity, ..
    }=parameters;
    get_jump_diffusion_vol(
        *sigma, *lambda,
        *mu_l, *sig_l, 
        *maturity
    )
}

fn main()-> Result<(), io::Error> {
    let args: Vec<String> = env::args().collect();
    let fn_choice:i32=args[1].parse().unwrap();
    let mut parameters:constraints::OptionParameters=serde_json::from_str(&args[2])?;
    constraints::check_constraints(
        &parameters, 
        &constraints::get_constraints()
    )?;
    let x_max_density=get_vol_from_parameters(&parameters)*5.0;
    let x_max_options=x_max_density*2.0;
    parameters.extend_k(x_max_options);
    
    let constraints::OptionParameters {
        maturity,
        rate,
        asset,
        lambda,
        mu_l,
        sig_l,
        sigma,
        v0,
        speed,
        eta_v,
        rho,
        strikes,
        quantile,
        num_u:num_u_base
    }=parameters; //destructure
    
    let num_u=(2 as usize).pow(num_u_base as u32);
    let strikes=Vec::from(strikes);
    //note...if pass by reference doesn't work 
    //I can always move this value since I only
    //use it once.  However, if I ever want 
    //the binary to stay "live" for multiple
    //calls I'll need to keep this reference around
    let inst_cf=cf_functions::merton_time_change_cf(
        maturity, rate, lambda, mu_l, sig_l, sigma, v0,
        speed, eta_v, rho
    );

    match fn_choice {
        CALL_PRICE => print_call_prices(
                &strikes,
                &option_pricing::fang_oost_call_price(
                    num_u, asset, 
                    &strikes, rate, 
                    maturity, &inst_cf
                ),
                asset, rate, maturity
            ),
        PUT_PRICE => print_greeks(
                &strikes,
                &option_pricing::fang_oost_put_price(
                    num_u, asset, 
                    &strikes, rate, 
                    maturity, &inst_cf
                )
            ),
        CALL_DELTA => print_greeks(
                &strikes,
                &option_pricing::fang_oost_call_delta(
                    num_u, asset, 
                    &strikes, rate, 
                    maturity, &inst_cf
                )
            ),
        PUT_DELTA => print_greeks(
                &strikes,
                &option_pricing::fang_oost_put_delta(
                    num_u, asset, 
                    &strikes, rate, 
                    maturity, &inst_cf
                )
            ),
        CALL_GAMMA => print_greeks(
                &strikes,
                &option_pricing::fang_oost_call_gamma(
                    num_u, asset, 
                    &strikes, rate, 
                    maturity, &inst_cf
                )
            ),
        PUT_GAMMA => print_greeks(
                &strikes,
                &option_pricing::fang_oost_put_gamma(
                    num_u, asset, 
                    &strikes, rate, 
                    maturity, &inst_cf
                )
            ),
        CALL_THETA => print_greeks(
                &strikes,
                &option_pricing::fang_oost_call_theta(
                    num_u, asset, 
                    &strikes, rate, 
                    maturity, &inst_cf
                )
            ),
        PUT_THETA => print_greeks(
                &strikes,
                &option_pricing::fang_oost_put_theta(
                    num_u, asset, 
                    &strikes, rate, 
                    maturity, &inst_cf
                )
            ),
        DENSITY => adjust_density(
                num_u, x_max_density, &inst_cf
            ),
        RISK_MEASURES => print_risk_measures(
                cf_dist_utils::get_expected_shortfall_and_value_at_risk(
                    quantile, num_u, -x_max_density, x_max_density, &inst_cf
                )
            ),
        _ => println!("wow, nothing")
    }
    Ok(())
}



#[cfg(test)]
mod tests {
    use super::*;
    fn get_rng_seed(seed:[u8; 32])->StdRng{
        SeedableRng::from_seed(seed) 
    }
    fn get_over_region(lower:f64, upper:f64, rand:f64)->f64{
        lower+(upper-lower)*rand
    }
    #[test]
    fn test_many_inputs(){
        let seed:[u8; 32]=[2; 32];
        let mut rng_seed=get_rng_seed(seed);
        let uniform=Uniform::new(0.0f64, 1.0);
        let constr=constraints::get_constraints();
        let asset=178.46;
        let num_u=256;
        let strikes=vec![
            95.0,130.0,150.0,160.0,
            165.0,170.0,175.0,185.0,
            190.0,195.0,200.0,210.0,240.0,250.0
        ];
        let maturity=0.86;
        let rate=0.02;
        let num_total:usize=10000;
        let mut num_bad:usize=0;
        (0..num_total).for_each(|index|{
            let lambda_sim=get_over_region(
                constr.lambda.lower,
                constr.lambda.upper,
                uniform.sample(&mut rng_seed)
            );
            let mu_l_sim=get_over_region(
                constr.mu_l.lower,
                constr.mu_l.upper,
                uniform.sample(&mut rng_seed)
            );
            let sig_l_sim=get_over_region(
                constr.sig_l.lower,
                constr.sig_l.upper,
                uniform.sample(&mut rng_seed)
            );
            let sigma_sim=get_over_region(
                constr.sigma.lower,
                constr.sigma.upper,
                uniform.sample(&mut rng_seed)
            );
            let v0_sim=get_over_region(
                constr.v0.lower,
                constr.v0.upper,
                uniform.sample(&mut rng_seed)
            );
            let speed_sim=get_over_region(
                constr.speed.lower,
                constr.speed.upper,
                uniform.sample(&mut rng_seed)
            );
            let eta_v_sim=get_over_region(
                constr.eta_v.lower,
                constr.eta_v.upper,
                uniform.sample(&mut rng_seed)
            );
            let rho_sim=get_over_region(
                constr.rho.lower,
                constr.rho.upper,
                uniform.sample(&mut rng_seed)
            );

            let inst_cf=cf_functions::merton_time_change_cf(
                maturity, rate, lambda_sim, 
                mu_l_sim, sig_l_sim, 
                sigma_sim, v0_sim,
                speed_sim, eta_v_sim, rho_sim
            );
            let opt_prices=option_pricing::fang_oost_call_price(
                num_u, asset, 
                &strikes, rate, 
                maturity, &inst_cf
            );
            
            for option_price in opt_prices.iter(){
                if option_price.is_nan()||option_price.is_infinite(){
                    println!("lambda: {}", lambda_sim);
                    println!("mu_l: {}", mu_l_sim);
                    println!("sig_l: {}", sig_l_sim);
                    println!("sigma: {}", sigma_sim);
                    println!("v0: {}", v0_sim);
                    println!("speed: {}", speed_sim);
                    println!("eta_v: {}", eta_v_sim);
                    println!("rho: {}", rho_sim);
                    num_bad=num_bad+1;
                    break;
                }
                //assert_eq!(!option_price.is_nan());
            }
        });
        let bad_rate=(num_bad as f64)/(num_total as f64);
        println!("Bad rate: {}", bad_rate);
        assert_eq!(bad_rate, 0.0);
    }

}