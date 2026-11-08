//! The components around an inline edit's field, and where each goes: the "left" section stacks
//! leftwards from the field and the "right" section rightwards, each in rows of 25 px
//! (src/ui/quickedit-manager.js:274-311; extensions/essential/*/quickedits/*.json). A component is
//! shown only when the edited element allows it, as the templates' `visibility` keys do
//! (quickedit-manager.js:279-283).

use crate::app::{App, EditKind, Editing};
use crate::canvas::{item_buttons, ItemKey};
use crate::model::Eid;
use crate::related::Related;
use crate::views::{self, is_edge};

/// The height of a row of components (BUTTON_HEIGHT, quickedit-manager.js:60).
pub const ROW_H: f32 = 25.0;

/// Which side of the field a component sits on.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Side {
    Left,
    Right,
}

/// What a button does. All but Add Qualifier close the edit first.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Act {
    /// An item key of the edit, run on this view (the edited box, or the edited line)
    Key(Eid, ItemKey),
    /// An "add related element" button of a box
    Related(Related),
    /// A line's Add Note (a BPMN flow's Add Text Annotation), Add Constraint
    EdgeNote(Eid),
    EdgeConstraint(Eid),
    /// A communication connector's Add Forward (true) or Reverse Message
    CommMessage(Eid, bool),
    /// A SysML connector's Create Item Flow
    ItemFlow(Eid),
    /// A template binding's Add Template Parameter Substitution
    Subst(Eid),
    /// An association end's Add Qualifier: the edit stays open
    Qualifier(Eid),
}

