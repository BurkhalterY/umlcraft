//! File › Import › StarUML 1 File (.uml): reads a StarUML 1 (XPD XML) project and converts it to
//! the current metamodel, following extensions/default/staruml-v1 (main.js, reader.js, uml-readers.js).
//!
//! The original reads the XML into plain JSON objects with one reader per V1 type (each one calls the
//! reader of its super-type first), runs a list of post-processors over every object it read, and then
//! loads the JSON as the new project. This module does the same: the objects live in an arena, an
//! object held by another one is written as a placeholder `{"$obj": index}` until the end, where the
//! tree is expanded into .mdj JSON. See MECHANICS.md "StarUML 1 import".

use crate::meta;
use crate::model::{new_id, num, Repo};
use indexmap::IndexMap;
use roxmltree::{Document, Node};
use serde_json::{json, Map, Value};

/// Read a StarUML 1 file as a new project (main.js:34 loadFile).
pub fn import(text: &str) -> Result<Repo, String> {
    let v = convert(text)?;
    Repo::parse(&serde_json::to_string(&v).map_err(|e| e.to_string())?)
}

/// The project JSON of a StarUML 1 file.
pub fn convert(text: &str) -> Result<Value, String> {
    let doc = Document::parse(text).map_err(|e| format!("Not a valid StarUML 1 file: {e}"))?;
    // getElementsByTagName("BODY") or "XPD:BODY" (main.js:45)
    let body = doc.descendants().find(|n| n.is_element() && n.tag_name().name() == "BODY").ok_or("This file has no BODY element.")?;
    let mut imp = Imp::default();
    let root = imp.read_obj(body, "DocumentElement").ok_or("This file has no DocumentElement.")?;
    imp.objs[root].remove("_parent");
    imp.postprocess();
    Ok(imp.expand(root, None))
}

// ------------------------------------------------------------ XML access (reader.js)

fn guid(n: Node) -> Option<String> {
    n.attribute("guid").map(|s| s.to_string())
}

/// Child elements with this local name (ATTR, REF, OBJ; the XPD: prefix is ignored) and this `name`.
fn tagged<'a, 'i>(n: Node<'a, 'i>, tag: &str, name: &str) -> Vec<Node<'a, 'i>> {
    n.children().filter(|c| c.is_element() && c.tag_name().name() == tag && c.attribute("name") == Some(name)).collect()
}

fn text(n: Node) -> String {
    n.text().unwrap_or("").to_string()
}

/// ATTR value by its type: string, integer, real, boolean or Points (reader.js:61 readAttr).
fn attr(n: Node, name: &str) -> Option<Value> {
    for c in tagged(n, "ATTR", name) {
        let t = text(c);
        match c.attribute("type") {
            Some("string") => return Some(json!(t)),
            Some("integer") => return Some(t.trim().parse::<i64>().map(|i| json!(i)).unwrap_or(Value::Null)),
            Some("real") => return Some(t.trim().parse::<f64>().map(num).unwrap_or(Value::Null)),
            Some("boolean") => return Some(json!(t.to_lowercase() == "true")),
            // "x,y;x,y" becomes "x:y;x:y"
            Some("Points") => return Some(json!(t.replace(',', ":"))),
            _ => {}
        }
    }
    None
}

fn attr_or(n: Node, name: &str, default: Value) -> Value {
    attr(n, name).unwrap_or(default)
}

fn attr_null(n: Node, name: &str) -> Value {
    attr_or(n, name, Value::Null)
}

/// The V1 enumerations and the values they map to (uml-readers.js:233-395, 1390).
fn enum_literal(ty: &str, lit: &str) -> Option<Value> {
    let v = match (ty, lit) {
        ("LineStyleKind", "lsRectilinear") => json!(0),
        ("LineStyleKind", "lsOblique") => json!(1),
        ("EdgePositionKind", "epHead") => json!(0),
        ("EdgePositionKind", "epMiddle") => json!(1),
        ("EdgePositionKind", "epTail") => json!(2),
        ("UMLAggregationKind", "akNone") => json!("none"),
        ("UMLAggregationKind", "akAggregate") => json!("shared"),
        ("UMLAggregationKind", "akComposite") => json!("composite"),
        ("UMLCallConcurrencyKind", "cckSequential") => json!("sequential"),
        ("UMLCallConcurrencyKind", "cckGuarded") => json!("guarded"),
        ("UMLCallConcurrencyKind", "cckConcurrent") => json!("concurrent"),
        ("UMLChangeableKind", "ckChangeable") => json!(0),
        ("UMLChangeableKind", "ckFrozen") => json!(1),
        ("UMLChangeableKind", "ckAddOnly") => json!(2),
        ("UMLOrderingKind", "okUnordered") => json!(0),
        ("UMLOrderingKind", "okOrdered") => json!(1),
        ("UMLParameterDirectionKind", "pdkIn") => json!("in"),
        ("UMLParameterDirectionKind", "pdkInout") => json!("inout"),
        ("UMLParameterDirectionKind", "pdkOut") => json!("out"),
        ("UMLParameterDirectionKind", "pdkReturn") => json!("return"),
        ("UMLScopeKind", "skInstance") => json!(0),
        ("UMLScopeKind", "skClassifier") => json!(1),
        ("UMLVisibilityKind", "vkPublic") => json!("public"),
        ("UMLVisibilityKind", "vkProtected") => json!("protected"),
        ("UMLVisibilityKind", "vkPrivate") => json!("private"),
        ("UMLVisibilityKind", "vkPackage") => json!("package"),
        ("UMLPseudostateKind", "pkChoice") => json!("choice"),
        ("UMLPseudostateKind", "pkDeepHistory") => json!("deepHistory"),
        // fork or join, decided by the transitions afterwards
        ("UMLPseudostateKind", "pkSynchronization") => json!(0),
        ("UMLPseudostateKind", "pkInitial") => json!("initial"),
        ("UMLPseudostateKind", "pkJunction") => json!("junction"),
        ("UMLPseudostateKind", "pkShallowHistory") => json!("shallowHistory"),
        // a decision node in an activity, a choice elsewhere
        ("UMLPseudostateKind", "pkDecision") => json!(1),
        ("UMLInteractionOperatorKind", "iokSeq") => json!("seq"),
        ("UMLInteractionOperatorKind", "iokAlt") => json!("alt"),
        ("UMLInteractionOperatorKind", "iokOpt") => json!("opt"),
        ("UMLInteractionOperatorKind", "iokBreak") => json!("seq"),
        ("UMLInteractionOperatorKind", "iokPar") => json!("par"),
        // the original maps it to an undefined constant (so "seq"); UMLCraft keeps "strict"
        ("UMLInteractionOperatorKind", "iokStrict") => json!("strict"),
        ("UMLInteractionOperatorKind", "iokLoop") => json!("loop"),
        ("UMLInteractionOperatorKind", "iokRegion") => json!("seq"),
        ("UMLInteractionOperatorKind", "iokNeg") => json!("neg"),
        ("UMLInteractionOperatorKind", "iokAssert") => json!("assert"),
        ("UMLInteractionOperatorKind", "iokIgnore") => json!("ignore"),
        ("UMLInteractionOperatorKind", "iokConsider") => json!("consider"),
        ("UMLMessageSignatureKind", "mskNone") => json!(0),
        ("UMLMessageSignatureKind", "mskTypeOnly") => json!(1),
        ("UMLMessageSignatureKind", "mskNameOnly") => json!(2),
        ("UMLMessageSignatureKind", "mskNameAndType") => json!(3),
        ("UMLStereotypeDisplayKind", "sdkLabel") => json!("label"),
        ("UMLStereotypeDisplayKind", "sdkIcon") => json!("icon"),
        ("UMLStereotypeDisplayKind", "sdkNone") => json!("none"),
        ("UMLStereotypeDisplayKind", "sdkDecoration") => json!("decoration"),
        ("UMLSwimlaneDirectionKind", "slkVertical") => json!(0),
        ("UMLSwimlaneDirectionKind", "slkHorizontal") => json!(1),
        _ => return None,
    };
    Some(v)
}

/// ATTR of an enumeration type (reader.js:94 readEnum).
fn en(n: Node, name: &str) -> Option<Value> {
    tagged(n, "ATTR", name).into_iter().find_map(|c| enum_literal(c.attribute("type").unwrap_or(""), text(c).trim()))
}

fn en_or(n: Node, name: &str, default: Value) -> Value {
    en(n, name).unwrap_or(default)
}

fn en_is(n: Node, name: &str, v: i64) -> Value {
    json!(en(n, name).and_then(|x| x.as_i64()) == Some(v))
}

/// REF value (reader.js:121 readRef).
fn rf(n: Node, name: &str) -> Value {
    tagged(n, "REF", name).first().copied().map(|c| json!({ "$ref": text(c) })).unwrap_or(Value::Null)
}

/// "$00BBGGRR" or a Delphi colour name to "#RRGGBB" (reader.js:232 readColor).
fn color(n: Node, name: &str, default: &str) -> Value {
    for c in tagged(n, "ATTR", name) {
        let mut v = text(c);
        if v.starts_with("cl") {
            let hex = match v.as_str() {
                "clNone" => "$1FFFFFFF",
                "clAqua" => "$00FFFF00",
                "clBlack" => "$00000000",
                "clBlue" => "$00FF0000",
                "clCream" => "$00F0FBFF",
                "clDkGray" => "$00808080",
                "clFuchsia" => "$00FF00FF",
                "clGray" => "$00808080",
                "clGreen" => "$00008000",
                "clLime" => "$0000FF00",
                "clLtGray" => "$00C0C0C0",
                "clMaroon" => "$00000080",
                "clMedGray" => "$00A4A0A0",
                "clMoneyGreen" => "$00C0DCC0",
                "clNavy" => "$00800000",
                "clOlive" => "$00008080",
                "clPurple" => "$00800080",
                "clRed" => "$000000FF",
                "clSilver" => "$00C0C0C0",
                "clSkyBlue" => "$00F0CAA6",
                "clTeal" => "$00808000",
                "clWhite" => "$00FFFFFF",
                "clYellow" => "$0000FFFF",
                _ => "",
            };
            if !hex.is_empty() {
                v = hex.to_string();
            }
        }
        if v.starts_with('$') && v.len() == 9 && v.is_ascii() {
            return json!(format!("#{}{}{}", &v[7..9], &v[5..7], &v[3..5]));
        }
    }
    json!(default)
}

