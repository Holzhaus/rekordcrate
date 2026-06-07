use binrw::BinRead;
use rekordcrate::anlz::{Content, ANLZ};
use std::fmt::Write as _;
use std::fs::File;
use std::path::{Path, PathBuf};
use xmlwriter::{Options, XmlWriter};

const WAVEFORM_LABEL_WIDTH: u32 = 72;
const WAVEFORM_PIXELS_PER_SECOND: u32 = 72;
const WAVEFORM_SECTION_GAP: u32 = 16;
const WAVEFORM_MARKER_HEIGHT: u32 = 24;
const WAVEFORM_PADDING: u32 = 12;
const WAVEFORM_BLUE_FILL: &str = "#2563eb";
const WAVEFORM_BLUE_STROKE: &str = "#60a5fa";
const WAVEFORM_LOW_FILL: &str = "#2563eb";
const WAVEFORM_LOW_STROKE: &str = "#60a5fa";
const WAVEFORM_MID_FILL: &str = "#fde047";
const WAVEFORM_MID_STROKE: &str = "#fcd34d";
const WAVEFORM_HIGH_FILL: &str = "#f8fafc";
const WAVEFORM_HIGH_STROKE: &str = "#ffffff";
const WAVEFORM_OVERLAP_FILL: &str = "#d97706";

