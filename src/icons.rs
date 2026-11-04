//! Small line icons drawn for UMLCraft (simple shapes; not the original's artwork).

use crate::model::{Eid, Repo};
use egui::{Color32, Painter, Pos2, Rect, Shape, Stroke, Vec2};

const Y: Color32 = Color32::from_rgb(0xd9, 0xa5, 0x21); // models
const G: Color32 = Color32::from_rgb(0x36, 0xb3, 0x7e); // diagrams
const B: Color32 = Color32::from_rgb(0x3d, 0x9b, 0xe9); // relationships
const RED: Color32 = Color32::from_rgb(0xe5, 0x53, 0x3d);

/// Icon key for a model element.
pub fn key_for(r: &Repo, e: Eid) -> &'static str {
    let t = r.ty(e);
    match t {
        "Project" => "project",
        "UMLModel" => "model",
        "UMLPackage" | "UMLSubsystem" => "package",
        "UMLClass" => "class",
        "UMLInterface" => "interface",
        "UMLEnumeration" => "enumeration",
        "UMLDataType" | "UMLPrimitiveType" | "UMLSignal" => "datatype",
        "UMLAttribute" | "UMLEnumerationLiteral" => "attribute",
        "UMLOperation" => "operation",
        "UMLParameter" => "parameter",
        "UMLUseCase" => "usecase",
        "UMLActor" => "actor",
        "UMLUseCaseSubject" => "subject",
        "UMLComponent" => "component",
        "UMLArtifact" => "artifact",
        "UMLNode" => "node",
        "UMLComponentRealization" => "realization",
        "UMLDeployment" => "dependency",
        "UMLCommunicationPath" => "association",
        "UMLClassDiagram" | "UMLPackageDiagram" => "diagram-class",
        "UMLUseCaseDiagram" => "diagram-usecase",
        "UMLAssociation" => "association",
        "UMLAssociationEnd" => "end",
        "UMLGeneralization" => "generalization",
        "UMLDependency" | "UMLInclude" | "UMLExtend" => "dependency",
        "UMLInterfaceRealization" | "UMLRealization" => "realization",
        "Tag" => "attribute",
        _ if r.kind_of(e, "Diagram") || t.ends_with("Diagram") => "diagram",
        _ if t.ends_with("Relationship") || t.ends_with("Flow") || t.ends_with("Link") || t.ends_with("Connector") || t.ends_with("Transition") || t.ends_with("Message") => "association",
        _ => "generic",
    }
}

