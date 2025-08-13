//! IGES (Initial Graphics Exchange Specification) B-spline surface export
//!
//! This module provides functionality to export OpenSubdiv patches as
//! B-spline surfaces in IGES format. IGES entity 128 represents a
//! rational B-spline surface.

use crate::far::{PatchTable, PatchType};
use std::io::{self, Write};

/// Error type for IGES export
#[derive(Debug)]
pub enum IgesExportError {
    /// Unsupported patch type
    UnsupportedPatchType(PatchType),
    /// Invalid control points
    InvalidControlPoints,
    /// IO error
    Io(io::Error),
}

impl From<io::Error> for IgesExportError {
    fn from(err: io::Error) -> Self {
        IgesExportError::Io(err)
    }
}

impl std::fmt::Display for IgesExportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedPatchType(t) => write!(f, "Unsupported patch type: {t:?}"),
            Self::InvalidControlPoints => write!(f, "Invalid control point configuration"),
            Self::Io(err) => write!(f, "IO error: {err}"),
        }
    }
}

impl std::error::Error for IgesExportError {}

/// Result type for IGES export
pub type Result<T> = std::result::Result<T, IgesExportError>;

/// IGES file writer helper
struct IgesWriter<W: Write> {
    writer: W,
    sequence: i32,
}

impl<W: Write> IgesWriter<W> {
    fn new(writer: W) -> Self {
        Self {
            writer,
            sequence: 1,
        }
    }

    /// Write a line to the Start section (S)
    fn write_start_line(&mut self, line: &str) -> io::Result<()> {
        // IGES lines must be exactly 80 characters (72 data + 8 for section/sequence)
        let padded = format!("{line:<72}");
        writeln!(self.writer, "{}S{:7}", padded, self.sequence)?;
        self.sequence += 1;
        Ok(())
    }

    /// Write a line to the Global section (G)
    fn write_global_line(&mut self, line: &str) -> io::Result<()> {
        let padded = format!("{line:<72}");
        writeln!(self.writer, "{}G{:7}", padded, self.sequence)?;
        self.sequence += 1;
        Ok(())
    }

    /// Write a Directory Entry (D) - takes two lines
    fn write_directory_entry(&mut self, entity: &DirectoryEntry) -> io::Result<i32> {
        let de_number = self.sequence;

        // First line - each field is exactly 8 characters
        writeln!(
            self.writer,
            "{:>8}{:>8}{:>8}{:>8}{:>8}{:>8}{:>8}{:>8}{:>8}{:>8}D{:>7}",
            entity.entity_type,
            entity.parameter_pointer,
            entity.structure,
            entity.line_font_pattern,
            entity.level,
            entity.view,
            entity.transformation_matrix,
            entity.label_display,
            &entity.status_number[0..8], // First 8 chars of status
            de_number,
            self.sequence
        )?;
        self.sequence += 1;

        // Second line
        let label = if entity.entity_label.is_empty() {
            "        ".to_string() // 8 spaces
        } else {
            format!(
                "{:<8}",
                &entity.entity_label[..entity.entity_label.len().min(8)]
            )
        };

        writeln!(
            self.writer,
            "{:>8}{:>8}{:>8}{:>8}{:>8}{:>8}{:>8}{:>8}{:>8}        D{:>7}",
            entity.entity_type,
            entity.line_weight,
            entity.color,
            entity.parameter_line_count,
            entity.form,
            "0",
            "0",
            label,
            "0", // Subscript number
            self.sequence
        )?;
        self.sequence += 1;

        Ok(de_number)
    }

    /// Write a Parameter Data (P) line
    fn write_parameter_line(&mut self, de_pointer: i32, line: &str) -> io::Result<()> {
        // Parameter lines: 64 chars data + space + 8 chars DE pointer + P + 7 chars
        // sequence
        let padded_line = format!("{line:<64}");
        writeln!(
            self.writer,
            "{} {:7}P{:7}",
            padded_line, de_pointer, self.sequence
        )?;
        self.sequence += 1;
        Ok(())
    }

    /// Write the Terminate section (T)
    fn write_terminate(
        &mut self,
        s_count: i32,
        g_count: i32,
        d_count: i32,
        p_count: i32,
    ) -> io::Result<()> {
        writeln!(
            self.writer,
            "S{:>7}G{:>7}D{:>7}P{:>7}{:>40}T{:>7}",
            s_count, g_count, d_count, p_count, "", 1
        )?;
        Ok(())
    }
}

