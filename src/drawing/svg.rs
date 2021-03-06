use linked_hash_map::LinkedHashMap;
use svgdom::*;

use super::prelude::*;

use crate::error::*;

#[derive(Clone, Default, Debug)]
pub struct SvgPoint {
    pub x: f64,
    pub y: f64,
    pub marker: bool,
}

#[derive(Default, Debug)]
pub struct SvgLine {
    pub p: (SvgPoint, SvgPoint),
    pub width: f64,
}

#[derive(Default, Debug)]
pub struct SvgHLine {
    pub x0: f64,
    pub x1: f64,
    pub y: f64,
    pub width: f64,
}

impl SvgHLine {
    pub fn cx(&self) -> f64 {
        (self.x0 + self.x1) / 2.0
    }

    pub fn len(&self) -> f64 {
        (self.x1 - self.x0).abs()
    }
}

#[derive(Default, Debug)]
pub struct SvgVLine {
    pub x: f64,
    pub y0: f64,
    pub y1: f64,
    pub width: f64,
}

impl SvgVLine {
    pub fn cy(&self) -> f64 {
        (self.y0 + self.y1) / 2.0
    }

    pub fn len(&self) -> f64 {
        (self.y1 - self.y0).abs()
    }
}

#[derive(Default, Debug)]
pub struct SvgPolygon {
    pub p: Vec<SvgPoint>,
    pub line_width: f64,
    pub filled: bool,
}

#[derive(Default, Debug)]
pub struct SvgRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub line_width: f64,
    pub filled: bool,
}

#[derive(Default, Debug)]
pub struct SvgEllipse {
    pub cx: f64,
    pub cy: f64,
    pub rx: f64,
    pub ry: f64,
    pub line_width: f64,
    pub filled: bool,
}

#[derive(Default, Debug)]
pub struct SvgText {
    pub x: f64,
    pub y: f64,
    pub height: f64,
    pub text: String,
    pub halign: HAlign,
    pub valign: VAlign,
}

#[derive(Debug)]
pub enum SvgElement {
    HLine(SvgHLine),
    VLine(SvgVLine),
    Line(SvgLine),
    Polygon(SvgPolygon),
    Rect(SvgRect),
    Ellipse(SvgEllipse),
    Text(SvgText),
}

pub type SvgHash = LinkedHashMap<String, SvgElement>;

#[derive(Default, Debug)]
struct Svg {
    elements: SvgHash,
    id_counter: usize,
}

impl Svg {
    fn new() -> Self {
        Self::default()
    }