// ------------------------------------------------------------ the arena

/// Placeholder for an object held by another one.
fn ph(i: usize) -> Value {
    json!({ "$obj": i })
}

fn ph_of(v: &Value) -> Option<usize> {
    v.as_object().filter(|m| m.len() == 1).and_then(|m| m.get("$obj")).and_then(|x| x.as_u64()).map(|x| x as usize)
}

fn ref_id(v: Option<&Value>) -> Option<&str> {
    v.and_then(|v| v.get("$ref")).and_then(|x| x.as_str())
}

fn refv(id: &str) -> Value {
    json!({ "$ref": id })
}

#[derive(Default)]
struct Imp {
    objs: Vec<Map<String, Value>>,
    /// Every object read from the file by its guid, in reading order (reader.js:40 idMap).
    id_map: IndexMap<String, usize>,
}

fn set(j: &mut Map<String, Value>, k: &str, v: Value) {
    j.insert(k.to_string(), v);
}

impl Imp {
    fn push(&mut self, m: Map<String, Value>) -> usize {
        self.objs.push(m);
        self.objs.len() - 1
    }
    fn new_obj(&mut self, ty: &str, props: Value) -> usize {
        let mut m = Map::new();
        m.insert("_id".into(), json!(new_id()));
        m.insert("_type".into(), json!(ty));
        if let Value::Object(p) = props {
            for (k, v) in p {
                m.insert(k, v);
            }
        }
        self.push(m)
    }
    fn ty(&self, i: usize) -> &str {
        self.objs[i].get("_type").and_then(|t| t.as_str()).unwrap_or("")
    }
    fn id(&self, i: usize) -> String {
        self.objs[i].get("_id").and_then(|t| t.as_str()).unwrap_or("").to_string()
    }
    fn get(&self, id: Option<&str>) -> Option<usize> {
        id.and_then(|id| self.id_map.get(id).copied())
    }
    /// The object a ref field points to (Reader.get(elem.key.$ref)).
    fn deref(&self, i: usize, key: &str) -> Option<usize> {
        self.get(ref_id(self.objs[i].get(key)))
    }
    fn parent(&self, i: usize) -> Option<usize> {
        self.deref(i, "_parent")
    }
    fn list(&self, i: usize, key: &str) -> Vec<usize> {
        match self.objs[i].get(key) {
            Some(Value::Array(a)) => a.iter().filter_map(ph_of).collect(),
            _ => vec![],
        }
    }
    fn child(&self, i: usize, key: &str) -> Option<usize> {
        self.objs[i].get(key).and_then(ph_of)
    }
    fn arr_push(&mut self, i: usize, key: &str, v: Value) {
        if let Some(Value::Array(a)) = self.objs[i].get_mut(key) {
            a.push(v);
        }
    }

    /// OBJ child by name, converted by its type's reader (reader.js:142 readObj).
    fn read_obj(&mut self, n: Node, name: &str) -> Option<usize> {
        let parent = guid(n);
        for c in tagged(n, "OBJ", name) {
            let ty = c.attribute("type").unwrap_or("");
            if let Some(mut j) = self.read(ty, c) {
                if !j.contains_key("_type") {
                    set(&mut j, "_type", json!(ty));
                }
                if j.get("_id").map(|v| v.is_null()).unwrap_or(true) {
                    set(&mut j, "_id", json!(guid(c).unwrap_or_else(new_id)));
                }
                if j.get("_parent").map(|v| v.is_null()).unwrap_or(true) {
                    set(&mut j, "_parent", parent.as_deref().map(refv).unwrap_or(Value::Null));
                }
                let id = j["_id"].as_str().unwrap_or("").to_string();
                let i = self.push(j);
                // a guid read again keeps its place (a JavaScript object key)
                self.id_map.insert(id, i);
                return Some(i);
            }
        }
        None
    }

    /// "#Name" count, then Name[0]..Name[n-1] (reader.js:179 readObjArray).
    fn read_obj_array(&mut self, n: Node, name: &str) -> Vec<usize> {
        let count = format!("#{name}");
        if tagged(n, "ATTR", &count).first().copied().is_none() {
            return vec![];
        }
        let size = attr(n, &count).and_then(|v| v.as_i64()).unwrap_or(0);
        (0..size).filter_map(|k| self.read_obj(n, &format!("{name}[{k}]"))).collect()
    }

    fn read_ref_array(&mut self, n: Node, name: &str) -> Vec<Value> {
        let count = format!("#{name}");
        if tagged(n, "ATTR", &count).first().copied().is_none() {
            return vec![];
        }
        let size = attr(n, &count).and_then(|v| v.as_i64()).unwrap_or(0);
        (0..size).map(|k| rf(n, &format!("{name}[{k}]"))).filter(|v| !v.is_null()).collect()
    }

    fn objs_value(items: &[usize]) -> Value {
        Value::Array(items.iter().map(|i| ph(*i)).collect())
    }

    // -------------------------------------------- helpers of uml-readers.js:53-128

    /// Add an element to json[field] and make obj its parent (addTo).
    fn add_to(&mut self, n: Node, j: &mut Map<String, Value>, field: &str, elem: Option<usize>) {
        if let (Some(Value::Array(a)), Some(e)) = (j.get_mut(field), elem) {
            set(&mut self.objs[e], "_parent", guid(n).as_deref().map(refv).unwrap_or(Value::Null));
            a.push(ph(e));
        }
    }
    fn append_to(&mut self, n: Node, j: &mut Map<String, Value>, field: &str, elems: Vec<usize>) {
        for e in elems {
            self.add_to(n, j, field, Some(e));
        }
    }
    /// Read a sub-view (or use the given default), add it to subViews and point json[field] to it.
    fn add_sub_view(&mut self, n: Node, child: &str, j: &mut Map<String, Value>, field: &str, default: Option<Value>) -> Option<usize> {
        let sub = self.read_obj(n, child).or_else(|| {
            default.map(|d| {
                let ty = d["_type"].as_str().unwrap_or("").to_string();
                let mut d = d;
                d.as_object_mut().map(|m| m.remove("_type"));
                self.new_obj(&ty, d)
            })
        });
        if let Some(s) = sub {
            if j.get("subViews").map(|v| v.is_array()).unwrap_or(false) {
                let id = self.id(s);
                set(j, field, refv(&id));
                set(&mut self.objs[s], "_parent", guid(n).as_deref().map(refv).unwrap_or(Value::Null));
                self.arr_push_map(j, "subViews", ph(s));
            }
        }
        sub
    }
    fn arr_push_map(&self, j: &mut Map<String, Value>, key: &str, v: Value) {
        if let Some(Value::Array(a)) = j.get_mut(key) {
            a.push(v);
        }
    }
    fn add_tag(&mut self, n: Node, j: &mut Map<String, Value>, name: &str, value: Value) {
        if value.is_null() {
            return;
        }
        let p = guid(n).as_deref().map(refv).unwrap_or(Value::Null);
        let t = self.new_obj("Tag", json!({ "_parent": p, "name": name, "value": value }));
        self.arr_push_map(j, "tags", ph(t));
    }
    fn label_view() -> Value {
        json!({ "_type": "LabelView" })
    }
    fn edge_label_view() -> Value {
        json!({ "_type": "EdgeLabelView", "parentStyle": true })
    }
    /// The diagram that holds a view, by following the parents (getDiagram).
    fn diagram_of(&self, mut i: usize) -> Option<usize> {
        for _ in 0..10_000 {
            if self.ty(i).ends_with("Diagram") {
                return Some(i);
            }
            i = self.parent(i)?;
        }
        None
    }

    // -------------------------------------------- readers (uml-readers.js:133-1960)

