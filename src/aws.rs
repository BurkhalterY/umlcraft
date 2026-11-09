//! AWS architecture diagrams: groups (whose icon sets their colour and dashes), generic groups,
//! availability zones and security groups hold the other boxes; services, resources and general
//! resources are icons with their name under them; callouts are numbered black circles; arrows end
//! in an open arrow head (extensions/essential/aws: metamodel.json, elements.js, aws-factory.js,
//! aws-commands.js).

use crate::cloud::{self, draw_label, outline, rect_pts};
use crate::model::{Eid, Repo};
use crate::render::{parse_color, Canvas, Rgba};
use crate::views::{self, Measure, R};
use serde_json::json;

/// Padding inside a general box, a group's icon and an icon's smallest size (elements.js:30-40).
const PAD: f64 = 10.0;
const GROUP_ICON: f64 = 32.0;
const ICON_MIN: f64 = 48.0;

/// The AWS palette (elements.js:42-51).
pub const SQUID: &str = "#232F3E";
pub const GALAXY: &str = "#8C4FFF";
pub const COSMOS: &str = "#E7157B";
pub const SMILE: &str = "#ED7100";
pub const ENDOR: &str = "#7AA116";
pub const ORBIT: &str = "#01A88D";
/// A security group's red (AWSSecurityGroupView.drawObject).
pub const SECURITY_RED: &str = "#CB464F";

pub fn is_node_type(ty: &str) -> bool {
    matches!(ty, "AWSGroup" | "AWSGenericGroup" | "AWSAvailabilityZone" | "AWSSecurityGroup" | "AWSService" | "AWSResource" | "AWSGeneralResource" | "AWSCallout")
}

/// The views that hold others (AWSGroupView and the three other group views).
pub fn is_group_view(ty: &str) -> bool {
    matches!(ty, "AWSGroupView" | "AWSGenericGroupView" | "AWSAvailabilityZoneView" | "AWSSecurityGroupView")
}

/// The icon views (AWSIconNodeView's subclasses).
pub fn is_icon_view(ty: &str) -> bool {
    matches!(ty, "AWSServiceView" | "AWSResourceView" | "AWSGeneralResourceView")
}

/// A group holds every other AWS box (canContainViewKind: AWSGeneralNodeView or AWSIconNodeView).
pub fn can_contain(container_ty: &str, view_ty: &str) -> bool {
    is_group_view(container_ty) && view_ty.starts_with("AWS") && cloud::is_node_view(view_ty)
}

/// The icon folder and renamed field of the types with an icon (`__assetBaseDir`).
pub fn icon_base(ty: &str) -> Option<(&'static str, &'static str)> {
    Some(match ty {
        "AWSGroup" => ("aws/group-icons", "name"),
        "AWSService" => ("aws/service-icons", "name"),
        "AWSResource" => ("aws/resource-icons", "name"),
        "AWSGeneralResource" => ("aws/general-icons", "name"),
        _ => return None,
    })
}

/// The icon a new element has (the constructors, elements.js:130-205).
pub fn default_icon(ty: &str) -> Option<&'static str> {
    Some(match ty {
        "AWSGroup" => "AWS-Cloud.svg",
        "AWSService" => "Amazon-EC2.svg",
        "AWSResource" => "Amazon-EC2_Instance.svg",
        "AWSGeneralResource" => "Users.svg",
        _ => return None,
    })
}

/// A callout is black with white text (AWSCalloutView constructor).
pub fn init_view(r: &mut Repo, v: Eid) {
    if r.ty(v) == "AWSCalloutView" {
        r.set(v, "fillColor", json!("#000000"));
        r.set(v, "fontColor", json!("#ffffff"));
    }
}

/// A group's line colour and dashes by its icon (AWSGroupView.update, elements.js:395-440).
pub fn group_style(icon: &str) -> (&'static str, bool) {
    match icon {
        "Region.svg" => (ORBIT, true),
        "Private-subnet.svg" => (ORBIT, false),
        "EC2-instance-contents.svg" | "Spot-Fleet.svg" | "Elastic-Beanstalk-container.svg" => (SMILE, false),
        "Auto-Scaling-group.svg" => (SMILE, true),
        "Virtual-private-cloud-(VPC).svg" => (GALAXY, false),
        "Public-subnet.svg" | "AWS-IoT-Greengrass-Deployment.svg" | "AWS-IoT-Greengrass.svg" => (ENDOR, false),
        "Server-contents.svg" | "Corporate-data-center.svg" => (SQUID, false),
        "AWS-account.svg" | "AWS-Step-Functions-workflow.svg" => (COSMOS, false),
        _ => ("#000000", false),
    }
}

