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
        "UMLObject" => "object",
        "UMLArtifactInstance" => "artifact",
        "UMLComponentInstance" => "component",
        "UMLNodeInstance" => "node",
        "UMLSlot" => "attribute",
        "UMLLink" => "association",
        "UMLObjectDiagram" | "UMLInformationFlowDiagram" => "diagram-class",
        "UMLInformationItem" | "UMLStereotype" | "UMLMetaClass" => "class",
        "UMLExtension" => "generalization",
        "UMLProfileDiagram" => "diagram-class",
        "UMLInformationFlow" => "dependency",
        "UMLComponentRealization" => "realization",
        "Hyperlink" => "hyperlink",
        "UMLNaryAssociationNode" => "nary",
        "UMLCollaboration" | "UMLCollaborationUse" => "collaboration",
        "UMLState" => "state",
        "UMLFinalState" => "final",
        "UMLPseudostate" => "initial",
        "UMLConnectionPointReference" => "connection-point",
        "UMLLifeline" => "lifeline",
        "UMLMessage" => "message",
        "UMLEndpoint" => "endpoint",
        "UMLGate" => "gate",
        "UMLSequenceDiagram" => "diagram",
        // a flowchart element's icon is its type (the shapes are drawn by key)
        t if t.starts_with("FC") || t.starts_with("DFD") || t.starts_with("MM") || t.starts_with("ERD") || t.starts_with("C4") || t.starts_with("WF") => crate::meta::static_name(t),
        "UMLTransition" => "dependency",
        // activities: the action by its kind, nodes by their marks (elements.js:3176-3210)
        "UMLAction" => match r.str(e, "kind").as_str() {
            "sendSignal" | "triggerEvent" => "send-signal",
            "acceptSignal" | "acceptEvent" => "accept-signal",
            "timeEvent" => "time-event",
            _ => "action",
        },
        "UMLObjectNode" | "UMLCentralBufferNode" | "UMLDataStoreNode" => "object-node",
        "UMLActivityParameterNode" => "param-node",
        "UMLStructuredActivityNode" => "structured",
        "UMLExpansionRegion" => "expansion-region",
        "UMLInterruptibleActivityRegion" => "interruptible",
        "UMLActivityEdgeConnector" => "edge-connector",
        "UMLExpansionNode" => "expansion-node",
        "UMLExceptionHandler" | "UMLActivityInterrupt" => "interrupt",
        "UMLInitialNode" => "initial",
        "UMLActivityFinalNode" => "final",
        "UMLFlowFinalNode" => "flow-final",
        "UMLForkNode" | "UMLJoinNode" => "fork",
        "UMLMergeNode" | "UMLDecisionNode" => "choice",
        "UMLControlFlow" | "UMLObjectFlow" => "dependency",
        "UMLInputPin" | "UMLOutputPin" => "pin",
        "UMLActivityPartition" => "swimlane-vert",
        "UMLDeployment" | "UMLTemplateBinding" => "dependency",
        "UMLCommunicationPath" => "association",
        "UMLClassDiagram" | "UMLPackageDiagram" | "UMLCompositeStructureDiagram" => "diagram-class",
        "UMLUseCaseDiagram" => "diagram-usecase",
        "UMLAssociation" => "association",
        "UMLAssociationEnd" => "end",
        "UMLGeneralization" => "generalization",
        "UMLDependency" | "UMLInclude" | "UMLExtend" => "dependency",
        "UMLInterfaceRealization" | "UMLRealization" => "realization",
        "Tag" => "tag",
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
        "object" => {
            rect_at(2.0, 3.5, 12.0, 9.0, Y);
            line(&[(4.5, 7.0), (11.5, 7.0)], Y);
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
        "frame" => {
            rect_at(1.5, 2.5, 13.0, 11.0, W);
            line(&[(1.5, 6.5), (6.0, 6.5), (8.0, 4.5), (8.0, 2.5)], W);
        }
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
        "associationclass" => {
            line(&[(2.0, 4.0), (14.0, 4.0)], B);
            dashed(&[(8.0, 4.0), (8.0, 9.0)], B);
            rect_at(4.5, 9.5, 7.0, 5.0, B);
        }
        "tag" => {
            line(&[(2.0, 7.5), (7.5, 2.0), (14.0, 2.0), (14.0, 8.5), (8.5, 14.0), (2.0, 7.5)], W);
            p.circle_stroke(at(11.0, 5.0), 1.2 * s, st(W));
        }
        "freeline" => line(&[(2.0, 13.0), (6.0, 5.0), (10.0, 11.0), (14.0, 3.0)], W),
        "containment" => {
            line(&[(8.0, 14.0), (8.0, 7.0)], B);
            p.circle_stroke(at(8.0, 4.5), 2.5 * s, st(B));
            line(&[(6.5, 4.5), (9.5, 4.5)], B);
            line(&[(8.0, 3.0), (8.0, 6.0)], B);
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
        "rect" => rect_at(2.0, 3.0, 12.0, 10.0, W),
        "picture" => {
            rect_at(1.5, 2.5, 13.0, 11.0, W);
            line(&[(3.0, 12.0), (6.5, 7.5), (9.0, 10.0), (11.0, 8.0), (13.0, 12.0)], W);
        }
        "port" => {
            rect_at(2.0, 2.0, 10.0, 12.0, Y);
            rect_at(9.5, 6.0, 4.0, 4.0, Y);
        }
        "state" => {
            p.rect_stroke(Rect::from_min_size(at(1.5, 3.5), Vec2::new(13.0 * s, 9.0 * s)), (3.5 * s) as f32, st(Y), egui::StrokeKind::Middle);
        }
        "initial" => {
            p.circle_filled(at(8.0, 8.0), (4.5 * s) as f32, Y);
        }
        "final" => {
            p.circle_stroke(at(8.0, 8.0), (6.0 * s) as f32, st(Y));
            p.circle_filled(at(8.0, 8.0), (3.5 * s) as f32, Y);
        }
        "choice" => line(&[(8.0, 2.0), (14.0, 8.0), (8.0, 14.0), (2.0, 8.0), (8.0, 2.0)], Y),
        "fork" => {
            p.rect_filled(Rect::from_min_size(at(6.5, 1.5), Vec2::new(3.0 * s, 13.0 * s)), 0.0, Y);
        }
        "history" => {
            p.circle_stroke(at(8.0, 8.0), (6.0 * s) as f32, st(Y));
            line(&[(5.5, 5.0), (5.5, 11.0)], Y);
            line(&[(10.5, 5.0), (10.5, 11.0)], Y);
            line(&[(5.5, 8.0), (10.5, 8.0)], Y);
        }
        "action" => {
            p.rect_stroke(Rect::from_min_size(at(1.5, 3.5), Vec2::new(13.0 * s, 9.0 * s)), (3.0 * s) as f32, st(Y), egui::StrokeKind::Middle);
        }
        "object-node" => rect_at(1.5, 4.5, 13.0, 7.0, Y),
        "swimlane-vert" => {
            rect_at(1.5, 1.5, 13.0, 13.0, Y);
            line(&[(1.5, 5.0), (14.5, 5.0)], Y);
        }
        "swimlane-horz" => {
            rect_at(1.5, 1.5, 13.0, 13.0, Y);
            line(&[(5.0, 1.5), (5.0, 14.5)], Y);
        }
        "param-node" => {
            rect_at(1.5, 1.5, 13.0, 13.0, Y);
            rect_at(10.0, 5.0, 4.5, 6.0, Y);
        }
        "structured" | "interruptible" => {
            let pts = [(1.5, 3.0), (14.5, 3.0), (14.5, 13.0), (1.5, 13.0), (1.5, 3.0)];
            dashed(&pts, Y);
        }
        "expansion-region" => {
            dashed(&[(3.0, 3.0), (14.5, 3.0), (14.5, 14.5), (3.0, 14.5), (3.0, 3.0)], Y);
            rect_at(1.0, 5.0, 4.0, 8.0, Y);
            line(&[(1.0, 9.0), (5.0, 9.0)], Y);
        }
        "expansion-node" => {
            rect_at(1.5, 5.5, 13.0, 5.0, Y);
            for x in [4.75, 8.0, 11.25] {
                line(&[(x, 5.5), (x, 10.5)], Y);
            }
        }
        // sequence diagram tools
        "lifeline" => {
            rect_at(3.0, 1.5, 10.0, 4.5, Y);
            dashed(&[(8.0, 6.0), (8.0, 15.0)], Y);
        }
        "message" | "async-message" | "reply-message" => {
            let l = [(1.5, 8.0), (14.0, 8.0)];
            if key == "reply-message" { dashed(&l, B) } else { line(&l, B) }
            if key == "message" {
                p.add(Shape::convex_polygon(vec![at(14.5, 8.0), at(10.5, 5.5), at(10.5, 10.5)], B, st(B)));
            } else {
                line(&[(11.0, 5.5), (14.5, 8.0), (11.0, 10.5)], B);
            }
        }
        "self-message" => {
            line(&[(4.0, 4.0), (12.0, 4.0), (12.0, 11.0), (4.0, 11.0)], B);
            line(&[(7.0, 8.5), (4.0, 11.0), (7.0, 13.5)], B);
        }
        // communication diagram tools: a connector with a short arrow beside it
        "timing-lifeline" => {
            line(&[(1.0, 14.0), (15.0, 14.0)], Y);
            line(&[(4.0, 3.0), (4.0, 11.0)], Y);
            line(&[(6.0, 5.0), (10.0, 5.0), (10.0, 10.0), (15.0, 10.0)], B);
        }
        "state-condition" => {
            line(&[(1.0, 5.0), (6.0, 5.0)], Y);
            line(&[(1.0, 11.0), (6.0, 11.0)], Y);
            dashed(&[(8.0, 5.0), (15.0, 5.0)], B);
            dashed(&[(8.0, 11.0), (15.0, 11.0)], B);
        }
        "time-segment" => line(&[(1.0, 11.0), (7.0, 11.0), (7.0, 5.0), (15.0, 5.0)], B),
        "time-tick" => {
            line(&[(1.0, 6.0), (15.0, 6.0)], Y);
            for x in [3.0, 8.0, 13.0] {
                line(&[(x, 6.0), (x, 10.0)], Y);
            }
        }
        "time-constraint" => {
            line(&[(1.0, 13.0), (15.0, 13.0)], B);
            dashed(&[(8.0, 13.0), (8.0, 7.0)], B);
            line(&[(2.5, 3.0), (1.5, 4.5), (2.5, 6.0)], Y);
            line(&[(13.5, 3.0), (14.5, 4.5), (13.5, 6.0)], Y);
            line(&[(5.0, 4.5), (11.0, 4.5)], Y);
        }
        "duration-constraint" => {
            line(&[(4.0, 1.5), (12.0, 1.5)], B);
            line(&[(4.0, 14.5), (12.0, 14.5)], B);
            line(&[(8.0, 1.5), (8.0, 14.5)], B);
            line(&[(6.0, 4.5), (8.0, 1.5), (10.0, 4.5)], B);
            line(&[(6.0, 11.5), (8.0, 14.5), (10.0, 11.5)], B);
        }
        "self-connector" => {
            rect_at(1.5, 6.0, 7.0, 6.0, Y);
            line(&[(8.5, 7.5), (13.5, 7.5), (13.5, 2.5), (5.0, 2.5), (5.0, 6.0)], B);
        }
        "forward-message" | "reverse-message" => {
            line(&[(1.0, 12.0), (15.0, 12.0)], B);
            line(&[(4.0, 7.0), (12.0, 7.0)], B);
            if key == "forward-message" {
                p.add(Shape::convex_polygon(vec![at(13.0, 7.0), at(10.0, 5.0), at(10.0, 9.0)], B, st(B)));
            } else {
                p.add(Shape::convex_polygon(vec![at(3.0, 7.0), at(6.0, 5.0), at(6.0, 9.0)], B, st(B)));
            }
        }
        "found-message" | "lost-message" => {
            line(&[(4.0, 8.0), (12.0, 8.0)], B);
            line(&[(9.5, 5.5), (12.5, 8.0), (9.5, 10.5)], B);
            let x = if key == "found-message" { 3.0 } else { 14.0 };
            p.circle_filled(at(x, 8.0), 2.0 * s, B);
        }
        "endpoint" => {
            p.circle_filled(at(8.0, 8.0), 4.0 * s, Y);
        }
        "gate" => rect_at(5.0, 5.0, 6.0, 6.0, Y),
        // C4 elements
        "C4Model" => {
            rect_at(1.5, 1.5, 13.0, 13.0, Y);
            p.rect_stroke(Rect::from_min_size(at(4.0, 4.0), Vec2::new(8.0 * s, 8.0 * s)), (1.5 * s) as f32, st(Y), egui::StrokeKind::Middle);
        }
        "C4Diagram" => {
            rect_at(1.5, 1.5, 13.0, 13.0, G);
            p.rect_stroke(Rect::from_min_size(at(4.0, 5.0), Vec2::new(8.0 * s, 6.0 * s)), (1.5 * s) as f32, st(G), egui::StrokeKind::Middle);
        }
        "C4Person" => {
            p.circle_stroke(at(8.0, 4.0), 2.5 * s, st(Y));
            p.rect_stroke(Rect::from_min_size(at(2.5, 7.5), Vec2::new(11.0 * s, 7.0 * s)), (2.0 * s) as f32, st(Y), egui::StrokeKind::Middle);
        }
        "C4SoftwareSystem" | "C4Container" | "C4Component" | "C4Element" => {
            p.rect_stroke(Rect::from_min_size(at(1.5, 3.5), Vec2::new(13.0 * s, 9.0 * s)), (2.0 * s) as f32, st(Y), egui::StrokeKind::Middle);
            match key {
                "C4Container" => line(&[(1.5, 6.5), (14.5, 6.5)], Y),
                "C4Component" => {
                    line(&[(4.0, 6.5), (12.0, 6.5)], Y);
                    line(&[(4.0, 9.5), (12.0, 9.5)], Y);
                }
                _ => {}
            }
        }
        "C4ContainerDatabase" => {
            p.add(Shape::ellipse_stroke(at(8.0, 4.0), Vec2::new(6.0 * s, 2.0 * s), st(Y)));
            line(&[(2.0, 4.0), (2.0, 12.0)], Y);
            line(&[(14.0, 4.0), (14.0, 12.0)], Y);
            line(&[(2.0, 12.0), (4.0, 13.5), (8.0, 14.0), (12.0, 13.5), (14.0, 12.0)], Y);
        }
        "C4Relationship" => dashed(&[(2.0, 13.0), (13.0, 3.0)], B),
        // ER elements
        "ERDDataModel" => {
            p.add(Shape::ellipse_stroke(at(8.0, 4.0), Vec2::new(6.0 * s, 2.0 * s), st(Y)));
            line(&[(2.0, 4.0), (2.0, 12.0)], Y);
            line(&[(14.0, 4.0), (14.0, 12.0)], Y);
            line(&[(2.0, 12.0), (4.0, 13.5), (8.0, 14.0), (12.0, 13.5), (14.0, 12.0)], Y);
        }
        "ERDDiagram" => {
            rect_at(1.5, 1.5, 13.0, 13.0, G);
            rect_at(3.5, 4.0, 4.0, 4.0, G);
            rect_at(8.5, 8.0, 4.0, 4.0, G);
            line(&[(7.5, 6.0), (10.5, 6.0), (10.5, 8.0)], G);
        }
        "ERDEntity" => {
            rect_at(1.5, 2.5, 13.0, 11.0, Y);
            line(&[(1.5, 6.0), (14.5, 6.0)], Y);
            line(&[(5.0, 6.0), (5.0, 13.5)], Y);
        }
        "ERDColumn" => {
            rect_at(1.5, 5.5, 13.0, 5.0, Y);
            line(&[(5.0, 5.5), (5.0, 10.5)], Y);
        }
        "ERDRelationship" | "ERDRelationshipOneToMany" | "ERDRelationshipManyToMany" => {
            line(&[(2.0, 8.0), (14.0, 8.0)], B);
            line(&[(4.0, 5.5), (4.0, 10.5)], B);
            if key == "ERDRelationship" {
                line(&[(12.0, 5.5), (12.0, 10.5)], B);
            } else {
                line(&[(10.0, 8.0), (14.0, 5.0)], B);
                line(&[(10.0, 8.0), (14.0, 11.0)], B);
            }
            if key == "ERDRelationshipManyToMany" {
                line(&[(6.0, 8.0), (2.0, 5.0)], B);
                line(&[(6.0, 8.0), (2.0, 11.0)], B);
            }
        }
        // data flow and mind map elements
        "DFDDataFlowModel" | "MMMindmap" => {
            rect_at(5.0, 1.5, 6.0, 4.0, Y);
            line(&[(8.0, 5.5), (8.0, 8.0), (3.0, 8.0), (3.0, 10.5)], Y);
            line(&[(8.0, 8.0), (13.0, 8.0), (13.0, 10.5)], Y);
            rect_at(1.0, 10.5, 4.5, 4.0, Y);
            rect_at(10.5, 10.5, 4.5, 4.0, Y);
        }
        "DFDDiagram" | "MMMindmapDiagram" => {
            rect_at(1.5, 1.5, 13.0, 13.0, G);
            if key == "DFDDiagram" {
                rect_at(3.5, 5.5, 4.0, 4.0, G);
                line(&[(7.5, 7.5), (12.5, 7.5)], G);
                line(&[(10.5, 5.5), (12.5, 7.5), (10.5, 9.5)], G);
            } else {
                p.rect_stroke(Rect::from_min_size(at(5.5, 6.5), Vec2::new(5.0 * s, 3.0 * s)), (1.5 * s) as f32, st(G), egui::StrokeKind::Middle);
                line(&[(5.5, 8.0), (3.5, 4.5)], G);
                line(&[(10.5, 8.0), (12.5, 11.5)], G);
            }
        }
        "DFDExternalEntity" => rect_at(1.5, 3.5, 13.0, 9.0, Y),
        "DFDProcess" => {
            p.rect_stroke(Rect::from_min_size(at(1.5, 2.5), Vec2::new(13.0 * s, 11.0 * s)), (3.0 * s) as f32, st(Y), egui::StrokeKind::Middle);
            line(&[(1.5, 6.0), (14.5, 6.0)], Y);
        }
        "DFDDataStore" => {
            line(&[(14.5, 4.0), (1.5, 4.0), (1.5, 12.0), (14.5, 12.0)], Y);
            line(&[(5.0, 4.0), (5.0, 12.0)], Y);
        }
        "DFDDataFlow" => {
            line(&[(8.0, 14.0), (8.0, 4.0)], B);
            p.add(Shape::convex_polygon(vec![at(8.0, 2.0), at(10.5, 6.0), at(5.5, 6.0)], B, st(B)));
        }
        "MMNode" => {
            p.rect_stroke(Rect::from_min_size(at(1.5, 4.0), Vec2::new(13.0 * s, 8.0 * s)), (4.0 * s) as f32, st(Y), egui::StrokeKind::Middle);
        }
        "MMEdge" => line(&[(2.0, 13.0), (14.0, 3.0)], B),
        // wireframe elements: their outlines in small (wireframe/elements.js)
        "WFWireframe" => {
            rect_at(1.5, 2.5, 13.0, 11.0, Y);
            line(&[(1.5, 5.5), (14.5, 5.5)], Y);
            rect_at(3.5, 8.0, 5.0, 3.5, Y);
        }
        "WFWireframeDiagram" => {
            rect_at(1.5, 1.5, 13.0, 13.0, G);
            rect_at(4.0, 4.0, 8.0, 3.0, G);
            line(&[(4.0, 10.0), (12.0, 10.0)], G);
        }
        "WFFrame" => {
            p.rect_stroke(Rect::from_min_size(at(1.5, 1.5), Vec2::new(13.0 * s, 13.0 * s)), (3.0 * s) as f32, st(Y), egui::StrokeKind::Middle);
        }
        "WFMobileFrame" => {
            p.rect_stroke(Rect::from_min_size(at(4.0, 1.0), Vec2::new(8.0 * s, 14.0 * s)), (2.0 * s) as f32, st(Y), egui::StrokeKind::Middle);
            rect_at(5.0, 3.0, 6.0, 10.0, Y);
        }
        "WFWebFrame" | "WFDesktopFrame" => {
            rect_at(1.5, 2.5, 13.0, 11.0, Y);
            line(&[(1.5, 5.5), (14.5, 5.5)], Y);
            if key == "WFWebFrame" {
                p.circle_stroke(at(12.5, 4.0), 0.8 * s, st(Y));
            } else {
                line(&[(11.5, 3.2), (13.0, 4.8)], Y);
                line(&[(13.0, 3.2), (11.5, 4.8)], Y);
            }
        }
        "WFButton" | "WFInput" | "WFDropdown" => {
            p.rect_stroke(Rect::from_min_size(at(1.5, 4.5), Vec2::new(13.0 * s, 7.0 * s)), (2.0 * s) as f32, st(Y), egui::StrokeKind::Middle);
            match key {
                "WFButton" => line(&[(5.0, 8.0), (11.0, 8.0)], Y),
                "WFInput" => line(&[(4.0, 6.5), (4.0, 9.5)], Y),
                _ => line(&[(10.0, 7.0), (11.5, 9.0), (13.0, 7.0)], Y),
            }
        }
        "WFText" | "WFLink" => {
            let c = if key == "WFLink" { B } else { Y };
            line(&[(3.0, 3.0), (13.0, 3.0)], c);
            line(&[(8.0, 3.0), (8.0, 12.0)], c);
            if key == "WFLink" {
                line(&[(3.0, 14.0), (13.0, 14.0)], c);
            }
        }
        "WFRadio" => {
            p.circle_stroke(at(5.0, 8.0), 3.5 * s, st(Y));
            p.circle_filled(at(5.0, 8.0), 1.5 * s, Y);
            line(&[(10.0, 8.0), (15.0, 8.0)], Y);
        }
        "WFCheckbox" => {
            rect_at(1.5, 4.5, 7.0, 7.0, Y);
            line(&[(2.5, 7.5), (4.5, 10.0), (8.0, 4.0)], Y);
            line(&[(10.0, 8.0), (15.0, 8.0)], Y);
        }
        "WFSwitch" => {
            p.rect_stroke(Rect::from_min_size(at(1.5, 4.5), Vec2::new(13.0 * s, 7.0 * s)), (3.5 * s) as f32, st(Y), egui::StrokeKind::Middle);
            p.circle_filled(at(11.0, 8.0), 2.2 * s, Y);
        }
        "WFTabList" | "WFTab" => {
            line(&[(1.5, 12.5), (1.5, 4.5), (7.5, 4.5), (7.5, 12.5)], Y);
            if key == "WFTabList" {
                line(&[(7.5, 7.0), (13.5, 7.0), (13.5, 12.5)], Y);
                line(&[(0.5, 12.5), (15.5, 12.5)], Y);
            }
        }
        "WFPanel" => rect_at(1.5, 2.5, 13.0, 11.0, Y),
        "WFImage" => {
            rect_at(1.5, 2.5, 13.0, 11.0, Y);
            line(&[(1.5, 2.5), (14.5, 13.5)], Y);
            line(&[(14.5, 2.5), (1.5, 13.5)], Y);
        }
        "WFSeparator" => line(&[(1.0, 8.0), (15.0, 8.0)], Y),
        "WFAvatar" => {
            p.circle_stroke(at(8.0, 8.0), 6.5 * s, st(Y));
            p.circle_stroke(at(8.0, 6.0), 2.0 * s, st(Y));
            line(&[(4.5, 12.0), (5.5, 9.5), (10.5, 9.5), (11.5, 12.0)], Y);
        }
        "WFSlider" => {
            line(&[(1.0, 8.0), (15.0, 8.0)], Y);
            p.circle_filled(at(6.0, 8.0), 2.5 * s, Y);
        }
        // flowchart shapes in small (flowchart/elements.js)
        k if k.starts_with("FC") => {
            let c = Y;
            let circle = |r: f32| p.circle_stroke(at(8.0, 8.0), r * s, st(c));
            match k {
                "FCFlowchart" => {
                    rect_at(5.0, 1.5, 6.0, 4.0, c);
                    line(&[(8.0, 5.5), (8.0, 8.0), (3.0, 8.0), (3.0, 10.5)], c);
                    line(&[(8.0, 8.0), (13.0, 8.0), (13.0, 10.5)], c);
                    rect_at(1.0, 10.5, 4.5, 4.0, c);
                    rect_at(10.5, 10.5, 4.5, 4.0, c);
                }
                "FCFlowchartDiagram" => {
                    rect_at(1.5, 1.5, 13.0, 13.0, G);
                    rect_at(5.0, 4.0, 6.0, 3.0, G);
                    line(&[(8.0, 7.0), (8.0, 9.0)], G);
                    line(&[(8.0, 9.0), (10.5, 11.0), (8.0, 13.0), (5.5, 11.0), (8.0, 9.0)], G);
                }
                "FCFlow" => {
                    line(&[(8.0, 14.0), (8.0, 4.0)], B);
                    p.add(Shape::convex_polygon(vec![at(8.0, 2.0), at(10.5, 6.0), at(5.5, 6.0)], B, st(B)));
                }
                "FCTerminator" => {
                    p.rect_stroke(Rect::from_min_size(at(1.5, 4.5), Vec2::new(13.0 * s, 7.0 * s)), (3.5 * s) as f32, st(c), egui::StrokeKind::Middle);
                }
                "FCAlternateProcess" => {
                    p.rect_stroke(Rect::from_min_size(at(1.5, 3.5), Vec2::new(13.0 * s, 9.0 * s)), (2.5 * s) as f32, st(c), egui::StrokeKind::Middle);
                }
                "FCDecision" | "FCSort" => {
                    line(&[(1.5, 8.0), (8.0, 2.5), (14.5, 8.0), (8.0, 13.5), (1.5, 8.0)], c);
                    if k == "FCSort" {
                        line(&[(1.5, 8.0), (14.5, 8.0)], c);
                    }
                }
                "FCDelay" => line(&[(1.5, 3.5), (10.0, 3.5), (12.5, 4.5), (14.0, 6.5), (14.0, 9.5), (12.5, 11.5), (10.0, 12.5), (1.5, 12.5), (1.5, 3.5)], c),
                "FCPredefinedProcess" => {
                    rect_at(1.5, 3.5, 13.0, 9.0, c);
                    line(&[(3.5, 3.5), (3.5, 12.5)], c);
                    line(&[(12.5, 3.5), (12.5, 12.5)], c);
                }
                "FCData" => line(&[(4.0, 3.5), (14.5, 3.5), (12.0, 12.5), (1.5, 12.5), (4.0, 3.5)], c),
                "FCDocument" | "FCMultiDocument" => {
                    if k == "FCMultiDocument" {
                        line(&[(4.0, 3.0), (4.0, 1.5), (14.5, 1.5), (14.5, 10.0), (13.0, 10.0)], c);
                    }
                    line(&[(1.5, 3.5), (12.5, 3.5), (12.5, 12.0), (10.0, 11.0), (7.0, 12.5), (4.0, 14.0), (1.5, 13.0), (1.5, 3.5)], c);
                }
                "FCPreparation" => line(&[(1.5, 8.0), (4.0, 3.5), (12.0, 3.5), (14.5, 8.0), (12.0, 12.5), (4.0, 12.5), (1.5, 8.0)], c),
                "FCDisplay" => line(&[(1.5, 8.0), (4.0, 3.5), (11.5, 3.5), (13.8, 4.8), (14.5, 8.0), (13.8, 11.2), (11.5, 12.5), (4.0, 12.5), (1.5, 8.0)], c),
                "FCManualInput" => line(&[(1.5, 6.0), (14.5, 3.0), (14.5, 12.5), (1.5, 12.5), (1.5, 6.0)], c),
                "FCManualOperation" => line(&[(1.5, 3.5), (14.5, 3.5), (12.0, 12.5), (4.0, 12.5), (1.5, 3.5)], c),
                "FCCard" => line(&[(4.5, 3.5), (14.5, 3.5), (14.5, 12.5), (1.5, 12.5), (1.5, 6.5), (4.5, 3.5)], c),
                "FCPunchedTape" => line(&[(1.5, 4.0), (5.0, 5.5), (10.0, 2.5), (14.5, 4.0), (14.5, 12.0), (11.0, 10.5), (6.0, 13.5), (1.5, 12.0), (1.5, 4.0)], c),
                "FCConnector" => {
                    circle(5.5);
                }
                "FCOffPageConnector" => line(&[(3.0, 2.0), (13.0, 2.0), (13.0, 10.0), (8.0, 14.0), (3.0, 10.0), (3.0, 2.0)], c),
                "FCOr" | "FCSummingJunction" => {
                    circle(6.0);
                    if k == "FCOr" {
                        line(&[(8.0, 2.0), (8.0, 14.0)], c);
                        line(&[(2.0, 8.0), (14.0, 8.0)], c);
                    } else {
                        line(&[(4.0, 4.0), (12.0, 12.0)], c);
                        line(&[(12.0, 4.0), (4.0, 12.0)], c);
                    }
                }
                "FCCollate" => line(&[(3.0, 2.0), (13.0, 2.0), (3.0, 14.0), (13.0, 14.0), (3.0, 2.0)], c),
                "FCMerge" => line(&[(2.0, 3.0), (14.0, 3.0), (8.0, 13.0), (2.0, 3.0)], c),
                "FCExtract" => line(&[(8.0, 3.0), (14.0, 13.0), (2.0, 13.0), (8.0, 3.0)], c),
                "FCStoredData" => line(&[(14.5, 3.5), (13.0, 5.0), (12.5, 8.0), (13.0, 11.0), (14.5, 12.5), (4.0, 12.5), (2.2, 11.0), (1.5, 8.0), (2.2, 5.0), (4.0, 3.5), (14.5, 3.5)], c),
                "FCDatabase" => {
                    p.add(Shape::ellipse_stroke(at(8.0, 4.0), Vec2::new(6.0 * s, 2.0 * s), st(c)));
                    line(&[(2.0, 4.0), (2.0, 12.0)], c);
                    line(&[(14.0, 4.0), (14.0, 12.0)], c);
                    line(&[(2.0, 12.0), (4.0, 13.5), (8.0, 14.0), (12.0, 13.5), (14.0, 12.0)], c);
                }
                "FCDirectAccessStorage" => {
                    line(&[(4.0, 3.5), (12.0, 3.5), (13.8, 5.0), (14.5, 8.0), (13.8, 11.0), (12.0, 12.5), (4.0, 12.5), (2.2, 11.0), (1.5, 8.0), (2.2, 5.0), (4.0, 3.5)], c);
                    line(&[(12.0, 3.5), (10.2, 5.0), (9.5, 8.0), (10.2, 11.0), (12.0, 12.5)], c);
                }
                "FCInternalStorage" => {
                    rect_at(1.5, 2.5, 13.0, 11.0, c);
                    line(&[(4.0, 2.5), (4.0, 13.5)], c);
                    line(&[(1.5, 5.0), (14.5, 5.0)], c);
                }
                _ => rect_at(1.5, 3.5, 13.0, 9.0, c),
            }
        }
        "connection-point" => {
            rect_at(1.5, 1.5, 10.0, 10.0, Y);
            p.circle_stroke(at(11.5, 11.5), (3.5 * s) as f32, st(Y));
        }
        "edge-connector" => {
            p.circle_stroke(at(8.0, 8.0), (6.0 * s) as f32, st(Y));
            line(&[(6.0, 11.0), (8.0, 5.0), (10.0, 11.0)], Y);
        }
        "interrupt" => line(&[(2.0, 3.0), (12.0, 3.0), (4.0, 10.0), (14.0, 10.0), (11.0, 13.0)], B),
        "pin" => {
            rect_at(1.5, 3.5, 9.0, 9.0, Y);
            rect_at(10.5, 6.0, 4.0, 4.0, Y);
        }
        "send-signal" => line(&[(1.5, 3.5), (11.0, 3.5), (14.5, 8.0), (11.0, 12.5), (1.5, 12.5), (1.5, 3.5)], Y),
        "accept-signal" => line(&[(1.5, 3.5), (14.5, 3.5), (14.5, 12.5), (1.5, 12.5), (5.0, 8.0), (1.5, 3.5)], Y),
        "time-event" => line(&[(3.0, 2.0), (13.0, 2.0), (3.0, 14.0), (13.0, 14.0), (3.0, 2.0)], Y),
        "flow-final" => {
            p.circle_stroke(at(8.0, 8.0), (6.0 * s) as f32, st(Y));
            line(&[(4.0, 4.0), (12.0, 12.0)], Y);
            line(&[(4.0, 12.0), (12.0, 4.0)], Y);
        }
        "terminate" => {
            line(&[(3.0, 3.0), (13.0, 13.0)], Y);
            line(&[(3.0, 13.0), (13.0, 3.0)], Y);
        }
        "part" => {
            rect_at(1.5, 1.5, 13.0, 13.0, Y);
            rect_at(4.0, 6.0, 8.0, 5.0, Y);
        }
        "collaboration" => {
            let pts: Vec<(f32, f32)> = (0..=24).map(|i| { let a = std::f32::consts::TAU * i as f32 / 24.0; (8.0 + 6.5 * a.cos(), 8.0 + 4.5 * a.sin()) }).collect();
            dashed(&pts, Y);
        }
        "nary" => line(&[(1.5, 8.0), (8.0, 3.5), (14.5, 8.0), (8.0, 12.5), (1.5, 8.0)], Y),
        "roundrect" => {
            p.rect_stroke(Rect::from_min_size(at(2.0, 3.0), Vec2::new(12.0 * s, 10.0 * s)), (3.0 * s) as f32, st(W), egui::StrokeKind::Middle);
        }
        "ellipse" => {
            p.add(Shape::ellipse_stroke(at(8.0, 8.0), Vec2::new(6.5 * s, 5.0 * s), st(W)));
        }
        "hyperlink" => {
            // two chain links
            rect_at(1.5, 5.5, 7.0, 5.0, W);
            rect_at(7.5, 5.5, 7.0, 5.0, W);
        }
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
        // the Markdown mark: an M and a down arrow in a frame
        "markdown" => {
            rect_at(1.0, 3.5, 14.0, 9.0, W);
            line(&[(3.5, 10.0), (3.5, 6.0), (5.5, 8.0), (7.5, 6.0), (7.5, 10.0)], W);
            line(&[(11.0, 6.0), (11.0, 10.0)], W);
            line(&[(9.5, 8.5), (11.0, 10.0), (12.5, 8.5)], W);
        }
        _ => rect_at(3.0, 3.0, 10.0, 10.0, W),
    }
}