    /// The reader of a V1 type: the fields it adds after those of its super-type. None for an unknown type.
    fn read(&mut self, ty: &str, n: Node) -> Option<Map<String, Value>> {
        let mut j;
        match ty {
            // ---- core elements (uml-readers.js:133)
            "Element" => {
                j = Map::new();
                set(&mut j, "tags", json!([]));
                set(&mut j, "ownedElements", json!([]));
                self.add_tag(n, &mut j, "_tag", attr_null(n, "Tag"));
            }
            "Model" => {
                j = self.read("Element", n)?;
                set(&mut j, "name", attr_or(n, "Name", json!("")));
                set(&mut j, "documentation", attr_or(n, "Documentation", json!("")));
                let d = self.read_obj_array(n, "OwnedDiagrams");
                self.append_to(n, &mut j, "ownedElements", d);
                if let Some(a) = attr(n, "Attachments").filter(|a| a.as_str().map(|s| !s.is_empty()).unwrap_or(false)) {
                    self.add_tag(n, &mut j, "attachments", a);
                }
            }
            "View" => {
                j = self.read("Element", n)?;
                set(&mut j, "lineColor", color(n, "LineColor", "#000000"));
                set(&mut j, "fillColor", color(n, "FillColor", "#FFFFFF"));
                set(&mut j, "fontColor", color(n, "FontColor", "#000000"));
                let face = attr(n, "FontFace").and_then(|v| v.as_str().map(|s| s.to_string())).unwrap_or_else(|| "Arial".into());
                let size = attr(n, "FontSize").and_then(|v| v.as_f64()).unwrap_or(8.0);
                // V1 sizes are in points
                let size = (size / 0.75 + 0.5).floor() as i64;
                let fs = attr(n, "FontStyle").and_then(|v| v.as_i64()).unwrap_or(0);
                let (bold, italic) = (fs & 1 != 0, fs & 2 != 0);
                let style = match (bold, italic) {
                    (true, false) => 1,
                    (false, true) => 2,
                    (true, true) => 3,
                    _ => 0,
                };
                set(&mut j, "font", json!(format!("{face};{size};{style}")));
                set(&mut j, "visible", attr_or(n, "Visible", json!(true)));
                set(&mut j, "model", rf(n, "Model"));
                set(&mut j, "subViews", json!([]));
                let cv = self.read_obj_array(n, "ContainedViews");
                set(&mut j, "containedViews", Self::objs_value(&cv));
                set(&mut j, "containerView", rf(n, "ContainerView"));
            }
            "Diagram" => {
                j = self.read("Model", n)?;
                set(&mut j, "defaultDiagram", attr_or(n, "DefaultDiagram", json!(false)));
                let dv = self.read_obj(n, "DiagramView");
                set(&mut j, "ownedViews", json!([]));
                if let Some(dv) = dv {
                    for v in self.list(dv, "ownedViews") {
                        self.add_to(n, &mut j, "ownedViews", Some(v));
                    }
                }
            }
            "DiagramView" => {
                j = self.read("View", n)?;
                let v = self.read_obj_array(n, "OwnedViews");
                set(&mut j, "ownedViews", Self::objs_value(&v));
            }
            "ExtensibleModel" => {
                j = self.read("Model", n)?;
                set(&mut j, "stereotype", attr_null(n, "StereotypeName"));
                let c = self.read_obj_array(n, "Constraints");
                self.append_to(n, &mut j, "ownedElements", c);
                for t in self.read_obj_array(n, "TaggedValues") {
                    let s = |k: &str| self.objs[t].get(k).and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let name = format!("{}.{}.{}", s("profileName"), s("tagDefinitionSetName"), s("name"));
                    let value = self.objs[t].get("dataValue").cloned().unwrap_or(json!(""));
                    self.add_tag(n, &mut j, &name, value);
                }
            }
            "Constraint" => {
                j = self.read("Element", n)?;
                set(&mut j, "name", attr_or(n, "Name", json!("")));
                set(&mut j, "specification", attr_or(n, "Body", json!("")));
                set(&mut j, "_type", json!("UMLConstraint"));
            }
            "TaggedValue" => {
                j = self.read("Element", n)?;
                for (k, a) in [("profileName", "ProfileName"), ("tagDefinitionSetName", "TagDefinitionSetName"), ("name", "Name"), ("dataValue", "DataValue")] {
                    set(&mut j, k, attr_or(n, a, json!("")));
                }
            }

            // ---- view core (uml-readers.js:233)
            "NodeView" => {
                j = self.read("View", n)?;
                for (k, a) in [("left", "Left"), ("top", "Top"), ("width", "Width"), ("height", "Height")] {
                    set(&mut j, k, attr_or(n, a, json!(0)));
                }
                set(&mut j, "autoResize", attr_or(n, "AutoResize", json!(false)));
            }
            "EdgeView" => {
                j = self.read("View", n)?;
                set(&mut j, "head", rf(n, "Head"));
                set(&mut j, "tail", rf(n, "Tail"));
                set(&mut j, "lineStyle", en_or(n, "LineStyle", json!(1)));
                set(&mut j, "points", attr_or(n, "Points", json!("")));
            }
            "LabelView" => {
                j = self.read("NodeView", n)?;
                set(&mut j, "text", attr_or(n, "Text", json!("")));
            }
            "ParasiticView" => {
                j = self.read("NodeView", n)?;
                set(&mut j, "alpha", attr_or(n, "Alpha", json!(0)));
                set(&mut j, "distance", attr_or(n, "Distance", json!(0)));
            }
            "NodeParasiticView" => {
                j = self.read("ParasiticView", n)?;
                set(&mut j, "hostNode", rf(n, "HostNode"));
            }
            "EdgeParasiticView" => {
                j = self.read("ParasiticView", n)?;
                set(&mut j, "hostEdge", rf(n, "HostEdge"));
                set(&mut j, "edgePosition", en_or(n, "EdgePosition", json!(1)));
            }
            "NodeNodeView" | "NodeLabelView" => j = self.read("NodeParasiticView", n)?,
            "EdgeNodeView" | "EdgeLabelView" => j = self.read("EdgeParasiticView", n)?,

            // ---- foundation core (uml-readers.js:405)
            "UMLElement" => j = self.read("ExtensibleModel", n)?,
            "UMLModelElement" => {
                j = self.read("UMLElement", n)?;
                set(&mut j, "visibility", en_or(n, "Visibility", json!("public")));
                set(&mut j, "isSpecification", attr_or(n, "IsSpecification", json!(false)));
                set(&mut j, "_parent", rf(n, "Namespace"));
                let tp = self.read_obj_array(n, "TemplateParameters");
                set(&mut j, "templateParameters", Self::objs_value(&tp));
                let b = self.read_obj_array(n, "Behaviors");
                self.append_to(n, &mut j, "ownedElements", b);
            }
            "UMLTemplateParameter" => {
                j = self.read("UMLModelElement", n)?;
                set(&mut j, "parameterType", attr_or(n, "ParameterType", json!("")));
                set(&mut j, "DefaultValue", attr_or(n, "DefaultValue", json!("")));
            }
            "UMLFeature" => {
                j = self.read("UMLModelElement", n)?;
                set(&mut j, "isStatic", en_is(n, "OwnerScope", 1));
            }
            "UMLStructuralFeature" => {
                j = self.read("UMLFeature", n)?;
                set(&mut j, "multiplicity", attr_or(n, "Multiplicity", json!("")));
                set(&mut j, "isReadOnly", en_is(n, "Changeability", 1));
                set(&mut j, "isOrdered", en_is(n, "Ordering", 1));
                set(&mut j, "type", Self::type_of(n));
            }
            "UMLAttribute" => {
                j = self.read("UMLStructuralFeature", n)?;
                set(&mut j, "defaultValue", attr_or(n, "InitialValue", json!("")));
            }
            "UMLParameter" => {
                j = self.read("UMLModelElement", n)?;
                set(&mut j, "defaultValue", attr_or(n, "DefaultValue", json!("")));
                set(&mut j, "direction", en_or(n, "DirectionKind", json!("in")));
                set(&mut j, "type", Self::type_of(n));
            }
            "UMLBehavioralFeature" => {
                j = self.read("UMLFeature", n)?;
                set(&mut j, "isQuery", attr_or(n, "IsQuery", json!(false)));
                set(&mut j, "raisedExceptions", Value::Array(self.read_ref_array(n, "RaisedSignals")));
            }
            "UMLOperation" => {
                j = self.read("UMLBehavioralFeature", n)?;
                let p = self.read_obj_array(n, "Parameters");
                set(&mut j, "parameters", Self::objs_value(&p));
                set(&mut j, "concurrency", en_or(n, "Concurrency", json!("sequential")));
                set(&mut j, "isLeaf", attr_or(n, "IsLeaf", json!(false)));
                set(&mut j, "isAbstract", attr_or(n, "IsAbstract", json!(false)));
                set(&mut j, "specification", attr_or(n, "Specification", json!("")));
                for f in ["OwnedCollaborations", "OwnedCollaborationInstanceSets"] {
                    let c = self.read_obj_array(n, f);
                    self.append_to(n, &mut j, "ownedElements", c);
                }
            }
            "UMLGeneralizableElement" => {
                j = self.read("UMLModelElement", n)?;
                set(&mut j, "isRoot", attr_or(n, "IsRoot", json!(false)));
                set(&mut j, "isLeaf", attr_or(n, "IsLeaf", json!(false)));
                set(&mut j, "isAbstract", attr_or(n, "IsAbstract", json!(false)));
            }
            "UMLNamespace" => {
                j = self.read("UMLGeneralizableElement", n)?;
                let o = self.read_obj_array(n, "OwnedElements");
                self.append_to(n, &mut j, "ownedElements", o);
            }
            "UMLClassifier" => {
                j = self.read("UMLNamespace", n)?;
                set(&mut j, "attributes", json!([]));
                set(&mut j, "operations", json!([]));
                for (field, v1) in [
                    ("attributes", "Attributes"),
                    ("operations", "Operations"),
                    ("attributes", "OwnedPorts"),
                    ("ownedElements", "OwnedCollaborations"),
                    ("ownedElements", "OwnedCollaborationInstanceSets"),
                ] {
                    let items = self.read_obj_array(n, v1);
                    self.append_to(n, &mut j, field, items);
                }
            }
            "UMLClass" => {
                j = self.read("UMLClassifier", n)?;
                set(&mut j, "isActive", attr_or(n, "IsActive", json!(false)));
            }
            "UMLInterface" | "UMLDataType" | "UMLSignal" | "UMLArtifact" | "UMLActor" | "UMLPackage" => j = self.read("UMLClassifier", n)?,
            "UMLPrimitive" => {
                // the original keeps "UMLPrimitive", which the current metamodel does not have (so the
                // element is lost); UMLCraft makes it a UMLPrimitiveType
                j = self.read("UMLDataType", n)?;
                set(&mut j, "_type", json!("UMLPrimitiveType"));
            }
            "UMLEnumeration" => {
                j = self.read("UMLDataType", n)?;
                let l = self.read_obj_array(n, "Literals");
                set(&mut j, "literals", Self::objs_value(&l));
            }
            "UMLEnumerationLiteral" | "UMLRelationship" | "UMLInteractionFragment" | "UMLEvent" => j = self.read("UMLModelElement", n)?,
            "UMLException" => {
                j = self.read("UMLSignal", n)?;
                set(&mut j, "_type", json!("UMLSignal"));
                set(&mut j, "stereotype", json!("exception"));
            }
            "UMLComponent" => {
                j = self.read("UMLClassifier", n)?;
                let me = guid(n).unwrap_or_default();
                for r in self.read_ref_array(n, "Residents") {
                    let cr = self.new_obj("UMLComponentRealization", json!({ "source": r, "target": refv(&me) }));
                    self.add_to(n, &mut j, "ownedElements", Some(cr));
                }
            }
            "UMLNode" => {
                j = self.read("UMLClassifier", n)?;
                let me = guid(n).unwrap_or_default();
                for f in ["DeployedComponents", "DeployedArtifacts"] {
                    for r in self.read_ref_array(n, f) {
                        let d = self.new_obj("UMLDeployment", json!({ "source": r, "target": refv(&me) }));
                        self.add_to(n, &mut j, "ownedElements", Some(d));
                    }
                }
            }
            "UMLPort" => {
                j = self.read("UMLStructuralFeature", n)?;
                set(&mut j, "isBehavior", attr_null(n, "IsBehavior"));
                set(&mut j, "isService", attr_null(n, "IsService"));
            }
            "UMLConnectorEnd" => {
                j = self.read("UMLModelElement", n)?;
                set(&mut j, "multiplicity", attr_or(n, "Multiplicity", json!("")));
                set(&mut j, "isOrdered", attr_null(n, "IsOrdered"));
                set(&mut j, "isUnique", attr_null(n, "IsUnique"));
                set(&mut j, "reference", rf(n, "Role"));
            }
            "UMLConnector" => {
                j = self.read("UMLFeature", n)?;
                let ends = self.read_obj_array(n, "Ends");
                if ends.len() == 2 {
                    set(&mut j, "end1", ph(ends[0]));
                    set(&mut j, "end2", ph(ends[1]));
                }
            }
            "UMLGeneralization" => {
                j = self.read("UMLRelationship", n)?;
                set(&mut j, "source", rf(n, "Child"));
                set(&mut j, "target", rf(n, "Parent"));
                set(&mut j, "discriminator", attr_or(n, "Discriminator", json!("")));
            }
            "UMLDependency" => {
                j = self.read("UMLRelationship", n)?;
                set(&mut j, "source", rf(n, "Client"));
                set(&mut j, "target", rf(n, "Supplier"));
                set(&mut j, "mapping", attr_or(n, "Mapping", json!("")));
            }
            "UMLRealization" => {
                j = self.read("UMLDependency", n)?;
                set(&mut j, "_type", json!("UMLInterfaceRealization"));
            }
            "UMLAssociationEnd" => {
                j = self.read("UMLModelElement", n)?;
                let nav = attr_or(n, "IsNavigable", json!(true)).as_bool().unwrap_or(true);
                set(&mut j, "navigable", json!(if nav { "navigable" } else { "unspecified" }));
                set(&mut j, "isOrdered", en_is(n, "Ordering", 1));
                set(&mut j, "aggregation", en_or(n, "Aggregation", json!("none")));
                set(&mut j, "multiplicity", attr_or(n, "Multiplicity", json!("")));
                set(&mut j, "isReadOnly", en_is(n, "Changeability", 1));
                set(&mut j, "reference", rf(n, "Participant"));
                let q = self.read_obj_array(n, "Qualifiers");
                set(&mut j, "qualifiers", Self::objs_value(&q));
            }
            "UMLAssociation" => {
                j = self.read("UMLRelationship", n)?;
                let c = self.read_obj_array(n, "Connections");
                if c.len() == 2 {
                    set(&mut j, "end1", ph(c[0]));
                    set(&mut j, "end2", ph(c[1]));
                }
            }

            // ---- diagrams (uml-readers.js:710): the diagram type mapping
            "UMLDiagram" => j = self.read("Diagram", n)?,
            "UMLClassDiagram" | "UMLUseCaseDiagram" | "UMLComponentDiagram" | "UMLDeploymentDiagram" | "UMLCompositeStructureDiagram" | "UMLStatechartDiagram"
            | "UMLActivityDiagram" => j = self.read("UMLDiagram", n)?,
            "UMLSequenceDiagram" | "UMLSequenceRoleDiagram" => {
                j = self.read("UMLDiagram", n)?;
                set(&mut j, "showSequenceNumber", attr_or(n, "ShowSequenceNumber", json!(true)));
                set(&mut j, "showSignature", json!(en(n, "MessageSignature").and_then(|v| v.as_i64()).unwrap_or(0) == 3));
                set(&mut j, "showActivation", attr_or(n, "ShowActivation", json!(true)));
                set(&mut j, "_type", json!("UMLSequenceDiagram"));
            }
            "UMLCollaborationDiagram" | "UMLCollaborationRoleDiagram" => {
                j = self.read("UMLDiagram", n)?;
                set(&mut j, "_type", json!("UMLCommunicationDiagram"));
            }
            t if t.starts_with("UML") && t.ends_with("DiagramView") => j = self.read("DiagramView", n)?,

            // ---- common behavior (uml-readers.js:841)
            "UMLAction" => {
                j = self.read("UMLModelElement", n)?;
                set(&mut j, "recurrence", attr_null(n, "Recurrence"));
                set(&mut j, "target", attr_null(n, "Target"));
                set(&mut j, "isAsynchronous", attr_null(n, "IsAsynchronous"));
                set(&mut j, "script", attr_null(n, "Script"));
            }
            "UMLCreateAction" => {
                j = self.read("UMLAction", n)?;
                set(&mut j, "instantiation", rf(n, "Instantiation"));
            }
            "UMLCallAction" => {
                j = self.read("UMLAction", n)?;
                set(&mut j, "operation", rf(n, "Operation"));
            }
            "UMLSendAction" => {
                j = self.read("UMLAction", n)?;
                set(&mut j, "signal", rf(n, "Signal"));
            }
            "UMLReturnAction" | "UMLTerminalAction" | "UMLDestroyAction" => j = self.read("UMLAction", n)?,
            "UMLUninterpretedAction" => {
                j = self.read("UMLAction", n)?;
                set(&mut j, "_type", json!("UMLOpaqueBehavior"));
            }
            "UMLAttributeLink" => {
                j = self.read("UMLModelElement", n)?;
                set(&mut j, "definingFeature", rf(n, "Attribute"));
                set(&mut j, "value", attr_or(n, "ValueExpression", json!("")));
                set(&mut j, "_type", json!("UMLSlot"));
            }
            "UMLInstance" => {
                j = self.read("UMLModelElement", n)?;
                set(&mut j, "classifier", rf(n, "Classifier"));
                let s = self.read_obj_array(n, "Slots");
                set(&mut j, "slots", Self::objs_value(&s));
            }
            "UMLComponentInstance" | "UMLNodeInstance" => j = self.read("UMLInstance", n)?,

            // ---- collaborations (uml-readers.js:924)
            "UMLCollaboration" => {
                j = self.read("UMLNamespace", n)?;
                set(&mut j, "attributes", json!([]));
                let i = self.read_obj_array(n, "Interactions");
                self.append_to(n, &mut j, "ownedElements", i);
            }
            "UMLInteraction" | "UMLInteractionInstanceSet" => {
                j = self.read("UMLModelElement", n)?;
                let m = self.read_obj_array(n, if ty == "UMLInteraction" { "Messages" } else { "ParticipatingStimuli" });
                set(&mut j, "messages", Self::objs_value(&m));
                set(&mut j, "participants", json!([]));
                let f = self.read_obj_array(n, "Fragments");
                set(&mut j, "fragments", Self::objs_value(&f));
                set(&mut j, "_type", json!("UMLInteraction"));
            }
            "UMLCollaborationInstanceSet" => {
                j = self.read("UMLModelElement", n)?;
                set(&mut j, "attributes", json!([]));
                let i = self.read_obj_array(n, "InteractionInstanceSets");
                self.append_to(n, &mut j, "ownedElements", i);
                let p = self.read_obj_array(n, "ParticipatingInstances");
                self.append_to(n, &mut j, "attributes", p);
                let l = self.read_obj_array(n, "ParticipatingLinks");
                self.append_to(n, &mut j, "ownedElements", l);
                set(&mut j, "_type", json!("UMLCollaboration"));
            }
            "UMLClassifierRole" => {
                j = self.read("UMLClassifier", n)?;
                set(&mut j, "multiplicity", attr_or(n, "Multiplicity", json!("")));
                set(&mut j, "type", rf(n, "Base"));
                set(&mut j, "_type", json!("UMLAttribute"));
            }
            "UMLObject" => {
                j = self.read("UMLInstance", n)?;
                set(&mut j, "isMultiInstance", attr_or(n, "IsMultiInstance", json!(false)));
            }
            "UMLMessage" | "UMLStimulus" => {
                j = self.read("UMLModelElement", n)?;
                set(&mut j, "arguments", attr_or(n, "Arguments", json!("")));
                set(&mut j, "assignmentTarget", attr_or(n, "Return", json!("")));
                set(&mut j, "source", rf(n, "Sender"));
                set(&mut j, "target", rf(n, "Receiver"));
                set(&mut j, "connector", rf(n, if ty == "UMLMessage" { "CommunicationConnection" } else { "CommunicationLink" }));
                self.add_tag(n, &mut j, "_iteration", attr_null(n, "Iteration"));
                self.add_tag(n, &mut j, "_branch", attr_null(n, "Branch"));
                self.message_sort(n, &mut j);
                set(&mut j, "_type", json!("UMLMessage"));
            }
            "UMLInteractionOperand" => {
                j = self.read("UMLInteractionFragment", n)?;
                set(&mut j, "guard", attr_or(n, "Guard", json!("")));
            }
            "UMLCombinedFragment" => {
                j = self.read("UMLInteractionFragment", n)?;
                set(&mut j, "interactionOperator", en_or(n, "InteractionOperator", json!("seq")));
                let o = self.read_obj_array(n, "Operands");
                set(&mut j, "operands", Self::objs_value(&o));
            }
            "UMLAssociationEndRole" => {
                j = self.read("UMLAssociationEnd", n)?;
                set(&mut j, "_type", json!("UMLConnectorEnd"));
            }
            "UMLAssociationRole" => {
                j = self.read("UMLAssociation", n)?;
                self.add_tag(n, &mut j, "_multiplicity", attr_null(n, "Multiplicity"));
                set(&mut j, "type", rf(n, "Base"));
                set(&mut j, "_type", json!("UMLConnector"));
            }
            "UMLLinkEnd" => {
                j = self.read("UMLModelElement", n)?;
                set(&mut j, "instance", rf(n, "Instance"));
                let more = self.read("UMLConnectorEnd", n)?;
                j.extend(more);
            }
            "UMLLink" => {
                j = self.read("UMLModelElement", n)?;
                let c = self.read_obj_array(n, "Connections");
                if c.len() == 2 {
                    set(&mut j, "end1", ph(c[0]));
                    set(&mut j, "end2", ph(c[1]));
                }
                let more = self.read("UMLConnector", n)?;
                j.extend(more);
                set(&mut j, "association", rf(n, "Association"));
            }

            // ---- use cases (uml-readers.js:1126)
            "UMLUseCase" => {
                j = self.read("UMLClassifier", n)?;
                let x = self.read_obj_array(n, "ExtensionPoints");
                set(&mut j, "extensionPoints", Self::objs_value(&x));
            }
            "UMLExtensionPoint" => {
                j = self.read("UMLModelElement", n)?;
                set(&mut j, "location", attr_or(n, "Location", json!("")));
            }
            "UMLInclude" | "UMLExtend" => {
                j = self.read("UMLRelationship", n)?;
                set(&mut j, "source", rf(n, "Base"));
                set(&mut j, "target", rf(n, if ty == "UMLInclude" { "Addition" } else { "Extension" }));
            }

            // ---- state machines (uml-readers.js:1167)
            "UMLSignalEvent" | "UMLCallEvent" | "UMLTimeEvent" | "UMLChangeEvent" => {
                j = self.read("UMLEvent", n)?;
                match ty {
                    "UMLSignalEvent" => {
                        set(&mut j, "kind", json!("signal"));
                        set(&mut j, "targetSignal", rf(n, "Signal"));
                    }
                    "UMLCallEvent" => {
                        set(&mut j, "kind", json!("call"));
                        set(&mut j, "targetOperation", rf(n, "Operation"));
                    }
                    "UMLTimeEvent" => {
                        set(&mut j, "kind", json!("time"));
                        set(&mut j, "expression", attr_or(n, "When", json!("")));
                    }
                    _ => {
                        set(&mut j, "kind", json!("change"));
                        set(&mut j, "expression", attr_or(n, "ChangeExpression", json!("")));
                    }
                }
                set(&mut j, "_type", json!("UMLEvent"));
            }
            "UMLStateMachine" => {
                j = self.read("UMLModelElement", n)?;
                set(&mut j, "regions", json!([]));
                let top = self.read_obj(n, "Top");
                set(&mut j, "__top", top.map(ph).unwrap_or(Value::Null));
                let t = self.read_obj_array(n, "Transitions");
                set(&mut j, "__transitions", Self::objs_value(&t));
            }
            "UMLStateVertex" => {
                j = self.read("UMLModelElement", n)?;
                set(&mut j, "regions", json!([]));
            }
            "UMLPseudostate" => {
                j = self.read("UMLStateVertex", n)?;
                set(&mut j, "kind", en_or(n, "PseudostateKind", json!("initial")));
            }
            "UMLSyncState" | "UMLStubState" => j = self.read("UMLStateVertex", n)?,
            "UMLState" => {
                j = self.read("UMLStateVertex", n)?;
                for (k, v1) in [("entryActivities", "EntryActions"), ("doActivities", "DoActivities"), ("exitActivities", "ExitActions")] {
                    let a = self.read_obj_array(n, v1);
                    set(&mut j, k, Self::objs_value(&a));
                }
            }
            "UMLCompositeState" => {
                j = self.read("UMLState", n)?;
                let s = self.read_obj_array(n, "Subvertices");
                set(&mut j, "__subvertices", Self::objs_value(&s));
                self.add_tag(n, &mut j, "_isConcurrent", attr_null(n, "IsConcurrent"));
                set(&mut j, "_type", json!("UMLState"));
            }
            "UMLSubmachineState" => {
                j = self.read("UMLCompositeState", n)?;
                set(&mut j, "submachine", rf(n, "Submachine"));
                set(&mut j, "_type", json!("UMLState"));
            }
            "UMLSimpleState" => {
                // the original keeps "UMLSimpleState", which the current metamodel does not have (so
                // the state is lost); UMLCraft makes it a UMLState
                j = self.read("UMLState", n)?;
                set(&mut j, "_type", json!("UMLState"));
            }
            "UMLFinalState" => j = self.read("UMLState", n)?,
            "UMLFlowFinalState" => {
                j = self.read("UMLFinalState", n)?;
                set(&mut j, "kind", json!("terminate"));
                set(&mut j, "_type", json!("UMLPseudostate"));
            }
            "UMLTransition" => {
                j = self.read("UMLModelElement", n)?;
                set(&mut j, "source", rf(n, "Source"));
                set(&mut j, "target", rf(n, "Target"));
                set(&mut j, "guard", attr_or(n, "GuardCondition", json!("")));
                let e = self.read_obj_array(n, "Effects");
                set(&mut j, "effects", Self::objs_value(&e));
                let t = self.read_obj_array(n, "Triggers");
                set(&mut j, "triggers", Self::objs_value(&t));
            }

            // ---- activity graphs (uml-readers.js:1292)
            "UMLActivityGraph" => {
                j = self.read("UMLStateMachine", n)?;
                set(&mut j, "nodes", json!([]));
                set(&mut j, "edges", json!([]));
                let g = self.read_obj_array(n, "Partitions");
                set(&mut j, "groups", Self::objs_value(&g));
                set(&mut j, "_type", json!("UMLActivity"));
            }
            "UMLPartition" => {
                j = self.read("UMLModelElement", n)?;
                set(&mut j, "_type", json!("UMLActivityPartition"));
            }
            "UMLActionState" | "UMLSubactivityState" => {
                if ty == "UMLActionState" {
                    j = self.read("UMLSimpleState", n)?;
                } else {
                    j = self.read("UMLSubmachineState", n)?;
                    set(&mut j, "subactivity", rf(n, "Submachine"));
                }
                self.add_tag(n, &mut j, "_isDynamic", attr_null(n, "IsDynamic"));
                self.add_tag(n, &mut j, "_dynamicArguments", attr_null(n, "DynamicArguments"));
                self.add_tag(n, &mut j, "_dynamicMultiplicity", attr_null(n, "DynamicMultiplicity"));
                set(&mut j, "_type", json!("UMLAction"));
            }
            "UMLObjectFlowState" => {
                j = self.read("UMLSimpleState", n)?;
                set(&mut j, "type", rf(n, "Type_"));
                self.add_tag(n, &mut j, "_isSynch", attr_null(n, "IsSynch"));
                set(&mut j, "_type", json!("UMLObjectNode"));
            }
            "UMLSignalSendState" | "UMLSignalAcceptState" => {
                j = self.read("UMLActionState", n)?;
                set(&mut j, "kind", json!(if ty == "UMLSignalSendState" { "sendSignal" } else { "acceptSignal" }));
                set(&mut j, "_type", json!("UMLAction"));
            }

            // ---- model management (uml-readers.js:1354)
            "UMLProject" => {
                j = self.read("UMLPackage", n)?;
                set(&mut j, "name", attr_or(n, "Title", json!("")));
                set(&mut j, "author", attr_or(n, "Author", json!("")));
                set(&mut j, "company", attr_or(n, "Company", json!("")));
                set(&mut j, "copyright", attr_or(n, "Copyright", json!("")));
                set(&mut j, "_type", json!("Project"));
            }
            "UMLModel" => j = self.read("UMLPackage", n)?,
            "UMLSubsystem" => {
                j = self.read("UMLPackage", n)?;
                set(&mut j, "isIndirectlyInstantiated", attr_or(n, "IsInstantiable", json!(true)));
            }

            // ---- UML views (uml-readers.js:1386)
            "UMLNameCompartmentView" => {
                j = self.read("NodeView", n)?;
                self.add_sub_view(n, "StereotypeLabel", &mut j, "stereotypeLabel", None);
                self.add_sub_view(n, "NameLabel", &mut j, "nameLabel", None);
                self.add_sub_view(n, "NamespaceLabel", &mut j, "namespaceLabel", Some(Self::label_view()));
                self.add_sub_view(n, "PropertyLabel", &mut j, "propertyLabel", Some(Self::label_view()));
            }
            "UMLListCompartmentView" => j = self.read("NodeView", n)?,
            "UMLAttributeCompartmentView"
            | "UMLOperationCompartmentView"
            | "UMLEnumerationLiteralCompartmentView"
            | "UMLTemplateParameterCompartmentView"
            | "UMLQualifierCompartmentView"
            | "UMLExtensionPointCompartmentView"
            | "UMLSlotCompartmentView" => j = self.read("UMLListCompartmentView", n)?,
            "UMLGeneralNodeView" => {
                j = self.read("NodeView", n)?;
                set(&mut j, "stereotypeDisplay", en_or(n, "StereotypeDisplay", json!("label")));
                set(&mut j, "showNamespace", attr_or(n, "ShowParentName", json!(false)));
                set(&mut j, "showProperty", attr_or(n, "ShowProperty", json!(false)));
                self.add_sub_view(n, "NameCompartment", &mut j, "nameCompartment", None);
            }
            "UMLGeneralEdgeView" => {
                j = self.read("EdgeView", n)?;
                set(&mut j, "showProperty", attr_or(n, "ShowProperty", json!(false)));
                self.edge_labels(n, &mut j, true);
            }
            "UMLClassifierView" | "UMLClassView" | "UMLInterfaceView" | "UMLSignalView" | "UMLArtifactView" | "UMLActorView" | "UMLComponentView" | "UMLNodeView" => {
                j = self.read("UMLGeneralNodeView", n)?;
                set(&mut j, "showOperationSignature", attr_or(n, "ShowOperationSignature", json!(true)));
                set(&mut j, "showVisibility", attr_or(n, "ShowCompartmentVisibility", json!(true)));
                let hidden = |ty: &str| json!({ "_type": ty, "parentStyle": true, "visible": false });
                let ac = self.add_sub_view(n, "AttributeCompartment", &mut j, "attributeCompartment", Some(hidden("UMLAttributeCompartmentView")));
                let oc = self.add_sub_view(n, "OperationCompartment", &mut j, "operationCompartment", Some(hidden("UMLOperationCompartmentView")));
                let tp = json!({ "_type": "UMLTemplateParameterCompartmentView", "parentStyle": true });
                self.add_sub_view(n, "TemplateParameterCompartment", &mut j, "templateParameterCompartment", Some(tp));
                set(&mut j, "suppressAttributes", json!(!self.visible(ac)));
                set(&mut j, "suppressOperations", json!(!self.visible(oc)));
            }
            "UMLExceptionView" => {
                j = self.read("UMLSignalView", n)?;
                set(&mut j, "_type", json!("UMLSignalView"));
            }
            "UMLPackageView" | "UMLSubsystemView" | "UMLModelView" | "UMLPartView" | "UMLCustomComponentView" | "UMLCustomNodeView" | "UMLCustomColObjectView" => {
                j = self.read("UMLGeneralNodeView", n)?
            }
            "UMLComponentInstanceView" => j = self.read("UMLCustomComponentView", n)?,
            "UMLNodeInstanceView" => j = self.read("UMLCustomNodeView", n)?,
            "UMLEnumerationView" => {
                j = self.read("UMLGeneralNodeView", n)?;
                set(&mut j, "showOperationSignature", attr_null(n, "ShowOperationSignature"));
                set(&mut j, "showVisibility", attr_null(n, "ShowCompartmentVisibility"));
                let tp = json!({ "_type": "UMLTemplateParameterCompartmentView", "parentStyle": true });
                self.add_sub_view(n, "TemplateParameterCompartment", &mut j, "templateParameterCompartment", Some(tp));
                let ac = json!({ "_type": "UMLAttributeCompartmentView", "parentStyle": true });
                self.add_sub_view(n, "AttributeCompartment", &mut j, "attributeCompartment", Some(ac));
                let lc = self.add_sub_view(n, "EnumerationLiteralCompartment", &mut j, "enumerationLiteralCompartment", None);
                let oc = self.add_sub_view(n, "OperationCompartment", &mut j, "operationCompartment", None);
                // the original fails on a missing compartment; UMLCraft shows it
                set(&mut j, "suppressLiterals", json!(lc.is_some() && !self.visible(lc)));
                set(&mut j, "suppressOperations", json!(oc.is_some() && !self.visible(oc)));
            }
            "UMLGeneralizationView" | "UMLDependencyView" | "UMLIncludeView" | "UMLExtendView" | "UMLTransitionView" => j = self.read("UMLGeneralEdgeView", n)?,
            "UMLRealizationView" => {
                j = self.read("UMLGeneralEdgeView", n)?;
                set(&mut j, "_type", json!("UMLInterfaceRealizationView"));
            }
            "UMLAssociationView" | "UMLAssociationRoleView" => {
                j = self.read("UMLGeneralEdgeView", n)?;
                set(&mut j, "showVisibility", attr_or(n, "ShowCompartmentVisibility", json!(true)));
                for (v1, field) in [
                    ("HeadRoleNameLabel", "headRoleNameLabel"),
                    ("TailRoleNameLabel", "tailRoleNameLabel"),
                    ("HeadMultiplicityLabel", "headMultiplicityLabel"),
                    ("TailMultiplicityLabel", "tailMultiplicityLabel"),
                    ("HeadPropertyLabel", "headPropertyLabel"),
                    ("TailPropertyLabel", "tailPropertyLabel"),
                    ("HeadQualifierCompartment", "headQualifiersCompartment"),
                    ("TailQualifierCompartment", "tailQualifiersCompartment"),
                ] {
                    self.add_sub_view(n, v1, &mut j, field, None);
                }
                if ty == "UMLAssociationRoleView" {
                    set(&mut j, "_type", json!("UMLConnectorView"));
                }
            }
            "UMLPortView" => {
                j = self.read("NodeView", n)?;
                set(&mut j, "showProperty", attr_or(n, "ShowProperty", json!(false)));
                for (v1, field) in [("StereotypeLabel", "stereotypeLabel"), ("NameLabel", "nameLabel"), ("PropertyLabel", "propertyLabel")] {
                    self.add_sub_view(n, v1, &mut j, field, None);
                }
            }
            "UMLConnectorView" => {
                j = self.read("UMLGeneralEdgeView", n)?;
                for (v1, field) in [
                    ("HeadRoleNameLabel", "headRoleNameLabel"),
                    ("TailRoleNameLabel", "tailRoleNameLabel"),
                    ("HeadMultiplicityLabel", "headMultiplicityLabel"),
                    ("TailMultiplicityLabel", "tailMultiplicityLabel"),
                ] {
                    self.add_sub_view(n, v1, &mut j, field, None);
                }
            }
            "UMLUseCaseView" => {
                j = self.read("UMLClassifierView", n)?;
                let x = json!({ "_type": "UMLExtensionPointCompartmentView", "parentStyle": true });
                self.add_sub_view(n, "ExtensionPointCompartment", &mut j, "extensionPointCompartment", Some(x));
            }
            "UMLSystemBoundaryView" => {
                j = self.read("NodeView", n)?;
                set(&mut j, "_type", json!("RectangleView"));
            }
            "UMLLifeLineView" => {
                j = self.read("NodeView", n)?;
                set(&mut j, "_type", json!("UMLLinePartView"));
            }
            "UMLCustomSeqObjectView" | "UMLSeqObjectView" | "UMLSeqClassifierRoleView" => {
                j = self.read("UMLGeneralNodeView", n)?;
                self.add_sub_view(n, "LifeLine", &mut j, "linePart", None);
                if ty != "UMLCustomSeqObjectView" {
                    set(&mut j, "_type", json!("UMLSeqLifelineView"));
                }
            }
            "UMLActivationView" | "UMLFinalStateView" | "ShapeView" => j = self.read("NodeView", n)?,
            "UMLCustomSeqMessageView" | "UMLSeqMessageView" | "UMLSeqStimulusView" => {
                j = self.read("EdgeView", n)?;
                set(&mut j, "showProperty", attr_or(n, "ShowProperty", json!(false)));
                self.edge_labels(n, &mut j, true);
                self.add_sub_view(n, "Activation", &mut j, "activation", None);
                if ty == "UMLSeqStimulusView" {
                    set(&mut j, "_type", json!("UMLSeqMessageView"));
                }
            }
            "UMLCustomFrameView" => {
                j = self.read("NodeView", n)?;
                self.add_sub_view(n, "NameLabel", &mut j, "nameLabel", None);
                self.add_sub_view(n, "FrameTypeLabel", &mut j, "frameTypeLabel", None);
            }
            "UMLCombinedFragmentView" => {
                j = self.read("UMLCustomFrameView", n)?;
                // the operand compartment takes the frame's own geometry (uml-readers.js:1750)
                // (read without registering again the views it reads a second time)
                let saved = self.id_map.clone();
                let mut oc = self.read("NodeView", n)?;
                self.id_map = saved;
                // the original also reads the frame's contained views a second time here, which puts
                // them twice in the diagram; UMLCraft leaves them out
                set(&mut oc, "containedViews", json!([]));
                oc.remove("_id");
                let oc = self.new_obj("UMLInteractionOperandCompartmentView", Value::Object(oc));
                let id = self.id(oc);
                set(&mut j, "operandCompartment", refv(&id));
                self.add_to(n, &mut j, "subViews", Some(oc));
            }
            "UMLColObjectView" => {
                j = self.read("UMLCustomColObjectView", n)?;
                self.add_sub_view(n, "SlotCompartment", &mut j, "slotCompartment", None);
            }
            "UMLColClassifierRoleView" => {
                j = self.read("UMLCustomColObjectView", n)?;
                set(&mut j, "_type", json!("UMLCommLifelineView"));
            }
            "UMLLinkView" => {
                j = self.read("UMLGeneralEdgeView", n)?;
                self.add_sub_view(n, "HeadEndLabel", &mut j, "headRoleNameLabel", None);
                self.add_sub_view(n, "TailEndLabel", &mut j, "tailRoleNameLabel", None);
            }
            "UMLCustomColMessageView" | "UMLColMessageView" | "UMLColStimulusView" => {
                j = self.read("EdgeNodeView", n)?;
                set(&mut j, "showProperty", attr_or(n, "ShowProperty", json!(false)));
                self.edge_labels(n, &mut j, false);
                if ty != "UMLCustomColMessageView" {
                    set(&mut j, "_type", json!("UMLCommMessageView"));
                }
            }
            "UMLInternalTransitionCompartmentView" => {
                j = self.read("UMLListCompartmentView", n)?;
                set(&mut j, "_type", json!("UMLInternalActivityCompartmentView"));
            }
            "UMLCustomStateView" | "UMLStateView" | "UMLSubmachineStateView" => {
                j = self.read("UMLGeneralNodeView", n)?;
                let ia = json!({ "_type": "UMLInternalActivityCompartmentView" });
                self.add_sub_view(n, "InternalTransitionCompartment", &mut j, "internalActivityCompartment", Some(ia));
                let dc = json!({ "_type": "UMLDecompositionCompartmentView", "subViews": [] });
                self.add_sub_view(n, "DecompositionCompartment", &mut j, "decompositionCompartment", Some(dc));
                if ty == "UMLSubmachineStateView" {
                    set(&mut j, "_type", json!("UMLStateView"));
                }
            }
            "UMLPseudostateView" | "UMLFlowFinalStateView" => {
                j = self.read("NodeView", n)?;
                for (v1, field) in [("StereotypeLabel", "stereotypeLabel"), ("NameLabel", "nameLabel"), ("PropertyLabel", "propertyLabel")] {
                    self.add_sub_view(n, v1, &mut j, field, Some(json!({ "_type": "NodeLabelView" })));
                }
                set(&mut j, "_type", json!("UMLPseudostateView"));
            }
            "UMLActionStateView" | "UMLSubactivityStateView" | "UMLSignalAcceptStateView" | "UMLSignalSendStateView" => {
                j = self.read("UMLGeneralNodeView", n)?;
                set(&mut j, "_type", json!("UMLActionView"));
            }
            "UMLObjectFlowStateView" => {
                j = self.read("UMLGeneralNodeView", n)?;
                set(&mut j, "_type", json!("UMLObjectNodeView"));
            }
            "UMLSwimlaneView" => {
                j = self.read("NodeView", n)?;
                self.add_sub_view(n, "NameLabel", &mut j, "nameLabel", None);
                set(&mut j, "isVertical", json!(en(n, "Direction").and_then(|v| v.as_i64()) != Some(1)));
            }
            "UMLCustomTextView" | "UMLTextView" | "UMLNoteView" => {
                j = self.read("NodeView", n)?;
                set(&mut j, "text", attr_or(n, "Text", json!("")));
            }
            "UMLNoteLinkView" => j = self.read("EdgeView", n)?,
            "RectangleView" | "RoundRectView" | "EllipseView" => j = self.read("ShapeView", n)?,
            _ => return None,
        }
        Some(j)
    }

