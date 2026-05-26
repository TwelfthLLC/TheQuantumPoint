use egui::{Color32, Pos2, Vec2};
use graph_model::GraphLayer;

use crate::theme::Palette;

pub(crate) const NODE_W: f32 = 200.0;
pub(crate) const NODE_H: f32 = 88.0;
pub(crate) const MARQUEE_DRAG_THRESHOLD: f32 = 6.0;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum HandleKind {
    ExecIn,
    ExecOut,
    TrueOut,
    FalseOut,
    DoneOut,
    BodyOut,
    TryOut,
    Case1Out,
    Case2Out,
    Case3Out,
    Case4Out,
    Case5Out,
    Case6Out,
    DefaultOut,
    CatchOut,
}

pub enum GraphAction {
    None,
    Changed,
}

pub(crate) enum NodeAction {
    None,
    Select { additive: bool },
    DragStart { offset: Vec2 },
    HandleClick(HandleKind),
    HandleDisconnect(HandleKind),
}

pub(crate) struct Marquee {
    pub start: Pos2,
    pub current: Pos2,
}

pub(crate) struct NodePickerMenu {
    pub open: bool,
    pub screen_pos: Pos2,
    pub world_pos: Pos2,
    pub search: String,
    pub pending_connect: Option<(String, HandleKind)>,
}

pub(crate) struct NodeCatalogEntry {
    pub kind: &'static str,
    pub title: &'static str,
    pub keywords: &'static str,
    pub color_fn: fn(&Palette) -> Color32,
}

const VIEW_CATALOG: &[NodeCatalogEntry] = &[
    NodeCatalogEntry {
        kind: graph_model::NODE_START,
        title: "Start",
        keywords: "start kirish",
        color_fn: |p| p.start,
    },
    NodeCatalogEntry {
        kind: graph_model::NODE_UI_PAGE,
        title: "Page",
        keywords: "page sahifa ui ekran",
        color_fn: |p| p.accent,
    },
    NodeCatalogEntry {
        kind: graph_model::NODE_UI_BUTTON,
        title: "Button",
        keywords: "button tugma click",
        color_fn: |p| p.success,
    },
    NodeCatalogEntry {
        kind: graph_model::NODE_UI_LABEL,
        title: "Label",
        keywords: "label matn text",
        color_fn: |p| p.log,
    },
    NodeCatalogEntry {
        kind: graph_model::NODE_UI_INPUT,
        title: "Input",
        keywords: "input field forma",
        color_fn: |p| p.warn,
    },
    NodeCatalogEntry {
        kind: graph_model::NODE_UI_EVENT,
        title: "Event",
        keywords: "event hodisa on_click",
        color_fn: |p| p.if_node,
    },
];