/// IGES Directory Entry
struct DirectoryEntry {
    entity_type: i32,
    parameter_pointer: i32,
    structure: i32,
    line_font_pattern: i32,
    level: i32,
    view: i32,
    transformation_matrix: i32,
    label_display: i32,
    status_number: String,
    line_weight: i32,
    color: i32,
    parameter_line_count: i32,
    form: i32,
    entity_label: String,
}

impl DirectoryEntry {
    /// Create a directory entry for a B-spline surface (entity 128)
    fn bspline_surface(parameter_pointer: i32, parameter_line_count: i32) -> Self {
        Self {
            entity_type: 128,
            parameter_pointer,
            structure: 0,
            line_font_pattern: 0,
            level: 0,
            view: 0,
            transformation_matrix: 0,
            label_display: 0,
            status_number: "00010001".to_string(), // Visible, independent
            line_weight: 0,
            color: 0, // No color assigned
            parameter_line_count,
            form: 0, // Form 0 for B-spline surface
            entity_label: "".to_string(),
        }
    }
}

/// Export OpenSubdiv patches as B-spline surfaces to IGES format
pub fn export_patches_as_iges<W: Write>(
    writer: &mut W,
    patch_table: &PatchTable,
    control_points: &[[f32; 3]],
) -> Result<()> {
    let mut iges = IgesWriter::new(writer);

    // Start section
    iges.write_start_line("OpenSubdiv B-spline Surface Export")?;
    iges.write_start_line("Generated by opensubdiv-petite")?;
    let s_count = iges.sequence - 1;

    // Global section - IGES standard format
    // Each global parameter on its own, separated by commas
    iges.write_global_line(
        "1H,,1H;,14HOpenSubdiv-1.0,18Hb-spline-export.igs,10HOpenSubdiv,5H1.0.0,32,",
    )?;
    iges.write_global_line("38,6,308,15,10HOpenSubdiv,1.0,2,2HMM,1,0.1,15H20250126.120000,")?;
    iges.write_global_line("0.001,1000.0,6HAuthor,10HOpenSubdiv,11,0,15H20250126.120000;")?;
    let g_count = iges.sequence - s_count - 1;

    // Track directory and parameter counts
    let mut d_count = 0;
    let mut p_count = 0;
    let mut parameter_data = Vec::new();
    let mut directory_entries = Vec::new();

    // Process patches
    let mut _patch_global_idx = 0;
    for array_idx in 0..patch_table.patch_array_count() {
        if let Some(desc) = patch_table.patch_array_descriptor(array_idx) {
            if desc.patch_type() != PatchType::Regular {
                continue;
            }

            let num_patches = patch_table.patch_array_patch_count(array_idx);
            if let Some(cv_indices) = patch_table.patch_array_vertices(array_idx) {
                const REGULAR_PATCH_SIZE: usize = 16; // 4x4 control points

                for patch_idx in 0..num_patches {
                    let start = patch_idx * REGULAR_PATCH_SIZE;
                    let patch_cvs = &cv_indices[start..start + REGULAR_PATCH_SIZE];

                    // Build parameter data for this surface
                    let mut param_lines = Vec::new();

                    // Entity 128: Rational B-Spline Surface
                    // Format: 128,K1,K2,M1,M2,PROP1,PROP2,PROP3,PROP4,PROP5,
                    //         S(1),S(2),...,S(K1+K2+2),
                    //         T(1),T(2),...,T(M1+M2+2),
                    //         W(1,1),W(1,2),...,W(K2+1,M2+1),
                    //         X(1,1),X(1,2),...,X(K2+1,M2+1),
                    //         Y(1,1),Y(1,2),...,Y(K2+1,M2+1),
                    //         Z(1,1),Z(1,2),...,Z(K2+1,M2+1),
                    //         U0,U1,V0,V1;

                    let k1 = 3; // upper index of control points in u (0-based, so 4 points)
                    let k2 = 3; // upper index of control points in v
                    let m1 = 3; // degree in u
                    let m2 = 3; // degree in v

                    // PROP1 = 0: polynomial (non-rational), 1: rational
                    // PROP2 = 0: non-periodic in u, 1: periodic in u
                    // PROP3 = 0: non-periodic in v, 1: periodic in v
                    // PROP4 = 0: non-uniform knots in u, 1: uniform knots in u
                    // PROP5 = 0: non-uniform knots in v, 1: uniform knots in v
                    let mut line = format!("128,{k1},{k2},{m1},{m2},0,0,0,1,1,");

                    // U knot vector: -3,-2,-1,0,1,2,3,4
                    let u_knots = vec![-3.0, -2.0, -1.0, 0.0, 1.0, 2.0, 3.0, 4.0];
                    for knot in &u_knots {
                        line.push_str(&format!("{knot},"));
                    }

                    // V knot vector: -3,-2,-1,0,1,2,3,4
                    let v_knots = vec![-3.0, -2.0, -1.0, 0.0, 1.0, 2.0, 3.0, 4.0];
                    for knot in &v_knots {
                        line.push_str(&format!("{knot},"));
                    }

                    // Weights (all 1.0 for non-rational)
                    for _ in 0..16 {
                        line.push_str("1.0,");
                    }

                    // Control points X, Y, Z coordinates
                    // Control points are in row-major order (V varies fastest)
                    // Write X, Y, Z coordinates separately as required by IGES format
                    for coord_idx in 0..3 {
                        for cv in patch_cvs.iter().take(16) {
                            let cv_idx = cv.0 as usize;
                            if cv_idx >= control_points.len() {
                                return Err(IgesExportError::InvalidControlPoints);
                            }
                            let cp = &control_points[cv_idx];
                            line.push_str(&format!("{:.6},", cp[coord_idx]));
                        }
                    }

                    // Parameter range [U0,U1,V0,V1]
                    // Using the actual evaluation range for the surface
                    line.push_str("0.0,1.0,0.0,1.0;");

                    // Split into 64-character chunks for parameter section
                    let mut param_chars = line.chars().collect::<Vec<_>>();
                    while !param_chars.is_empty() {
                        let chunk_size = param_chars.len().min(64);
                        let chunk: String = param_chars.drain(..chunk_size).collect();
                        param_lines.push(chunk);
                    }

                    // Create directory entry
                    // Parameter pointer must point to the correct sequence number in P section
                    // P section starts after S, G, and D sections
                    let param_pointer = s_count + g_count + d_count + p_count + 1;
                    let entry =
                        DirectoryEntry::bspline_surface(param_pointer, param_lines.len() as i32);

                    directory_entries.push(entry);
                    let param_lines_len = param_lines.len();
                    parameter_data.push(param_lines);

                    p_count += param_lines_len as i32;
                    d_count += 2; // Each directory entry takes 2 lines

                    _patch_global_idx += 1;
                }
            }
        }
    }

    // Write Directory section
    let d_start = iges.sequence;
    for entry in &directory_entries {
        iges.write_directory_entry(entry)?;
    }

    // Write Parameter section
    for (i, param_lines) in parameter_data.iter().enumerate() {
        let de_pointer = d_start + i as i32 * 2;
        for line in param_lines {
            iges.write_parameter_line(de_pointer, line)?;
        }
    }

    // Terminate section
    iges.write_terminate(s_count, g_count, d_count, p_count)?;

    Ok(())
}

/// Extension trait for PatchTable to provide IGES export functionality
pub trait PatchTableIgesExt {
    /// Export patches as B-spline surfaces to IGES format
    fn export_iges_surfaces<W: Write>(
        &self,
        writer: &mut W,
        control_points: &[[f32; 3]],
    ) -> Result<()>;

    /// Export patches to IGES file
    fn export_iges_file(&self, path: &str, control_points: &[[f32; 3]]) -> Result<()>;
}

impl PatchTableIgesExt for PatchTable {
    fn export_iges_surfaces<W: Write>(
        &self,
        writer: &mut W,
        control_points: &[[f32; 3]],
    ) -> Result<()> {
        export_patches_as_iges(writer, self, control_points)
    }

    fn export_iges_file(&self, path: &str, control_points: &[[f32; 3]]) -> Result<()> {
        let mut file = std::fs::File::create(path)?;
        self.export_iges_surfaces(&mut file, control_points)
    }
}