    /// TypeExpression, or the Type_ reference when there is one.
    fn type_of(n: Node) -> Value {
        let r = rf(n, "Type_");
        if ref_id(Some(&r)).is_some() {
            r
        } else {
            attr_or(n, "TypeExpression", json!(""))
        }
    }

    fn visible(&self, sub: Option<usize>) -> bool {
        sub.and_then(|s| self.objs[s].get("visible")).and_then(|v| v.as_bool()).unwrap_or(true)
    }

    /// Stereotype, name and property labels of an edge (with EdgeLabelView defaults) or of a
    /// communication message (without).
    fn edge_labels(&mut self, n: Node, j: &mut Map<String, Value>, defaults: bool) {
        for (v1, field) in [("StereotypeLabel", "stereotypeLabel"), ("NameLabel", "nameLabel"), ("PropertyLabel", "propertyLabel")] {
            self.add_sub_view(n, v1, j, field, defaults.then(Self::edge_label_view));
        }
    }

    /// messageSort and signature from the V1 action of a message (uml-readers.js:976).
    fn message_sort(&mut self, n: Node, j: &mut Map<String, Value>) {
        let Some(a) = self.read_obj(n, "Action") else { return };
        // matched on the V1 type: the original tests the converted type, so its "UMLUninterpretedAction"
        // and "UMLTerminateAction" cases never match and those messages keep the default sort
        let v1 = tagged(n, "OBJ", "Action").first().copied().and_then(|c| c.attribute("type")).unwrap_or("").to_string();
        let asynch = self.objs[a].get("isAsynchronous").and_then(|v| v.as_bool()) == Some(true);
        let call = if asynch { "asynchCall" } else { "synchCall" };
        let field = |k: &str| self.objs[a].get(k).cloned().unwrap_or(Value::Null);
        let (sort, sig) = match v1.as_str() {
            "UMLCreateAction" => ("createMessage", field("instantiation")),
            "UMLCallAction" => (call, field("operation")),
            "UMLReturnAction" => ("reply", Value::Null),
            "UMLSendAction" => ("asynchSignal", field("signal")),
            "UMLTerminalAction" | "UMLUninterpretedAction" => (call, Value::Null),
            "UMLDestroyAction" => ("deleteMessage", Value::Null),
            _ => return,
        };
        set(j, "messageSort", json!(sort));
        if !sig.is_null() {
            set(j, "signature", sig);
        }
    }

