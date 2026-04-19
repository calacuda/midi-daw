use std::path::PathBuf;

use rack::prelude::*;
use rustc_hash::FxHashMap;
use tracing::*;

struct PluginInfo {
    info: rack::prelude::PluginInfo,
    param_names: FxHashMap<String, ParameterInfo>,
}

impl PluginInfo {
    fn new(plugin: &Plugin) -> Self {
        let info = plugin.info().clone();
        let mut param_names = FxHashMap::default();

        for param_num in 0..plugin.preset_count().unwrap_or_default() {
            if let Ok(param_info) = plugin.parameter_info(param_num) {
                param_names.insert(param_info.name.clone(), param_info);
            }
        }

        Self { info, param_names }
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

pub fn plugin_from_path(plugin_path: PathBuf) -> Option<(PluginInfo, Plugin)> {
    do_plugin_from_path(plugin_path)
        .ok()
        .map(|plugin| (PluginInfo::new(&plugin), plugin))
}