    fn add_node(&mut self, node: &Node) -> Result<()> {
        if let Some(id) = node.tag_id() {
            match id {
                ElementId::Defs => return Ok(()), // Skip <defs>
                ElementId::Path => {
                    let mut path_id = node.id().to_string();
                    if path_id.is_empty() {
                        path_id = self.id_counter.to_string();
                        self.id_counter += 1;
                    }
                    let polygon = self.to_polygon(&node.attributes())?;
                    if polygon.p.len() == 2 {
                        if (polygon.p[0].y - polygon.p[1].y).abs() < f64::EPSILON {
                            let line = SvgHLine {
                                x0: polygon.p[0].x,
                                x1: polygon.p[1].x,
                                y: polygon.p[0].y,
                                width: polygon.line_width,
                            };
                            self.elements.insert(path_id, SvgElement::HLine(line));
                        } else if (polygon.p[0].x - polygon.p[1].x).abs() < f64::EPSILON {
                            let line = SvgVLine {
                                x: polygon.p[0].x,
                                y0: polygon.p[0].y,
                                y1: polygon.p[1].y,
                                width: polygon.line_width,
                            };
                            self.elements.insert(path_id, SvgElement::VLine(line));
                        } else {
                            let line = SvgLine {
                                p: (polygon.p[0].clone(), polygon.p[1].clone()),
                                width: polygon.line_width,
                            };
                            self.elements.insert(path_id, SvgElement::Line(line));
                        }
                    } else {
                        self.elements.insert(path_id, SvgElement::Polygon(polygon));
                    }
                }
                ElementId::Rect => {
                    let rect_id = node.id().to_string();
                    let rect = self.to_rect(&node.attributes())?;
                    self.elements.insert(rect_id, SvgElement::Rect(rect));
                }
                ElementId::Ellipse => {
                    let ellipse_id = node.id().to_string();
                    let ellipse = self.to_ellipse(&node.attributes())?;
                    self.elements
                        .insert(ellipse_id, SvgElement::Ellipse(ellipse));
                }
                ElementId::Text => {
                    let text_id = node.id().to_string();
                    let mut text = self.to_text(&node.attributes())?;
                    text.text = node.text().to_string();
                    if node.has_children() {
                        for child in node.children() {
                            if let Some(ElementId::Tspan) = child.tag_id() {
                                let tspan = self.to_text(&child.attributes())?;
                                text.x = tspan.x;
                                text.y = tspan.y;
                                text.halign = tspan.halign;
                                if child.has_children() {
                                    for grandchild in child.children() {
                                        if grandchild.is_text() {
                                            text.text = grandchild.text().to_string();
                                        }
                                    }
                                }
                            } else if child.is_text() {
                                text.text = child.text().to_string();
                            }
                        }
                    }
                    self.elements.insert(text_id, SvgElement::Text(text));
                }
                _ => (),
            }
        }

        if node.has_children() {
            for child in node.children() {
                self.add_node(&child)?;
            }
        }
        Ok(())
    }

    fn convert_units(len: &Length) -> Result<f64> {
        match len.unit {
            LengthUnit::None => Ok(len.num),
            LengthUnit::Mm => Ok(len.num),
            LengthUnit::Pt => Ok(len.num * 25.4 / 72.),
            _ => Err(QedaError::UnsupportedSvgUnits(format!("{:?}", len.unit)).into()),
        }
    }

    fn to_ellipse(&self, attributes: &Attributes) -> Result<SvgEllipse> {
        let mut result = SvgEllipse::default();
        for attr in attributes.iter() {
            match attr.id().ok_or(QedaError::InvalidSvgPath)? {
                AttributeId::Cx => {
                    if let AttributeValue::Length(ref len) = attr.value {
                        result.cx = Svg::convert_units(len)?;
                    }
                }
                AttributeId::Cy => {
                    if let AttributeValue::Length(ref len) = attr.value {
                        result.cy = Svg::convert_units(len)?;
                    }
                }
                AttributeId::Rx => {
                    if let AttributeValue::Length(ref len) = attr.value {
                        result.rx = Svg::convert_units(len)?;
                    }
                }
                AttributeId::Ry => {
                    if let AttributeValue::Length(ref len) = attr.value {
                        result.ry = Svg::convert_units(len)?;
                    }
                }
                AttributeId::StrokeWidth => {
                    if let AttributeValue::Length(ref len) = attr.value {
                        result.line_width = Svg::convert_units(len)?;
                    }
                }
                AttributeId::Fill => {
                    result.filled = attr.value != AttributeValue::None;
                }
                _ => (),
            }
        }
        Ok(result)
    }

