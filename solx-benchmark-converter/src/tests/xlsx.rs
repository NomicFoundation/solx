//!
//! Tests for the XLSX output format.
//!

use crate::output::xlsx::Xlsx;
use crate::output::xlsx::sheet::Sheet;

#[test]
fn every_sheet_indexes_its_own_worksheet() {
    // `sheet as usize` indexes the worksheets `new` filled in `ALL` order:
    // a variant added mid-enum but appended to `ALL` would silently write
    // every later sheet's data to the wrong tab.
    let mut xlsx = Xlsx::new().expect("workbook creation");
    assert_eq!(xlsx.worksheets.len(), Sheet::ALL.len());
    for (index, sheet) in Sheet::ALL.into_iter().enumerate() {
        assert_eq!(sheet as usize, index, "{sheet:?}");
        let (name, headers) = sheet.spec();
        assert_eq!(xlsx.sheet(sheet).headers, headers, "{name}");
    }
}
