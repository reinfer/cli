use anyhow::{bail, Context, Result};
use dialoguer::{FuzzySelect, Input, MultiSelect, Select};
use log::info;
use ordered_float::NotNan;
use reinfer_client::{resources::comment::PropertyFilter, Client, LabelDef, PropertyValue};
use structopt::StructOpt;

use crate::{commands::get::comments::get_user_properties_filter_interactively, printer::Printer};

#[derive(Debug, StructOpt)]
pub struct GetCustomLabelTrendReportArgs {}

pub fn get(
    client: &Client,
    _args: &GetCustomLabelTrendReportArgs,
    printer: &Printer,
) -> Result<()> {
    info!("Getting datasets...");
    let datasets = client.get_datasets()?;

    let dataset_selection = FuzzySelect::new()
        .with_prompt("Which dataset do you want to run this report for?")
        .items(
            &datasets
                .iter()
                .map(|dataset| dataset.full_name().0)
                .collect::<Vec<String>>(),
        )
        .interact()?;

    let dataset = &datasets[dataset_selection];

    info!("Getting dataset summary...");
    let summary_response = client.dataset_summary(&dataset.full_name(), &Default::default())?;

    info!("Getting labellers...");
    let labellers = client.get_labellers(&dataset.full_name())?;

    if labellers.is_empty() {
        bail!("Cannot get a label trend report for a dataset without any pinned models")
    }

    let label_defs = dataset.label_defs.iter();

    let label_inclusion_selections = MultiSelect::new()
        .with_prompt("Select which label(s) you want to include in your report")
        .items(
            &label_defs
                .map(|label| label.name.0.clone())
                .collect::<Vec<String>>(),
        )
        .interact()?;

    let excludable_label_defs: Vec<LabelDef> = dataset
        .label_defs
        .iter()
        .enumerate()
        .filter_map(|(idx, label_def)| {
            if label_inclusion_selections.contains(&idx) {
                None
            } else {
                Some(label_def.clone())
            }
        })
        .collect();

    let label_exclusion_selections = MultiSelect::new()
        .with_prompt("Select which label(s) you want to filter out of your report")
        .items(
            &dataset
                .label_defs
                .iter()
                .enumerate()
                .filter_map(|(idx, label)| {
                    if label_inclusion_selections.contains(&idx) {
                        None
                    } else {
                        Some(label.name.0.clone())
                    }
                })
                .collect::<Vec<String>>(),
        )
        .interact()?;

    let get_threshold_for_selections =
        |selections: Vec<usize>, label_defs: Vec<LabelDef>| -> Vec<(LabelDef, NotNan<f64>)> {
            selections
                .iter()
                .map(|selection| {
                    let label_def = &label_defs[*selection];

                    let confidence_str = Input::new()
                        .with_prompt(format!(
                            "What confidence do you want to use for the label \"{}\"",
                            label_def.name.0
                        ))
                        .validate_with(|input: &String| match input.trim().parse::<NotNan<f64>>() {
                            Ok(number) => {
                                if number >= NotNan::new(0.0).unwrap()
                                    && number <= NotNan::new(1.0).unwrap()
                                {
                                    Ok(())
                                } else {
                                    Err("Please enter a number between 0 and 1")
                                }
                            }

                            Err(_) => Err("Please enter a number between 0 and 1"),
                        })
                        .interact()
                        .expect("Could not get confidence string from user");

                    let confidence = confidence_str
                        .trim()
                        .parse::<NotNan<f64>>()
                        .expect("Could not parse user input");
                    (label_def.clone(), confidence)
                })
                .collect()
        };

    let label_inclusion_by_confidence =
        get_threshold_for_selections(label_inclusion_selections, dataset.label_defs.clone());
    let label_exclusion_by_confidence =
        get_threshold_for_selections(label_exclusion_selections, excludable_label_defs);

    let model_version_selections = MultiSelect::new()
        .with_prompt("Select which model version(s) you want to run this report for")
        .items(
            &labellers
                .iter()
                .map(|labeller| labeller.version.to_string())
                .collect::<Vec<String>>(),
        )
        .interact()?;
    get_user_properties_filter_interactively(client, dataset);

    todo!()
}
