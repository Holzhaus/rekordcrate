// Copyright (c) 2026 Jan Holthuis <jan.holthuis@rub.de>
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

//! Rendering of Rekordbox waveform data to SVG.

use std::io::Write;

use svg::node::element::path::Data;
use svg::node::element::Path;
use svg::Document;

use crate::anlz::{Content, WaveformPreviewColumn, ANLZ};
use crate::Result;

/// Renders supported ANLZ waveform sections to SVG.
#[derive(Debug, Clone)]
pub struct WaveformRenderer {
    /// Height of the waveform plot in SVG units.
    pub height: u32,
    /// Fill color for the waveform.
    pub color: String,
    /// Background color for the SVG document.
    pub background: String,
}

impl Default for WaveformRenderer {
    fn default() -> Self {
        Self {
            height: 144,
            color: String::from("#2563eb"),
            background: String::from("#05070c"),
        }
    }
}

impl WaveformRenderer {
    /// Creates a renderer with default appearance settings.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Renders the first supported waveform in an ANLZ file to an SVG document.
    pub fn render_anlz(&self, anlz: &ANLZ) -> Result<Document> {
        let columns = anlz
            .sections
            .iter()
            .find_map(|section| match &section.content {
                Content::WaveformPreview(preview) => Some(preview.data.as_slice()),
                _ => None,
            });

        let columns = columns
            .ok_or_else(|| std::io::Error::other("no supported waveform preview section found"))?;
        self.render_columns(columns)
    }

    /// Renders monochrome waveform columns to an SVG document.
    pub fn render_columns(&self, columns: &[WaveformPreviewColumn]) -> Result<Document> {
        if columns.is_empty() {
            return Err(std::io::Error::other("waveform preview contains no columns").into());
        }

        let height = self.height.max(1);
        let width = u32::try_from(columns.len())
            .map_err(|_| std::io::Error::other("waveform preview contains too many columns"))?;
        let center = height as f32 / 2.0;
        let half_height = center;
        let maximum = 31.0_f32;
        let divisor = (columns.len().saturating_sub(1).max(1)) as f32;

        let mut upper = Data::new().move_to((0.0, center));
        for (index, column) in columns.iter().enumerate() {
            let x = width as f32 * index as f32 / divisor;
            let y = center - (f32::from(column.height()) / maximum) * half_height;
            upper = upper.line_to((x, y));
        }
        for (index, column) in columns.iter().enumerate().rev() {
            let x = width as f32 * index as f32 / divisor;
            let y = center + (f32::from(column.height()) / maximum) * half_height;
            upper = upper.line_to((x, y));
        }
        let waveform = Path::new()
            .set("d", upper.close())
            .set("fill", self.color.clone())
            .set("fill-opacity", 0.9);

        let document = Document::new()
            .set("xmlns", "http://www.w3.org/2000/svg")
            .set("width", width)
            .set("height", height)
            .set("viewBox", (0, 0, width, height))
            .set("role", "img")
            .add(
                svg::node::element::Rectangle::new()
                    .set("width", width)
                    .set("height", height)
                    .set("fill", self.background.clone()),
            )
            .add(waveform);

        Ok(document)
    }

    /// Renders an ANLZ file and writes the SVG document to a writer.
    pub fn render_to<W: Write>(&self, anlz: &ANLZ, writer: W) -> Result<()> {
        let document = self.render_anlz(anlz)?;
        svg::write(writer, &document)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_columns_as_svg() {
        let columns = [
            WaveformPreviewColumn::new().with_height(0),
            WaveformPreviewColumn::new().with_height(31),
            WaveformPreviewColumn::new().with_height(0),
        ];
        let document = WaveformRenderer::default()
            .render_columns(&columns)
            .expect("columns should render");
        let svg = document.to_string();
        assert!(svg.contains("viewBox=\"0 0 3 144\""));
        assert!(svg.contains("<path"));
        assert!(svg.contains("#2563eb"));
    }
}
