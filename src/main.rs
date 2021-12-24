use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, ValueHint};
use plotlib::page::Page;
use plotlib::repr::Plot;
use plotlib::style::{PointMarker, PointStyle};
use plotlib::view::ContinuousView;

use usl::{Measurement, Model};

/// Build and evaluate Universal Scalability Law models.
#[derive(Debug, Parser)]
#[clap(author, version, about)]
struct Opts {
    /// Path to input CSV file.
    #[clap(value_hint = ValueHint::FilePath)]
    input: PathBuf,

    /// Show plot of data.
    #[clap(long)]
    plot: bool,

    /// Predict the throughput at the given concurrency levels.
    predictions: Vec<u32>,
}

fn main() -> Result<()> {
    let opts: Opts = Opts::parse();

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
        let observed = measurments.iter().map(|m| (m.n, m.x)).collect::<Vec<(f64, f64)>>();
        let max_n = observed.iter().map(|&(n, _)| n).fold(0.0, f64::max);
        let observed =
            Plot::new(observed).point_style(PointStyle::new().marker(PointMarker::Square));

        let predicted = (0..(max_n as usize))
            .step_by(max_n as usize / 10)
            .map(|n| (n as f64, model.throughput_at_concurrency(n as u32)))
            .collect();
        let predicted =
            Plot::new(predicted).point_style(PointStyle::new().marker(PointMarker::Circle));

        let extrapolated = opts
            .predictions
            .iter()
            .map(|&n| (n as f64, model.throughput_at_concurrency(n)))
            .collect();
        let extrapolated =
            Plot::new(extrapolated).point_style(PointStyle::new().marker(PointMarker::Cross));

        let v = ContinuousView::new()
            .add(observed)
            .add(predicted)
            .add(extrapolated)
            .x_range(0.0, max_n)
            .y_range(0.0, model.max_throughput())
            .x_label("concurrency")
            .y_label("throughput");

        println!("{}", Page::single(&v).dimensions(80, 20).to_text().unwrap());
    }

    for n in opts.predictions {
        println!("{},{}", n, model.throughput_at_concurrency(n));
    }

    Ok(())
}