#[derive(Clone, Debug, PartialEq, Eq)]
struct WaveformSection {
    label: &'static str,
    render_style: WaveformRenderStyle,
    vertical_alignment: WaveformVerticalAlignment,
    layers: Vec<WaveformLayer>,
    color_columns: Vec<WaveformRenderColumn>,
    color_column_class_name: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WaveformRenderStyle {
    SharedAxis,
    SharedAxisBlend,
    Stacked,
    ColorColumns,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WaveformVerticalAlignment {
    Bottom,
    Center,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct WaveformLayer {
    class_name: &'static str,
    values: Vec<u8>,
    max_value: u8,
    fill: &'static str,
    stroke: &'static str,
    fill_opacity_percent: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct WaveformRenderColumn {
    height: u16,
    max_value: u16,
    red: u8,
    green: u8,
    blue: u8,
    color_max_value: u16,
    whiteness: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct BeatMarker {
    beat_number: u16,
    time_ms: u32,
}

impl WaveformSection {
    fn new(
        label: &'static str,
        render_style: WaveformRenderStyle,
        vertical_alignment: WaveformVerticalAlignment,
        layers: Vec<WaveformLayer>,
    ) -> Self {
        Self {
            label,
            render_style,
            vertical_alignment,
            layers,
            color_columns: Vec::new(),
            color_column_class_name: "",
        }
    }

    fn with_color_columns(
        label: &'static str,
        vertical_alignment: WaveformVerticalAlignment,
        color_column_class_name: &'static str,
        color_columns: Vec<WaveformRenderColumn>,
    ) -> Self {
        Self {
            label,
            render_style: WaveformRenderStyle::ColorColumns,
            vertical_alignment,
            layers: Vec::new(),
            color_columns,
            color_column_class_name,
        }
    }
}

impl WaveformLayer {
    fn blue(values: Vec<u8>, max_value: u8) -> Self {
        Self {
            class_name: "waveform-layer waveform-layer-blue",
            values,
            max_value,
            fill: WAVEFORM_BLUE_FILL,
            stroke: WAVEFORM_BLUE_STROKE,
            fill_opacity_percent: 45,
        }
    }

    fn low(values: Vec<u8>, max_value: u8) -> Self {
        Self {
            class_name: "waveform-layer waveform-layer-low",
            values,
            max_value,
            fill: WAVEFORM_LOW_FILL,
            stroke: WAVEFORM_LOW_STROKE,
            fill_opacity_percent: 100,
        }
    }

    fn mid(values: Vec<u8>, max_value: u8) -> Self {
        Self {
            class_name: "waveform-layer waveform-layer-mid",
            values,
            max_value,
            fill: WAVEFORM_MID_FILL,
            stroke: WAVEFORM_MID_STROKE,
            fill_opacity_percent: 64,
        }
    }

    fn high(values: Vec<u8>, max_value: u8) -> Self {
        Self {
            class_name: "waveform-layer waveform-layer-high",
            values,
            max_value,
            fill: WAVEFORM_HIGH_FILL,
            stroke: WAVEFORM_HIGH_STROKE,
            fill_opacity_percent: 78,
        }
    }
}

impl WaveformRenderColumn {
    fn blue(height: u8, max_value: u8, whiteness: u8) -> Self {
        Self {
            height: u16::from(height),
            max_value: u16::from(max_value.max(1)),
            red: 0x25,
            green: 0x63,
            blue: 0xeb,
            color_max_value: u16::from(u8::MAX),
            whiteness: whiteness.min(7),
        }
    }

    fn rgb_preview(
        entry: &rekordcrate::anlz::WaveformRGBPreviewColumn,
        max_height_sum: u16,
    ) -> Self {
        let red = u16::from(entry.red_bass);
        let green = u16::from(entry.green_mids);
        let blue = u16::from(entry.blue_highs);
        let height = red + green + blue;
        let scale_channel = |value: u16| -> u8 {
            if height == 0 {
                return 0;
            }
            (((u32::from(value) * u32::from(u8::MAX)) + (u32::from(height) / 2))
                / u32::from(height))
            .min(u32::from(u8::MAX)) as u8
        };
        Self {
            height,
            max_value: max_height_sum.max(1),
            red: scale_channel(red),
            green: scale_channel(green),
            blue: scale_channel(blue),
            color_max_value: u16::from(u8::MAX),
            whiteness: 0,
        }
    }

    fn rgb_detail(height: u8, red: u8, green: u8, blue: u8) -> Self {
        Self {
            height: u16::from(height),
            max_value: 0x1f,
            red,
            green,
            blue,
            color_max_value: 0x07,
            whiteness: 0,
        }
    }
}

fn read_anlz(path: &Path) -> rekordcrate::Result<ANLZ> {
    let mut reader = File::open(path)?;
    Ok(ANLZ::read(&mut reader)?)
}

fn detect_sibling_path(path: &Path, extension: &str) -> Option<PathBuf> {
    let uppercase = path.with_extension(extension);
    if uppercase.exists() {
        return Some(uppercase);
    }

    let lowercase = path.with_extension(extension.to_ascii_lowercase());
    lowercase.exists().then_some(lowercase)
}

fn collect_related_anlz_paths(path: &Path, no_ext: bool, no_2ex: bool) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let mut push_unique = |candidate: Option<PathBuf>| {
        if let Some(candidate) = candidate {
            if !paths.contains(&candidate) {
                paths.push(candidate);
            }
        }
    };

    push_unique(detect_sibling_path(path, "DAT"));
    if !no_ext {
        push_unique(detect_sibling_path(path, "EXT"));
    }
    if !no_2ex {
        push_unique(detect_sibling_path(path, "2EX"));
    }
    if paths.is_empty() {
        paths.push(path.to_path_buf());
    }
    paths
}

fn scale_waveform_band(value: u8, gain: u16) -> u8 {
    (((u32::from(value) * u32::from(gain)) + 50) / 100).min(u32::from(u8::MAX)) as u8
}

fn scale_color_channel(value: u16, max_value: u16) -> u8 {
    if max_value == 0 {
        return 0;
    }
    (((u32::from(value) * 255) + (u32::from(max_value) / 2)) / u32::from(max_value))
        .min(u32::from(u8::MAX)) as u8
}

fn waveform_render_column_fill(column: &WaveformRenderColumn) -> String {
    let whiteness = u16::from(column.whiteness);
    let brighten = |value: u8| -> u8 {
        let value = u16::from(scale_color_channel(
            u16::from(value),
            column.color_max_value,
        ));
        (value + ((255 - value) * whiteness + 3) / 7) as u8
    };
    format!(
        "#{:02x}{:02x}{:02x}",
        brighten(column.red),
        brighten(column.green),
        brighten(column.blue)
    )
}

fn write_svg_path(
    svg: &mut XmlWriter,
    class_name: &str,
    path_data: &str,
    fill: &str,
    fill_opacity: f32,
    stroke: &str,
) {
    svg.start_element("path");
    svg.write_attribute("class", class_name);
    svg.write_attribute("d", path_data);
    svg.write_attribute("fill", fill);
    svg.write_attribute("fill-opacity", &format!("{fill_opacity:.2}"));
    svg.write_attribute("stroke", stroke);
    svg.write_attribute("stroke-width", "0");
    svg.end_element();
}

fn write_svg_rect(
    svg: &mut XmlWriter,
    class_name: Option<&str>,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    fill: &str,
    stroke: &str,
) {
    svg.start_element("rect");
    if let Some(class_name) = class_name {
        svg.write_attribute("class", class_name);
    }
    svg.write_attribute("x", &format!("{x:.2}"));
    svg.write_attribute("y", &format!("{y:.2}"));
    svg.write_attribute("width", &format!("{width:.2}"));
    svg.write_attribute("height", &format!("{height:.2}"));
    svg.write_attribute("fill", fill);
    svg.write_attribute("stroke", stroke);
    svg.end_element();
}

fn write_svg_line(
    svg: &mut XmlWriter,
    class_name: Option<&str>,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    stroke: &str,
    stroke_width: &str,
) {
    svg.start_element("line");
    if let Some(class_name) = class_name {
        svg.write_attribute("class", class_name);
    }
    svg.write_attribute("x1", &format!("{x1:.2}"));
    svg.write_attribute("y1", &format!("{y1:.2}"));
    svg.write_attribute("x2", &format!("{x2:.2}"));
    svg.write_attribute("y2", &format!("{y2:.2}"));
    svg.write_attribute("stroke", stroke);
    svg.write_attribute("stroke-width", stroke_width);
    svg.end_element();
}

fn write_svg_text(
    svg: &mut XmlWriter,
    class_name: Option<&str>,
    x: f32,
    y: f32,
    fill: &str,
    font_size: &str,
    extra_attrs: &[(&str, &str)],
    text: &str,
) {
    svg.start_element("text");
    if let Some(class_name) = class_name {
        svg.write_attribute("class", class_name);
    }
    svg.write_attribute("x", &format!("{x:.2}"));
    svg.write_attribute("y", &format!("{y:.2}"));
    svg.write_attribute("fill", fill);
    svg.write_attribute("font-family", "monospace");
    svg.write_attribute("font-size", font_size);
    for &(name, value) in extra_attrs {
        svg.write_attribute(name, value);
    }
    svg.write_text(text);
    svg.end_element();
}

fn render_section_scaffold(
    svg: &mut XmlWriter,
    section: &WaveformSection,
    plot_left: u32,
    plot_width: u32,
    top: u32,
    section_height: u32,
    beat_markers: &[BeatMarker],
    axis_markers: &[BeatMarker],
) {
    write_svg_rect(
        svg,
        None,
        plot_left as f32,
        top as f32,
        plot_width as f32,
        section_height as f32,
        "#0a0f18",
        "#1f2937",
    );
    write_svg_text(
        svg,
        None,
        WAVEFORM_PADDING as f32,
        (top + section_height / 2) as f32,
        "#d1d5db",
        "14",
        &[("dominant-baseline", "middle")],
        section.label,
    );

    for beat in beat_markers {
        let x = plot_x_for_time(plot_left, beat.time_ms);
        write_svg_line(
            svg,
            Some(if beat.beat_number == 1 {
                "beat-grid beat-grid-bar"
            } else {
                "beat-grid beat-grid-beat"
            }),
            x as f32,
            top as f32,
            x as f32,
            (top + section_height) as f32,
            if beat.beat_number == 1 {
                "#4b5563"
            } else {
                "#1f2937"
            },
            if beat.beat_number == 1 { "1.2" } else { "1.0" },
        );
    }

    for marker in axis_markers {
        let x = plot_x_for_time(plot_left, marker.time_ms);
        write_svg_line(
            svg,
            Some("axis-marker"),
            x as f32,
            top as f32,
            x as f32,
            (top + section_height) as f32,
            "#94a3b8",
            "1",
        );
        write_svg_line(
            svg,
            Some("axis-marker-tick"),
            x as f32,
            (top + section_height) as f32,
            x as f32,
            (top + section_height + 6) as f32,
            "#cbd5e1",
            "1",
        );
        write_svg_text(
            svg,
            Some("axis-label"),
            x as f32,
            (top + section_height + 18) as f32,
            "#e5e7eb",
            "11",
            &[("text-anchor", "middle")],
            &format_axis_timestamp(marker.time_ms),
        );
    }
}

fn calibrate_layers(
    low: Vec<u8>,
    mid: Vec<u8>,
    high: Vec<u8>,
    calibration: Option<rekordcrate::anlz::Waveform3BandCalibration>,
) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    let Some(calibration) = calibration else {
        return (low, mid, high);
    };

    (
        low.into_iter()
            .map(|value| scale_waveform_band(value, calibration.low_gain))
            .collect(),
        mid.into_iter()
            .map(|value| scale_waveform_band(value, calibration.mid_gain))
            .collect(),
        high.into_iter()
            .map(|value| scale_waveform_band(value, calibration.high_gain))
            .collect(),
    )
}

fn three_band_layers(
    low: Vec<u8>,
    mid: Vec<u8>,
    high: Vec<u8>,
    max_value: u8,
) -> Vec<WaveformLayer> {
    vec![
        WaveformLayer::low(low, max_value),
        WaveformLayer::mid(mid, max_value),
        WaveformLayer::high(high, max_value),
    ]
}

fn three_band_layers_with_calibration(
    low: Vec<u8>,
    mid: Vec<u8>,
    high: Vec<u8>,
    calibration: Option<rekordcrate::anlz::Waveform3BandCalibration>,
) -> Vec<WaveformLayer> {
    let (low, mid, high) = calibrate_layers(low, mid, high, calibration);
    three_band_layers(low, mid, high, u8::MAX)
}

fn waveform_rank(label: &str) -> usize {
    match label {
        "PWAV" => 0,
        "PWV2" => 1,
        "PWV3" => 2,
        "PWV4" => 3,
        "PWV5" => 4,
        "PWV6" => 5,
        "PWV7" => 6,
        _ => usize::MAX,
    }
}

fn collect_waveform_calibration(
    anlzs: &[ANLZ],
) -> Option<rekordcrate::anlz::Waveform3BandCalibration> {
    anlzs
        .iter()
        .flat_map(|anlz| anlz.sections.iter())
        .find_map(|section| match &section.content {
            Content::Waveform3BandCalibration(calibration) => Some(*calibration),
            _ => None,
        })
}

fn collect_waveform_sections(anlzs: &[ANLZ]) -> Vec<WaveformSection> {
    let mut sections = Vec::new();
    let calibration = collect_waveform_calibration(anlzs);
    for anlz in anlzs {
        for section in &anlz.sections {
            match &section.content {
                Content::WaveformBluePreview(preview) => {
                    sections.push(WaveformSection::with_color_columns(
                        "PWAV",
                        WaveformVerticalAlignment::Bottom,
                        "waveform-column waveform-column-color-preview",
                        preview
                            .data
                            .iter()
                            .map(|entry| {
                                WaveformRenderColumn::blue(entry.height(), 0x1f, entry.whiteness())
                            })
                            .collect(),
                    ))
                }
                Content::WaveformBlueTinyPreview(preview) => sections.push(WaveformSection::new(
                    "PWV2",
                    WaveformRenderStyle::SharedAxis,
                    WaveformVerticalAlignment::Bottom,
                    vec![WaveformLayer::blue(
                        preview.data.iter().map(|entry| entry.height()).collect(),
                        0x0f,
                    )],
                )),
                Content::WaveformBlueDetail(detail) => {
                    sections.push(WaveformSection::with_color_columns(
                        "PWV3",
                        WaveformVerticalAlignment::Center,
                        "waveform-column waveform-column-color-detail",
                        detail
                            .data
                            .iter()
                            .map(|entry| {
                                WaveformRenderColumn::blue(entry.height(), 0x1f, entry.whiteness())
                            })
                            .collect(),
                    ))
                }
                Content::WaveformRGBPreview(preview) => {
                    let max_height_sum = preview
                        .data
                        .iter()
                        .map(|entry| {
                            u16::from(entry.red_bass)
                                + u16::from(entry.green_mids)
                                + u16::from(entry.blue_highs)
                        })
                        .max()
                        .unwrap_or(1);
                    sections.push(WaveformSection::with_color_columns(
                        "PWV4",
                        WaveformVerticalAlignment::Bottom,
                        "waveform-column waveform-column-color-preview",
                        preview
                            .data
                            .iter()
                            .map(|entry| WaveformRenderColumn::rgb_preview(entry, max_height_sum))
                            .collect(),
                    ))
                }
                Content::WaveformRGBDetail(detail) => {
                    sections.push(WaveformSection::with_color_columns(
                        "PWV5",
                        WaveformVerticalAlignment::Center,
                        "waveform-column waveform-column-color-detail",
                        detail
                            .data
                            .iter()
                            .map(|entry| {
                                WaveformRenderColumn::rgb_detail(
                                    entry.height(),
                                    entry.red_bass(),
                                    entry.green_mids(),
                                    entry.blue_highs(),
                                )
                            })
                            .collect(),
                    ))
                }
                Content::Waveform3BandPreview(preview) => sections.push(WaveformSection::new(
                    "PWV6",
                    WaveformRenderStyle::Stacked,
                    WaveformVerticalAlignment::Bottom,
                    three_band_layers(
                        preview.data.iter().map(|entry| entry.low).collect(),
                        preview.data.iter().map(|entry| entry.mid).collect(),
                        preview.data.iter().map(|entry| entry.high).collect(),
                        u8::MAX,
                    ),
                )),
                Content::Waveform3BandDetail(detail) => sections.push(WaveformSection::new(
                    "PWV7",
                    WaveformRenderStyle::SharedAxisBlend,
                    WaveformVerticalAlignment::Center,
                    three_band_layers_with_calibration(
                        detail.data.iter().map(|entry| entry.low).collect(),
                        detail.data.iter().map(|entry| entry.mid).collect(),
                        detail.data.iter().map(|entry| entry.high).collect(),
                        calibration,
                    ),
                )),
                _ => {}
            }
        }
    }
    sections.sort_by_key(|section| waveform_rank(section.label));
    sections
}

fn collect_beat_markers(anlzs: &[ANLZ]) -> Vec<BeatMarker> {
    anlzs
        .iter()
        .flat_map(|anlz| anlz.sections.iter())
        .filter_map(|section| match &section.content {
            Content::BeatGrid(beat_grid) => Some(&beat_grid.beats),
            _ => None,
        })
        .max_by_key(|beats| beats.len())
        .map(|beats| {
            beats
                .iter()
                .map(|beat| BeatMarker {
                    beat_number: beat.beat_number,
                    time_ms: beat.time,
                })
                .collect()
        })
        .unwrap_or_default()
}

fn estimate_track_duration_ms(anlzs: &[ANLZ]) -> Option<u32> {
    let beat_markers = collect_beat_markers(anlzs);
    let detail_duration = anlzs
        .iter()
        .flat_map(|anlz| anlz.sections.iter())
        .filter_map(|section| match &section.content {
            Content::WaveformBlueDetail(detail) => Some(detail.data.len()),
            Content::WaveformRGBDetail(detail) => Some(detail.data.len()),
            Content::Waveform3BandDetail(detail) => Some(detail.data.len()),
            _ => None,
        })
        .max()
        .map(|len| ((len as u64) * 1000 / 150) as u32);

    let beat_grid_duration = estimate_beat_grid_duration_ms(&beat_markers);

    detail_duration.or(beat_grid_duration)
}

fn estimate_beat_grid_duration_ms(beat_markers: &[BeatMarker]) -> Option<u32> {
    let last = beat_markers.last()?;
    let interval = if beat_markers.len() >= 2 {
        last.time_ms
            .saturating_sub(beat_markers[beat_markers.len() - 2].time_ms)
    } else {
        500
    };
    Some(last.time_ms.saturating_add(interval))
}

fn beat_axis_markers(beat_markers: &[BeatMarker]) -> Vec<BeatMarker> {
    beat_markers.iter().copied().step_by(64).collect()
}

fn format_axis_timestamp(ms: u32) -> String {
    let rounded_seconds = (ms + 500) / 1000;
    let seconds = rounded_seconds % 60;
    let minutes = (rounded_seconds / 60) % 60;
    let hours = rounded_seconds / 3600;

    if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes:02}:{seconds:02}")
    }
}

fn plot_width_for_duration(duration_ms: u32) -> u32 {
    (((u64::from(duration_ms.max(1)) * u64::from(WAVEFORM_PIXELS_PER_SECOND)) + 999) / 1000) as u32
        + 1
}

fn plot_x_for_time(plot_left: u32, time_ms: u32) -> u32 {
    plot_left
        + ((u64::from(time_ms) * u64::from(WAVEFORM_PIXELS_PER_SECOND)) / 1000)
            .try_into()
            .unwrap_or(u32::MAX)
}

fn downsample_waveform(values: &[u8], target_len: usize) -> Vec<u8> {
    if values.is_empty() || target_len == 0 || values.len() <= target_len {
        return values.to_vec();
    }

    (0..target_len)
        .map(|bucket| {
            let start = bucket * values.len() / target_len;
            let end = ((bucket + 1) * values.len() / target_len).max(start + 1);
            *values[start..end].iter().max().unwrap_or(&0)
        })
        .collect()
}

fn downsample_color_columns(
    columns: &[WaveformRenderColumn],
    target_len: usize,
) -> Vec<WaveformRenderColumn> {
    if columns.is_empty() || target_len == 0 || columns.len() <= target_len {
        return columns.to_vec();
    }

    (0..target_len)
        .map(|bucket| {
            let start = bucket * columns.len() / target_len;
            let end = ((bucket + 1) * columns.len() / target_len).max(start + 1);
            let slice = &columns[start..end];
            let max_height = slice.iter().map(|column| column.height).max().unwrap_or(0);
            let weight_sum = slice
                .iter()
                .map(|column| u32::from(column.height.max(1)))
                .sum::<u32>()
                .max(1);
            let avg_channel = |channel: fn(&WaveformRenderColumn) -> u8| -> u8 {
                ((slice
                    .iter()
                    .map(|column| u32::from(channel(column)) * u32::from(column.height.max(1)))
                    .sum::<u32>()
                    + (weight_sum / 2))
                    / weight_sum) as u8
            };
            WaveformRenderColumn {
                height: max_height,
                max_value: slice[0].max_value,
                red: avg_channel(|column| column.red),
                green: avg_channel(|column| column.green),
                blue: avg_channel(|column| column.blue),
                color_max_value: slice[0].color_max_value,
                whiteness: avg_channel(|column| column.whiteness),
            }
        })
        .collect()
}

fn waveform_path_data(
    values: &[u8],
    max_value: u8,
    left: f32,
    top: f32,
    width: f32,
    height: f32,
    vertical_alignment: WaveformVerticalAlignment,
) -> String {
    if values.is_empty() {
        return String::new();
    }

    let divisor = values.len().saturating_sub(1).max(1) as f32;
    match vertical_alignment {
        WaveformVerticalAlignment::Bottom => {
            let bottom = top + height;
            let mut path = format!("M {left:.2} {bottom:.2}");
            for (index, value) in values.iter().enumerate() {
                let x = left + (width * index as f32 / divisor);
                let normalized_height = *value as f32 / f32::from(max_value.max(1));
                let y = bottom - normalized_height * height;
                let _ = write!(path, " L {x:.2} {y:.2}");
            }
            let _ = write!(path, " L {:.2} {:.2} Z", left + width, bottom);
            path
        }
        WaveformVerticalAlignment::Center => {
            let center = top + (height / 2.0);
            let half_height = height / 2.0;
            let mut path = format!("M {left:.2} {center:.2}");
            for (index, value) in values.iter().enumerate() {
                let x = left + (width * index as f32 / divisor);
                let normalized_height = *value as f32 / f32::from(max_value.max(1));
                let y = center - normalized_height * half_height;
                let _ = write!(path, " L {x:.2} {y:.2}");
            }
            for (index, value) in values.iter().enumerate().rev() {
                let x = left + (width * index as f32 / divisor);
                let normalized_height = *value as f32 / f32::from(max_value.max(1));
                let y = center + normalized_height * half_height;
                let _ = write!(path, " L {x:.2} {y:.2}");
            }
            path.push_str(" Z");
            path
        }
    }
}

fn waveform_band_path_data(
    lower_values: &[u16],
    upper_values: &[u16],
    max_value: u16,
    left: f32,
    top: f32,
    width: f32,
    height: f32,
    vertical_alignment: WaveformVerticalAlignment,
) -> String {
    if lower_values.is_empty() || upper_values.is_empty() {
        return String::new();
    }

    let divisor = upper_values.len().saturating_sub(1).max(1) as f32;
    let scale = f32::from(max_value.max(1));
    match vertical_alignment {
        WaveformVerticalAlignment::Bottom => {
            let bottom = top + height;
            let scale_y = |value: u16| bottom - (f32::from(value) / scale) * height;
            let mut path = format!("M {left:.2} {:.2}", scale_y(lower_values[0]));
            for (index, value) in upper_values.iter().enumerate() {
                let x = left + (width * index as f32 / divisor);
                let y = scale_y(*value);
                let _ = write!(path, " L {x:.2} {y:.2}");
            }
            for (index, value) in lower_values.iter().enumerate().rev() {
                let x = left + (width * index as f32 / divisor);
                let y = scale_y(*value);
                let _ = write!(path, " L {x:.2} {y:.2}");
            }
            path.push_str(" Z");
            path
        }
        WaveformVerticalAlignment::Center => {
            let center = top + (height / 2.0);
            let half_height = height / 2.0;
            let top_y = |value: u16| center - (f32::from(value) / scale) * half_height;
            let bottom_y = |value: u16| center + (f32::from(value) / scale) * half_height;
            let mut path = format!("M {left:.2} {:.2}", top_y(lower_values[0]));
            for (index, value) in upper_values.iter().enumerate() {
                let x = left + (width * index as f32 / divisor);
                let y = top_y(*value);
                let _ = write!(path, " L {x:.2} {y:.2}");
            }
            for (index, value) in upper_values.iter().enumerate().rev() {
                let x = left + (width * index as f32 / divisor);
                let y = bottom_y(*value);
                let _ = write!(path, " L {x:.2} {y:.2}");
            }
            for (index, value) in lower_values.iter().enumerate() {
                let x = left + (width * index as f32 / divisor);
                let y = bottom_y(*value);
                let _ = write!(path, " L {x:.2} {y:.2}");
            }
            for (index, value) in lower_values.iter().enumerate().rev() {
                let x = left + (width * index as f32 / divisor);
                let y = top_y(*value);
                let _ = write!(path, " L {x:.2} {y:.2}");
            }
            path.push_str(" Z");
            path
        }
    }
}

fn render_waveform_svg_from_paths(
    paths: &[PathBuf],
    section_height: u32,
) -> rekordcrate::Result<String> {
    let anlzs = paths
        .iter()
        .map(|path| read_anlz(path))
        .collect::<rekordcrate::Result<Vec<_>>>()?;
    let sections = collect_waveform_sections(&anlzs);
    if sections.is_empty() {
        return Err(std::io::Error::other("no supported waveform sections found").into());
    }

    let duration_ms = estimate_track_duration_ms(&anlzs).ok_or_else(|| {
        std::io::Error::other("could not determine track duration for axis labels")
    })?;
    let beat_markers = collect_beat_markers(&anlzs);
    let section_height = section_height.max(1);
    let axis_markers = if beat_markers.is_empty() {
        vec![BeatMarker {
            beat_number: 1,
            time_ms: 0,
        }]
    } else {
        beat_axis_markers(&beat_markers)
    };

    let plot_left = WAVEFORM_PADDING + WAVEFORM_LABEL_WIDTH;
    let plot_width = plot_width_for_duration(duration_ms);
    let total_width = plot_left + plot_width + WAVEFORM_PADDING;
    let row_height = section_height + WAVEFORM_MARKER_HEIGHT;
    let total_height = WAVEFORM_PADDING * 2
        + row_height * u32::try_from(sections.len()).unwrap_or(0)
        + WAVEFORM_SECTION_GAP * u32::try_from(sections.len().saturating_sub(1)).unwrap_or(0);

    let mut svg = XmlWriter::new(Options::default());
    svg.start_element("svg");
    svg.write_attribute("xmlns", "http://www.w3.org/2000/svg");
    svg.write_attribute("width", &total_width.to_string());
    svg.write_attribute("height", &total_height.to_string());
    svg.write_attribute("viewBox", &format!("0 0 {total_width} {total_height}"));
    svg.write_attribute("role", "img");
    write_svg_rect(
        &mut svg,
        None,
        0.0,
        0.0,
        total_width as f32,
        total_height as f32,
        "#05070c",
        "none",
    );

    for (index, section) in sections.iter().enumerate() {
        let top = WAVEFORM_PADDING
            + u32::try_from(index).unwrap_or(0) * (row_height + WAVEFORM_SECTION_GAP);
        let plot_left_f = plot_left as f32;
        let plot_width_f = plot_width as f32;
        let top_f = top as f32;
        let section_height_f = section_height as f32;

        render_section_scaffold(
            &mut svg,
            section,
            plot_left,
            plot_width,
            top,
            section_height,
            &beat_markers,
            &axis_markers,
        );

        let resampled_layers = section
            .layers
            .iter()
            .map(|layer| {
                (
                    layer,
                    downsample_waveform(&layer.values, plot_width as usize),
                )
            })
            .collect::<Vec<_>>();
        match section.render_style {
            WaveformRenderStyle::ColorColumns => {
                let resampled_columns =
                    downsample_color_columns(&section.color_columns, plot_width as usize);
                let divisor = resampled_columns.len().max(1) as f32;
                let center_y = top_f + section_height_f / 2.0;
                for (index, column) in resampled_columns.iter().enumerate() {
                    if column.height == 0 {
                        continue;
                    }
                    let x = plot_left_f + (plot_width_f * index as f32 / divisor);
                    let next_x = plot_left_f + (plot_width_f * (index + 1) as f32 / divisor);
                    let width = (next_x - x).max(1.0);
                    let normalized_height = column.height as f32 / column.max_value.max(1) as f32;
                    let bar_height = normalized_height * section_height_f;
                    let y = match section.vertical_alignment {
                        WaveformVerticalAlignment::Bottom => top_f + section_height_f - bar_height,
                        WaveformVerticalAlignment::Center => center_y - (bar_height / 2.0),
                    };
                    let fill = waveform_render_column_fill(column);
                    write_svg_rect(
                        &mut svg,
                        Some(section.color_column_class_name),
                        x,
                        y,
                        width,
                        bar_height,
                        &fill,
                        "none",
                    );
                }
            }
            WaveformRenderStyle::SharedAxis => {
                for (layer, resampled) in &resampled_layers {
                    let path_data = waveform_path_data(
                        resampled,
                        layer.max_value,
                        plot_left_f,
                        top_f,
                        plot_width_f,
                        section_height_f,
                        section.vertical_alignment,
                    );
                    write_svg_path(
                        &mut svg,
                        layer.class_name,
                        &path_data,
                        layer.fill,
                        f32::from(layer.fill_opacity_percent) / 100.0,
                        layer.stroke,
                    );
                }
            }
            WaveformRenderStyle::SharedAxisBlend => {
                if let [low, mid, high] = &resampled_layers[..] {
                    let max_value =
                        u16::from(low.0.max_value.max(mid.0.max_value).max(high.0.max_value));
                    let zeros = vec![0u16; low.1.len()];
                    let overlap = low
                        .1
                        .iter()
                        .zip(&mid.1)
                        .map(|(low_value, mid_value)| u16::from((*low_value).min(*mid_value)))
                        .collect::<Vec<_>>();
                    let low_band = low
                        .1
                        .iter()
                        .map(|value| u16::from(*value))
                        .collect::<Vec<_>>();
                    let mid_band = mid
                        .1
                        .iter()
                        .map(|value| u16::from(*value))
                        .collect::<Vec<_>>();

                    let overlap_path = waveform_band_path_data(
                        &zeros,
                        &overlap,
                        max_value,
                        plot_left_f,
                        top_f,
                        plot_width_f,
                        section_height_f,
                        section.vertical_alignment,
                    );
                    write_svg_path(
                        &mut svg,
                        "waveform-layer waveform-layer-overlap",
                        &overlap_path,
                        WAVEFORM_OVERLAP_FILL,
                        1.0,
                        WAVEFORM_OVERLAP_FILL,
                    );

                    let low_path = waveform_band_path_data(
                        &overlap,
                        &low_band,
                        max_value,
                        plot_left_f,
                        top_f,
                        plot_width_f,
                        section_height_f,
                        section.vertical_alignment,
                    );
                    write_svg_path(
                        &mut svg,
                        low.0.class_name,
                        &low_path,
                        low.0.fill,
                        1.0,
                        low.0.stroke,
                    );

                    let mid_path = waveform_band_path_data(
                        &overlap,
                        &mid_band,
                        max_value,
                        plot_left_f,
                        top_f,
                        plot_width_f,
                        section_height_f,
                        section.vertical_alignment,
                    );
                    write_svg_path(
                        &mut svg,
                        mid.0.class_name,
                        &mid_path,
                        mid.0.fill,
                        1.0,
                        mid.0.stroke,
                    );

                    let high_path = waveform_path_data(
                        &high.1,
                        high.0.max_value,
                        plot_left_f,
                        top_f,
                        plot_width_f,
                        section_height_f,
                        section.vertical_alignment,
                    );
                    write_svg_path(
                        &mut svg,
                        high.0.class_name,
                        &high_path,
                        high.0.fill,
                        1.0,
                        high.0.stroke,
                    );
                }
            }
            WaveformRenderStyle::Stacked => {
                let len = resampled_layers
                    .first()
                    .map(|(_, values)| values.len())
                    .unwrap_or(0);
                let mut totals = vec![0u16; len];
                for (_, values) in &resampled_layers {
                    for (index, value) in values.iter().enumerate() {
                        totals[index] = totals[index].saturating_add(u16::from(*value));
                    }
                }
                let max_total = totals.into_iter().max().unwrap_or(1);
                let mut lower = vec![0u16; len];
                for (layer, values) in &resampled_layers {
                    let upper = values
                        .iter()
                        .enumerate()
                        .map(|(index, value)| lower[index].saturating_add(u16::from(*value)))
                        .collect::<Vec<_>>();
                    let path_data = waveform_band_path_data(
                        &lower,
                        &upper,
                        max_total,
                        plot_left_f,
                        top_f,
                        plot_width_f,
                        section_height_f,
                        section.vertical_alignment,
                    );
                    write_svg_path(
                        &mut svg,
                        layer.class_name,
                        &path_data,
                        layer.fill,
                        f32::from(layer.fill_opacity_percent) / 100.0,
                        layer.stroke,
                    );
                    lower = upper;
                }
            }
        }
    }

    svg.end_element();
    Ok(svg.end_document())
}

pub(crate) fn render_waveforms(
    path: &Path,
    output: &Path,
    no_ext: bool,
    no_2ex: bool,
    section_height: u32,
) -> rekordcrate::Result<()> {
    let paths = collect_related_anlz_paths(path, no_ext, no_2ex);
    let svg = render_waveform_svg_from_paths(&paths, section_height)?;
    std::fs::write(output, svg)?;
    println!("Rendered waveform SVG to {}", output.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        collect_related_anlz_paths, collect_waveform_sections, read_anlz,
        render_waveform_svg_from_paths, waveform_render_column_fill, WaveformRenderColumn,
        WaveformRenderStyle, WaveformVerticalAlignment,
    };
    use std::path::PathBuf;

    fn fixture_dat_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("data/complete_export/demo_tracks/PIONEER/USBANLZ/P016/0000875E/ANLZ0000.DAT")
    }