    fn to_polygon(&self, attributes: &Attributes) -> Result<SvgPolygon> {
        let mut result = SvgPolygon::default();
        let mut current_x = 0.0;
        let mut current_y = 0.0;

        for attr in attributes.iter() {
            match attr.id().ok_or(QedaError::InvalidSvgPath)? {
                AttributeId::D => {
                    if let AttributeValue::Path(ref path) = attr.value {
                        for command in path.iter() {
                            match command {
                                PathSegment::MoveTo { abs, x, y }
                                | PathSegment::LineTo { abs, x, y } => {
                                    let mut x = *x;
                                    let mut y = *y;
                                    if !abs {
                                        x += current_x;
                                        y += current_y;
                                    }
                                    current_x = x;
                                    current_y = y;
                                    result.p.push(SvgPoint {
                                        x,
                                        y,
                                        marker: false,
                                    });
                                }
                                PathSegment::HorizontalLineTo { abs, x } => {
                                    let mut x = *x;
                                    let y = current_y;
                                    if !abs {
                                        x += current_x;
                                    }
                                    current_x = x;
                                    result.p.push(SvgPoint {
                                        x,
                                        y,
                                        marker: false,
                                    });
                                }
                                PathSegment::VerticalLineTo { abs, y } => {
                                    let x = current_x;
                                    let mut y = *y;
                                    if !abs {
                                        y += current_y;
                                    }
                                    current_y = y;
                                    result.p.push(SvgPoint {
                                        x,
                                        y,
                                        marker: false,
                                    });
                                }
                                _ => (),
                            }
                        }
                    }
                }
                AttributeId::StrokeWidth => {
                    if let AttributeValue::Length(ref len) = attr.value {
                        result.line_width = Svg::convert_units(len)?;
                    }
                }
                AttributeId::Fill => {
                    result.filled = attr.value != AttributeValue::None;
                }
                _ => (),
            }
        }
        Ok(result)
    }

    fn to_rect(&self, attributes: &Attributes) -> Result<SvgRect> {
        let mut result = SvgRect::default();
        for attr in attributes.iter() {
            match attr.id().ok_or(QedaError::InvalidSvgPath)? {
                AttributeId::X => {
                    if let AttributeValue::Length(ref len) = attr.value {
                        result.x = Svg::convert_units(len)?;
                    }
                }
                AttributeId::Y => {
                    if let AttributeValue::Length(ref len) = attr.value {
                        result.y = Svg::convert_units(len)?;
                    }
                }
                AttributeId::Width => {
                    if let AttributeValue::Length(ref len) = attr.value {
                        result.width = Svg::convert_units(len)?;
                    }
                }
                AttributeId::Height => {
                    if let AttributeValue::Length(ref len) = attr.value {
                        result.height = Svg::convert_units(len)?;
                    }
                }
                AttributeId::StrokeWidth => {
                    if let AttributeValue::Length(ref len) = attr.value {
                        result.line_width = Svg::convert_units(len)?;
                    }
                }
                AttributeId::Fill => {
                    result.filled = attr.value != AttributeValue::None;
                }
                _ => (),
            }
        }
        Ok(result)
    }

    fn to_text(&self, attributes: &Attributes) -> Result<SvgText> {
        let mut result = SvgText::default();
        for attr in attributes.iter() {
            match attr.id().ok_or(QedaError::InvalidSvgPath)? {
                AttributeId::X => {
                    if let AttributeValue::LengthList(ref len_list) = attr.value {
                        result.x = Svg::convert_units(len_list.first().unwrap())?;
                    }
                }
                AttributeId::Y => {
                    if let AttributeValue::LengthList(ref len_list) = attr.value {
                        result.y = Svg::convert_units(len_list.first().unwrap())?;
                    }
                }
                AttributeId::FontSize => {
                    if let AttributeValue::Length(ref len) = attr.value {
                        result.height = Svg::convert_units(len)?;
                    }
                }
                AttributeId::TextAnchor => {
                    if let AttributeValue::String(ref string) = attr.value {
                        result.halign = match string.as_ref() {
                            "middle" => HAlign::Center,
                            "end" => HAlign::Right,
                            _ => HAlign::default(),
                        }
                    }
                }
                AttributeId::DominantBaseline => {
                    if let AttributeValue::String(ref string) = attr.value {
                        result.valign = match string.as_ref() {
                            "middle" => VAlign::Middle,
                            "text-before-edge" => VAlign::Top,
                            _ => VAlign::default(),
                        }
                    }
                }
                _ => (),
            }
        }
        Ok(result)
    }
}

pub fn to_elements(svg: &str) -> Result<SvgHash> {
    let svg_doc = svgdom::Document::from_str(svg)?;
    let mut svg = Svg::new();
    svg.add_node(&svg_doc.root())?;
    Ok(svg.elements)
}
