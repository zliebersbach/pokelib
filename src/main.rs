#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod str_ext;

use std::sync::{Arc, Mutex};

use eframe::egui;
use egui::{Image, ScrollArea, TextureOptions};
use egui_extras::{Column, TableBuilder};
use rustemon::{
    client::RustemonClient,
    model::{
        pokemon::{Pokemon, PokemonSpecies},
        resource::NamedApiResource,
    },
    Follow,
};
use str_ext::StrExt;

const APP_NAME: &str = "PokéLib";

#[tokio::main]
async fn main() {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 600.0]),
        ..Default::default()
    };
    let _ = eframe::run_native(
        APP_NAME,
        options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);

            Box::<MyApp>::default()
        }),
    );
}

struct MyApp {
    loading: Arc<Mutex<bool>>,
    pkmn_species: Arc<Mutex<Option<Vec<NamedApiResource<PokemonSpecies>>>>>,
    search_term: String,
    pkmn_selected: Arc<Mutex<Option<SelectedPokemonSpecies>>>,
}

#[derive(Debug)]
struct SelectedPokemonSpecies {
    species: PokemonSpecies,
    pokemon: Vec<Pokemon>,
}

impl SelectedPokemonSpecies {
    pub fn new(species: PokemonSpecies, pokemon: Vec<Pokemon>) -> Self {
        Self { species, pokemon }
    }
}

impl MyApp {
    fn load_pkmn_species(&self) {
        // Fetch Pokemon species async
        let loading_cln = self.loading.clone();
        let pkmn_species_cln = self.pkmn_species.clone();

        *loading_cln.lock().unwrap() = true;
        *pkmn_species_cln.lock().unwrap() = None;
        tokio::spawn(async move {
            let client = RustemonClient::default();
            let mut species = rustemon::pokemon::pokemon_species::get_all_entries(&client)
                .await
                .expect("failed to load pokemon species");
            species.sort_by_key(|s| s.name.clone());
            *pkmn_species_cln.lock().unwrap() = Some(species);
            *loading_cln.lock().unwrap() = false;
        });
    }

    fn load_selected_pkmn(&self, species_resource: NamedApiResource<PokemonSpecies>) {
        // Fetch Pokemon species async
        let loading_cln = self.loading.clone();
        let pkmn_selected_cln = self.pkmn_selected.clone();

        *loading_cln.lock().unwrap() = true;
        *pkmn_selected_cln.lock().unwrap() = None;
        tokio::spawn(async move {
            let client = RustemonClient::default();
            let species = species_resource
                .follow(&client)
                .await
                .expect("failed to follow pokemon species from resource");
            let pokemon_resources = rustemon::pokemon::pokemon::get_all_entries(&client)
                .await
                .expect("failed to load pokemon");
            let mut pokemon = vec![];
            for pokemon_resource in pokemon_resources
                .into_iter()
                .filter(|p| p.name.starts_with(&species.name))
            {
                pokemon.push(
                    pokemon_resource
                        .follow(&client)
                        .await
                        .expect("failed to follow pokemon from resource"),
                );
            }

            *pkmn_selected_cln.lock().unwrap() =
                Some(SelectedPokemonSpecies::new(species, pokemon));
            *loading_cln.lock().unwrap() = false;
        });
    }
}

impl Default for MyApp {
    fn default() -> Self {
        let s = Self {
            loading: Arc::new(Mutex::from(false)),
            pkmn_species: Arc::new(Mutex::from(None)),
            search_term: "".to_owned(),
            pkmn_selected: Arc::new(Mutex::new(None)),
        };

        s.load_pkmn_species();

        s
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.columns(2, |cols| {
                let is_loading = *self.loading.lock().unwrap();

                cols[0].heading(APP_NAME);

                cols[0].text_edit_singleline(&mut self.search_term);

                match &*self.pkmn_species.lock().unwrap() {
                    Some(value) => {
                        ScrollArea::vertical()
                            .id_source("pkmn_species")
                            .show(&mut cols[0], |ui| {
                                for result in value.iter() {
                                    if result.name.contains(&self.search_term) {
                                        // TODO: btn.highlight() if result.id == pkmn_selected.species.id
                                        if ui.button(&result.name.capitalize()).clicked() {
                                            self.load_selected_pkmn(result.clone());
                                        }
                                    }
                                }
                            });
                    }
                    None => {
                        if is_loading {
                            cols[0].spinner();
                        }
                    }
                }

                match &*self.pkmn_selected.lock().unwrap() {
                    Some(value) => {
                        // TODO: Break into smaller functions
                        cols[1].heading(value.species.name.capitalize());

                        ScrollArea::vertical().id_source(&value.species.name).show(
                            &mut cols[1],
                            |ui| {
                                for pokemon in value.pokemon.iter() {
                                    ui.strong(&pokemon.name.capitalize());
                                    match &pokemon.sprites.front_default {
                                        Some(sprite) => ui.add(
                                            Image::new(sprite)
                                                .fit_to_original_size(2.0)
                                                .texture_options(TextureOptions::NEAREST),
                                        ),
                                        None => ui.label("missing sprite"),
                                    };

                                    ui.push_id(&pokemon.name, |ui| {
                                        TableBuilder::new(ui)
                                            .vscroll(false)
                                            .striped(true)
                                            .cell_layout(egui::Layout::left_to_right(
                                                egui::Align::Center,
                                            ))
                                            .column(Column::auto())
                                            .column(Column::remainder())
                                            .header(20.0, |mut header| {
                                                header.col(|ui| {
                                                    ui.strong("Stat");
                                                });
                                                header.col(|ui| {
                                                    ui.strong("Value");
                                                });
                                            })
                                            .body(|mut body| {
                                                for stat in &pokemon.stats {
                                                    body.row(20.0, |mut row| {
                                                        row.col(|col| {
                                                            col.label(&stat.stat.name.capitalize());
                                                        });
                                                        row.col(|col| {
                                                            col.label(&stat.base_stat.to_string());
                                                        });
                                                    });
                                                }
                                            });
                                    });
                                }
                            },
                        );
                    }
                    None => {
                        if is_loading {
                            cols[1].spinner();
                        }
                    }
                }
            });
        });
    }
}