/// What a dropdown item sets. It applies at once and the edit stays open.
#[derive(Clone, PartialEq, Debug)]
pub enum Pick {
    Visibility(Eid, &'static str),
    Set(Eid, &'static str, serde_json::Value),
    Owner(Eid, Option<Eid>),
}

#[derive(Clone, PartialEq, Debug)]
pub enum Comp {
    Button { label: String, hint: String, act: Act },
    /// A dropdown: the shown value, its hint, and its items (text, current, what it sets)
    Menu { shown: String, hint: String, items: Vec<(String, bool, Pick)> },
    /// An editable multiplicity with its choices (quickedits/uml.json "uml.multiplicity", a combo)
    Combo(Eid),
}

#[derive(Clone, PartialEq, Debug)]
pub struct Placed {
    pub side: Side,
    /// 1 to 5
    pub row: usize,
    pub comp: Comp,
}

/// An association, link or connector end.
pub fn is_end(ty: &str) -> bool {
    matches!(ty, "UMLAssociationEnd" | "UMLLinkEnd" | "UMLConnectorEnd")
}

/// An association end that shows its owner attribute instead of itself: its name, qualifiers,
/// aggregation, visibility and multiplicity are hidden in its quick edit (`_tailEndSole`,
/// `_headEndSole`, elements.js:5631-5651; quickedits/uml.json "uml.association-end1/2").
pub fn end_owned(r: &crate::model::Repo, end: Eid) -> bool {
    r.ty(end) == "UMLAssociationEnd" && r.refd(end, "ownerAttribute").map(|a| r.alive(a) && r.ty(a) == "UMLAttribute").unwrap_or(false)
}

/// The elements whose quick edit has no visibility dropdown (quickedits/uml.json: "uml.message",
/// "uml.state", "uml.action", "uml.transition", "uml.activity-edge", "uml.combined-fragment",
/// "uml.interaction-operand", "uml.continuation", "uml.partition", "uml.state-invariant").
fn no_visibility(ty: &str) -> bool {
    matches!(
        ty,
        "UMLMessage" | "UMLState" | "UMLAction" | "UMLTransition" | "UMLControlFlow" | "UMLObjectFlow" | "UMLCombinedFragment" | "UMLInteractionOperand" | "UMLContinuation" | "UMLActivityPartition" | "UMLStateInvariant"
    )
}

/// The side and row of a button, by the edited element and the button's hint, as the quick-edit
/// templates place it; a button the templates do not list goes on the first row of the right.
pub fn place(model_ty: &str, view_ty: &str, hint: &str) -> (Side, usize) {
    use Side::*;
    let uml = model_ty.starts_with("UML") || model_ty.starts_with("SysML");
    match hint {
        // a line of an activity or a transition has Add Note on the first row
        "Add Note" if matches!(model_ty, "UMLTransition" | "UMLControlFlow" | "UMLObjectFlow") => return (Left, 1),
        "Add Note" if model_ty == "ERDRelationship" => return (Left, 2),
        "Add Note" | "Add Constraint" => return (Left, if uml { 2 } else { 1 }),
        "Add Text Annotation" | "Add Node Left" => return (Left, 1),
        "Add Output Pin" | "Add Input Pin" | "Add Region" | "Add Connection Point Reference" => return (Left, 1),
        _ => {}
    }
    let row = match (model_ty, hint) {
        ("UMLClass", "Add Sub-Class" | "Add Super-Class" | "Add Provided Interface" | "Add Required Interface") => 2,
        ("UMLClass", "Add Associated Class" | "Add Aggregated Class" | "Add Composited Class") => 3,
        ("UMLClass", "Add Port" | "Add Part") => 4,
        ("UMLInterface", "Add Sub-Interface" | "Add Super-Interface" | "Add Realizing Class") => 2,
        ("UMLPackage" | "UMLModel", "Add Dependant Package" | "Add Depending Package") => 2,
        ("UMLComponent", "Add Provided Interface" | "Add Required Interface") => 2,
        ("UMLComponent", "Add Port" | "Add Part") => 3,
        ("UMLNode", "Add Communicating Node") => 2,
        ("UMLNode", "Add Deployed Artifact" | "Add Deployed Component") => 3,
        ("UMLUseCase", "Add Associated Actor" | "Add Associated Use Case" | "Add Included Use Case" | "Add Extended Use Case") => 2,
        ("UMLActor", "Add Sub-Actor" | "Add Super-Actor") => 2,
        ("UMLActor", "Add Associated Use Case") => 3,
        // UMLCraft's own Sub-Type and Super-Type of data types, signals and enumerations
        (_, "Add Sub-Type" | "Add Super-Type") => 2,
        ("UMLPort" | "UMLAttribute", "Add Provided Interface" | "Add Required Interface" | "Add Connected Part") => 2,
        ("UMLObject", "Add Linked Object") => 2,
        ("UMLStereotype", "Add Sub-Stereotype" | "Add Super-Stereotype") => 2,
        ("UMLLifeline", "Add Message with Lifeline" | "Add Create Message with Lifeline" | "Add Self Message") => 2,
        ("UMLLifeline", "Add Found Message" | "Add Lost Message") => 3,
        ("UMLLifeline", "Add Message from Gate" | "Add Message to Gate") => 4,
        ("UMLLifeline", "Add Connected Lifeline" | "Add Self Connector") if view_ty == "UMLCommLifelineView" => 2,
        ("UMLLifeline", "Add Forward Message" | "Add Reverse Message") if view_ty == "UMLCommLifelineView" => 3,
        ("UMLMessage", "Add Reply Message") => 2,
        ("UMLState", "Add Outgoing Transition" | "Add Incoming Transition") => 2,
        ("UMLState", "Add Initial State" | "Add Final State" | "Add Choice") => 3,
        ("UMLState", "Add Fork" | "Add Join") => 4,
        ("UMLAction", "Add Outgoing Control Flow" | "Add Incoming Control Flow" | "Add Outgoing Object Flow" | "Add Incoming Object Flow") => 2,
        ("UMLAction", "Add Decision" | "Add Merge" | "Add Fork" | "Add Join") => 3,
        ("UMLAction", "Add Initial Node" | "Add Final Node") => 4,
        ("UMLObjectNode" | "UMLCentralBufferNode" | "UMLDataStoreNode" | "UMLActivityParameterNode", "Add Outgoing Control Flow" | "Add Incoming Control Flow") => 2,
        ("ERDEntity", "Add One to One" | "Add One to Many" | "Add Many to Many") => 2,
        (_, "Add Outgoing Decision" | "Add Incoming Decision") => 2,
        // BPMN events: five definitions a row; a choreography's lower participant and return
        // message, then its initiating participant (quickedits/bpmn.json)
        (t, h) if t.starts_with("BPMN") && t.ends_with("Event") && ["Timer", "Escalation", "Message", "Terminate", "Conditional"].iter().any(|k| h.contains(k)) => 2,
        (_, "Assign Participant (Lower)" | "Create Participant (Lower)" | "Create Return Message") => 2,
        (_, "Initiating Participant") => 3,
        // SysML blocks and parts (quickedits/sysml.json "sysml.block", "sysml.part", "sysml.port")
        ("SysMLBlock" | "SysMLInterfaceBlock", "Add Constraint Property" | "Add Property" | "Add Operation" | "Add Reception") => 2,
        ("SysMLBlock" | "SysMLInterfaceBlock", "Add Port" | "Add Composited Block") => 3,
        (_, "Add Port") if view_ty == "SysMLPartView" => 2,
        (_, "Add Provided Interface" | "Add Required Interface") if view_ty == "SysMLPortView" => 2,
        _ => 1,
    };
    (Right, row)
}

impl App {
    /// The components of an inline edit around its field, each with its side and row.
    pub fn quick_components(&self, ed: &Editing) -> Vec<Placed> {
        let r = &self.repo;
        let mut out: Vec<Placed> = vec![];
        let m = match ed.kind {
            EditKind::Name(m) | EditKind::Feature(m) => m,
            EditKind::Text(_) => return out,
        };
        if !r.alive(m) || ed.multiline {
            return out;
        }
        let ty = r.ty(m).to_string();
        let view_ty = ed.host.map(|h| r.ty(h).to_string()).unwrap_or_default();
        let owned = end_owned(r, m);
        let menu = |shown: String, hint: &str, items: Vec<(String, bool, Pick)>| Comp::Menu { shown, hint: hint.to_string(), items };
        let button = |label: &str, hint: &str, act: Act| Comp::Button { label: label.to_string(), hint: hint.to_string(), act };
        let mut left1: Vec<Comp> = vec![];

        // an end: aggregation, navigability and visibility; then its qualifier and owner attribute;
        // its multiplicity on the right (quickedits/uml.json "uml.association-end1/2", "uml.link-end1/2",
        // "uml.connector-end1/2")
        if let (EditKind::Name(_), true) = (&ed.kind, is_end(&ty)) {
            let field_menu = |field: &'static str, hint: &str, options: &[&str]| {
                let cur = r.str(m, field);
                let shown = if cur.is_empty() { "—".to_string() } else { cur.clone() };
                menu(shown, hint, options.iter().map(|o| (o.to_string(), cur == *o, Pick::Set(m, field, serde_json::json!(o)))).collect())
            };
            if !owned {
                left1.push(field_menu("aggregation", "Aggregation Kind", &["none", "shared", "composite"]));
            }
            left1.push(field_menu("navigable", "Navigability", &["unspecified", "navigable", "notNavigable"]));
            if ty == "UMLAssociationEnd" {
                if !owned {
                    out.push(Placed { side: Side::Left, row: 2, comp: button("+ Qual", "Add Qualifier", Act::Qualifier(m)) });
                }
                let cur = r.refd(m, "ownerAttribute");
                let shown = cur.map(|c| r.name(c)).unwrap_or_else(|| "—".to_string());
                let mut items = vec![("(none)".to_string(), cur.is_none(), Pick::Owner(m, None))];
                items.extend(self.owner_attribute_options(m).into_iter().map(|a| (r.name(a), cur == Some(a), Pick::Owner(m, Some(a)))));
                out.push(Placed { side: Side::Left, row: 2, comp: menu(shown, "Owner Attribute", items) });
            }
            if !owned {
                out.push(Placed { side: Side::Right, row: 1, comp: Comp::Combo(m) });
            }
        }
        // the line of a name edit: the edited line, or the line showing the edited element
        let edge = match (ed.host, &ed.kind) {
            (Some(h), EditKind::Name(_)) if is_edge(r, h) => Some(h),
            (None, EditKind::Name(_)) => self.diagram.and_then(|d| r.list(d, "ownedViews").into_iter().find(|v| is_edge(r, *v) && r.refd(*v, "model") == Some(m))),
            _ => None,
        };
        // an association's Order of Ends, a property of its line (quickedits/uml.json "uml.end-order")
        if let (Some(e), true) = (edge, matches!(ty.as_str(), "UMLAssociation" | "UMLCommunicationPath")) {
            let cur = r.str(e, "showEndOrder");
            let opts = [("hide", "Hide"), ("forward", "Forward (End1 to End2)"), ("backward", "Backward (End2 to End1)")];
            let shown = opts.iter().find(|o| o.0 == cur).map(|o| o.1).unwrap_or("Hide");
            left1.push(menu(shown.split(' ').next().unwrap_or("").to_string(), "Order of Ends", opts.iter().map(|(v, l)| (l.to_string(), cur == *v, Pick::Set(e, "showEndOrder", serde_json::json!(v)))).collect()));
        }
        // a message's sort (quickedits/uml.json "uml.set-message-sort")
        if ty == "UMLMessage" {
            let cur = r.str(m, "messageSort");
            let opts = ["synchCall", "asynchCall", "asynchSignal", "createMessage", "deleteMessage", "reply"];
            left1.push(menu(cur.clone(), "MessageSort", opts.iter().map(|o| (o.to_string(), cur == *o, Pick::Set(m, "messageSort", serde_json::json!(o)))).collect()));
        }
        // the visibility, shown as its symbol (quickedits/uml.json "uml.visibility")
        if crate::meta::order_of(&ty, "visibility").is_some() && !owned && !no_visibility(&ty) {
            let cur = views::vis_symbol(&r.str(m, "visibility"));
            let items = [("+", "public"), ("#", "protected"), ("-", "private"), ("~", "package")].iter().map(|(s, v)| (format!("{s}  {v}"), views::vis_symbol(v) == cur, Pick::Visibility(m, v))).collect();
            left1.push(menu(if cur.is_empty() { "+".to_string() } else { cur.to_string() }, "Visibility", items));
        }
        // ERD (quickedits/erd.json): a relationship's Identifying and a column's Primary Key on the
        // left, a relationship end's Cardinality on the right
        match ty.as_str() {
            "ERDRelationshipEnd" => {
                let cur = r.str(m, "cardinality");
                let cur = if cur.is_empty() { "1".to_string() } else { cur };
                let shown = crate::erd::CARDINALITIES.iter().find(|c| c.0 == cur).map(|c| c.1).unwrap_or("One and Only One");
                let items = crate::erd::CARDINALITIES.iter().map(|(v, l)| (l.to_string(), cur == *v, Pick::Set(m, "cardinality", serde_json::json!(v)))).collect();
                out.push(Placed { side: Side::Right, row: 1, comp: menu(shown.to_string(), "Cardinality", items) });
            }
            "ERDRelationship" | "ERDColumn" => {
                let (field, hint, yes, no) = if ty == "ERDColumn" { ("primaryKey", "Primary Key", "Primary Key", "Not Key") } else { ("identifying", "Identifying", "Identifying", "Non-Identifying") };
                let cur = if ty == "ERDColumn" { r.bool(m, field) } else { r.raw(m, field).and_then(|v| v.as_bool()).unwrap_or(true) };
                let items = [(true, yes), (false, no)].iter().map(|(v, l)| (l.to_string(), cur == *v, Pick::Set(m, field, serde_json::json!(v)))).collect();
                left1.push(menu(if cur { yes } else { no }.to_string(), hint, items));
            }
            _ => {}
        }
        for c in left1 {
            out.push(Placed { side: Side::Left, row: 1, comp: c });
        }
        // the cloud quick edits' dropdowns: an Azure group's Dashed and Rounded, a GCP zone's Zone Type
        // (quickedits/azure.json, gcp.json)
        if let EditKind::Name(_) = ed.kind {
            for (field, hint, options) in crate::cloud::dropdowns(r, m) {
                let cur = r.val(m, field).unwrap_or(serde_json::Value::Null);
                let shown = options.iter().find(|o| o.0 == cur).map(|o| o.1).unwrap_or("—");
                let items = options.iter().map(|(v, l)| (l.to_string(), cur == *v, Pick::Set(m, field, v.clone()))).collect();
                out.push(Placed { side: Side::Right, row: 1, comp: menu(shown.to_string(), hint, items) });
            }
        }
        // the buttons with a key, on a box's name or item
        if let Some(h) = ed.host {
            let mut buttons = item_buttons(r, Some(h), &ed.kind);
            // a class also gets Add Template Parameter, after Add Operation (quickedits/uml.json "uml.class")
            if let (EditKind::Name(_), "UMLClass", true) = (&ed.kind, ty.as_str(), r.refd(h, "model") == Some(m)) {
                buttons.insert(2.min(buttons.len()), ("T", "Add Template Parameter", "", ItemKey::AddTemplateParameter));
            }
            for (label, hint, keys, k) in buttons {
                let tip = if keys.is_empty() { hint.to_string() } else { format!("{hint} ({keys})") };
                let (side, row) = match (&ed.kind, hint) {
                    // a reception's Select Signal is on the left (quickedits/uml.json "uml.reception")
                    (EditKind::Feature(_), "Select Signal") => (Side::Left, 1),
                    (EditKind::Feature(_), _) => (Side::Right, 1),
                    _ => place(&ty, &view_ty, hint),
                };
                out.push(Placed { side, row, comp: Comp::Button { label: label.to_string(), hint: tip, act: Act::Key(h, k) } });
            }
            // the "add related element" buttons of a box's name edit
            if let (EditKind::Name(_), true) = (&ed.kind, r.refd(h, "model") == Some(m) && !is_edge(r, h)) {
                for a in crate::related::actions(&ty) {
                    let (side, row) = place(&ty, &view_ty, a.label);
                    out.push(Placed { side, row, comp: button(&crate::canvas::short_label(a.label), a.label, Act::Related(a)) });
                }
            }
        }
        // a line's buttons (quickedits/uml.json, uml-commands.js:684)
        if let Some(e) = edge {
            let ety = r.ty(e).to_string();
            if crate::bpmn::is_edge_view(&ety) {
                // a BPMN flow's only button is Add Text Annotation (quickedits/bpmn.json "bpmn.sequence-flow")
                out.push(Placed { side: Side::Left, row: 1, comp: button("+ Text Annotation", "Add Text Annotation", Act::EdgeNote(e)) });
            } else {
                let (s, row) = place(&ty, &ety, "Add Note");
                out.push(Placed { side: s, row, comp: button("+ Note", "Add Note", Act::EdgeNote(e)) });
                // the other notations' lines have Add Note only (flowchart.json "fc.edge" and the like)
                if ty.starts_with("UML") || ty.starts_with("SysML") {
                    out.push(Placed { side: Side::Left, row: 2, comp: button("+ Constraint", "Add Constraint", Act::EdgeConstraint(e)) });
                }
            }
            // a transition's Add Trigger Event and Add Effect Behavior (quickedits/uml.json "uml.transition")
            if ty == "UMLTransition" {
                out.push(Placed { side: Side::Right, row: 1, comp: button("+ Trigger", "Add Trigger Event", Act::Key(e, ItemKey::AddTrigger)) });
                out.push(Placed { side: Side::Right, row: 1, comp: button("+ Effect", "Add Effect Behavior", Act::Key(e, ItemKey::AddEffect)) });
            }
            // a communication connector's Add Forward / Reverse Message (quickedits/uml.json "uml.connector-comm")
            if ety == "UMLConnectorView" && self.diagram.map(|d| r.ty(d) == "UMLCommunicationDiagram").unwrap_or(false) {
                out.push(Placed { side: Side::Right, row: 1, comp: button("+ Fwd", "Add Forward Message", Act::CommMessage(e, true)) });
                out.push(Placed { side: Side::Right, row: 1, comp: button("+ Rev", "Add Reverse Message", Act::CommMessage(e, false)) });
            }
            // a SysML connector's Create Item Flow (quickedits/sysml.json "sysml.connector")
            if ety == "SysMLConnectorView" {
                out.push(Placed { side: Side::Right, row: 1, comp: button("+ Item Flow", "Create Item Flow", Act::ItemFlow(m)) });
            }
            // a template binding adds a parameter substitution (quickedits/uml.json "uml.template-binding")
            if ty == "UMLTemplateBinding" {
                out.push(Placed { side: Side::Right, row: 1, comp: button("+ Subst", "Add Template Parameter Substitution", Act::Subst(m)) });
            }
        }
        out.iter_mut().for_each(|p| p.row = p.row.clamp(1, 5));
        out
    }
}
