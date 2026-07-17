//!
//! XLSX worksheet for benchmark data.
//!

use std::collections::HashMap;

///
/// XLSX worksheet for benchmark data.
///
pub struct Worksheet {
    /// The inner worksheet.
    pub worksheet: rust_xlsxwriter::Worksheet,
    /// Test indexes in the worksheet.
    pub rows: HashMap<String, u32>,
    /// Header names and their column widths.
    pub headers: Vec<(&'static str, usize)>,
}

impl Worksheet {
    /// Width of columns that contain values.
    const VALUE_COLUMN_WIDTH: usize = 12;

    /// Width of the PR-vs-`main` comparison columns.
    const COMPARISON_COLUMN_WIDTH: usize = 10;

    ///
    /// Creates a new worksheet with the given name.
    ///
    pub fn new(name: &str, headers: Vec<(&'static str, usize)>) -> anyhow::Result<Self> {
        let mut worksheet = rust_xlsxwriter::Worksheet::new();
        worksheet.set_name(name)?;

        for (header_index, (header_name, column_width)) in headers.iter().enumerate() {
            worksheet.write_with_format(
                0,
                header_index as u16,
                header_name.to_owned(),
                &Self::worksheet_caption_format(),
            )?;
            worksheet.set_column_width(header_index as u16, *column_width as f64)?;
        }

        Ok(Self {
            worksheet,
            rows: HashMap::new(),
            headers,
        })
    }

    ///
    /// Adds a new column for a toolchain.
    ///
    pub fn add_toolchain_column(
        &mut self,
        toolchain_name: &str,
        toolchain_id: u16,
    ) -> anyhow::Result<u16> {
        let column_toolchain_name = toolchain_name.replace('-', "\n");

        self.worksheet.set_column_width(
            (self.headers.len() as u16) + toolchain_id,
            Self::VALUE_COLUMN_WIDTH as f64,
        )?;
        self.worksheet.write_with_format(
            0,
            (self.headers.len() as u16) + toolchain_id,
            column_toolchain_name.clone(),
            &Self::column_header_format(),
        )?;

        Ok(toolchain_id)
    }

    ///
    /// Adds a new row for a test and writes a value.
    ///
    pub fn write_test_value(
        &mut self,
        project: &str,
        contract: Option<&str>,
        function: Option<&str>,
        toolchain_id: u16,
        value: u64,
    ) -> anyhow::Result<()> {
        let row_identifier = format!("{project}/{contract:?}.{function:?}");
        let row_index = if let Some(index) = self.rows.get(row_identifier.as_str()) {
            *index
        } else {
            let row_index = (self.rows.len() as u32) + 1;
            self.rows.insert(row_identifier, row_index);

            self.worksheet.write_with_format(
                row_index,
                0,
                project.to_owned(),
                &Self::row_header_format(),
            )?;
            if let Some(contract) = contract {
                self.worksheet.write_with_format(
                    row_index,
                    1,
                    contract.to_owned(),
                    &Self::row_header_format(),
                )?;
            }
            if let Some(function) = function {
                self.worksheet.write_with_format(
                    row_index,
                    2,
                    function.to_owned(),
                    &Self::row_header_format(),
                )?;
            }

            row_index
        };

        self.worksheet.write_with_format(
            row_index,
            (self.headers.len() as u16) + toolchain_id,
            value,
            &Self::value_format(),
        )?;
        Ok(())
    }

    ///
    /// Writes one measurement cell, allocating the toolchain column on
    /// demand.
    ///
    pub fn record(
        &mut self,
        toolchain_name: &str,
        toolchain_id: u16,
        project: &str,
        contract: Option<&str>,
        function: Option<&str>,
        value: u64,
    ) -> anyhow::Result<()> {
        self.add_toolchain_column(toolchain_name, toolchain_id)?;
        self.write_test_value(project, contract, function, toolchain_id, value)
    }

    ///
    /// Sets totals for each column in the worksheet.
    ///
    pub fn set_totals(&mut self, column_count: usize) -> anyhow::Result<()> {
        if self.rows.is_empty() {
            return Ok(());
        }
        let last_data_row_index = self.rows.len() + 1;

        let sum_criterion = (self.headers.len()..(self.headers.len() + column_count)).map(|column_index| {
            let column_name = rust_xlsxwriter::column_number_to_name(column_index as u16);
            format!(r#"{column_name}2:{column_name}{last_data_row_index},"<>", {column_name}2:{column_name}{last_data_row_index},"<>0""#)
        }).collect::<Vec<String>>().join(", ");
        let median_criterion = (self.headers.len()..(self.headers.len() + column_count)).map(|column_index| {
            let column_name = rust_xlsxwriter::column_number_to_name(column_index as u16);
            format!(r#"(${column_name}$2:${column_name}${last_data_row_index}<>"")*(${column_name}$2:${column_name}${last_data_row_index}<>0)"#)
        }).collect::<Vec<String>>().join("*");

        for (total_row_index, summary_name) in ["Total", "Median"].iter().enumerate() {
            let value_row_index = last_data_row_index + total_row_index;

            for column_index in 0..self.headers.len() {
                let total_caption = if column_index == self.headers.len() - 1 {
                    summary_name
                } else {
                    ""
                };
                self.worksheet.write_with_format(
                    value_row_index as u32,
                    column_index as u16,
                    total_caption,
                    &Self::row_header_summary_format(),
                )?;
            }

            for column_index in 0..column_count {
                let column_name = rust_xlsxwriter::column_number_to_name(
                    (self.headers.len() + column_index) as u16,
                );

                let formula = match total_row_index {
                    0 => {
                        format!(
                            "SUMIFS(
                        {column_name}2:{column_name}{last_data_row_index},
                        {sum_criterion})"
                        )
                    }
                    1 => {
                        format!(
                            "MEDIAN(FILTER(
                    {column_name}2:{column_name}{last_data_row_index},
                    {median_criterion}))"
                        )
                    }
                    _ => {
                        return Err(anyhow::anyhow!(
                            "Unexpected total row index: {total_row_index}",
                        ));
                    }
                };

                self.worksheet.write_formula_with_format(
                    value_row_index as u32,
                    (self.headers.len() + column_index) as u16,
                    formula.as_str(),
                    &Self::value_format(),
                )?;
            }
        }

        Ok(())
    }