    #[test]
    fn collects_related_anlz_paths() {
        let dat_path = fixture_dat_path();
        let paths = collect_related_anlz_paths(&dat_path, false, false);

        assert_eq!(
            paths,
            vec![
                dat_path.clone(),
                dat_path.with_extension("EXT"),
                dat_path.with_extension("2EX"),
            ]
        );
    }

    #[test]
    fn renders_waveforms_to_svg() {
        let dat_path = fixture_dat_path();
        let svg = render_waveform_svg_from_paths(
            &collect_related_anlz_paths(&dat_path, false, false),
            48,
        )
        .expect("svg should render");

        assert!(svg.contains("<svg"));
        assert!(svg.contains("PWAV"));
        assert!(svg.contains("PWV2"));
        assert!(svg.contains("PWV3"));
        assert!(svg.contains("PWV4"));
        assert!(svg.contains("PWV5"));
        assert!(svg.contains("PWV6"));
        assert!(svg.contains("PWV7"));
        assert!(svg.contains("00:00"));
        assert!(svg.contains("class=\"beat-grid beat-grid-bar\""));
        assert!(svg.contains("class=\"beat-grid beat-grid-beat\""));
        assert!(svg.contains("class=\"axis-label\""));
        assert!(svg.contains("class=\"waveform-layer waveform-layer-blue\""));
        assert!(svg.contains("class=\"waveform-layer waveform-layer-low\""));
        assert!(svg.contains("class=\"waveform-layer waveform-layer-mid\""));
        assert!(svg.contains("class=\"waveform-layer waveform-layer-high\""));
        assert!(svg.contains("class=\"waveform-layer waveform-layer-overlap\""));
        assert!(svg.contains("class=\"waveform-column waveform-column-color-preview\""));
        assert!(svg.contains("class=\"waveform-column waveform-column-color-detail\""));
    }

