//! Azure architecture diagrams: groups (filled, square or rounded, solid or dashed) hold the other
//! boxes; services are icons with their name under them; callouts are numbered black circles;
//! connectors end in a filled arrow head, dotted when dashed (extensions/essential/azure:
//! metamodel.json, elements.js, azure-factory.js, azure-commands.js).

use crate::cloud::{self, draw_label, outline, rect_pts};
use crate::model::{Eid, Repo};
use crate::render::{parse_color, Canvas, Rgba};
use crate::views::{self, Measure};
use serde_json::{json, Value};

/// Padding inside a group, a group's corner radius and an icon's smallest size (elements.js:30-41).
const PAD: f64 = 10.0;
const GROUP_ROUND: f64 = 10.0;
const ICON_MIN: f64 = 48.0;
/// The placeholder's colour (Azure blue; UMLCraft's own).
const TINT: &str = "#0078D4";

pub fn is_node_type(ty: &str) -> bool {
    matches!(ty, "AzureGroup" | "AzureService" | "AzureCallout")
}

pub fn is_icon_view(ty: &str) -> bool {
    ty == "AzureServiceView"
}

/// A group holds every other Azure box (AzureGroupView.canContainViewKind).
pub fn can_contain(container_ty: &str, view_ty: &str) -> bool {
    container_ty == "AzureGroupView" && view_ty.starts_with("Azure") && cloud::is_node_view(view_ty)
}

/// A service's icons are in the azure folder; the pick renames it (AzureService, azure-commands.js:31-50).
pub fn icon_base(ty: &str) -> Option<(&'static str, &'static str)> {
    (ty == "AzureService").then_some(("azure", "name"))
}

pub fn default_icon(ty: &str) -> Option<&'static str> {
    (ty == "AzureService").then_some("App-Services.svg")
}

/// A callout is black with white text (AzureCalloutView constructor).
pub fn init_view(r: &mut Repo, v: Eid) {
    if r.ty(v) == "AzureCalloutView" {
        r.set(v, "fillColor", json!("#000000"));
        r.set(v, "fontColor", json!("#ffffff"));
    }
}

/// The quick edits' Dashed and Rounded dropdowns (quickedits/azure.json "azure.set-dashed",
/// "azure.set-rounded"): a group has both, a connector Dashed.
pub fn dropdowns(ty: &str) -> Vec<(&'static str, &'static str, Vec<(Value, &'static str)>)> {
    let dashed = ("dashed", "Dashed", vec![(json!(false), "solid"), (json!(true), "dashed")]);
    match ty {
        "AzureGroup" => vec![dashed, ("rounded", "Rounded", vec![(json!(false), "rect"), (json!(true), "rounded")])],
        "AzureConnector" => vec![dashed],
        _ => vec![],
    }
}

pub fn layout(r: &mut Repo, ms: &dyn Measure, v: Eid) {
    match r.ty(v) {
        "AzureServiceView" => cloud::layout_icon(r, ms, v, ICON_MIN),
        "AzureCalloutView" => cloud::layout_callout(r, ms, v),
        _ => {
            cloud::layout_general(r, ms, v, PAD);
        }
    }
}

/// Draw a box (drawObject of each view, elements.js:350-440).
pub fn draw(c: &mut dyn Canvas, r: &Repo, v: Eid, fill: Rgba, line: Rgba, fc: Rgba) {
    let b = views::node_rect(r, v);
    let Some(m) = r.refd(v, "model") else { return };
    match r.ty(v) {
        "AzureServiceView" => {
            cloud::draw_icon(c, "azure", &r.str(m, "icon"), b, parse_color(TINT, line), line);
        }
        "AzureCalloutView" => {
            cloud::draw_callout(c, r, v, fill, fc);
            return;
        }
        _ => {
            let pts = if r.bool(m, "rounded") { crate::flowchart::round_box(b, GROUP_ROUND) } else { rect_pts(b) };
            c.polygon(&pts, Some(fill), None);
            outline(c, &pts, line, r.bool(m, "dashed").then_some((3.0, 3.0)));
        }
    }
    if let Some(l) = r.refd(v, "nameLabel") {
        draw_label(c, r, l, fc);
    }
}
