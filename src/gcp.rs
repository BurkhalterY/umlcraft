//! GCP architecture diagrams: zones (filled by their kind) hold the other cards; users, products and
//! services are cards with an icon; a product can show its expanded details (machine, disk and
//! modifier icons with their numbers) and stand for several instances; paths are coloured and
//! dotted by their kind (extensions/essential/gcp: metamodel.json, elements.js, gcp-factory.js,
//! gcp-commands.js).

use crate::cloud::{self, draw_label, measure_label, set_label};
use crate::flowchart::round_box;
use crate::model::{Eid, Repo};
use crate::render::{parse_color, Canvas, Rgba};
use crate::views::{self, Measure, P, R};
use serde_json::{json, Value};

/// Padding, card shadow and icon sizes (elements.js:28-45).
const PAD: f64 = 8.0;
const SHADOW_OFFSET: f64 = 2.0;
const SHADOW_ALPHA: f64 = 0.5;
const SHADOW_COLOR: &str = "#E2E2E2";
const USER_ICON: f64 = 48.0;
const PRODUCT_ICON: f64 = 32.0;
const SERVICE_ICON: f64 = 32.0;
const EXPANDED_ICON: f64 = 24.0;
const ROUND: f64 = 2.0;
/// A card's line colour (the GCPUserView, GCPProductView and GCPServiceView constructors).
pub const CARD_LINE: &str = "#E2E2E2";
/// The product line's and the expanded details' font colours (GCPProductView.update).
pub const PRODUCT_FONT: &str = "#707070";
pub const EXPANDED_FONT: &str = "#8E8E8E";

pub const GOOGLE_BLUE: &str = "#4284F3";
pub const GOOGLE_GRAY: &str = "#9E9E9E";
pub const GOOGLE_GREEN: &str = "#34A853";
pub const GOOGLE_RED: &str = "#EA4335";

/// The zone kinds with their quick edit label and fill colour (GCPZoneKind, metamodel.json;
/// quickedits/gcp.json "gcp.zone-type"; GCPZoneView.update, elements.js:575-640).
pub const ZONE_KINDS: [(&str, &str, &str); 20] = [
    ("project", "Project", "#F6F6F6"),
    ("logical-group", "Logical Group", "#E3F2FD"),
    ("region", "Region", "#ECEFF1"),
    ("zone", "Zone", "#FFF3E0"),
    ("sub-network", "SubNetwork", "#EDE7F6"),
    ("firewall", "Firewall", "#FBE9E7"),
    ("instance-group", "Instance Group", "#F9FBE7"),
    ("replica-pool", "Replica Pool", "#E0F7FA"),
    ("kubernetes-cluster", "Kubernetes Cluster", "#FCE4EC"),
    ("pod", "Pod", "#E8F5E9"),
    ("account", "Account", "#E8EAF6"),
    ("optional", "Optional Component", "#FFFFFF"),
    ("system", "System", "#F1F8E9"),
    ("infrastructure-system", "Infrastructure System", "#F3E5F5"),
    ("on-premises", "On-Premises", "#EFEBE9"),
    ("external-saas-providers", "External SaaS Providers", "#FFEBEE"),
    ("external-data-sources", "External Data Sources", "#FFF8E1"),
    ("external-thirdparty-infrastructure", "External 3rd-party Infrastructure", "#E0F2F1"),
    ("external-firstparty-infrastructure", "External 1st-party Infrastructure", "#E1F5FE"),
    ("user", "User", "#FFFFFF"),
];

/// The zone kinds alone, for the Properties panel.
pub const ZONE_TYPES: [&str; 20] = [
    "project", "logical-group", "region", "zone", "sub-network", "firewall", "instance-group", "replica-pool", "kubernetes-cluster", "pod", "account", "optional", "system",
    "infrastructure-system", "on-premises", "external-saas-providers", "external-data-sources", "external-thirdparty-infrastructure", "external-firstparty-infrastructure", "user",
];

