//! Tools > Diagram Generator: new diagrams drawn from a package's contents, laid out automatically
//! (extensions/default/diagram-generator/main.js).

use crate::app::App;
use crate::layout::{Dir, Seps, SEPS};
use crate::model::Eid;
use crate::views::{self, Measure, P};
use serde_json::json;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Generator {
    Overview,
    OverviewExpanded,
    TypeHierarchy,
    PackageStructure,
}

impl Generator {
    /// Tools › Diagram Generator, in menu order.
    pub const MENU: [(Generator, &'static str); 4] = [
        (Generator::Overview, "Overview"),
        (Generator::OverviewExpanded, "Overview Expanded"),
        (Generator::TypeHierarchy, "Type Hierarchy"),
        (Generator::PackageStructure, "Package Structure"),
    ];

    fn diagram_name(self) -> &'static str {
        match self {
            Generator::Overview | Generator::OverviewExpanded => "Overview",
            Generator::TypeHierarchy => "Type Hierarchy",
            Generator::PackageStructure => "Package Structure",
        }
    }
}

/// The kinds a type hierarchy shows (main.js:226).
const HIERARCHY_KINDS: [&str; 5] = ["UMLClass", "UMLInterface", "UMLEnumeration", "UMLSignal", "UMLDataType"];

impl App {
    /// Generate a diagram in the selected package: one undo step; the new diagram is laid out and opened.
    /// Without a selected package, an alert asks for one.
    pub fn generate_diagram(&mut self, g: Generator, ms: &dyn Measure) -> Option<Eid> {
        let pkg = self.focus.filter(|f| self.repo.alive(*f) && self.repo.kind_of(*f, "UMLPackage"));
        let Some(pkg) = pkg else {
            self.info_box(&format!("To generate {} diagram, select a Package.", g.diagram_name().to_lowercase()));
            return None;
        };
        Some(self.generate_diagram_in(pkg, g, ms))
    }

    /// Generate a diagram in a given package (also used after a code reverse), as one undo step.
    pub fn generate_diagram_in(&mut self, pkg: Eid, g: Generator, ms: &dyn Measure) -> Eid {
        let undo = self.undo.len();
        let d = self.change(|a| {
            let ty = if g == Generator::PackageStructure { "UMLPackageDiagram" } else { "UMLClassDiagram" };
            let d = a.add_diagram(Some(pkg), ty);
            a.repo.set(d, "name", json!(g.diagram_name()));
            match g {
                Generator::Overview | Generator::OverviewExpanded => {
                    let members: Vec<Eid> = a.repo.list(pkg, "ownedElements").into_iter().filter(|m| views::node_kind(a.repo.ty(*m)).is_some()).collect();
                    let placed = a.place_all(d, &members, g == Generator::Overview);
                    a.connect_all(d, &placed, |_| true);
                }
                Generator::TypeHierarchy => {
                    let mut members = vec![];
                    a.collect(pkg, &mut |a, e| HIERARCHY_KINDS.contains(&a.repo.ty(e)), &mut members);
                    let placed = a.place_all(d, &members, true);
                    a.connect_all(d, &placed, |t| t == "UMLGeneralization" || t == "UMLInterfaceRealization");
                }
                Generator::PackageStructure => {
                    let mut members = vec![pkg];
                    a.collect(pkg, &mut |a, e| a.repo.kind_of(e, "UMLPackage"), &mut members);
                    let placed = a.place_all(d, &members, true);
                    // each package hangs from its parent by a containment line
                    for &(m, v) in &placed {
                        let parent = a.repo.parent(m).and_then(|p| placed.iter().find(|x| x.0 == p)).map(|x| x.1);
                        if let Some(pv) = parent {
                            let font = json!(views::FONT);
                            let (h, t) = (a.repo.ref_to(pv), a.repo.ref_to(v));
                            let props = [("font", font), ("head", h), ("tail", t), ("lineStyle", json!(1)), ("points", json!("0:0;0:0"))];
                            a.repo.create("UMLContainmentView", &props, Some((d, "ownedViews")));
                        }
                    }
                }
            }
            d
        });
        // lay it out with the real box sizes, still as the same undo step
        self.info = views::layout_diagram(&mut self.repo, ms, d);
        let (dir, seps) = match g {
            Generator::TypeHierarchy => (Dir::LeftRight, Seps { node: 10.0, edge: 10.0, rank: 100.0 }),
            _ => (Dir::TopBottom, SEPS),
        };
        self.layout_diagram_with(dir, seps);
        self.undo.truncate(undo + 1);
        self.info = views::layout_diagram(&mut self.repo, ms, d);
        self.focus = Some(d);
        d
    }

    /// Every element under `e` (not `e` itself) that passes the test, in model order.
    fn collect(&self, e: Eid, keep: &mut dyn FnMut(&App, Eid) -> bool, out: &mut Vec<Eid>) {
        for c in self.repo.list(e, "ownedElements") {
            if keep(self, c) {
                out.push(c);
            }
            self.collect(c, keep, out);
        }
    }

    /// A view for each model, in a loose grid that the layout then rearranges. `brief`: attributes,
    /// operations and literals are suppressed and interfaces drawn as icons.
    fn place_all(&mut self, d: Eid, models: &[Eid], brief: bool) -> Vec<(Eid, Eid)> {
        let mut out = vec![];
        for (i, &m) in models.iter().enumerate() {
            let p = P { x: 20.0 + (i % 8) as f64 * 200.0, y: 20.0 + (i / 8) as f64 * 160.0 };
            let Some(v) = self.place_node(d, m, p) else { continue };
            if brief {
                for f in ["suppressAttributes", "suppressOperations", "suppressLiterals"] {
                    if crate::meta::order_of(self.repo.ty(v), f).is_some() {
                        self.repo.set(v, f, json!(true));
                    }
                }
                if self.repo.ty(m) == "UMLInterface" {
                    self.repo.set(v, "stereotypeDisplay", json!("icon"));
                }
            }
            out.push((m, v));
        }
        out
    }

    /// One line for each relationship of an accepted kind whose two ends are shown, self-relationships
    /// included (main.js:58).
    pub(crate) fn connect_all(&mut self, d: Eid, placed: &[(Eid, Eid)], accept: impl Fn(&str) -> bool) {
        let view_of = |m: Eid| placed.iter().find(|x| x.0 == m).map(|x| x.1);
        let rels: Vec<Eid> = self.repo.all().filter(|x| views::edge_view_of(self.repo.ty(*x)).is_some() && accept(self.repo.ty(*x))).collect();
        for rel in rels {
            let Some((t, h)) = self.rel_ends(rel) else { continue };
            if let (Some(tv), Some(hv)) = (view_of(t), view_of(h)) {
                views::create_edge_view(&mut self.repo, d, Some(rel), tv, hv);
            }
        }
    }
}
