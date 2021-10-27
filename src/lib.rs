mod circuit_config;
mod constants;
mod parser;
mod table_data;
pub mod translator;

pub use circuit_config::CircuitConfig;
pub use table_data::TableData;

pub fn parse(data: Vec<&str>) -> Result<Vec<TableData>, String> {
    match parser::core::parse(data) {
        Err(parsing_error) => Err(format!("{}", parsing_error)),
        Ok(td_vec) => Ok(td_vec),
    }
}