/// GCPPathKind, GCPMachineType, GCPDiskType, GCPAdditionalModifierType (metamodel.json).
pub const PATH_KINDS: [&str; 6] = ["primary", "optional-primary", "secondary", "optional-secondary", "success", "failure"];
pub const MACHINE_TYPES: [&str; 7] = ["none", "standard", "shared-core-f1", "shared-core-g1", "high-cpu", "high-memory", "custom-virtual"];
pub const DISK_TYPES: [&str; 4] = ["none", "pd", "pd-ssd", "local-ssd"];
pub const MODIFIERS: [&str; 3] = ["none", "autoscaling", "preemptable"];

pub fn is_node_type(ty: &str) -> bool {
    matches!(ty, "GCPUser" | "GCPZone" | "GCPProduct" | "GCPService")
}

/// A zone holds every other GCP card (GCPZoneView.canContainViewKind: GCPGeneralNodeView).
pub fn can_contain(container_ty: &str, view_ty: &str) -> bool {
    container_ty == "GCPZoneView" && view_ty.starts_with("GCP") && cloud::is_node_view(view_ty)
}

/// The icon folders: a user's and a service's pick renames them, a product's sets its product
/// line (handleSetIcon, handleSetProductIcon, gcp-commands.js:31-80).
pub fn icon_base(ty: &str) -> Option<(&'static str, &'static str)> {
    Some(match ty {
        "GCPUser" => ("gcp/users-and-devices", "name"),
        "GCPProduct" => ("gcp/products", "product"),
        "GCPService" => ("gcp/services", "name"),
        _ => return None,
    })
}

pub fn default_icon(ty: &str) -> Option<&'static str> {
    Some(match ty {
        "GCPUser" => "Users.png",
        "GCPProduct" => "Compute-Engine.svg",
        "GCPService" => "Application-System.png",
        _ => return None,
    })
}

/// A new product is a Compute Engine (GCPProduct constructor).
pub fn init_model(r: &mut Repo, m: Eid) {
    if r.ty(m) == "GCPProduct" {
        r.set(m, "product", json!("Compute Engine"));
    }
}

/// Cards have a light grey line (the view constructors).
pub fn init_view(r: &mut Repo, v: Eid) {
    if matches!(r.ty(v), "GCPUserView" | "GCPProductView" | "GCPServiceView") {
        r.set(v, "lineColor", json!(CARD_LINE));
    }
}

/// The product view's two more labels (GCPProductView constructor).
pub fn add_labels(r: &mut Repo, v: Eid, model: Eid) {
    if r.ty(v) != "GCPProductView" {
        return;
    }
    let m = r.ref_to(model);
    for f in ["productLabel", "expandedLabel"] {
        let l = r.create("LabelView", &[("model", m.clone()), ("font", json!(views::FONT)), ("parentStyle", json!(true)), ("horizontalAlignment", json!(0)), ("wordWrap", json!(true))], Some((v, "subViews")));
        r.set_ref(v, f, Some(l));
    }
}

/// The quick edit's Zone Type dropdown (quickedits/gcp.json "gcp.zone-type").
pub fn dropdowns(ty: &str) -> Vec<(&'static str, &'static str, Vec<(Value, &'static str)>)> {
    if ty == "GCPZone" {
        vec![("zoneType", "Zone Type", ZONE_KINDS.iter().map(|z| (json!(z.0), z.1)).collect())]
    } else {
        vec![]
    }
}

/// A zone's fill colour by its kind.
pub fn zone_fill(kind: &str) -> Option<&'static str> {
    ZONE_KINDS.iter().find(|z| z.0 == kind).map(|z| z.2)
}

/// A path's colour and whether it is dotted, by its kind (GCPPathView.update, elements.js:1015-1045).
pub fn path_style(kind: &str) -> Option<(&'static str, bool)> {
    Some(match kind {
        "primary" => (GOOGLE_BLUE, false),
        "optional-primary" => (GOOGLE_BLUE, true),
        "secondary" => (GOOGLE_GRAY, false),
        "optional-secondary" => (GOOGLE_GRAY, true),
        "success" => (GOOGLE_GREEN, false),
        "failure" => (GOOGLE_RED, false),
        _ => return None,
    })
}

/// What a path's update sets on its view: its colour.
pub fn update_edge(r: &mut Repo, e: Eid) {
    let Some(m) = r.refd(e, "model") else { return };
    let kind = r.str(m, "pathType");
    let kind = if kind.is_empty() { "primary".to_string() } else { kind };
    if let Some((c, _)) = path_style(&kind) {
        r.set(e, "lineColor", json!(c));
    }
}

