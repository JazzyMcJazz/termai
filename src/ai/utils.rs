use console::style;
use dialoguer::{MultiSelect, Select};

use crate::{config::Config, provider::Provider, utils::console::get_select_theme};

pub const NO_MODELS_FOUND_MSG: &str =
    "Unable to change model. Select a model in the Options menu to fix this issue.";

pub const NO_SEARCH_MODELS_FOUND_MSG: &str = "No search models available.";

pub async fn on_the_fly_change_model(
    cfg: &mut Config,
    active_model_id: Option<String>,
    search: bool,
) -> Option<Provider> {
    let models = if search {
        cfg.available_search_models().to_owned()
    } else {
        cfg.available_completion_models().to_owned()
    };

    // Find the index of the active model
    let active_model = if let Some(model) = active_model_id {
        models.iter().position(|(_, m, _)| *m == model).unwrap_or(0)
    } else if let Some(model) = cfg.active_model(search) {
        models
            .iter()
            .position(|(_, m, _)| *m == model.0)
            .unwrap_or(0)
    } else {
        0
    };

    // Get the list of models
    let items = models
        .iter()
        .map(|(provider, _, display_name)| {
            let spaces: String = (0..26 - display_name.to_string().len())
                .map(|_| ' ')
                .collect();
            format!("{}{} {}", display_name, spaces, provider)
        })
        .collect::<Vec<String>>();

    if items.is_empty() {
        return None;
    }

    // Create a selection dialog
    let selection = Select::with_theme(&get_select_theme())
        .with_prompt("Select model")
        .items(&items)
        .default(active_model)
        .interact()
        .unwrap_or_else(|_| std::process::exit(0));

    let Some((provider_name, model_id, _)) = models.get(selection) else {
        println!("Invalid selection");
        return None;
    };

    if search {
        cfg.set_search_model(provider_name.to_owned(), model_id.to_owned());
    } else {
        cfg.set_completion_model(provider_name.to_owned(), model_id.to_owned());
    }

    let provider = cfg.find_provider(provider_name);
    match provider {
        Some(p) => {
            let mut provider = p.clone();
            if search {
                provider.set_search_model(model_id.to_string());
            } else {
                provider.set_completion_model(model_id.to_string());
            }
            Some(provider)
        }
        None => None,
    }
}

pub fn on_the_fly_select_mcp_client(cfg: &mut Config) {
    let clients = cfg.mcp_clients_mut();

    let items = clients
        .iter()
        .map(|client| format!("{} ({})", client.name(), client.version()))
        .collect::<Vec<String>>();

    if items.is_empty() {
        println!("{} No MCP servers configured", style("✗").red());
        return;
    }

    let defaults = clients
        .iter()
        .map(|client| client.is_enabled())
        .collect::<Vec<bool>>();

    let selections = MultiSelect::with_theme(&get_select_theme())
        .with_prompt("Enable / disable MCP servers")
        .items(&items[..])
        .defaults(&defaults[..])
        .interact()
        .unwrap_or_else(|_| std::process::exit(0));

    for (i, client) in clients.iter_mut().enumerate() {
        client.set_enabled(selections.contains(&i));
    }

    cfg.save();
}
