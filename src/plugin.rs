use libloading::{Library, Symbol};
struct Plugin {
    load_order: usize,
    lib: Library,
    ac_state: PluginActivateState,
}
use std::error::Error;
use std::{collections::HashMap, path::PathBuf};
pub struct PluginManager {
    plugin_list: HashMap<String, Plugin>,
    order: Vec<String>,
    base_path: PathBuf,
}
// Asc: 読み込んだ順
// Desc: 読み込んだ逆順
#[derive(Debug, PartialEq)]
pub enum CallOrder {
    Asc,
    Desc,
}
#[derive(Debug, PartialEq, Clone)]
pub enum PluginActivateState {
    Activate,
    Disable,
}
impl PluginManager {
    pub fn new(base_path: &str) -> Self {
        PluginManager {
            plugin_list: HashMap::new(),
            order: Vec::new(),
            base_path: PathBuf::from(base_path),
        }
    }
    pub fn load_plugin<S: Into<String>>(
        &mut self,
        lib_name: S,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut base = self.base_path.clone();
        let lib_name = lib_name.into();
        base.push(&lib_name);
        if let Some(_) = self.plugin_list.get(&lib_name) {
            return Err(Box::new(PluginError::new(
                PluginErrorId::AlreadyLoaded,
                format!(
                    "指定されたファイル名:{}\nフルパス:{}\nその他情報: ロード済みのプラグインのためスキップします",
                    lib_name,
                    base.to_str().unwrap(),
                ),
            )));
        }
        let lib = match unsafe { libloading::Library::new(base.as_os_str()) } {
            Ok(lib) => lib,
            Err(e) => {
                return Err(Box::new(PluginError::new(
                    PluginErrorId::FileNotFound,
                    format!(
                        "指定されたファイル名:{}\nフルパス:{}\nその他情報: {}",
                        lib_name,
                        base.to_str().unwrap(),
                        e.source().unwrap()
                    ),
                )));
            }
        };
        self.order.push(lib_name.clone());
        self.plugin_list.insert(
            lib_name.clone(),
            Plugin {
                load_order: self.order.len() - 1,
                lib: lib,
                ac_state: PluginActivateState::Disable, // ロード直後はすべて無効とする
            },
        );
        Ok(())
    }
    pub fn get_all_plugin_func_with_order<T>(
        &self,
        function_name: &str,
        co: CallOrder,
    ) -> Vec<libloading::Symbol<T>> {
        let mut result = Vec::new();
        if co == CallOrder::Asc {
            for name in &self.order {
                if let Ok(func) = self.get_plugin_function::<T>(&name, function_name) {
                    result.push(func)
                }
            }
        } else {
            let len = self.order.len();
            for i in 0..len {
                let name = &self.order[len - 1 - i];
                if let Ok(func) = self.get_plugin_function::<T>(&name, function_name) {
                    result.push(func)
                }
            }
        }
        result
    }
    pub fn get_plugin_function<T>(
        &self,
        plugin_name: &str,
        function_name: &str,
    ) -> Result<Symbol<T>, PluginError> {
        let _ = if let Some((_, ac_state)) = self.get_plugin_activate_state(plugin_name) {
            if ac_state == PluginActivateState::Disable {
                return Err(PluginError::new(
                    PluginErrorId::PluginDisable,
                    "\"{plugin_name}\" は読み込まれていますが、ユーザにより無効化されています",
                ));
            }
            ac_state
        } else {
            return Err(PluginError::new(
                PluginErrorId::NotReady,
                format!("\"{plugin_name}\" はロードされていません"),
            ));
        };
        let func: Symbol<T> = match self.plugin_list.get(plugin_name) {
            None => {
                return Err(PluginError::new(
                    PluginErrorId::NotReady,
                    format!("\"{plugin_name}\" はロードされていません"),
                ))
            }
            Some(plugin) => unsafe {
                match plugin.lib.get(function_name.as_bytes()) {
                    Err(_e) => {
                        return Err(PluginError::new(
                            PluginErrorId::SymbolNotFound,
                            format!("\"{function_name}\" が見つかりません"),
                        ))
                    }
                    Ok(f) => f,
                }
            },
        };
        Ok(func)
    }
    pub fn get_plugin_activate_state_with_order(
        &self,
        index: usize,
    ) -> Option<(String, PluginActivateState)> {
        if self.order.len() <= index {
            return None;
        }
        self.get_plugin_activate_state(&self.order[index].clone())
    }

    pub fn get_plugin_activate_state(
        &self,
        plugin_name: &str,
    ) -> Option<(String, PluginActivateState)> {
        if let Some(plugin) = self.plugin_list.get(plugin_name) {
            Some((plugin_name.to_owned(), plugin.ac_state.clone()))
        } else {
            None
        }
    }
    pub fn set_plugin_activate_state_with_order(
        &mut self,
        index: usize,
        state: PluginActivateState,
    ) -> Option<PluginActivateState> {
        if self.order.len() <= index {
            return None;
        }
        let name = &self.order[index].clone();
        self.set_plugin_activate_state(&name, state)
    }
    pub fn set_plugin_activate_state(
        &mut self,
        plugin_name: &str,
        state: PluginActivateState,
    ) -> Option<PluginActivateState> {
        if let Some(plugin) = self.plugin_list.get_mut(plugin_name) {
            plugin.ac_state = state.clone();
            Some(state)
        } else {
            None
        }
    }
    pub fn loaded_plugin_counts(&self) -> usize {
        self.order.len()
    }
    pub fn get_plugin_ordered_list(&self) -> &Vec<String> {
        &self.order
    }
    pub fn unload_specify_plugin_with_name(&mut self, name: &str) {
        if let Some(plugin) = self.plugin_list.remove(name) {
            self.order.remove(plugin.load_order);
        }
    }
    pub fn unload_specify_plugin_with_index(&mut self, index: usize) {
        if self.order.len() > index {
            let name = self.order.remove(index);
            self.plugin_list.remove(&name);
        }
    }
    pub fn unload_all_plugin(&mut self) {
        self.plugin_list.clear();
        self.order.clear();
    }
}
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PluginErrorId {
    AlreadyLoaded,
    FileNotFound,
    NotReady,
    SymbolNotFound,
    PluginDisable,
}
#[derive(Debug, Clone)]
pub struct PluginError {
    id: PluginErrorId,
    msg: String,
}
impl PluginError {
    pub fn new<S: Into<String>>(id: PluginErrorId, msg: S) -> Self {
        PluginError {
            id: id,
            msg: msg.into(),
        }
    }
    pub fn plugin_error_id(&self) -> PluginErrorId {
        self.id
    }
}
impl std::error::Error for PluginError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.source()
    }
}

impl std::fmt::Display for PluginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}
use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MasterConfig {
    pub addon_name: String,
    pub plugin_directory: String,
}
impl Default for MasterConfig {
    fn default() -> Self {
        let mut installdir = std::env::current_exe().unwrap();
        installdir.pop();
        installdir.push("multiline_paster_plugins");
        MasterConfig {
            addon_name: "main_logic.dll".to_owned(),
            plugin_directory: installdir.into_os_string().into_string().unwrap(),
        }
    }
}