pub fn edge_dotted(r: &Repo, m: Eid) -> bool {
    path_style(&r.str(m, "pathType")).map(|s| s.1).unwrap_or(false)
}

/// The sizes of a product's parts: icon, main (name and product) and expanded details
/// (GCPProductView.getPartSizes, elements.js:705-740): (iw, ih, mw, mh, ew, eh).
fn part_sizes(r: &Repo, ms: &dyn Measure, m: Eid, w: f64) -> (f64, f64, f64, f64, f64, f64) {
    let size = ms.size();
    let iw = PAD + PRODUCT_ICON + PAD;
    let ih = PAD + PRODUCT_ICON + PAD;
    let lw = (w - iw).max(1.0);
    let (nw, nl) = measure_label(ms, &r.name(m), lw, true, 0);
    let (pw, pl) = measure_label(ms, &r.str(m, "product"), lw, true, 0);
    let mw = nw.max(pw);
    let mh = PAD + nl as f64 * size + PAD / 2.0 + pl as f64 * size + PAD;
    let expanded = r.bool(m, "expanded");
    let has_icon = [("machineType", "none"), ("diskType", "none"), ("additionalModifier", "none")].iter().any(|(f, d)| {
        let v = r.str(m, f);
        !v.is_empty() && v != *d
    });
    let (ew, eh) = if expanded {
        let (xw, xl) = measure_label(ms, &r.str(m, "expandedInfo"), lw, true, 0);
        (PAD + xw.max((EXPANDED_ICON + PAD) * 3.0) + PAD, PAD + xl as f64 * size + if has_icon { PAD + EXPANDED_ICON + PAD } else { 0.0 } + PAD)
    } else {
        (0.0, 0.0)
    };
    (iw, ih, mw, mh, ew, eh)
}

/// Size a card and place its labels (sizeObject and arrangeObject of each view, elements.js:470-1000).
pub fn layout(r: &mut Repo, ms: &dyn Measure, v: Eid) {
    let ty = r.ty(v).to_string();
    let m = r.refd(v, "model").unwrap();
    let (face, size) = cloud::font_of(r, v);
    let sm = views::Scaled { ms, face: face.clone(), size };
    let font = views::font_str(&face, size, 0);
    let wrap = r.raw(v, "wordWrap").and_then(|x| x.as_bool()).unwrap_or(true);
    let name = r.name(m);
    let (x, y, w0) = (r.f(v, "left"), r.f(v, "top"), r.f(v, "width"));
    let label = |r: &mut Repo, field: &str, text: &str, b: R| {
        if let Some(l) = r.refd(v, field) {
            set_label(r, l, text, &font, b, wrap);
        }
    };
    match ty.as_str() {
        "GCPZoneView" => {
            // the fill colour of its kind, which is saved with the view (GCPZoneView.update)
            let kind = r.str(m, "zoneType");
            if let Some(f) = zone_fill(if kind.is_empty() { "project" } else { &kind }) {
                r.set(v, "fillColor", json!(f));
            }
            cloud::layout_general(r, ms, v, PAD);
        }
        "GCPUserView" => {
            let (min_lw, _) = measure_label(&sm, &name, 0.0, wrap, 0);
            let lw = (w0.max(min_lw + 2.0 * PAD).max(USER_ICON) - 2.0 * PAD).max(1.0);
            let (_, lines) = measure_label(&sm, &name, lw, wrap, 0);
            let lh = lines as f64 * size;
            let (w, _) = cloud::fit(r, v, (min_lw + 2.0 * PAD).max(USER_ICON), USER_ICON + PAD + lh + PAD);
            label(r, "nameLabel", &name, R { x: x + PAD, y: y + USER_ICON + PAD, w: w - 2.0 * PAD, h: lh });
        }
        "GCPServiceView" => {
            let (min_lw, _) = measure_label(&sm, &name, 0.0, wrap, 0);
            let lead = PAD + SERVICE_ICON + PAD;
            let lw = (w0.max(lead + min_lw + PAD) - lead - PAD).max(1.0);
            let (_, lines) = measure_label(&sm, &name, lw, wrap, 0);
            let lh = lines as f64 * size;
            let (w, h) = cloud::fit(r, v, lead + min_lw + PAD, (SERVICE_ICON + 2.0 * PAD).max(lh + 2.0 * PAD));
            label(r, "nameLabel", &name, R { x: x + lead, y: y + ((h - lh) / 2.0).round(), w: w - lead - PAD, h: lh });
        }
        "GCPProductView" => {
            let (iw, ih, mw, mh, ew, eh) = part_sizes(r, &sm, m, w0);
            let (w, h) = cloud::fit(r, v, iw + mw.max(ew), ih.max(mh + eh));
            let (iw, _, _, mh, _, _) = part_sizes(r, &sm, m, w);
            let ch = ih.max(mh);
            let lw = w - iw;
            let (_, nl) = measure_label(&sm, &name, lw, wrap, 0);
            let product = r.str(m, "product");
            let (_, pl) = measure_label(&sm, &product, lw, wrap, 0);
            let info = r.str(m, "expandedInfo");
            let (_, xl) = measure_label(&sm, &info, lw, wrap, 0);
            let ny = y + PAD + ((ch - mh) / 2.0).round();
            label(r, "nameLabel", &name, R { x: x + iw, y: ny, w: lw, h: nl as f64 * size });
            label(r, "productLabel", &product, R { x: x + iw, y: ny + nl as f64 * size + PAD / 2.0, w: lw, h: pl as f64 * size });
            label(r, "expandedLabel", &info, R { x: x + iw, y: y + mh + PAD, w: lw, h: xl as f64 * size });
            let expanded = r.bool(m, "expanded");
            for (f, c) in [("productLabel", PRODUCT_FONT), ("expandedLabel", EXPANDED_FONT)] {
                if let Some(l) = r.refd(v, f) {
                    r.set(l, "fontColor", json!(c));
                }
            }
            if let Some(l) = r.refd(v, "expandedLabel") {
                if expanded {
                    r.remove(l, "visible");
                } else {
                    r.set_raw(l, "visible", json!(false));
                }
            }
            let _ = h;
        }
        _ => {}
    }
}

