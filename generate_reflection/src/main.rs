#![recursion_limit="128"]

mod api_dump;
mod reflection_database;
mod roblox_install;
mod run_in_roblox;

use std::{
    collections::HashMap,
    fs::File,
    io::Write,
    path::PathBuf,
    error::Error,
};

use serde_derive::Deserialize;
use rbx_dom_weak::{RbxTree, RbxValue, RbxInstanceProperties};

use crate::{
    run_in_roblox::{inject_plugin_main, run_in_roblox},
    api_dump::Dump,
};

static PLUGIN_MODEL: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/plugin.rbxmx"));

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum PluginMessage {
    Version {
        version: [u32; 4],
    },

    #[serde(rename_all = "camelCase")]
    DefaultProperties {
        class_name: String,
        properties: HashMap<String, RbxValue>,
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let output_dir = {
        let mut output_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        output_dir.pop();
        output_dir.push("rbx_reflection");
        output_dir.push("src");
        output_dir
    };

    println!("Output at {}", output_dir.display());

    let (dump_source, dump) = Dump::read_with_source()?;

    let plugin = {
        let mut plugin = RbxTree::new(RbxInstanceProperties {
            name: String::from("generate_rbx_reflection plugin"),
            class_name: String::from("Folder"),
            properties: Default::default(),
        });

        let root_id = plugin.get_root_id();

        rbx_xml::decode(&mut plugin, root_id, PLUGIN_MODEL)
            .expect("Couldn't deserialize built-in plugin");

        inject_plugin_main(&mut plugin);
        inject_api_dump(&mut plugin, dump_source);

        plugin
    };

    let messages = run_in_roblox(&plugin);

    let mut default_properties = HashMap::new();
    let mut studio_version = [0, 0, 0, 0];

    for message in &messages {
        if let Ok(str) = std::str::from_utf8(message) {
            println!("{}", str);
        }

        let deserialized = serde_json::from_slice(&message)
            .expect("Couldn't deserialize message");

        match deserialized {
            PluginMessage::Version { version } => {
                studio_version = version;
            }
            PluginMessage::DefaultProperties { class_name, properties } => {
                default_properties.insert(class_name, properties);
            }
        }
    }

    let dump_code = reflection_database::generate(&dump, &default_properties);

    let mut dump_file = File::create(output_dir.join("reflection_database.rs"))?;
    writeln!(dump_file, "//! This file is automatically generated by generate_rbx_reflection.")?;
    writeln!(dump_file, "//! To update it, make sure you have Roblox Studio and Rojo installed and run")?;
    writeln!(dump_file, "//! `gen-reflection` in the root.")?;
    writeln!(dump_file, "#![allow(unused_mut)]")?;
    write!(dump_file, "{}", dump_code)?;

    let mut version_file = File::create(output_dir.join("version.rs"))?;
    writeln!(version_file, "pub const VERSION_MAJOR: u32 = {};", studio_version[0])?;
    writeln!(version_file, "pub const VERSION_MINOR: u32 = {};", studio_version[1])?;
    writeln!(version_file, "pub const VERSION_PATCH: u32 = {};", studio_version[2])?;
    writeln!(version_file, "pub const VERSION_BUILD: u32 = {};", studio_version[3])?;

    Ok(())
}

fn inject_api_dump(plugin: &mut RbxTree, source: String) {
    let root_id = plugin.get_root_id();

    let dump_node = RbxInstanceProperties {
        class_name: String::from("StringValue"),
        name: String::from("ApiDump"),
        properties: {
            let mut properties = HashMap::new();

            properties.insert(
                String::from("Value"),
                RbxValue::String { value: source },
            );

            properties
        },
    };

    plugin.insert_instance(dump_node, root_id);
}