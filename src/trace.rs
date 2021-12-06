use super::*;

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ContractDebugInfo {
    pub assembly_code: String,
    pub pc_line_mapping: HashMap<u16, u16>,
    pub active_lines: HashSet<u16>,
}