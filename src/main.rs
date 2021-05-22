use std::path::PathBuf;

use anyhow::Result;
use argh::FromArgs;
use textplots::{Chart, Plot, Shape};

use usl::{Measurement, Model};

#[derive(Debug, FromArgs)]
#[argh(description = "build and evaluate Universal Scalability Law models")]
struct Opts {
    #[argh(positional, description = "path to input CSV")]
    input: PathBuf,

    #[argh(switch, long = "plot", description = "show plot of data")]
    plot: bool,

    #[argh(positional, description = "predict the throughput at the given concurrency levels")]
    predictions: Vec<u32>,
}

fn main() -> Result<()> {
    let opts: Opts = argh::from_env();

    let mut measurments = Vec::new();
    let mut input = csv::Reader::from_path(&opts.input)?;
    for record in input.records() {
        let record = record?;
        let m = Measurement::concurrency_and_throughput(record[0].parse()?, record[1].parse()?);
        measurments.push(m);
    }

    let model = Model::build(&measurments);
    println!("USL parameters: σ={:.6}, κ={:.6}, λ={:.6}", model.sigma, model.lambda, model.kappa);
    println!(
        "\tmax throughput: {:.6}, max concurrency: {:.6}",
        model.max_throughput(),
        model.max_concurrency()
    );
    if model.is_contention_constrained() {
        println!("\tcontention constrained");
    } else if model.is_coherency_constrained() {
        println!("\tcoherency constrained");
    } else if model.is_limitless() {
        println!("\tlinearly scalable");
    }

    if opts.plot {
        let points =
            measurments.iter().map(|m| (m.n as f32, m.x as f32)).collect::<Vec<(f32, f32)>>();
        Chart::new(200, 40, 0.0, model.max_throughput() as f32)
            .lineplot(&Shape::Continuous(Box::new(|n| {
                model.throughput_at_concurrency(n as u32) as f32
            })))
            .lineplot(&Shape::Points(&points))
            .nice();
    }

    for n in opts.predictions {
        println!("{},{}", n, model.throughput_at_concurrency(n));
    }

    Ok(())
}