    ///
    /// Sets diffs for the first two data columns in the worksheet.
    ///
    pub fn set_diffs(
        &mut self,
        toolchain_id_1: u16,
        toolchain_name_1: &str,
        toolchain_id_2: u16,
        toolchain_name_2: &str,
        total_toolchains: u16,
        diff_index: u16,
    ) -> anyhow::Result<()> {
        if self.rows.is_empty() {
            return Ok(());
        }
        let column_identifier = format!(
            "{}\n------- vs -------\n{}",
            toolchain_name_1.replace('-', "\n"),
            toolchain_name_2.replace('-', "\n")
        );
        let column_index = (self.headers.len() as u16) + total_toolchains + diff_index;
        self.worksheet.write_with_format(
            0,
            column_index,
            column_identifier,
            &Self::column_comparison_header_format(),
        )?;
        self.worksheet
            .set_column_width(column_index, Self::COMPARISON_COLUMN_WIDTH as f64)?;

        for row_id in 0..self.rows.len() + 2 {
            self.worksheet.write_formula_with_format(
                (row_id + 1) as u32,
                column_index,
                format!(
                    r#"=IF(AND({0}{2}<>"", {1}{2}<>"", {0}{2}<>0, {1}{2}<>0), ({0}{2}-{1}{2}) / {1}{2}, "")"#,
                    rust_xlsxwriter::column_number_to_name((self.headers.len() as u16) + toolchain_id_1),
                    rust_xlsxwriter::column_number_to_name((self.headers.len() as u16) + toolchain_id_2),
                    row_id + 2
                )
                .as_str(),
                &Self::percent_format(),
            )?;
        }

        Ok(())
    }

    ///
    /// Finalizes the worksheet and returns its inner object.
    ///
    pub fn into_inner(self) -> rust_xlsxwriter::Worksheet {
        self.worksheet
    }

    ///
    /// Returns the eponymous cell format.
    ///
    fn worksheet_caption_format() -> rust_xlsxwriter::Format {
        rust_xlsxwriter::Format::new()
            .set_bold()
            .set_font_size(16)
            .set_font_color("#FFFFFF")
            .set_background_color("#4C6EF5")
            .set_align(rust_xlsxwriter::FormatAlign::Center)
            .set_align(rust_xlsxwriter::FormatAlign::VerticalCenter)
            .set_border(rust_xlsxwriter::FormatBorder::None)
    }

    ///
    /// Returns the eponymous cell format.
    ///
    fn column_header_format() -> rust_xlsxwriter::Format {
        rust_xlsxwriter::Format::new()
            .set_bold()
            .set_font_size(12)
            .set_font_color("#1E1E1E")
            .set_background_color("#EEF3FF")
            .set_align(rust_xlsxwriter::FormatAlign::Center)
            .set_align(rust_xlsxwriter::FormatAlign::Top)
            .set_border(rust_xlsxwriter::FormatBorder::None)
    }

    ///
    /// Returns the eponymous cell format.
    ///
    fn column_comparison_header_format() -> rust_xlsxwriter::Format {
        rust_xlsxwriter::Format::new()
            .set_bold()
            .set_font_size(11)
            .set_font_color("#1E1E1E")
            .set_background_color("#EEF3FF")
            .set_align(rust_xlsxwriter::FormatAlign::Center)
            .set_align(rust_xlsxwriter::FormatAlign::VerticalCenter)
            .set_border(rust_xlsxwriter::FormatBorder::None)
    }

    ///
    /// Returns the eponymous cell format.
    ///
    fn row_header_format() -> rust_xlsxwriter::Format {
        rust_xlsxwriter::Format::new()
            .set_font_size(12)
            .set_font_color("#1E1E1E")
            .set_background_color("#DDE6FF")
            .set_align(rust_xlsxwriter::FormatAlign::Left)
            .set_border(rust_xlsxwriter::FormatBorder::None)
    }

    ///
    /// Returns the eponymous cell format.
    ///
    fn row_header_summary_format() -> rust_xlsxwriter::Format {
        Self::row_header_format()
            .set_font_size(16)
            .set_bold()
            .set_align(rust_xlsxwriter::FormatAlign::Right)
    }

    ///
    /// Returns the eponymous cell format.
    ///
    fn value_format() -> rust_xlsxwriter::Format {
        rust_xlsxwriter::Format::new()
            .set_font_size(12)
            .set_font_color("#000000")
            .set_background_color("#FFFFFF")
            .set_align(rust_xlsxwriter::FormatAlign::Right)
            .set_border(rust_xlsxwriter::FormatBorder::None)
    }

    ///
    /// Returns the eponymous cell format.
    ///
    fn percent_format() -> rust_xlsxwriter::Format {
        Self::value_format().set_num_format("0.000%")
    }
}