    // -------------------------------------------- post-processors (uml-readers.js:1971-2312)

    fn all(&self) -> Vec<usize> {
        self.id_map.values().copied().collect()
    }

    fn postprocess(&mut self) {
        // contained views move to the diagram's ownedViews
        for e in self.all() {
            let cv = self.list(e, "containedViews");
            if cv.is_empty() {
                continue;
            }
            let Some(d) = self.diagram_of(e) else { continue };
            let did = self.id(d);
            let mut refs = vec![];
            for v in cv {
                set(&mut self.objs[v], "_parent", refv(&did));
                self.arr_push(d, "ownedViews", ph(v));
                refs.push(refv(&self.id(v)));
            }
            set(&mut self.objs[e], "containedViews", Value::Array(refs));
        }
        // a realization of a component is a component realization
        for e in self.all() {
            if self.ty(e) == "UMLInterfaceRealization" {
                if let Some(t) = self.deref(e, "target") {
                    if matches!(self.ty(t), "UMLComponent" | "UMLSubsystem" | "UMLUseCaseSubject") {
                        set(&mut self.objs[e], "_type", json!("UMLComponentRealization"));
                    }
                }
            }
        }
        for e in self.all() {
            if self.ty(e) == "UMLInterfaceRealizationView" {
                // the realizations built from Residents are not in the id map, as in the original
                if self.deref(e, "model").map(|m| self.ty(m) == "UMLComponentRealization").unwrap_or(false) {
                    set(&mut self.objs[e], "_type", json!("UMLComponentRealizationView"));
                }
            }
        }
        // objects and links of a collaboration become attributes and connectors
        for e in self.all() {
            if self.ty(e) == "UMLObject" && self.parent(e).map(|o| self.ty(o) == "UMLCollaboration").unwrap_or(false) {
                let o = &mut self.objs[e];
                let c = o.get("classifier").cloned().unwrap_or(Value::Null);
                o.insert("type".into(), c);
                if o.get("isMultiInstance").and_then(|v| v.as_bool()) == Some(true) {
                    o.insert("multiplicity".into(), json!("0..*"));
                }
                o.insert("_type".into(), json!("UMLAttribute"));
            }
        }
        for e in self.all() {
            if self.ty(e) == "UMLLink" && self.parent(e).map(|o| self.ty(o) == "UMLCollaboration").unwrap_or(false) {
                set(&mut self.objs[e], "_type", json!("UMLConnector"));
                for k in ["end1", "end2"] {
                    if let Some(end) = self.child(e, k) {
                        let inst = self.objs[end].get("instance").cloned().unwrap_or(Value::Null);
                        set(&mut self.objs[end], "_type", json!("UMLConnectorEnd"));
                        set(&mut self.objs[end], "reference", inst);
                    }
                }
            }
        }
        // attributes in ownedElements move to attributes
        for e in self.all() {
            if self.ty(e) != "UMLAttribute" {
                continue;
            }
            let Some(o) = self.parent(e) else { continue };
            let owned = self.objs[o].get("ownedElements").and_then(|v| v.as_array()).and_then(|a| a.iter().position(|x| ph_of(x) == Some(e)));
            if let (Some(pos), true) = (owned, self.objs[o].get("attributes").map(|v| v.is_array()).unwrap_or(false)) {
                if let Some(Value::Array(a)) = self.objs[o].get_mut("ownedElements") {
                    a.remove(pos);
                }
                self.arr_push(o, "attributes", ph(e));
            }
        }
        for e in self.all() {
            if self.ty(e) == "UMLColObjectView" {
                let comm = self.parent(e).map(|d| self.ty(d) == "UMLCommunicationDiagram").unwrap_or(false);
                set(&mut self.objs[e], "_type", json!(if comm { "UMLCommLifelineView" } else { "UMLObjectView" }));
            }
        }
        // one lifeline per lifeline view, in the interaction that owns the diagram
        for e in self.all() {
            if !matches!(self.ty(e), "UMLSeqLifelineView" | "UMLCommLifelineView") {
                continue;
            }
            let role = self.deref(e, "model");
            let inter = self.parent(e).and_then(|d| self.parent(d));
            if let (Some(role), Some(inter)) = (role, inter) {
                if self.ty(inter) != "UMLInteraction" {
                    continue;
                }
                let r = &self.objs[role];
                let props = json!({
                    "_parent": refv(&self.id(inter)),
                    "name": r.get("name").cloned().unwrap_or(Value::Null),
                    "stereotype": r.get("stereotype").cloned().unwrap_or(Value::Null),
                    "visibility": r.get("visibility").cloned().unwrap_or(Value::Null),
                    "represent": refv(&self.id(role)),
                });
                let l = self.new_obj("UMLLifeline", props);
                self.arr_push(inter, "participants", ph(l));
                let lid = self.id(l);
                set(&mut self.objs[e], "model", refv(&lid));
            }
        }
        // messages go between the lifelines of their sender and receiver
        for e in self.all() {
            if self.ty(e) != "UMLMessage" {
                continue;
            }
            let (Some(inter), Some(s), Some(t)) = (self.parent(e), self.deref(e, "source"), self.deref(e, "target")) else { continue };
            if self.ty(inter) != "UMLInteraction" {
                continue;
            }
            let (sid, tid) = (self.id(s), self.id(t));
            let (mut sl, mut tl) = (None, None);
            for p in self.list(inter, "participants") {
                let rep = ref_id(self.objs[p].get("represent")).map(|x| x.to_string());
                if rep.as_deref() == Some(sid.as_str()) {
                    sl = Some(self.id(p));
                }
                if rep.as_deref() == Some(tid.as_str()) {
                    tl = Some(self.id(p));
                }
            }
            if let (Some(sl), Some(tl)) = (sl, tl) {
                set(&mut self.objs[e], "source", refv(&sl));
                set(&mut self.objs[e], "target", refv(&tl));
            }
        }
        // the top state of a state machine becomes its region
        for e in self.all() {
            if self.ty(e) != "UMLStateMachine" {
                continue;
            }
            let Some(top) = self.child(e, "__top") else { continue };
            let verts = self.list(top, "__subvertices");
            let trans = self.list(e, "__transitions");
            let eid = self.id(e);
            let region = self.new_obj("UMLRegion", json!({ "_parent": refv(&eid), "vertices": Self::objs_value(&verts), "transitions": Self::objs_value(&trans) }));
            let rid = self.id(region);
            for x in verts.iter().chain(trans.iter()) {
                set(&mut self.objs[*x], "_parent", refv(&rid));
            }
            self.objs[top].remove("__subvertices");
            self.objs[e].remove("__transitions");
            self.arr_push(e, "regions", ph(region));
        }
        // the substates of a composite state go to a region
        for e in self.all() {
            if self.ty(e) != "UMLState" || !self.objs[e].contains_key("__subvertices") {
                continue;
            }
            let verts = self.list(e, "__subvertices");
            let eid = self.id(e);
            let region = self.new_obj("UMLRegion", json!({ "_parent": refv(&eid), "vertices": Self::objs_value(&verts), "containedViews": [] }));
            let rid = self.id(region);
            for x in &verts {
                set(&mut self.objs[*x], "_parent", refv(&rid));
            }
            self.objs[e].remove("__subvertices");
            self.arr_push(e, "regions", ph(region));
        }
        // a state view inside another one goes to the region view of its decomposition compartment
        for e in self.all() {
            if self.ty(e) != "UMLStateView" {
                continue;
            }
            let Some(cv) = self.deref(e, "containerView") else { continue };
            if self.ty(cv) != "UMLStateView" {
                continue;
            }
            let region = self.deref(cv, "model").and_then(|m| self.list(m, "regions").first().copied());
            let Some(dc) = self.list(cv, "subViews").into_iter().filter(|s| self.ty(*s) == "UMLDecompositionCompartmentView").last() else { continue };
            set(&mut self.objs[cv], "containedViews", json!([]));
            let mut rv = self.list(dc, "subViews").first().copied();
            if rv.is_none() {
                let Some(region) = region else { continue };
                let rid = self.id(region);
                let v = self.new_obj("UMLRegionView", json!({ "model": refv(&rid), "containedViews": [] }));
                self.arr_push(dc, "subViews", ph(v));
                rv = Some(v);
            }
            let rv = rv.unwrap();
            if !self.objs[rv].get("containedViews").map(|v| v.is_array()).unwrap_or(false) {
                set(&mut self.objs[rv], "containedViews", json!([]));
            }
            let me = self.id(e);
            self.arr_push(rv, "containedViews", refv(&me));
            let rvid = self.id(rv);
            set(&mut self.objs[e], "containerView", refv(&rvid));
        }
        // a synchronization bar is a join when it has more incoming than outgoing transitions
        let transitions: Vec<usize> = self.all().into_iter().filter(|t| self.ty(*t) == "UMLTransition").collect();
        for e in self.all() {
            if self.ty(e) != "UMLPseudostate" || self.objs[e].get("kind").and_then(|k| k.as_i64()) != Some(0) {
                continue;
            }
            let me = self.id(e);
            let outgoing = transitions.iter().filter(|t| ref_id(self.objs[**t].get("source")) == Some(me.as_str())).count();
            let incoming = transitions.iter().filter(|t| ref_id(self.objs[**t].get("target")) == Some(me.as_str())).count();
            set(&mut self.objs[e], "kind", json!(if incoming > outgoing { "join" } else { "fork" }));
        }
        // an activity takes the vertices of its top state as nodes and its transitions as control flows
        for e in self.all() {
            if self.ty(e) != "UMLActivity" {
                continue;
            }
            let eid = self.id(e);
            let region = self.child(e, "__top").and_then(|t| self.list(t, "regions").first().copied());
            if let Some(r) = region {
                for v in self.list(r, "vertices") {
                    set(&mut self.objs[v], "_parent", refv(&eid));
                    self.arr_push(e, "nodes", ph(v));
                }
            }
            for t in self.list(e, "__transitions") {
                set(&mut self.objs[t], "_parent", refv(&eid));
                set(&mut self.objs[t], "_type", json!("UMLControlFlow"));
                self.arr_push(e, "edges", ph(t));
            }
        }
        for e in self.all() {
            if self.ty(e) != "UMLActivity" {
                continue;
            }
            for node in self.list(e, "nodes") {
                let new = match self.ty(node) {
                    "UMLPseudostate" => match self.objs[node].get("kind") {
                        Some(k) if k.as_str() == Some("initial") => Some("UMLInitialNode"),
                        Some(k) if k.as_str() == Some("join") => Some("UMLJoinNode"),
                        Some(k) if k.as_str() == Some("fork") => Some("UMLForkNode"),
                        Some(k) if k.as_str() == Some("terminate") => Some("UMLFlowFinalNode"),
                        Some(k) if k.as_i64() == Some(1) => Some("UMLDecisionNode"),
                        _ => None,
                    },
                    "UMLFinalState" => Some("UMLActivityFinalNode"),
                    _ => None,
                };
                if let Some(t) = new {
                    set(&mut self.objs[node], "_type", json!(t));
                }
            }
        }
        for e in self.all() {
            let ty = self.ty(e).to_string();
            if ty != "UMLPseudostateView" && ty != "UMLTransitionView" {
                continue;
            }
            if self.diagram_of(e).map(|d| self.ty(d) != "UMLActivityDiagram").unwrap_or(true) {
                continue;
            }
            if ty == "UMLTransitionView" {
                set(&mut self.objs[e], "_type", json!("UMLControlFlowView"));
            } else if self.deref(e, "model").map(|m| CONTROL_NODES.contains(&self.ty(m))).unwrap_or(false) {
                set(&mut self.objs[e], "_type", json!("UMLControlNodeView"));
            }
        }
        for e in self.all() {
            if self.ty(e) != "UMLControlFlowView" {
                continue;
            }
            let Some(m) = self.deref(e, "model") else { continue };
            let (s, t) = (self.deref(m, "source"), self.deref(m, "target"));
            let obj = |x: Option<usize>| x.map(|x| OBJECT_NODES.contains(&self.ty(x))).unwrap_or(false);
            if s.is_some() && t.is_some() && (obj(s) || obj(t)) {
                set(&mut self.objs[e], "_type", json!("UMLObjectFlowView"));
                set(&mut self.objs[m], "_type", json!("UMLObjectFlow"));
            }
        }
        // a decision outside an activity is a choice (the original leaves the number 1, which is no kind)
        for e in self.all() {
            if self.ty(e) == "UMLPseudostate" && self.objs[e].get("kind").and_then(|k| k.as_i64()) == Some(1) {
                set(&mut self.objs[e], "kind", json!("choice"));
            }
        }
    }