/// The line colour each group view sets in update or drawObject, which is saved with it.
fn line_color(r: &Repo, v: Eid, m: Eid) -> Option<&'static str> {
    Some(match r.ty(v) {
        "AWSGroupView" => group_style(&r.str(m, "icon")).0,
        "AWSGenericGroupView" => SQUID,
        "AWSAvailabilityZoneView" => ORBIT,
        "AWSSecurityGroupView" => SECURITY_RED,
        _ => return None,
    })
}

/// Size a box and place its name (sizeObject, arrangeObject, update of each view).
pub fn layout(r: &mut Repo, ms: &dyn Measure, v: Eid) {
    let ty = r.ty(v).to_string();
    let m = r.refd(v, "model").unwrap();
    if is_icon_view(&ty) {
        cloud::layout_icon(r, ms, v, ICON_MIN);
        return;
    }
    if ty == "AWSCalloutView" {
        cloud::layout_callout(r, ms, v);
        return;
    }
    if let Some(c) = line_color(r, v, m) {
        r.set(v, "lineColor", json!(c));
    }
    let (b, lines, size) = cloud::layout_general(r, ms, v, PAD);
    // a group's name is right of its icon, 32 high; an auto scaling group's under the icon, centred
    // (AWSGroupView.arrangeObject, elements.js:370-395)
    if ty == "AWSGroupView" {
        let Some(l) = r.refd(v, "nameLabel") else { return };
        let (lb, align) = if r.str(m, "icon") == "Auto-Scaling-group.svg" {
            (R { x: b.x + PAD, y: b.y + GROUP_ICON + PAD, w: b.w - 2.0 * PAD, h: lines as f64 * size }, 2)
        } else {
            (R { x: b.x + GROUP_ICON + PAD, y: b.y, w: b.w - (GROUP_ICON + 2.0 * PAD), h: GROUP_ICON }, 0)
        };
        for (k, val) in [("left", lb.x), ("top", lb.y), ("width", lb.w), ("height", lb.h)] {
            r.set(l, k, json!(val));
        }
        r.set(l, "horizontalAlignment", json!(align));
    }
}

/// Draw a box (drawObject of each view, elements.js:440-640).
pub fn draw(c: &mut dyn Canvas, r: &Repo, v: Eid, fill: Rgba, line: Rgba, fc: Rgba) {
    let b = views::node_rect(r, v);
    let Some(m) = r.refd(v, "model") else { return };
    let ty = r.ty(v);
    let icon = r.str(m, "icon");
    let tint = parse_color(SMILE, line);
    match ty {
        _ if is_icon_view(ty) => {
            cloud::draw_icon(c, icon_base(r.ty(m)).map(|x| x.0).unwrap_or(""), &icon, b, tint, line);
            if let Some(l) = r.refd(v, "nameLabel") {
                draw_label(c, r, l, fc);
            }
            return;
        }
        "AWSCalloutView" => {
            cloud::draw_callout(c, r, v, fill, fc);
            return;
        }
        "AWSGroupView" => {
            let dashed = group_style(&icon).1;
            outline(c, &rect_pts(b), line, dashed.then_some((3.0, 3.0)));
            let ib = if icon == "Auto-Scaling-group.svg" {
                R { x: b.x + b.w / 2.0 - GROUP_ICON / 2.0, y: b.y, w: GROUP_ICON, h: GROUP_ICON }
            } else {
                R { x: b.x, y: b.y, w: GROUP_ICON, h: GROUP_ICON }
            };
            cloud::draw_icon(c, "aws/group-icons", &icon, ib, line, line);
        }
        "AWSGenericGroupView" => outline(c, &rect_pts(b), line, r.bool(m, "dashed").then_some((3.0, 3.0))),
        "AWSAvailabilityZoneView" => outline(c, &rect_pts(b), line, Some((3.0, 3.0))),
        _ => outline(c, &rect_pts(b), line, None),
    }
    if let Some(l) = r.refd(v, "nameLabel") {
        draw_label(c, r, l, fc);
    }
}