#[allow(non_snake_case)]
pub fn draw(p: &Painter, rect: Rect, key: &str) {
    // monochrome icons take the theme's text colour
    let W: Color32 = crate::ui::TEXT();
    let o = rect.min;
    let s = rect.width() / 16.0;
    let at = |x: f32, y: f32| Pos2::new(o.x + x * s, o.y + y * s);
    let st = |c: Color32| Stroke::new(1.2, c);
    let line = |pts: &[(f32, f32)], c: Color32| {
        p.add(Shape::line(pts.iter().map(|(x, y)| at(*x, *y)).collect(), st(c)));
    };
    let dashed = |pts: &[(f32, f32)], c: Color32| {
        let ps: Vec<Pos2> = pts.iter().map(|(x, y)| at(*x, *y)).collect();
        for sh in Shape::dashed_line(&ps, st(c), 2.0 * s, 1.5 * s) {
            p.add(sh);
        }
    };
    let rect_at = |x: f32, y: f32, w: f32, h: f32, c: Color32| {
        p.rect_stroke(Rect::from_min_size(at(x, y), Vec2::new(w * s, h * s)), 0.0, st(c), egui::StrokeKind::Middle);
    };
    let folder = |c: Color32| line(&[(1.5, 3.5), (6.5, 3.5), (7.5, 5.0), (14.5, 5.0), (14.5, 13.5), (1.5, 13.5), (1.5, 3.5)], c);
    let arrow_up = |c: Color32| line(&[(5.0, 5.5), (8.0, 2.5), (11.0, 5.5)], c);
    let triangle_up = |c: Color32| line(&[(8.0, 2.0), (11.5, 6.5), (4.5, 6.5), (8.0, 2.0)], c);
    match key {
        "project" => {
            line(&[(8.0, 1.5), (13.5, 4.5), (13.5, 11.5), (8.0, 14.5), (2.5, 11.5), (2.5, 4.5), (8.0, 1.5)], RED);
            line(&[(2.5, 4.5), (8.0, 7.5), (13.5, 4.5)], RED);
            line(&[(8.0, 7.5), (8.0, 14.5)], RED);
        }
        "model" => {
            folder(Y);
            line(&[(6.0, 9.0), (8.0, 7.5), (10.0, 9.0), (8.0, 10.5), (6.0, 9.0)], Y);
        }
        "package" => folder(Y),
        "class" => {
            rect_at(2.0, 2.5, 12.0, 11.0, Y);
            line(&[(2.0, 6.0), (14.0, 6.0)], Y);
            line(&[(2.0, 9.5), (14.0, 9.5)], Y);
        }
        "interface" => {
            p.circle_stroke(at(11.0, 8.0), 3.3 * s, st(Y));
            line(&[(1.0, 8.0), (7.5, 8.0)], Y);
        }
        "enumeration" => {
            for y in [4.0, 8.0, 12.0] {
                line(&[(4.0, y), (14.0, y)], Y);
                line(&[(1.5, y), (2.5, y)], Y);
            }
        }
        "datatype" => {
            rect_at(2.0, 3.0, 12.0, 10.0, Y);
            line(&[(5.0, 6.5), (11.0, 6.5)], Y);
            line(&[(5.0, 9.5), (11.0, 9.5)], Y);
        }
        "attribute" => line(&[(8.0, 3.5), (12.5, 8.0), (8.0, 12.5), (3.5, 8.0), (8.0, 3.5)], Y),
        "operation" => {
            p.circle_stroke(at(8.0, 8.0), 3.0 * s, st(Y));
            for (a, b) in [((8.0, 1.8), (8.0, 4.2)), ((8.0, 11.8), (8.0, 14.2)), ((1.8, 8.0), (4.2, 8.0)), ((11.8, 8.0), (14.2, 8.0))] {
                line(&[a, b], Y);
            }
        }
        "parameter" => {
            line(&[(12.0, 4.0), (12.0, 9.0), (5.0, 9.0)], Y);
            line(&[(7.5, 6.5), (5.0, 9.0), (7.5, 11.5)], Y);
        }
        "usecase" => {
            p.add(Shape::ellipse_stroke(at(8.0, 8.0), Vec2::new(6.5 * s, 4.0 * s), st(Y)));
        }
        "actor" => {
            p.circle_stroke(at(8.0, 3.5), 2.0 * s, st(Y));
            line(&[(8.0, 5.5), (8.0, 10.5)], Y);
            line(&[(4.0, 7.5), (12.0, 7.5)], Y);
            line(&[(5.0, 14.5), (8.0, 10.5), (11.0, 14.5)], Y);
        }
        "subject" => rect_at(3.0, 2.0, 10.0, 12.0, W),
        "diagram-class" | "diagram-usecase" | "diagram" => {
            rect_at(1.5, 1.5, 13.0, 13.0, G);
            match key {
                "diagram-class" => {
                    rect_at(4.0, 4.0, 4.0, 3.5, G);
                    rect_at(8.5, 9.0, 4.0, 3.5, G);
                    line(&[(6.0, 7.5), (6.0, 10.5), (8.5, 10.5)], G);
                }
                "diagram-usecase" => {
                    p.circle_stroke(at(8.0, 5.0), 1.6 * s, st(G));
                    line(&[(8.0, 6.6), (8.0, 10.0)], G);
                    line(&[(5.5, 8.0), (10.5, 8.0)], G);
                    line(&[(6.0, 12.5), (8.0, 10.0), (10.0, 12.5)], G);
                }
                _ => line(&[(4.0, 11.0), (7.0, 7.0), (9.0, 9.0), (12.0, 5.0)], G),
            }
        }
        "association" => line(&[(3.0, 2.0), (3.0, 13.0), (14.0, 13.0)], B),
        "directed" => {
            line(&[(3.0, 3.0), (3.0, 13.0), (13.0, 13.0)], B);
            line(&[(10.5, 10.5), (13.0, 13.0), (10.5, 15.5)], B);
        }
        "aggregation" | "composition" => {
            line(&[(5.0, 13.0), (5.0, 6.0)], B);
            line(&[(5.0, 13.0), (14.0, 13.0)], B);
            let d: Vec<Pos2> = [(5.0, 6.0), (3.0, 4.0), (5.0, 2.0), (7.0, 4.0)].iter().map(|(x, y)| at(*x, *y)).collect();
            let fill = if key == "composition" { B } else { Color32::TRANSPARENT };
            p.add(Shape::convex_polygon(d, fill, st(B)));
        }
        "end" => {
            line(&[(4.0, 12.0), (12.0, 12.0)], B);
            line(&[(12.0, 9.0), (12.0, 15.0)], B);
        }
        "generalization" => {
            line(&[(8.0, 14.0), (8.0, 6.5)], B);
            triangle_up(B);
        }
        "dependency" => {
            dashed(&[(8.0, 14.0), (8.0, 3.0)], B);
            arrow_up(B);
        }
        "realization" => {
            dashed(&[(8.0, 14.0), (8.0, 6.5)], B);
            triangle_up(B);
        }
        "include" => {
            dashed(&[(3.0, 13.0), (12.0, 4.0)], B);
            line(&[(8.0, 3.5), (12.5, 3.5), (12.5, 8.0)], B);
        }
        "note" => {
            line(&[(2.5, 2.5), (10.5, 2.5), (13.5, 5.5), (13.5, 13.5), (2.5, 13.5), (2.5, 2.5)], W);
            line(&[(10.5, 2.5), (10.5, 5.5), (13.5, 5.5)], W);
        }
        "notelink" => dashed(&[(3.0, 13.0), (13.0, 3.0)], W),
        "textbox" => {
            rect_at(2.0, 3.0, 12.0, 10.0, W);
            line(&[(5.0, 5.5), (11.0, 5.5)], W);
            line(&[(8.0, 5.5), (8.0, 11.0)], W);
        }
        "text" => {
            line(&[(3.0, 3.5), (13.0, 3.5)], W);
            line(&[(8.0, 3.5), (8.0, 13.5)], W);
        }
        "component" => {
            rect_at(4.0, 2.5, 10.0, 11.0, Y);
            rect_at(2.0, 4.5, 4.0, 2.5, Y);
            rect_at(2.0, 9.0, 4.0, 2.5, Y);
        }
        "artifact" => {
            line(&[(3.5, 2.0), (10.0, 2.0), (13.0, 5.0), (13.0, 14.0), (3.5, 14.0), (3.5, 2.0)], Y);
            line(&[(10.0, 2.0), (10.0, 5.0), (13.0, 5.0)], Y);
        }
        "node" => {
            rect_at(2.0, 5.0, 10.0, 9.0, Y);
            line(&[(2.0, 5.0), (5.0, 2.0), (15.0, 2.0), (15.0, 11.0), (12.0, 14.0)], Y);
            line(&[(12.0, 5.0), (15.0, 2.0)], Y);
        }
        "boundary" => {
            p.circle_stroke(at(9.5, 8.0), 4.5 * s, st(W));
            line(&[(2.5, 4.0), (2.5, 12.0)], W);
            line(&[(2.5, 8.0), (5.0, 8.0)], W);
        }
        "entity" => {
            p.circle_stroke(at(8.0, 7.5), 4.5 * s, st(W));
            line(&[(3.5, 12.5), (12.5, 12.5)], W);
        }
        "control" => {
            p.circle_stroke(at(8.0, 9.0), 4.5 * s, st(W));
            line(&[(9.5, 3.0), (7.5, 4.5), (9.5, 6.0)], W);
        }
        "select" => line(&[(4.0, 2.0), (12.0, 9.0), (8.5, 9.0), (10.5, 13.5), (9.0, 14.2), (7.0, 9.7), (4.0, 12.0), (4.0, 2.0)], W),
        "zoomIn" | "zoomOut" => {
            p.circle_stroke(at(7.0, 7.0), 4.5 * s, st(W));
            line(&[(10.3, 10.3), (14.3, 14.3)], W);
            line(&[(5.0, 7.0), (9.0, 7.0)], W);
            if key == "zoomIn" {
                line(&[(7.0, 5.0), (7.0, 9.0)], W);
            }
        }
        "fit" => {
            line(&[(2.0, 6.0), (2.0, 2.0), (6.0, 2.0)], W);
            line(&[(10.0, 2.0), (14.0, 2.0), (14.0, 6.0)], W);
            line(&[(14.0, 10.0), (14.0, 14.0), (10.0, 14.0)], W);
            line(&[(6.0, 14.0), (2.0, 14.0), (2.0, 10.0)], W);
        }
        "grid" => {
            for v in [5.5, 10.5] {
                line(&[(2.0, v), (14.0, v)], W);
                line(&[(v, 2.0), (v, 14.0)], W);
            }
        }
        "image" => {
            rect_at(2.0, 3.0, 12.0, 10.0, W);
            line(&[(2.0, 11.0), (6.0, 7.0), (9.0, 10.0), (11.0, 8.0), (14.0, 11.0)], W);
        }
        _ => rect_at(3.0, 3.0, 10.0, 10.0, W),
    }
}