fn shade(c: &mut dyn Canvas, b: R, off: f64) {
    let s = parse_color(SHADOW_COLOR, Rgba(0xE2, 0xE2, 0xE2, 255));
    let s = Rgba(s.0, s.1, s.2, (255.0 * SHADOW_ALPHA) as u8);
    c.polygon(&round_box(R { x: b.x + off, y: b.y + off, ..b }, ROUND), Some(s), None);
}

fn card(c: &mut dyn Canvas, b: R, fill: Rgba, line: Rgba) {
    let pts = round_box(b, ROUND);
    c.polygon(&pts, Some(fill), Some(line));
}

/// A number in a blue pill, white (drawCircleNumber, elements.js:85-105).
fn circle_number(c: &mut dyn Canvas, x: f64, y: f64, text: &str, size: f64) {
    let w = (c.width(text, 0) * size / 13.0 + 4.0).max(16.0);
    let b = R { x, y, w, h: 16.0 };
    c.polygon(&round_box(b, 8.0), Some(parse_color(GOOGLE_BLUE, crate::render::BLACK)), None);
    c.text(text, b, 2, 0, size, crate::render::WHITE, false);
}

/// A number as JavaScript prints it: no ".0" on a whole number.
fn js_number(v: f64) -> String {
    if v.fract() == 0.0 {
        format!("{}", v as i64)
    } else {
        format!("{v}")
    }
}

fn label_color(r: &Repo, l: Eid, def: Rgba) -> Rgba {
    parse_color(&r.str(l, "fontColor"), def)
}