    // -------------------------------------------- output

    /// The .mdj JSON of an object: `_type`, `_id`, `_parent` first, the other fields in the order the
    /// file format writes them; null, empty lists, default values and fields the current metamodel
    /// does not have are left out (the original drops them when it loads the JSON, repository.js:1597).
    fn expand(&self, i: usize, parent: Option<&str>) -> Value {
        let o = &self.objs[i];
        let ty = self.ty(i).to_string();
        let id = self.id(i);
        let mut m = Map::new();
        m.insert("_type".into(), json!(ty));
        m.insert("_id".into(), json!(id));
        if let Some(p) = parent {
            m.insert("_parent".into(), refv(p));
        }
        let mut rest: Vec<(&String, Value)> = vec![];
        for (k, v) in o {
            if k.starts_with('_') || DROPPED_FIELDS.contains(&k.as_str()) || (ty == "UMLOpaqueBehavior" && (k == "target" || k == "script")) {
                continue;
            }
            let v = match v {
                Value::Array(a) if a.iter().any(|x| ph_of(x).is_some()) => {
                    Value::Array(a.iter().filter_map(|x| ph_of(x)).filter(|c| self.kept(*c)).map(|c| self.expand(c, Some(&id))).collect())
                }
                x if ph_of(x).is_some() => {
                    let c = ph_of(x).unwrap();
                    if !self.kept(c) {
                        continue;
                    }
                    self.expand(c, Some(&id))
                }
                x => x.clone(),
            };
            let default = meta::default_of(&ty, k);
            let empty = v.is_null() || v.as_array().map(|a| a.is_empty()).unwrap_or(false) || (v.as_str() == Some("") && default.is_some());
            if empty || default.as_ref().map(|d| crate::model::json_eq(d, &v)).unwrap_or(false) {
                continue;
            }
            rest.push((k, v));
        }
        rest.sort_by_key(|(k, _)| meta::order_of(&ty, k).unwrap_or(usize::MAX));
        for (k, v) in rest {
            m.insert(k.clone(), v);
        }
        Value::Object(m)
    }