const CORE_CATALOG: &[NodeCatalogEntry] = &[
    NodeCatalogEntry {
        kind: graph_model::NODE_START,
        title: "Start",
        keywords: "start kirish",
        color_fn: |p| p.start,
    },
    NodeCatalogEntry {
        kind: graph_model::NODE_LOG,
        title: "Log",
        keywords: "log println",
        color_fn: |p| p.log,
    },
    NodeCatalogEntry {
        kind: graph_model::NODE_ASSIGN,
        title: "Assign",
        keywords: "assign let var",
        color_fn: |p| p.assign,
    },
    NodeCatalogEntry {
        kind: graph_model::NODE_IF,
        title: "If",
        keywords: "if shart branch",
        color_fn: |p| p.if_node,
    },
    NodeCatalogEntry {
        kind: graph_model::NODE_DB_READ,
        title: "DB Read",
        keywords: "database db read",
        color_fn: |p| p.success,
    },
    NodeCatalogEntry {
        kind: graph_model::NODE_SUBGRAPH,
        title: "Subgraph",
        keywords: "subgraph module import",
        color_fn: |p| p.accent,
    },
    NodeCatalogEntry {
        kind: graph_model::NODE_WHILE,
        title: "While",
        keywords: "while loop tsikl",
        color_fn: |p| p.if_node,
    },
    NodeCatalogEntry {
        kind: graph_model::NODE_FOR,
        title: "For",
        keywords: "for loop counter",
        color_fn: |p| p.if_node,
    },
    NodeCatalogEntry {
        kind: graph_model::NODE_FOREACH,
        title: "Foreach",
        keywords: "foreach collection loop",
        color_fn: |p| p.if_node,
    },
    NodeCatalogEntry {
        kind: graph_model::NODE_RETURN,
        title: "Return",
        keywords: "return chiqish",
        color_fn: |p| p.danger,
    },
    NodeCatalogEntry {
        kind: graph_model::NODE_SWITCH,
        title: "Switch",
        keywords: "switch match case",
        color_fn: |p| p.warn,
    },
    NodeCatalogEntry {
        kind: graph_model::NODE_BREAK,
        title: "Break",
        keywords: "break loop",
        color_fn: |p| p.danger,
    },
    NodeCatalogEntry {
        kind: graph_model::NODE_CONTINUE,
        title: "Continue",
        keywords: "continue loop",
        color_fn: |p| p.accent,
    },
    NodeCatalogEntry {
        kind: graph_model::NODE_TRY,
        title: "Try",
        keywords: "try catch xato",
        color_fn: |p| p.warn,
    },
    NodeCatalogEntry {
        kind: graph_model::NODE_EXPR,
        title: "Expr",
        keywords: "expression + - * /",
        color_fn: |p| p.assign,
    },
    NodeCatalogEntry {
        kind: graph_model::NODE_ASYNC,
        title: "Async",
        keywords: "async await",
        color_fn: |p| p.accent,
    },
    NodeCatalogEntry {
        kind: graph_model::NODE_FUNCTION,
        title: "Function",
        keywords: "function fn def",
        color_fn: |p| p.assign,
    },
    NodeCatalogEntry {
        kind: graph_model::NODE_CALL,
        title: "Call",
        keywords: "call invoke",
        color_fn: |p| p.log,
    },
    NodeCatalogEntry {
        kind: graph_model::NODE_CONST,
        title: "Const",
        keywords: "const immutable",
        color_fn: |p| p.assign,
    },
    NodeCatalogEntry {
        kind: graph_model::NODE_LIST,
        title: "List",
        keywords: "list array vec",
        color_fn: |p| p.warn,
    },
    NodeCatalogEntry {
        kind: graph_model::NODE_THROW,
        title: "Throw",
        keywords: "throw error panic",
        color_fn: |p| p.danger,
    },
    NodeCatalogEntry {
        kind: graph_model::NODE_AWAIT,
        title: "Await",
        keywords: "await async yield",
        color_fn: |p| p.accent,
    },
    NodeCatalogEntry {
        kind: graph_model::NODE_IMPORT,
        title: "Import",
        keywords: "import module use",
        color_fn: |p| p.success,
    },
    NodeCatalogEntry {
        kind: graph_model::NODE_STRUCT,
        title: "Struct",
        keywords: "struct record type",
        color_fn: |p| p.if_node,
    },
    NodeCatalogEntry {
        kind: graph_model::NODE_ENUM,
        title: "Enum",
        keywords: "enum variant",
        color_fn: |p| p.if_node,
    },
];

const BRIDGE_CATALOG: &[NodeCatalogEntry] = &[
    NodeCatalogEntry {
        kind: graph_model::NODE_START,
        title: "Start",
        keywords: "start kirish",
        color_fn: |p| p.start,
    },
    NodeCatalogEntry {
        kind: graph_model::NODE_API_ROUTE,
        title: "API Route",
        keywords: "api route endpoint http",
        color_fn: |p| p.accent,
    },
    NodeCatalogEntry {
        kind: graph_model::NODE_API_QUERY,
        title: "API Query",
        keywords: "query so'rov fetch",
        color_fn: |p| p.warn,
    },
    NodeCatalogEntry {
        kind: graph_model::NODE_EMIT_UI,
        title: "Emit UI",
        keywords: "emit ui view signal",
        color_fn: |p| p.danger,
    },
];

pub(crate) fn catalog_for(layer: GraphLayer) -> &'static [NodeCatalogEntry] {
    match layer {
        GraphLayer::View => VIEW_CATALOG,
        GraphLayer::Bridge => BRIDGE_CATALOG,
        GraphLayer::Core => CORE_CATALOG,
    }
}
