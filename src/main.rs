use std::path::PathBuf;

use home::home_dir;
use rocket::State;
use serde::{Deserialize, Serialize};

#[macro_use]
extern crate rocket;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Configuration {
    storage_path: String,
}

#[get("/")]
fn index(state: &State<Configuration>) -> String {
    state.storage_path.clone()
}

#[launch]
fn rocket() -> _ {
    let rocket = rocket::build().mount("/", routes![index]);
    let configuration = match Configuration::read_yaml_configuration() {
        Some(configuration) => {
            println!(
                "Using configuration located at: {:?}",
                Configuration::get_configuration_directory()
            );
            println!("Retrieved configuration:\n{:#?}", configuration);
            configuration
        }
        None => Configuration::handle_missing_configuration(),
    };
    rocket.manage(configuration)
}

impl Configuration {
    fn build_default_configuration() -> Option<Configuration> {
        home_dir().map(|mut path| {
            path.push("realsense_captures");
            Configuration {
                storage_path: path.into_os_string().into_string().unwrap(),
            }
        })
    }

    fn write_yaml(&self) {
        let config_directory = Configuration::get_configuration_directory().into_boxed_path();

        let config_as_yaml = serde_yaml::to_string(self).unwrap();

        if !config_directory.exists() {
            std::fs::create_dir(config_directory.as_ref())
                .expect("Could not create configuration directory");
        }
        let mut configuration_file = Configuration::get_configuration_directory();
        configuration_file.push("config.yaml");
        std::fs::write(configuration_file.into_boxed_path(), config_as_yaml)
            .expect("Unable to serialize file")
    }

    fn read_yaml_configuration() -> Option<Configuration> {
        let mut configuration_file = Configuration::get_configuration_directory();
        configuration_file.push("config.yaml");
        let configuration_file_path = configuration_file.into_boxed_path();
        if configuration_file_path.exists() {
            let configuration_file = std::fs::File::open(configuration_file_path.as_ref()).expect(
                String::as_str(&format!(
                    "Could not read configuration file at: {:?}",
                    configuration_file_path
                )),
            );
            let configuration: Configuration =
                serde_yaml::from_reader(configuration_file).expect(String::as_str(&format!(
                    "Could not read configuration file at: {:?}",
                    configuration_file_path
                )));
            Some(configuration)
        } else {
            None
        }
    }

    fn get_configuration_directory() -> PathBuf {
        match home_dir() {
            Some(mut path) => {
                path.push(".realsense-capture-service");
                path
            }
            None => panic!("Could not retrieve home directory"),
        }
    }

    fn handle_missing_configuration() -> Configuration {
        println!(
            "No configuration located at {:?}, using default configuration",
            Configuration::get_configuration_directory()
        );
        let configuration =
            Configuration::build_default_configuration().expect("Could not create default config");
        println!("Default Configuration:\n{:#?}", configuration);
        println!(
            "Storing configuration at: {:?}",
            Configuration::get_configuration_directory()
        );
        configuration.write_yaml();
        configuration
    }
}
