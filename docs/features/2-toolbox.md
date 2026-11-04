# Toolbox items

Part of the [StarUML feature inventory](README.md); the legend and how to update a status are there.

Every toolbox item of every notation, from the `toolbox/*.json` files of StarUML 7.1.1. One row per item.

UMLCraft shows the toolbox groups of the diagram kinds it edits: class, package, object, composite structure, component, deployment, use case, information flow, profile and statechart diagrams; every other diagram kind gets Annotations only (src/ui.rs:1523-1536).

| Group | Diagram types | Item | UMLCraft | Source |
|---|---|---|---|---|
| Robustness `uml.robustness` | UMLClassDiagram | Boundary | ✅ | extensions/default/robustness/toolbox/robustness.json |
| Robustness `uml.robustness` | UMLClassDiagram | Entity | ✅ | extensions/default/robustness/toolbox/robustness.json |
| Robustness `uml.robustness` | UMLClassDiagram | Control | ✅ | extensions/default/robustness/toolbox/robustness.json |
| AWS `aws.elements` | AWSDiagram | AWS Group | ❌ | extensions/essential/aws/toolbox/aws.json |
| AWS `aws.elements` | AWSDiagram | AWS Generic Group | ❌ | extensions/essential/aws/toolbox/aws.json |
| AWS `aws.elements` | AWSDiagram | AWS Availability Zone | ❌ | extensions/essential/aws/toolbox/aws.json |
| AWS `aws.elements` | AWSDiagram | AWS Security Group | ❌ | extensions/essential/aws/toolbox/aws.json |
| AWS `aws.elements` | AWSDiagram | AWS Service | ❌ | extensions/essential/aws/toolbox/aws.json |
| AWS `aws.elements` | AWSDiagram | AWS Resource | ❌ | extensions/essential/aws/toolbox/aws.json |
| AWS `aws.elements` | AWSDiagram | AWS General Resource | ❌ | extensions/essential/aws/toolbox/aws.json |
| AWS `aws.elements` | AWSDiagram | AWS Callout | ❌ | extensions/essential/aws/toolbox/aws.json |
| AWS `aws.elements` | AWSDiagram | AWS Arrow | ❌ | extensions/essential/aws/toolbox/aws.json |
| Azure `azure.elements` | AzureDiagram | Azure Group | ❌ | extensions/essential/azure/toolbox/azure.json |
| Azure `azure.elements` | AzureDiagram | Azure Service | ❌ | extensions/essential/azure/toolbox/azure.json |
| Azure `azure.elements` | AzureDiagram | Azure Callout | ❌ | extensions/essential/azure/toolbox/azure.json |
| Azure `azure.elements` | AzureDiagram | Azure Connector | ❌ | extensions/essential/azure/toolbox/azure.json |
| BPMN Basic `bpmn.basic` | BPMNDiagram | Task | ❌ | extensions/essential/bpmn/toolbox/bpmn.json |
| BPMN Basic `bpmn.basic` | BPMNDiagram | Gateway | ❌ | extensions/essential/bpmn/toolbox/bpmn.json |
| BPMN Basic `bpmn.basic` | BPMNDiagram | Start Event | ❌ | extensions/essential/bpmn/toolbox/bpmn.json |
| BPMN Basic `bpmn.basic` | BPMNDiagram | End Event | ❌ | extensions/essential/bpmn/toolbox/bpmn.json |
| BPMN Basic `bpmn.basic` | BPMNDiagram | Sequence Flow | ❌ | extensions/essential/bpmn/toolbox/bpmn.json |
| BPMN Artifact `bpmn.artifact` | BPMNDiagram | Text Annotation | ❌ | extensions/essential/bpmn/toolbox/bpmn.json |
| BPMN Artifact `bpmn.artifact` | BPMNDiagram | Group | ❌ | extensions/essential/bpmn/toolbox/bpmn.json |
| BPMN Artifact `bpmn.artifact` | BPMNDiagram | Association | ❌ | extensions/essential/bpmn/toolbox/bpmn.json |
| BPMN Swimlanes `bpmn.swimlanes` | BPMNDiagram | Pool | ❌ | extensions/essential/bpmn/toolbox/bpmn.json |
| BPMN Swimlanes `bpmn.swimlanes` | BPMNDiagram | Pool (Vert) | ❌ | extensions/essential/bpmn/toolbox/bpmn.json |
| BPMN Swimlanes `bpmn.swimlanes` | BPMNDiagram | Lane | ❌ | extensions/essential/bpmn/toolbox/bpmn.json |
| BPMN Swimlanes `bpmn.swimlanes` | BPMNDiagram | Lane (Vert) | ❌ | extensions/essential/bpmn/toolbox/bpmn.json |
| BPMN Activities `bpmn.activities` | BPMNDiagram | Task | ❌ | extensions/essential/bpmn/toolbox/bpmn.json |
| BPMN Activities `bpmn.activities` | BPMNDiagram | Send Task | ❌ | extensions/essential/bpmn/toolbox/bpmn.json |
| BPMN Activities `bpmn.activities` | BPMNDiagram | Receive Task | ❌ | extensions/essential/bpmn/toolbox/bpmn.json |
| BPMN Activities `bpmn.activities` | BPMNDiagram | Service Task | ❌ | extensions/essential/bpmn/toolbox/bpmn.json |
| BPMN Activities `bpmn.activities` | BPMNDiagram | User Task | ❌ | extensions/essential/bpmn/toolbox/bpmn.json |
| BPMN Activities `bpmn.activities` | BPMNDiagram | Manual Task | ❌ | extensions/essential/bpmn/toolbox/bpmn.json |
| BPMN Activities `bpmn.activities` | BPMNDiagram | Business Rule Task | ❌ | extensions/essential/bpmn/toolbox/bpmn.json |
| BPMN Activities `bpmn.activities` | BPMNDiagram | Script Task | ❌ | extensions/essential/bpmn/toolbox/bpmn.json |
| BPMN Activities `bpmn.activities` | BPMNDiagram | Call Activity | ❌ | extensions/essential/bpmn/toolbox/bpmn.json |
| BPMN Activities `bpmn.activities` | BPMNDiagram | Sub Process | ❌ | extensions/essential/bpmn/toolbox/bpmn.json |
| BPMN Activities `bpmn.activities` | BPMNDiagram | Ad-Hoc Sub Process | ❌ | extensions/essential/bpmn/toolbox/bpmn.json |
| BPMN Activities `bpmn.activities` | BPMNDiagram | Transaction | ❌ | extensions/essential/bpmn/toolbox/bpmn.json |
| BPMN Gateways `bpmn.gateways` | BPMNDiagram | Gateway (Exclusive) | ❌ | extensions/essential/bpmn/toolbox/bpmn.json |
| BPMN Gateways `bpmn.gateways` | BPMNDiagram | Gateway (Event-Based) | ❌ | extensions/essential/bpmn/toolbox/bpmn.json |
| BPMN Gateways `bpmn.gateways` | BPMNDiagram | Gateway (Inclusive) | ❌ | extensions/essential/bpmn/toolbox/bpmn.json |
| BPMN Gateways `bpmn.gateways` | BPMNDiagram | Gateway (Complex) | ❌ | extensions/essential/bpmn/toolbox/bpmn.json |
| BPMN Gateways `bpmn.gateways` | BPMNDiagram | Gateway (Parallel) | ❌ | extensions/essential/bpmn/toolbox/bpmn.json |
| BPMN Events `bpmn.events` | BPMNDiagram | Start Event | ❌ | extensions/essential/bpmn/toolbox/bpmn.json |
| BPMN Events `bpmn.events` | BPMNDiagram | Intermediate Event (Throw) | ❌ | extensions/essential/bpmn/toolbox/bpmn.json |
| BPMN Events `bpmn.events` | BPMNDiagram | Intermediate Event (Catch) | ❌ | extensions/essential/bpmn/toolbox/bpmn.json |
| BPMN Events `bpmn.events` | BPMNDiagram | Boundary Event | ❌ | extensions/essential/bpmn/toolbox/bpmn.json |
| BPMN Events `bpmn.events` | BPMNDiagram | End Event | ❌ | extensions/essential/bpmn/toolbox/bpmn.json |
| BPMN Data `bpmn.data` | BPMNDiagram | Data Object | ❌ | extensions/essential/bpmn/toolbox/bpmn.json |
| BPMN Data `bpmn.data` | BPMNDiagram | Data Store | ❌ | extensions/essential/bpmn/toolbox/bpmn.json |
| BPMN Data `bpmn.data` | BPMNDiagram | Data Input | ❌ | extensions/essential/bpmn/toolbox/bpmn.json |
| BPMN Data `bpmn.data` | BPMNDiagram | Data Output | ❌ | extensions/essential/bpmn/toolbox/bpmn.json |
| BPMN Data `bpmn.data` | BPMNDiagram | Data Association | ❌ | extensions/essential/bpmn/toolbox/bpmn.json |
| BPMN Message `bpmn.message` | BPMNDiagram | Message | ❌ | extensions/essential/bpmn/toolbox/bpmn.json |
| BPMN Message `bpmn.message` | BPMNDiagram | Message Flow | ❌ | extensions/essential/bpmn/toolbox/bpmn.json |
| BPMN Choreography `bpmn.choreography` | BPMNDiagram | Choreography Task | ❌ | extensions/essential/bpmn/toolbox/bpmn.json |
| BPMN Choreography `bpmn.choreography` | BPMNDiagram | Sub Choreography | ❌ | extensions/essential/bpmn/toolbox/bpmn.json |
| BPMN Choreography `bpmn.choreography` | BPMNDiagram | Message Link | ❌ | extensions/essential/bpmn/toolbox/bpmn.json |
| BPMN Conversation `bpmn.conversation` | BPMNDiagram | Conversation | ❌ | extensions/essential/bpmn/toolbox/bpmn.json |
| BPMN Conversation `bpmn.conversation` | BPMNDiagram | Sub Conversation | ❌ | extensions/essential/bpmn/toolbox/bpmn.json |
| BPMN Conversation `bpmn.conversation` | BPMNDiagram | Call Conversation | ❌ | extensions/essential/bpmn/toolbox/bpmn.json |
| BPMN Conversation `bpmn.conversation` | BPMNDiagram | Conversation Link | ❌ | extensions/essential/bpmn/toolbox/bpmn.json |
| C4 Model `c4` | C4Diagram | Person | ❌ | extensions/essential/c4/toolbox/c4.json |
| C4 Model `c4` | C4Diagram | Software System | ❌ | extensions/essential/c4/toolbox/c4.json |
| C4 Model `c4` | C4Diagram | Container | ❌ | extensions/essential/c4/toolbox/c4.json |
| C4 Model `c4` | C4Diagram | Container (Database) | ❌ | extensions/essential/c4/toolbox/c4.json |
| C4 Model `c4` | C4Diagram | Container (Web App) | ❌ | extensions/essential/c4/toolbox/c4.json |
| C4 Model `c4` | C4Diagram | Container (Desktop App) | ❌ | extensions/essential/c4/toolbox/c4.json |
| C4 Model `c4` | C4Diagram | Container (Mobile App) | ❌ | extensions/essential/c4/toolbox/c4.json |
| C4 Model `c4` | C4Diagram | Component | ❌ | extensions/essential/c4/toolbox/c4.json |
| C4 Model `c4` | C4Diagram | Element | ❌ | extensions/essential/c4/toolbox/c4.json |
| C4 Model `c4` | C4Diagram | Relationship | ❌ | extensions/essential/c4/toolbox/c4.json |
| Annotations `annotation` | all | Text | ✅ | extensions/essential/common/toolbox/common.json |
| Annotations `annotation` | all | Text Box | ✅ | extensions/essential/common/toolbox/common.json |
| Annotations `annotation` | all | Free Line | ✅ | extensions/essential/common/toolbox/common.json |
| Annotations `annotation` | all | Note | ✅ | extensions/essential/common/toolbox/common.json |
| Annotations `annotation` | all | Note Link | ✅ | extensions/essential/common/toolbox/common.json |
| Annotations `annotation` | all | Hyperlink | ✅ | a Hyperlink element in the diagram's owner and its view: a box with a bold "link" and the linked element's name, the URL, or with showName its own name; reference, url and showName are edited in the property editor (src/app.rs:821-827; src/views.rs:705-748; src/render.rs:514) | extensions/essential/common/toolbox/common.json |
| Annotations `annotation` | all | Rectangle | ✅ | a view-only shape: a filled rectangle, at least 50×50, with no shadow (src/app.rs:759-811; src/render.rs:564-576; src/views.rs:315-324) | extensions/essential/common/toolbox/common.json |
| Annotations `annotation` | all | Rounded Rectangle | ✅ | a view-only shape: a filled rectangle with corners rounded to an eighth of its smaller side, at least 50×50, with no shadow (src/app.rs:759-811; src/render.rs:564-576; src/views.rs:315-324) | extensions/essential/common/toolbox/common.json |
| Annotations `annotation` | all | Ellipse | ✅ | a view-only shape: a filled ellipse, at least 50×50, with no shadow (src/app.rs:759-811; src/render.rs:564-576; src/views.rs:315-324) | extensions/essential/common/toolbox/common.json |
| Annotations `annotation` | all | Image | ✅ | an ImageView, at least 100×100; with no picture yet the file picker opens at once, and the view takes the picture's natural size; the picture is kept as a data URL (src/app.rs:1507-1539; src/picture.rs; src/render.rs:553-563) | extensions/essential/common/toolbox/common.json |
| Data Flow `dfd` | DFDDiagram | External Entity | ❌ | extensions/essential/dfd/toolbox/dfd.json |
| Data Flow `dfd` | DFDDiagram | Process | ❌ | extensions/essential/dfd/toolbox/dfd.json |
| Data Flow `dfd` | DFDDiagram | Data Store | ❌ | extensions/essential/dfd/toolbox/dfd.json |
| Data Flow `dfd` | DFDDiagram | Data Flow | ❌ | extensions/essential/dfd/toolbox/dfd.json |
| Entity-Relationship `erd` | ERDDiagram | Entity | ❌ | extensions/essential/erd/toolbox/erd.json |
| Entity-Relationship `erd` | ERDDiagram | One to One Relationship | ❌ | extensions/essential/erd/toolbox/erd.json |
| Entity-Relationship `erd` | ERDDiagram | One to Many Relationship | ❌ | extensions/essential/erd/toolbox/erd.json |
| Entity-Relationship `erd` | ERDDiagram | Many to Many Relationship | ❌ | extensions/essential/erd/toolbox/erd.json |
| Flowchart `flowchart` | FCFlowchartDiagram | Flow | ❌ | extensions/essential/flowchart/toolbox/flowchart.json |
| Flowchart `flowchart` | FCFlowchartDiagram | Process | ❌ | extensions/essential/flowchart/toolbox/flowchart.json |
| Flowchart `flowchart` | FCFlowchartDiagram | Terminator | ❌ | extensions/essential/flowchart/toolbox/flowchart.json |
| Flowchart `flowchart` | FCFlowchartDiagram | Decision | ❌ | extensions/essential/flowchart/toolbox/flowchart.json |
| Flowchart `flowchart` | FCFlowchartDiagram | Delay | ❌ | extensions/essential/flowchart/toolbox/flowchart.json |
| Flowchart `flowchart` | FCFlowchartDiagram | Predefined Process | ❌ | extensions/essential/flowchart/toolbox/flowchart.json |
| Flowchart `flowchart` | FCFlowchartDiagram | Alternate Process | ❌ | extensions/essential/flowchart/toolbox/flowchart.json |
| Flowchart `flowchart` | FCFlowchartDiagram | Data | ❌ | extensions/essential/flowchart/toolbox/flowchart.json |
| Flowchart `flowchart` | FCFlowchartDiagram | Document | ❌ | extensions/essential/flowchart/toolbox/flowchart.json |
| Flowchart `flowchart` | FCFlowchartDiagram | Multi-Document | ❌ | extensions/essential/flowchart/toolbox/flowchart.json |
| Flowchart `flowchart` | FCFlowchartDiagram | Preparation | ❌ | extensions/essential/flowchart/toolbox/flowchart.json |
| Flowchart `flowchart` | FCFlowchartDiagram | Display | ❌ | extensions/essential/flowchart/toolbox/flowchart.json |
| Flowchart `flowchart` | FCFlowchartDiagram | Manual Input | ❌ | extensions/essential/flowchart/toolbox/flowchart.json |
| Flowchart `flowchart` | FCFlowchartDiagram | Manual Operation | ❌ | extensions/essential/flowchart/toolbox/flowchart.json |
| Flowchart `flowchart` | FCFlowchartDiagram | Card | ❌ | extensions/essential/flowchart/toolbox/flowchart.json |
| Flowchart `flowchart` | FCFlowchartDiagram | Punched Tape | ❌ | extensions/essential/flowchart/toolbox/flowchart.json |
| Flowchart `flowchart` | FCFlowchartDiagram | Connector | ❌ | extensions/essential/flowchart/toolbox/flowchart.json |
| Flowchart `flowchart` | FCFlowchartDiagram | Off-Page Connector | ❌ | extensions/essential/flowchart/toolbox/flowchart.json |
| Flowchart `flowchart` | FCFlowchartDiagram | Or | ❌ | extensions/essential/flowchart/toolbox/flowchart.json |
| Flowchart `flowchart` | FCFlowchartDiagram | Summing Junction | ❌ | extensions/essential/flowchart/toolbox/flowchart.json |
| Flowchart `flowchart` | FCFlowchartDiagram | Collate | ❌ | extensions/essential/flowchart/toolbox/flowchart.json |
| Flowchart `flowchart` | FCFlowchartDiagram | Sort | ❌ | extensions/essential/flowchart/toolbox/flowchart.json |
| Flowchart `flowchart` | FCFlowchartDiagram | Merge | ❌ | extensions/essential/flowchart/toolbox/flowchart.json |
| Flowchart `flowchart` | FCFlowchartDiagram | Extract | ❌ | extensions/essential/flowchart/toolbox/flowchart.json |
| Flowchart `flowchart` | FCFlowchartDiagram | Stored Data | ❌ | extensions/essential/flowchart/toolbox/flowchart.json |
| Flowchart `flowchart` | FCFlowchartDiagram | Database | ❌ | extensions/essential/flowchart/toolbox/flowchart.json |
| Flowchart `flowchart` | FCFlowchartDiagram | Direct Access Storage | ❌ | extensions/essential/flowchart/toolbox/flowchart.json |
| Flowchart `flowchart` | FCFlowchartDiagram | Internal Storage | ❌ | extensions/essential/flowchart/toolbox/flowchart.json |
| GCP `gcp.elements` | GCPDiagram | Zone | ❌ | extensions/essential/gcp/toolbox/gcp.json |
| GCP `gcp.elements` | GCPDiagram | User | ❌ | extensions/essential/gcp/toolbox/gcp.json |
| GCP `gcp.elements` | GCPDiagram | Product | ❌ | extensions/essential/gcp/toolbox/gcp.json |
| GCP `gcp.elements` | GCPDiagram | Service | ❌ | extensions/essential/gcp/toolbox/gcp.json |
| GCP `gcp.elements` | GCPDiagram | Path | ❌ | extensions/essential/gcp/toolbox/gcp.json |
| Mindmap `mindmap` | MMMindmapDiagram | Node | ❌ | extensions/essential/mindmap/toolbox/mindmap.json |
| Mindmap `mindmap` | MMMindmapDiagram | Edge | ❌ | extensions/essential/mindmap/toolbox/mindmap.json |
| Requirements `sysml.requirement` | SysMLRequirementDiagram | Package | ❌ | extensions/essential/sysml/toolbox/sysml.json |
| Requirements `sysml.requirement` | SysMLRequirementDiagram | Requirement | ❌ | extensions/essential/sysml/toolbox/sysml.json |
| Requirements `sysml.requirement` | SysMLRequirementDiagram | Frame | ❌ | extensions/essential/sysml/toolbox/sysml.json |
| Requirements `sysml.requirement` | SysMLRequirementDiagram | Copy | ❌ | extensions/essential/sysml/toolbox/sysml.json |
| Requirements `sysml.requirement` | SysMLRequirementDiagram | DeriveReqt | ❌ | extensions/essential/sysml/toolbox/sysml.json |
| Requirements `sysml.requirement` | SysMLRequirementDiagram | Verify | ❌ | extensions/essential/sysml/toolbox/sysml.json |
| Requirements `sysml.requirement` | SysMLRequirementDiagram | Satisfy | ❌ | extensions/essential/sysml/toolbox/sysml.json |
| Requirements `sysml.requirement` | SysMLRequirementDiagram | Refine | ❌ | extensions/essential/sysml/toolbox/sysml.json |
| Requirements `sysml.requirement` | SysMLRequirementDiagram | Containment | ❌ | extensions/essential/sysml/toolbox/sysml.json |
| Block `sysml.block` | SysMLBlockDefinitionDiagram | Block | ❌ | extensions/essential/sysml/toolbox/sysml.json |
| Block `sysml.block` | SysMLBlockDefinitionDiagram | Value Type | ❌ | extensions/essential/sysml/toolbox/sysml.json |
| Block `sysml.block` | SysMLBlockDefinitionDiagram | Interface Block | ❌ | extensions/essential/sysml/toolbox/sysml.json |
| Block `sysml.block` | SysMLBlockDefinitionDiagram | Constraint Block | ❌ | extensions/essential/sysml/toolbox/sysml.json |
| Block `sysml.block` | SysMLBlockDefinitionDiagram | Enumeration | ❌ | extensions/essential/sysml/toolbox/sysml.json |
| Block `sysml.block` | SysMLBlockDefinitionDiagram | Signal | ❌ | extensions/essential/sysml/toolbox/sysml.json |
| Block `sysml.block` | SysMLBlockDefinitionDiagram | Port | ❌ | extensions/essential/sysml/toolbox/sysml.json |
| Block `sysml.block` | SysMLBlockDefinitionDiagram | Object | ❌ | extensions/essential/sysml/toolbox/sysml.json |
| Block `sysml.block` | SysMLBlockDefinitionDiagram | Frame | ❌ | extensions/essential/sysml/toolbox/sysml.json |
| Block `sysml.block` | SysMLBlockDefinitionDiagram | Association | ❌ | extensions/essential/sysml/toolbox/sysml.json |
| Block `sysml.block` | SysMLBlockDefinitionDiagram | Directed Association | ❌ | extensions/essential/sysml/toolbox/sysml.json |
| Block `sysml.block` | SysMLBlockDefinitionDiagram | Aggregation | ❌ | extensions/essential/sysml/toolbox/sysml.json |
| Block `sysml.block` | SysMLBlockDefinitionDiagram | Composition | ❌ | extensions/essential/sysml/toolbox/sysml.json |
| Block `sysml.block` | SysMLBlockDefinitionDiagram | Dependency | ❌ | extensions/essential/sysml/toolbox/sysml.json |
| Block `sysml.block` | SysMLBlockDefinitionDiagram | Generalization | ❌ | extensions/essential/sysml/toolbox/sysml.json |
| Block `sysml.block` | SysMLBlockDefinitionDiagram | Interface Realization | ❌ | extensions/essential/sysml/toolbox/sysml.json |
| Block `sysml.block` | SysMLBlockDefinitionDiagram | Connector | ❌ | extensions/essential/sysml/toolbox/sysml.json |
| Block `sysml.block` | SysMLBlockDefinitionDiagram | Containment | ❌ | extensions/essential/sysml/toolbox/sysml.json |
| Internal Block `sysml.internal-block` | SysMLInternalBlockDiagram, SysMLParametricDiagram | Part | ❌ | extensions/essential/sysml/toolbox/sysml.json |
| Internal Block `sysml.internal-block` | SysMLInternalBlockDiagram, SysMLParametricDiagram | Reference | ❌ | extensions/essential/sysml/toolbox/sysml.json |
| Internal Block `sysml.internal-block` | SysMLInternalBlockDiagram, SysMLParametricDiagram | Value | ❌ | extensions/essential/sysml/toolbox/sysml.json |
| Internal Block `sysml.internal-block` | SysMLInternalBlockDiagram, SysMLParametricDiagram | Port | ❌ | extensions/essential/sysml/toolbox/sysml.json |
| Internal Block `sysml.internal-block` | SysMLInternalBlockDiagram, SysMLParametricDiagram | Frame | ❌ | extensions/essential/sysml/toolbox/sysml.json |
| Internal Block `sysml.internal-block` | SysMLInternalBlockDiagram, SysMLParametricDiagram | Connector | ❌ | extensions/essential/sysml/toolbox/sysml.json |
| Parametric `sysml.parametric` | SysMLParametricDiagram | Constraint Property | ❌ | extensions/essential/sysml/toolbox/sysml.json |
| Parametric `sysml.parametric` | SysMLParametricDiagram | Constraint Parameter | ❌ | extensions/essential/sysml/toolbox/sysml.json |
| Viewpoints `sysml.viewpoint` | SysMLRequirementDiagram, SysMLBlockDefinitionDiagram, UMLPackageDiagram | Stakeholder | ❌ | extensions/essential/sysml/toolbox/sysml.json |
| Viewpoints `sysml.viewpoint` | SysMLRequirementDiagram, SysMLBlockDefinitionDiagram, UMLPackageDiagram | View | ❌ | extensions/essential/sysml/toolbox/sysml.json |
| Viewpoints `sysml.viewpoint` | SysMLRequirementDiagram, SysMLBlockDefinitionDiagram, UMLPackageDiagram | Viewpoint | ❌ | extensions/essential/sysml/toolbox/sysml.json |
| Viewpoints `sysml.viewpoint` | SysMLRequirementDiagram, SysMLBlockDefinitionDiagram, UMLPackageDiagram | Conform | ❌ | extensions/essential/sysml/toolbox/sysml.json |
| Viewpoints `sysml.viewpoint` | SysMLRequirementDiagram, SysMLBlockDefinitionDiagram, UMLPackageDiagram | Expose | ❌ | extensions/essential/sysml/toolbox/sysml.json |
| Information Flows `uml.information-flow` | UMLInformationFlowDiagram | Information Item | ✅ | extensions/essential/uml/toolbox/uml.json |
| Information Flows `uml.information-flow` | UMLInformationFlowDiagram | Information Flow | ✅ | extensions/essential/uml/toolbox/uml.json |
| Use Cases `uml.usecase` | UMLUseCaseDiagram, UMLInformationFlowDiagram | Package | ✅ | extensions/essential/uml/toolbox/uml.json |
| Use Cases `uml.usecase` | UMLUseCaseDiagram, UMLInformationFlowDiagram | Use Case Subject | ✅ | extensions/essential/uml/toolbox/uml.json |
| Use Cases `uml.usecase` | UMLUseCaseDiagram, UMLInformationFlowDiagram | Use Case | ✅ | extensions/essential/uml/toolbox/uml.json |
| Use Cases `uml.usecase` | UMLUseCaseDiagram, UMLInformationFlowDiagram | Actor | ✅ | extensions/essential/uml/toolbox/uml.json |
| Use Cases `uml.usecase` | UMLUseCaseDiagram, UMLInformationFlowDiagram | Frame | ✅ | Frame asks "Select an element to be represented as Frame" (any element, or none) and then creates the frame, behind the other views; it shows the diagram kind in a bold tab and the element's name (src/canvas.rs:1231-1240; src/picker.rs:73-100; src/views.rs:480-702, 750-836; src/render.rs:360) | extensions/essential/uml/toolbox/uml.json |
| Use Cases `uml.usecase` | UMLUseCaseDiagram, UMLInformationFlowDiagram | Association | ✅ | extensions/essential/uml/toolbox/uml.json |
| Use Cases `uml.usecase` | UMLUseCaseDiagram, UMLInformationFlowDiagram | Directed Association | ✅ | extensions/essential/uml/toolbox/uml.json |
| Use Cases `uml.usecase` | UMLUseCaseDiagram, UMLInformationFlowDiagram | Generalization | ✅ | extensions/essential/uml/toolbox/uml.json |
| Use Cases `uml.usecase` | UMLUseCaseDiagram, UMLInformationFlowDiagram | Dependency | ✅ | extensions/essential/uml/toolbox/uml.json |
| Use Cases `uml.usecase` | UMLUseCaseDiagram, UMLInformationFlowDiagram | Include | ✅ | extensions/essential/uml/toolbox/uml.json |
| Use Cases `uml.usecase` | UMLUseCaseDiagram, UMLInformationFlowDiagram | Extend | ✅ | extensions/essential/uml/toolbox/uml.json |
| Classes (Basic) `uml.class` | UMLClassDiagram, UMLCompositeStructureDiagram, UMLInformationFlowDiagram | Class | ✅ | extensions/essential/uml/toolbox/uml.json |
| Classes (Basic) `uml.class` | UMLClassDiagram, UMLCompositeStructureDiagram, UMLInformationFlowDiagram | Interface | ✅ | extensions/essential/uml/toolbox/uml.json |
| Classes (Basic) `uml.class` | UMLClassDiagram, UMLCompositeStructureDiagram, UMLInformationFlowDiagram | Association | ✅ | extensions/essential/uml/toolbox/uml.json |
| Classes (Basic) `uml.class` | UMLClassDiagram, UMLCompositeStructureDiagram, UMLInformationFlowDiagram | Directed Association | ✅ | extensions/essential/uml/toolbox/uml.json |
| Classes (Basic) `uml.class` | UMLClassDiagram, UMLCompositeStructureDiagram, UMLInformationFlowDiagram | Aggregation | ✅ | extensions/essential/uml/toolbox/uml.json |
| Classes (Basic) `uml.class` | UMLClassDiagram, UMLCompositeStructureDiagram, UMLInformationFlowDiagram | Composition | ✅ | extensions/essential/uml/toolbox/uml.json |
| Classes (Basic) `uml.class` | UMLClassDiagram, UMLCompositeStructureDiagram, UMLInformationFlowDiagram | Dependency | ✅ | extensions/essential/uml/toolbox/uml.json |
| Classes (Basic) `uml.class` | UMLClassDiagram, UMLCompositeStructureDiagram, UMLInformationFlowDiagram | Generalization | ✅ | extensions/essential/uml/toolbox/uml.json |
| Classes (Basic) `uml.class` | UMLClassDiagram, UMLCompositeStructureDiagram, UMLInformationFlowDiagram | Interface Realization | ✅ | extensions/essential/uml/toolbox/uml.json |
| Classes (Advanced) `uml.class-adv` | UMLClassDiagram, UMLCompositeStructureDiagram | Signal | ✅ | extensions/essential/uml/toolbox/uml.json |
| Classes (Advanced) `uml.class-adv` | UMLClassDiagram, UMLCompositeStructureDiagram | DataType | ✅ | extensions/essential/uml/toolbox/uml.json |
| Classes (Advanced) `uml.class-adv` | UMLClassDiagram, UMLCompositeStructureDiagram | PrimitiveType | ✅ | extensions/essential/uml/toolbox/uml.json |
| Classes (Advanced) `uml.class-adv` | UMLClassDiagram, UMLCompositeStructureDiagram | Enumeration | ✅ | extensions/essential/uml/toolbox/uml.json |
| Classes (Advanced) `uml.class-adv` | UMLClassDiagram, UMLCompositeStructureDiagram | Frame | ✅ | Frame asks "Select an element to be represented as Frame" (any element, or none) and then creates the frame, behind the other views; it shows the diagram kind in a bold tab and the element's name (src/canvas.rs:1231-1240; src/picker.rs:73-100; src/views.rs:480-702, 750-836; src/render.rs:360) | extensions/essential/uml/toolbox/uml.json |
| Classes (Advanced) `uml.class-adv` | UMLClassDiagram, UMLCompositeStructureDiagram | N-ary Association Node | ✅ | an unnamed UMLNaryAssociationNode drawn as a diamond (at least 30×22, with shadow), its name, stereotype and properties floating around its centre; associations join classifiers to it, and the Association Class tool links a class to it (src/app.rs:814-820, 1202-1205, 1340-1346; src/views.rs:539-706; src/render.rs:497-511) | extensions/essential/uml/toolbox/uml.json |
| Classes (Advanced) `uml.class-adv` | UMLClassDiagram, UMLCompositeStructureDiagram | Association Class | ✅ | extensions/essential/uml/toolbox/uml.json |
| Classes (Advanced) `uml.class-adv` | UMLClassDiagram, UMLCompositeStructureDiagram | Template Binding | ✅ | extensions/essential/uml/toolbox/uml.json |
| Classes (Advanced) `uml.class-adv` | UMLClassDiagram, UMLCompositeStructureDiagram | Realization | ✅ | extensions/essential/uml/toolbox/uml.json |
| Packages `uml.package` | UMLClassDiagram, UMLPackageDiagram | Package | ✅ | extensions/essential/uml/toolbox/uml.json |
| Packages `uml.package` | UMLClassDiagram, UMLPackageDiagram | Model | ✅ | extensions/essential/uml/toolbox/uml.json |
| Packages `uml.package` | UMLClassDiagram, UMLPackageDiagram | Subsystem | ✅ | extensions/essential/uml/toolbox/uml.json |
| Packages `uml.package` | UMLClassDiagram, UMLPackageDiagram | Containment | ✅ | extensions/essential/uml/toolbox/uml.json |
| Packages `uml.package` | UMLClassDiagram, UMLPackageDiagram | Dependency | ✅ | extensions/essential/uml/toolbox/uml.json |
| Component `uml.component` | UMLComponentDiagram, UMLDeploymentDiagram | Component | ✅ | extensions/essential/uml/toolbox/uml.json |
| Component `uml.component` | UMLComponentDiagram, UMLDeploymentDiagram | Artifact | ✅ | extensions/essential/uml/toolbox/uml.json |
| Component `uml.component` | UMLComponentDiagram, UMLDeploymentDiagram | Interface | ✅ | extensions/essential/uml/toolbox/uml.json |
| Component `uml.component` | UMLComponentDiagram, UMLDeploymentDiagram | Frame | ✅ | Frame asks "Select an element to be represented as Frame" (any element, or none) and then creates the frame, behind the other views; it shows the diagram kind in a bold tab and the element's name (src/canvas.rs:1231-1240; src/picker.rs:73-100; src/views.rs:480-702, 750-836; src/render.rs:360) | extensions/essential/uml/toolbox/uml.json |
| Component `uml.component` | UMLComponentDiagram, UMLDeploymentDiagram | Dependency | ✅ | extensions/essential/uml/toolbox/uml.json |
| Component `uml.component` | UMLComponentDiagram, UMLDeploymentDiagram | Interface Realization | ✅ | extensions/essential/uml/toolbox/uml.json |
| Component `uml.component` | UMLComponentDiagram, UMLDeploymentDiagram | Component Realization | ✅ | extensions/essential/uml/toolbox/uml.json |
| Deployment `uml.deployment` | UMLComponentDiagram, UMLDeploymentDiagram | Node | ✅ | extensions/essential/uml/toolbox/uml.json |
| Deployment `uml.deployment` | UMLComponentDiagram, UMLDeploymentDiagram | Deployment | ✅ | extensions/essential/uml/toolbox/uml.json |
| Deployment `uml.deployment` | UMLComponentDiagram, UMLDeploymentDiagram | Communication Path | ✅ | extensions/essential/uml/toolbox/uml.json |
| Composite Structure `uml.compositestructure` | UMLClassDiagram, UMLCompositeStructureDiagram, UMLComponentDiagram | Collaboration | ✅ | in the Composite Structure group of class, component and deployment diagrams: a dashed ellipse at least √2 times its name and the use case minimum, the name centred; it keeps a hidden template parameter compartment, whose parameters are not drawn (src/ui.rs:97-111; src/views.rs:520-539; src/render.rs:461-496) | extensions/essential/uml/toolbox/uml.json |
| Composite Structure `uml.compositestructure` | UMLClassDiagram, UMLCompositeStructureDiagram, UMLComponentDiagram | Port | ✅ | a click on a classifier box adds a UMLPort "Port1" to its attributes and a 14 × 14 port on the box's border where the line from its centre through the click meets it; on a typed part it goes to the part's type; elsewhere "UMLPort cannot be placed here." (src/app.rs:1459-1503; src/views.rs:556-657; src/render.rs:474-483; src/canvas.rs:1245-1263) | extensions/essential/uml/toolbox/uml.json |
| Composite Structure `uml.compositestructure` | UMLClassDiagram, UMLCompositeStructureDiagram, UMLComponentDiagram | Part | ✅ | a box drawn in a classifier adds a composite attribute "Part1" and its box, contained in the classifier and showing "name: Type[mult]", dashed when not composite (src/app.rs:1459-1503; src/views.rs:573-585, 659-671; src/render.rs:484-496) | extensions/essential/uml/toolbox/uml.json |
| Composite Structure `uml.compositestructure` | UMLClassDiagram, UMLCompositeStructureDiagram, UMLComponentDiagram | Connector | ✅ | a line between two features (ports, parts, attributes), a UMLConnector with connector ends and end labels like an association's (src/app.rs:1220; src/views.rs:230, 891, 2007) | extensions/essential/uml/toolbox/uml.json |
| Composite Structure `uml.compositestructure` | UMLClassDiagram, UMLCompositeStructureDiagram, UMLComponentDiagram | Collaboration Use | ✅ | a dashed ellipse showing "name: Collaboration" once its type is set in Properties (src/ui.rs:97-111; src/views.rs:520-539; src/render.rs:461-496) | extensions/essential/uml/toolbox/uml.json |
| Composite Structure `uml.compositestructure` | UMLClassDiagram, UMLCompositeStructureDiagram, UMLComponentDiagram | Role Binding | ✅ | a dashed arrow from a collaboration use to a part or attribute, a UMLRoleBinding whose roleName (in Properties) shows near the head (src/app.rs:1222; src/views.rs:885-889) | extensions/essential/uml/toolbox/uml.json |
| Composite Structure `uml.compositestructure` | UMLClassDiagram, UMLCompositeStructureDiagram, UMLComponentDiagram | Realization | ✅ | the Realization line, in the Composite Structure group of class, component and deployment diagrams (src/ui.rs:97-111) | extensions/essential/uml/toolbox/uml.json |
| Instances `uml.instance` | UMLClassDiagram, UMLObjectDiagram, UMLComponentDiagram, UMLDeploymentDiagram | Object | ✅ | extensions/essential/uml/toolbox/uml.json |
| Instances `uml.instance` | UMLClassDiagram, UMLObjectDiagram, UMLComponentDiagram, UMLDeploymentDiagram | Artifact Instance | ✅ | extensions/essential/uml/toolbox/uml.json |
| Instances `uml.instance` | UMLClassDiagram, UMLObjectDiagram, UMLComponentDiagram, UMLDeploymentDiagram | Component Instance | ✅ | extensions/essential/uml/toolbox/uml.json |
| Instances `uml.instance` | UMLClassDiagram, UMLObjectDiagram, UMLComponentDiagram, UMLDeploymentDiagram | Node Instance | ✅ | extensions/essential/uml/toolbox/uml.json |
| Instances `uml.instance` | UMLClassDiagram, UMLObjectDiagram, UMLComponentDiagram, UMLDeploymentDiagram | Link | ✅ | extensions/essential/uml/toolbox/uml.json |
| Instances `uml.instance` | UMLClassDiagram, UMLObjectDiagram, UMLComponentDiagram, UMLDeploymentDiagram | Directed Link | ✅ | extensions/essential/uml/toolbox/uml.json |
| Instances `uml.instance` | UMLClassDiagram, UMLObjectDiagram, UMLComponentDiagram, UMLDeploymentDiagram | Link Object | ✅ | extensions/essential/uml/toolbox/uml.json |
| Interactions (Basic) `uml.sequence` | UMLSequenceDiagram | Lifeline | ❌ | extensions/essential/uml/toolbox/uml.json |
| Interactions (Basic) `uml.sequence` | UMLSequenceDiagram | Message | ❌ | extensions/essential/uml/toolbox/uml.json |
| Interactions (Basic) `uml.sequence` | UMLSequenceDiagram | Self Message | ❌ | extensions/essential/uml/toolbox/uml.json |
| Interactions (Basic) `uml.sequence` | UMLSequenceDiagram | Async Message | ❌ | extensions/essential/uml/toolbox/uml.json |
| Interactions (Basic) `uml.sequence` | UMLSequenceDiagram | Reply Message | ❌ | extensions/essential/uml/toolbox/uml.json |
| Interactions (Basic) `uml.sequence` | UMLSequenceDiagram | Create Message | ❌ | extensions/essential/uml/toolbox/uml.json |
| Interactions (Basic) `uml.sequence` | UMLSequenceDiagram | Delete Message | ❌ | extensions/essential/uml/toolbox/uml.json |
| Interactions (Basic) `uml.sequence` | UMLSequenceDiagram | Async Signal Message | ❌ | extensions/essential/uml/toolbox/uml.json |
| Interactions (Advanced) `uml.sequence-adv` | UMLSequenceDiagram | Found Message | ❌ | extensions/essential/uml/toolbox/uml.json |
| Interactions (Advanced) `uml.sequence-adv` | UMLSequenceDiagram | Lost Message | ❌ | extensions/essential/uml/toolbox/uml.json |
| Interactions (Advanced) `uml.sequence-adv` | UMLSequenceDiagram | Endpoint | ❌ | extensions/essential/uml/toolbox/uml.json |
| Interactions (Advanced) `uml.sequence-adv` | UMLSequenceDiagram | Gate | ❌ | extensions/essential/uml/toolbox/uml.json |
| Interactions (Advanced) `uml.sequence-adv` | UMLSequenceDiagram | State Invariant | ❌ | extensions/essential/uml/toolbox/uml.json |
| Interactions (Advanced) `uml.sequence-adv` | UMLSequenceDiagram | Continuation | ❌ | extensions/essential/uml/toolbox/uml.json |
| Interactions (Advanced) `uml.sequence-adv` | UMLSequenceDiagram | Combined Fragment | ❌ | extensions/essential/uml/toolbox/uml.json |
| Interactions (Advanced) `uml.sequence-adv` | UMLSequenceDiagram | Interaction Use | ❌ | extensions/essential/uml/toolbox/uml.json |
| Interactions (Advanced) `uml.sequence-adv` | UMLSequenceDiagram | Time Constraint | ❌ | extensions/essential/uml/toolbox/uml.json |
| Interactions (Advanced) `uml.sequence-adv` | UMLSequenceDiagram | Duration Constraint | ❌ | extensions/essential/uml/toolbox/uml.json |
| Interactions (Advanced) `uml.sequence-adv` | UMLSequenceDiagram | Frame | ❌ | extensions/essential/uml/toolbox/uml.json |
| Communication `uml.communication` | UMLCommunicationDiagram | Lifeline | ❌ | extensions/essential/uml/toolbox/uml.json |
| Communication `uml.communication` | UMLCommunicationDiagram | Connector | ❌ | extensions/essential/uml/toolbox/uml.json |
| Communication `uml.communication` | UMLCommunicationDiagram | Self Connector | ❌ | extensions/essential/uml/toolbox/uml.json |
| Communication `uml.communication` | UMLCommunicationDiagram | Forward Message | ❌ | extensions/essential/uml/toolbox/uml.json |
| Communication `uml.communication` | UMLCommunicationDiagram | Reverse Message | ❌ | extensions/essential/uml/toolbox/uml.json |
| Communication `uml.communication` | UMLCommunicationDiagram | Frame | ❌ | extensions/essential/uml/toolbox/uml.json |
| Timing `uml.timing` | UMLTimingDiagram | Lifeline | ❌ | extensions/essential/uml/toolbox/uml.json |
| Timing `uml.timing` | UMLTimingDiagram | State/Condition | ❌ | extensions/essential/uml/toolbox/uml.json |
| Timing `uml.timing` | UMLTimingDiagram | Time Segment | ❌ | extensions/essential/uml/toolbox/uml.json |
| Timing `uml.timing` | UMLTimingDiagram | Time Tick | ❌ | extensions/essential/uml/toolbox/uml.json |
| Timing `uml.timing` | UMLTimingDiagram | Time Constraint | ❌ | extensions/essential/uml/toolbox/uml.json |
| Timing `uml.timing` | UMLTimingDiagram | Duration Constraint | ❌ | extensions/essential/uml/toolbox/uml.json |
| Timing `uml.timing` | UMLTimingDiagram | Message | ❌ | extensions/essential/uml/toolbox/uml.json |
| Interaction Overview `uml.interaction-overview` | UMLInteractionOverviewDiagram | Interaction Use | ❌ | extensions/essential/uml/toolbox/uml.json |
| Interaction Overview `uml.interaction-overview` | UMLInteractionOverviewDiagram | Interaction (Inline) | ❌ | extensions/essential/uml/toolbox/uml.json |
| Interaction Overview `uml.interaction-overview` | UMLInteractionOverviewDiagram | Initial | ❌ | extensions/essential/uml/toolbox/uml.json |
| Interaction Overview `uml.interaction-overview` | UMLInteractionOverviewDiagram | Final | ❌ | extensions/essential/uml/toolbox/uml.json |
| Interaction Overview `uml.interaction-overview` | UMLInteractionOverviewDiagram | Fork | ❌ | extensions/essential/uml/toolbox/uml.json |
| Interaction Overview `uml.interaction-overview` | UMLInteractionOverviewDiagram | Join | ❌ | extensions/essential/uml/toolbox/uml.json |
| Interaction Overview `uml.interaction-overview` | UMLInteractionOverviewDiagram | Merge | ❌ | extensions/essential/uml/toolbox/uml.json |
| Interaction Overview `uml.interaction-overview` | UMLInteractionOverviewDiagram | Decision | ❌ | extensions/essential/uml/toolbox/uml.json |
| Interaction Overview `uml.interaction-overview` | UMLInteractionOverviewDiagram | Control Flow | ❌ | extensions/essential/uml/toolbox/uml.json |
| Statechart (Basic) `uml.statechart` | UMLStatechartDiagram | Simple State | ✅ | in the Statechart (Basic) group: a UMLState "State1" in the first region's vertices, drawn as a rounded box with its name and its entry/do/exit activities under a line (src/ui.rs:246-279; src/app.rs:749-814; src/views.rs:497-519; src/render.rs:393-425) | extensions/essential/uml/toolbox/uml.json |
| Statechart (Basic) `uml.statechart` | UMLStatechartDiagram | Initial State | ✅ | a UMLPseudostate of kind initial in the first region, a filled 20 px circle (size estimated) (src/views.rs:264-279, 1673-1690; src/render.rs:433-460) | extensions/essential/uml/toolbox/uml.json |
| Statechart (Basic) `uml.statechart` | UMLStatechartDiagram | Choice | ✅ | a pseudostate of kind choice, a 20 px diamond (src/views.rs:264-279; src/render.rs:433-460) | extensions/essential/uml/toolbox/uml.json |
| Statechart (Basic) `uml.statechart` | UMLStatechartDiagram | Join | ✅ | a pseudostate of kind join, a filled bar that keeps its drawn size (8 × 60 on a click, estimated) (src/views.rs:264-279; src/render.rs:433-460) | extensions/essential/uml/toolbox/uml.json |
| Statechart (Basic) `uml.statechart` | UMLStatechartDiagram | Fork | ✅ | a pseudostate of kind fork, a filled bar like Join (src/views.rs:264-279; src/render.rs:433-460) | extensions/essential/uml/toolbox/uml.json |
| Statechart (Basic) `uml.statechart` | UMLStatechartDiagram | Final State | ✅ | a UMLFinalState in the first region, a 26 px circle around a filled one (src/views.rs:1673-1690; src/render.rs:426-432) | extensions/essential/uml/toolbox/uml.json |
| Statechart (Basic) `uml.statechart` | UMLStatechartDiagram | Transition | ✅ | a UMLTransition between two vertices, owned by the source's region, a solid line with an open arrow and the label `triggers [guard] / effect` (src/app.rs:1239, 1295-1300; src/views.rs:1959-1978) | extensions/essential/uml/toolbox/uml.json |
| Statechart (Basic) `uml.statechart` | UMLStatechartDiagram | Self Transition | ✅ | a click on a state makes a transition from it to itself, drawn as a loop at its top right (src/ui.rs:258-259; src/canvas.rs:1302-1306) | extensions/essential/uml/toolbox/uml.json |
| Statechart (Advanced) `uml.statechart-adv` | UMLStatechartDiagram | Composite State | ✅ | a 200 × 150 state with one region, drawn with a line under its name; states and pseudostates drawn or dropped inside it are contained and go in its region (src/ui.rs:266; src/app.rs:776-809, 861-866, 913-932; src/render.rs:400-412) | extensions/essential/uml/toolbox/uml.json |
| Statechart (Advanced) `uml.statechart-adv` | UMLStatechartDiagram | Submachine State | 🟡 | a state whose `submachine` is a new state machine with one region, made next to the diagram's (src/ui.rs:267; src/app.rs:792-801); drawn as a plain state, without the submachine mark | extensions/essential/uml/toolbox/uml.json |
| Statechart (Advanced) `uml.statechart-adv` | UMLStatechartDiagram | Orthogonal State | ✅ | a 200 × 150 state with two regions, split by a dashed line; a state drawn or dropped in it goes in the region at its height (the split is estimated) (src/ui.rs:268; src/app.rs:783-791, 917-927; src/render.rs:406-411) | extensions/essential/uml/toolbox/uml.json |
| Statechart (Advanced) `uml.statechart-adv` | UMLStatechartDiagram | Junction | ✅ | in the Statechart (Advanced) group: a pseudostate of kind junction, a small filled circle (src/ui.rs:246-279; src/render.rs:433-460) | extensions/essential/uml/toolbox/uml.json |
| Statechart (Advanced) `uml.statechart-adv` | UMLStatechartDiagram | Shallow History | ✅ | a pseudostate of kind shallowHistory, a circle with "H" (src/render.rs:433-460) | extensions/essential/uml/toolbox/uml.json |
| Statechart (Advanced) `uml.statechart-adv` | UMLStatechartDiagram | Deep History | ✅ | a pseudostate of kind deepHistory, a circle with "H*" (src/render.rs:433-460) | extensions/essential/uml/toolbox/uml.json |
| Statechart (Advanced) `uml.statechart-adv` | UMLStatechartDiagram | Entry Point | 🟡 | a pseudostate of kind entryPoint, an empty circle; it is placed freely, not on a state's or frame's border (src/render.rs:433-460) | extensions/essential/uml/toolbox/uml.json |
| Statechart (Advanced) `uml.statechart-adv` | UMLStatechartDiagram | Exit Point | 🟡 | a pseudostate of kind exitPoint, a circle with an X; placed freely like Entry Point (src/render.rs:433-460) | extensions/essential/uml/toolbox/uml.json |
| Statechart (Advanced) `uml.statechart-adv` | UMLStatechartDiagram | Terminate | ✅ | a pseudostate of kind terminate, an X (src/render.rs:433-460) | extensions/essential/uml/toolbox/uml.json |
| Statechart (Advanced) `uml.statechart-adv` | UMLStatechartDiagram | Connection Point Reference | ❌ | extensions/essential/uml/toolbox/uml.json |
| Statechart (Advanced) `uml.statechart-adv` | UMLStatechartDiagram | Frame | ✅ | in the Statechart (Advanced) group, the same Frame tool as on other diagrams (src/ui.rs:246-279) | extensions/essential/uml/toolbox/uml.json |
| Activities (Basic) `uml.activity` | UMLActivityDiagram | Action | ❌ | extensions/essential/uml/toolbox/uml.json |
| Activities (Basic) `uml.activity` | UMLActivityDiagram | Initial | ❌ | extensions/essential/uml/toolbox/uml.json |
| Activities (Basic) `uml.activity` | UMLActivityDiagram | Final | ❌ | extensions/essential/uml/toolbox/uml.json |
| Activities (Basic) `uml.activity` | UMLActivityDiagram | Fork | ❌ | extensions/essential/uml/toolbox/uml.json |
| Activities (Basic) `uml.activity` | UMLActivityDiagram | Join | ❌ | extensions/essential/uml/toolbox/uml.json |
| Activities (Basic) `uml.activity` | UMLActivityDiagram | Merge | ❌ | extensions/essential/uml/toolbox/uml.json |
| Activities (Basic) `uml.activity` | UMLActivityDiagram | Decision | ❌ | extensions/essential/uml/toolbox/uml.json |
| Activities (Basic) `uml.activity` | UMLActivityDiagram | Control Flow | ❌ | extensions/essential/uml/toolbox/uml.json |
| Activities (Advanced) `uml.activity-adv` | UMLActivityDiagram | Swimlane (Vertical) | ❌ | extensions/essential/uml/toolbox/uml.json |
| Activities (Advanced) `uml.activity-adv` | UMLActivityDiagram | Swimlane (Horizontal) | ❌ | extensions/essential/uml/toolbox/uml.json |
| Activities (Advanced) `uml.activity-adv` | UMLActivityDiagram | Interruptible Activity Region | ❌ | extensions/essential/uml/toolbox/uml.json |
| Activities (Advanced) `uml.activity-adv` | UMLActivityDiagram | Structured Activity | ❌ | extensions/essential/uml/toolbox/uml.json |
| Activities (Advanced) `uml.activity-adv` | UMLActivityDiagram | Input Pin | ❌ | extensions/essential/uml/toolbox/uml.json |
| Activities (Advanced) `uml.activity-adv` | UMLActivityDiagram | Output Pin | ❌ | extensions/essential/uml/toolbox/uml.json |
| Activities (Advanced) `uml.activity-adv` | UMLActivityDiagram | Send Signal | ❌ | extensions/essential/uml/toolbox/uml.json |
| Activities (Advanced) `uml.activity-adv` | UMLActivityDiagram | Accept Signal | ❌ | extensions/essential/uml/toolbox/uml.json |
| Activities (Advanced) `uml.activity-adv` | UMLActivityDiagram | Accept Time Event | ❌ | extensions/essential/uml/toolbox/uml.json |
| Activities (Advanced) `uml.activity-adv` | UMLActivityDiagram | Flow Final | ❌ | extensions/essential/uml/toolbox/uml.json |
| Activities (Advanced) `uml.activity-adv` | UMLActivityDiagram | Object Node | ❌ | extensions/essential/uml/toolbox/uml.json |
| Activities (Advanced) `uml.activity-adv` | UMLActivityDiagram | Central Buffer | ❌ | extensions/essential/uml/toolbox/uml.json |
| Activities (Advanced) `uml.activity-adv` | UMLActivityDiagram | Datastore | ❌ | extensions/essential/uml/toolbox/uml.json |
| Activities (Advanced) `uml.activity-adv` | UMLActivityDiagram | Activity Parameter Node | ❌ | extensions/essential/uml/toolbox/uml.json |
| Activities (Advanced) `uml.activity-adv` | UMLActivityDiagram | Expansion Region | ❌ | extensions/essential/uml/toolbox/uml.json |
| Activities (Advanced) `uml.activity-adv` | UMLActivityDiagram | Input Expansion Node | ❌ | extensions/essential/uml/toolbox/uml.json |
| Activities (Advanced) `uml.activity-adv` | UMLActivityDiagram | Output Expansion Node | ❌ | extensions/essential/uml/toolbox/uml.json |
| Activities (Advanced) `uml.activity-adv` | UMLActivityDiagram | Frame | ❌ | extensions/essential/uml/toolbox/uml.json |
| Activities (Advanced) `uml.activity-adv` | UMLActivityDiagram | Object Flow | ❌ | extensions/essential/uml/toolbox/uml.json |
| Activities (Advanced) `uml.activity-adv` | UMLActivityDiagram | Exception Handler | ❌ | extensions/essential/uml/toolbox/uml.json |
| Activities (Advanced) `uml.activity-adv` | UMLActivityDiagram | Activity Interrupt | ❌ | extensions/essential/uml/toolbox/uml.json |
| Activities (Advanced) `uml.activity-adv` | UMLActivityDiagram | Activity Edge Connector | ❌ | extensions/essential/uml/toolbox/uml.json |
| Profiles `uml.profile` | UMLProfileDiagram | MetaClass | ✅ | extensions/essential/uml/toolbox/uml.json |
| Profiles `uml.profile` | UMLProfileDiagram | Stereotype | ✅ | extensions/essential/uml/toolbox/uml.json |
| Profiles `uml.profile` | UMLProfileDiagram | Enumeration | ✅ | extensions/essential/uml/toolbox/uml.json |
| Profiles `uml.profile` | UMLProfileDiagram | Extension | ✅ | extensions/essential/uml/toolbox/uml.json |
| Profiles `uml.profile` | UMLProfileDiagram | Generalization | ✅ | extensions/essential/uml/toolbox/uml.json |
| Wireframe Frames `wireframe.frames` | WFWireframeDiagram | Frame | ❌ | extensions/essential/wireframe/toolbox/wireframe.json |
| Wireframe Frames `wireframe.frames` | WFWireframeDiagram | Frame (Mobile) | ❌ | extensions/essential/wireframe/toolbox/wireframe.json |
| Wireframe Frames `wireframe.frames` | WFWireframeDiagram | Frame (Web) | ❌ | extensions/essential/wireframe/toolbox/wireframe.json |
| Wireframe Frames `wireframe.frames` | WFWireframeDiagram | Frame (Desktop) | ❌ | extensions/essential/wireframe/toolbox/wireframe.json |
| Wireframe Components `wireframe.components` | WFWireframeDiagram | Button | ❌ | extensions/essential/wireframe/toolbox/wireframe.json |
| Wireframe Components `wireframe.components` | WFWireframeDiagram | Text | ❌ | extensions/essential/wireframe/toolbox/wireframe.json |
| Wireframe Components `wireframe.components` | WFWireframeDiagram | Radio | ❌ | extensions/essential/wireframe/toolbox/wireframe.json |
| Wireframe Components `wireframe.components` | WFWireframeDiagram | Checkbox | ❌ | extensions/essential/wireframe/toolbox/wireframe.json |
| Wireframe Components `wireframe.components` | WFWireframeDiagram | Switch | ❌ | extensions/essential/wireframe/toolbox/wireframe.json |
| Wireframe Components `wireframe.components` | WFWireframeDiagram | Link | ❌ | extensions/essential/wireframe/toolbox/wireframe.json |
| Wireframe Components `wireframe.components` | WFWireframeDiagram | Tab List | ❌ | extensions/essential/wireframe/toolbox/wireframe.json |
| Wireframe Components `wireframe.components` | WFWireframeDiagram | Tab | ❌ | extensions/essential/wireframe/toolbox/wireframe.json |
| Wireframe Components `wireframe.components` | WFWireframeDiagram | Input | ❌ | extensions/essential/wireframe/toolbox/wireframe.json |
| Wireframe Components `wireframe.components` | WFWireframeDiagram | Dropdown | ❌ | extensions/essential/wireframe/toolbox/wireframe.json |
| Wireframe Components `wireframe.components` | WFWireframeDiagram | Panel | ❌ | extensions/essential/wireframe/toolbox/wireframe.json |
| Wireframe Components `wireframe.components` | WFWireframeDiagram | Image | ❌ | extensions/essential/wireframe/toolbox/wireframe.json |
| Wireframe Components `wireframe.components` | WFWireframeDiagram | Separator | ❌ | extensions/essential/wireframe/toolbox/wireframe.json |
| Wireframe Components `wireframe.components` | WFWireframeDiagram | Avatar | ❌ | extensions/essential/wireframe/toolbox/wireframe.json |
| Wireframe Components `wireframe.components` | WFWireframeDiagram | Slider | ❌ | extensions/essential/wireframe/toolbox/wireframe.json |