    #[test]
    fn waveform_sections_use_expected_alignment_and_styles() {
        let dat_path = fixture_dat_path();
        let anlzs = collect_related_anlz_paths(&dat_path, false, false)
            .into_iter()
            .map(|path| read_anlz(&path).expect("fixture should parse"))
            .collect::<Vec<_>>();
        let sections = collect_waveform_sections(&anlzs);

        let pwv3 = sections
            .iter()
            .find(|section| section.label == "PWV3")
            .expect("PWV3 section should exist");
        assert_eq!(pwv3.render_style, WaveformRenderStyle::ColorColumns);
        assert_eq!(pwv3.vertical_alignment, WaveformVerticalAlignment::Center);

        let pwv4 = sections
            .iter()
            .find(|section| section.label == "PWV4")
            .expect("PWV4 section should exist");
        assert_eq!(pwv4.render_style, WaveformRenderStyle::ColorColumns);
        assert_eq!(pwv4.vertical_alignment, WaveformVerticalAlignment::Bottom);

        let pwv5 = sections
            .iter()
            .find(|section| section.label == "PWV5")
            .expect("PWV5 section should exist");
        assert_eq!(pwv5.render_style, WaveformRenderStyle::ColorColumns);
        assert_eq!(pwv5.vertical_alignment, WaveformVerticalAlignment::Center);

        let pwv6 = sections
            .iter()
            .find(|section| section.label == "PWV6")
            .expect("PWV6 section should exist");
        assert_eq!(pwv6.render_style, WaveformRenderStyle::Stacked);
        assert_eq!(pwv6.vertical_alignment, WaveformVerticalAlignment::Bottom);

        let pwv7 = sections
            .iter()
            .find(|section| section.label == "PWV7")
            .expect("PWV7 section should exist");
        assert_eq!(pwv7.render_style, WaveformRenderStyle::SharedAxisBlend);
        assert_eq!(pwv7.vertical_alignment, WaveformVerticalAlignment::Center);
    }

    #[test]
    fn blue_waveform_whiteness_brightens_columns() {
        let dark = WaveformRenderColumn::blue(31, 0x1f, 0);
        let bright = WaveformRenderColumn::blue(31, 0x1f, 7);

        assert_eq!(waveform_render_column_fill(&dark), "#2563eb");
        assert_eq!(waveform_render_column_fill(&bright), "#ffffff");
    }
}
