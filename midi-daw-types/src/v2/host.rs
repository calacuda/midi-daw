use std::path::PathBuf;

use bincode::{Decode, Encode};
use crossbeam::channel::{Receiver, Sender, unbounded};
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
use rack::prelude::*;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use tracing::*;

pub type PluginParamMessage = (usize, PluginCmd);

#[cfg_attr(feature = "pyo3", pyclass(from_py_object))]
#[cfg_attr(feature = "pyo3", pyo3(get_all, set_all))]
#[derive(Serialize, Deserialize, Encode, Decode, PartialEq, PartialOrd, Clone)]
pub enum PluginCmd {
    StepPramBy(isize),
    SetParamTo(f32),
}

pub struct PluginInfo {
    info: rack::prelude::PluginInfo,
    param_names: FxHashMap<String, ParameterInfo>,
    tx: Sender<PluginParamMessage>,
}

impl PluginInfo {
    fn new(plugin: &Plugin, tx: Sender<PluginParamMessage>) -> Self {
        let info = plugin.info().clone();
        let mut param_names = FxHashMap::default();

        for param_num in 0..plugin.preset_count().unwrap_or_default() {
            if let Ok(param_info) = plugin.parameter_info(param_num) {
                param_names.insert(param_info.name.clone(), param_info);
            }
        }

        Self {
            info,
            param_names,
            tx,
        }
    }
}

fn do_plugin_from_path(plugin_path: PathBuf) -> Result<Plugin> {
    let scanner = Scanner::new()?;
    let plugins = scanner.scan()?;

    if plugins.is_empty() {
        error!("No plugins found!");
        return Err(Error::Other("no problems found".into()));
    }

    let plugin = plugins.iter().find(|p| p.path == plugin_path);

    if let Some(info) = plugin {
        let plugin = scanner.load(info)?;

        Ok(plugin)
    } else {
        error!("no plugin found at the path: {plugin_path:?}");

        Err(Error::Other("no plugin found".into()))
    }
}

pub fn plugin_from_path(
    plugin_path: PathBuf,
) -> Option<(
    PluginInfo,
    Plugin,
    // Sender<PluginParamMessage>,
    Receiver<PluginParamMessage>,
)> {
    let (tx, rx) = unbounded();

    do_plugin_from_path(plugin_path)
        .ok()
        .map(|plugin| (PluginInfo::new(&plugin, tx), plugin, rx))
}
