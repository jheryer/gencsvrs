use crate::output_types::lib::schema::Schema;
use arrow::array::StructArray;
use arrow::record_batch::RecordBatch;

use crate::output_types::lib::column_context::build_arrow_columns;

pub fn create_record_batch(schema: Vec<Schema>, size: usize) -> RecordBatch {
    let struct_array = StructArray::from(build_arrow_columns(schema, size));
    return RecordBatch::from(&struct_array);
}
