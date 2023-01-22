use libloading::{Library, Symbol};
struct Plugin {
    lib: Library,
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
        self.plugin_list
            .insert(lib_name.clone(), Plugin { lib: lib });
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
                if let Ok(func)=self.get_plugin_function::<T>(&name, function_name){
                    result.push(func)
                }
            }
        } else {
            let len = self.order.len();
            for i in 0..len {
                let name = &self.order[len-1-i];
                if let Ok(func)=self.get_plugin_function::<T>(&name, function_name){
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
        let func: Symbol<T> = match self.plugin_list.get(plugin_name) {
            None => {
                return Err(PluginError::new(
                    PluginErrorId::NotReady,
                    format!("プラグイン {plugin_name} はロードされていません"),
                ))
            }
            Some(plugin) => unsafe {
                match plugin.lib.get(function_name.as_bytes()) {
                    Err(e) => {
                        return Err(PluginError::new(
                            PluginErrorId::SymbolNotFound,
                            format!("シンボル名 \"{function_name}\" が見つかりません"),
                        ))
                    }
                    Ok(f) => f,
                }
            },
        };
        Ok(func)
    }
}
#[derive(Debug, PartialEq)]
pub enum PluginErrorId {
    FileNotFound,
    NotReady,
    SymbolNotFound,
}
#[derive(Debug)]
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