    /// Objects of a V1 type the current metamodel does not have are left out, as the original's
    /// loader drops them.
    fn kept(&self, i: usize) -> bool {
        let ty = self.ty(i);
        !(V1_ONLY.contains(&ty) || ty.starts_with("UMLCustom") || (ty.starts_with("UML") && ty.ends_with("DiagramView")))
    }
}

const CONTROL_NODES: &[&str] =
    &["UMLControlNode", "UMLInitialNode", "UMLFinalNode", "UMLActivityFinalNode", "UMLFlowFinalNode", "UMLForkNode", "UMLJoinNode", "UMLMergeNode", "UMLDecisionNode"];

const OBJECT_NODES: &[&str] = &["UMLObjectNode", "UMLCentralBufferNode", "UMLDataStoreNode", "UMLActivityParameterNode"];

/// Types the readers keep but the current metamodel does not have.
const V1_ONLY: &[&str] = &[
    "DiagramView",
    "TaggedValue",
    "UMLElement",
    "UMLGeneralizableElement",
    "UMLNamespace",
    "UMLRelationship",
    "UMLCreateAction",
    "UMLCallAction",
    "UMLReturnAction",
    "UMLSendAction",
    "UMLTerminalAction",
    "UMLDestroyAction",
    "UMLStateVertex",
    "UMLSyncState",
    "UMLStubState",
    "UMLColObjectView",
];

/// Fields the readers set that no type of the current metamodel has (V1 leftovers).
const DROPPED_FIELDS: &[&str] =
    &["DefaultValue", "dataValue", "instance", "instantiation", "isAsynchronous", "isRoot", "isSpecification", "operation", "profileName", "recurrence", "tagDefinitionSetName"];
