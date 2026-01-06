mod gpu_table;
mod process_table;
mod search;

pub use gpu_table::render_gpu_processes_with_focus;
pub use process_table::{render, render_with_focus};
pub use search::render_search_panel;