/// Draw a card (drawObject and drawShadow of each view, elements.js:520-1000).
pub fn draw(c: &mut dyn Canvas, r: &Repo, v: Eid, fill: Rgba, line: Rgba, fc: Rgba) {
    let b = views::node_rect(r, v);
    let Some(m) = r.refd(v, "model") else { return };
    let icon = r.str(m, "icon");
    let tint = parse_color(GOOGLE_BLUE, line);
    let shadow = r.bool(v, "showShadow");
    match r.ty(v) {
        "GCPZoneView" => {
            let pts = round_box(b, ROUND);
            c.polygon(&pts, Some(fill), None);
            if r.str(m, "zoneType") == "optional" {
                cloud::outline(c, &pts, line, Some((4.0, 4.0)));
            }
        }
        "GCPUserView" => {
            if shadow {
                shade(c, b, SHADOW_OFFSET);
            }
            card(c, b, fill, line);
            let ib = R { x: b.x + ((b.w - USER_ICON) / 2.0).round(), y: b.y, w: USER_ICON, h: USER_ICON };
            cloud::draw_icon(c, "gcp/users-and-devices", &icon, ib, tint, line);
        }
        "GCPServiceView" => {
            if shadow {
                shade(c, b, SHADOW_OFFSET);
            }
            card(c, b, fill, line);
            let ib = R { x: b.x + PAD, y: b.y + ((b.h - SERVICE_ICON) / 2.0).round(), w: SERVICE_ICON, h: SERVICE_ICON };
            cloud::draw_icon(c, "gcp/services", &icon, ib, tint, line);
        }
        "GCPProductView" => {
            // several instances: a second card 6 px behind
            if r.bool(m, "multiInstance") {
                let back = R { x: b.x + 6.0, y: b.y + 6.0, ..b };
                shade(c, back, SHADOW_OFFSET);
                card(c, back, fill, line);
            }
            shade(c, b, SHADOW_OFFSET);
            card(c, b, fill, line);
            let iw = PAD + PRODUCT_ICON + PAD;
            let size = views::view_font(r, v).1;
            let mh = r.refd(v, "expandedLabel").map(|l| r.f(l, "top") - PAD - b.y).unwrap_or(0.0);
            let expanded = r.bool(m, "expanded");
            if expanded {
                c.polyline(&[P { x: b.x + iw, y: b.y + mh }, P { x: b.x + b.w, y: b.y + mh }], line, false);
            }
            cloud::draw_icon(c, "gcp/products", &icon, R { x: b.x + PAD, y: b.y + PAD, w: PRODUCT_ICON, h: PRODUCT_ICON }, tint, line);
            if expanded {
                let eb = r.refd(v, "expandedLabel").map(|l| r.f(l, "top") + r.f(l, "height")).unwrap_or(b.y + mh);
                let top = eb + PAD;
                let y = top + EXPANDED_ICON;
                let at = |k: f64| R { x: b.x + iw + k, y: top, w: EXPANDED_ICON, h: EXPANDED_ICON };
                let machine = r.str(m, "machineType");
                if !machine.is_empty() && machine != "none" {
                    cloud::draw_icon(c, "gcp/expanded-icons", &format!("{machine}.png"), at(0.0), tint, line);
                    let cores = r.f(m, "cores");
                    if cores > 0.0 {
                        circle_number(c, b.x + iw - 8.0, y - 10.0, &js_number(cores), size);
                    }
                    let ram = r.f(m, "ram");
                    if ram > 0.0 {
                        circle_number(c, b.x + iw + EXPANDED_ICON - 8.0, y - 10.0, &js_number(ram), size);
                    }
                }
                let disk = r.str(m, "diskType");
                if !disk.is_empty() && disk != "none" {
                    cloud::draw_icon(c, "gcp/expanded-icons", "disk.png", at(EXPANDED_ICON + PAD), tint, line);
                    let disks = r.f(m, "disks");
                    if disks > 0.0 {
                        circle_number(c, b.x + iw + EXPANDED_ICON * 2.0 - 4.0, y - 10.0, &js_number(disks), size);
                    }
                }
                let modifier = r.str(m, "additionalModifier");
                if !modifier.is_empty() && modifier != "none" {
                    cloud::draw_icon(c, "gcp/expanded-icons", &format!("{modifier}.png"), at((EXPANDED_ICON + PAD) * 2.0), tint, line);
                }
            }
            for (f, def) in [("productLabel", PRODUCT_FONT), ("expandedLabel", EXPANDED_FONT)] {
                if let Some(l) = r.refd(v, f) {
                    let col = label_color(r, l, parse_color(def, fc));
                    draw_label(c, r, l, col);
                }
            }
        }
        _ => {}
    }
    if let Some(l) = r.refd(v, "nameLabel") {
        draw_label(c, r, l, fc);
    }
}
