use std::sync::Arc;

use arrow_array::{ArrayRef, BooleanArray, Float64Array, Int64Array, RecordBatch, StringArray};
use arrow_schema::{DataType, Field, Schema};
use parquet::arrow::ArrowWriter;
use parquet::basic::Compression;
use parquet::file::properties::WriterProperties;

use crate::error::Result;
use crate::render::{RenderOutput, Renderer, Rows, RunMeta, TemplateBundle};

pub struct ParquetRenderer;

impl Renderer for ParquetRenderer {
    fn render(rows: &Rows, _meta: &RunMeta, _template: &TemplateBundle) -> Result<RenderOutput> {
        // Infer schema from the first row (if any) — fall back to all-string if empty
        let fields: Vec<Field> = rows
            .columns
            .iter()
            .enumerate()
            .map(|(i, col)| {
                let dtype = rows
                    .data
                    .first()
                    .and_then(|r| r.get(i))
                    .map(infer_arrow_type)
                    .unwrap_or(DataType::Utf8);
                Field::new(col.as_str(), dtype, true)
            })
            .collect();

        let schema = Arc::new(Schema::new(fields));

        let props = WriterProperties::builder()
            .set_compression(Compression::SNAPPY)
            .build();

        let mut buf: Vec<u8> = Vec::new();
        let mut writer = ArrowWriter::try_new(&mut buf, schema.clone(), Some(props))?;

        // Build columns
        if !rows.data.is_empty() {
            let arrays: Vec<ArrayRef> = schema
                .fields()
                .iter()
                .enumerate()
                .map(|(col_idx, field)| build_column(rows, col_idx, field.data_type()))
                .collect();

            let batch = RecordBatch::try_new(schema, arrays)
                .map_err(|e| crate::error::ReportError::Renderer(e.to_string()))?;
            writer.write(&batch)?;
        }

        writer.close()?;
        Ok(RenderOutput::new(buf))
    }
}

fn infer_arrow_type(v: &serde_json::Value) -> DataType {
    match v {
        serde_json::Value::Bool(_) => DataType::Boolean,
        serde_json::Value::Number(n) => {
            if n.is_i64() {
                DataType::Int64
            } else {
                DataType::Float64
            }
        }
        _ => DataType::Utf8,
    }
}

fn build_column(rows: &Rows, col_idx: usize, dtype: &DataType) -> ArrayRef {
    match dtype {
        DataType::Boolean => {
            let vals: Vec<Option<bool>> = rows
                .data
                .iter()
                .map(|r| r.get(col_idx).and_then(|v| v.as_bool()))
                .collect();
            Arc::new(BooleanArray::from(vals))
        }
        DataType::Int64 => {
            let vals: Vec<Option<i64>> = rows
                .data
                .iter()
                .map(|r| r.get(col_idx).and_then(|v| v.as_i64()))
                .collect();
            Arc::new(Int64Array::from(vals))
        }
        DataType::Float64 => {
            let vals: Vec<Option<f64>> = rows
                .data
                .iter()
                .map(|r| r.get(col_idx).and_then(|v| v.as_f64()))
                .collect();
            Arc::new(Float64Array::from(vals))
        }
        _ => {
            let vals: Vec<Option<String>> = rows
                .data
                .iter()
                .map(|r| {
                    r.get(col_idx).map(|v| match v {
                        serde_json::Value::String(s) => s.clone(),
                        serde_json::Value::Null => String::new(),
                        other => other.to_string(),
                    })
                })
                .collect();
            Arc::new(StringArray::from(vals))
        }
    }
}
