use dialoguer::Select;

use crate::{config::Config, provider::Provider, utils::console::get_select_theme};

pub const NO_MODELS_FOUND_MSG: &str =
    "Unable to change model. Select a model in the Options menu to fix this issue.";

pub fn on_the_fly_change_model(
    cfg: &mut Config,
    active_model_id: Option<String>,
) -> Option<Provider> {
    let models = cfg.get_available_models(false);
    let active_model = if let Some(model) = active_model_id {
        models.iter().position(|(_, m, _)| *m == model).unwrap_or(0)
    } else if let Some(model) = cfg.active_model() {
        models
            .iter()
            .position(|(_, m, _)| *m == model.0)
            .unwrap_or(0)
    } else {
        0
    };

    let items = models
        .iter()
        .map(|(provider, _, display_name)| {
            let spaces: String = (0..18 - display_name.to_string().len())
                .map(|_| ' ')
                .collect();
            format!("{}{} {}", display_name, spaces, provider)
        })
        .collect::<Vec<String>>();

    if items.is_empty() {
        return None;
    }

    let Ok(selection) = Select::with_theme(&get_select_theme())
        .with_prompt("Select model")
        .items(&items)
        .default(active_model)
        .interact()
    else {
        // Exit if the user cancels the selection (e.g., Ctrl+C)
        std::process::exit(0);
    };

    let Some((provider_name, model_id, _)) = models.get(selection) else {
        println!("Invalid selection");
        return None;
    };

    let provider = cfg.find_provider(provider_name);
    match provider {
        Some(p) => {
            let mut provider = p.clone();
            provider.set_model(model_id.to_string());
            Some(provider)
        }
        None => None,
    }
}
