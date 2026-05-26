//! Node maturity and support hints for IDE + compiler messages.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeMaturity {
    /// Full IR + emit + runtime preview.
    Stable,
    /// IR + emit; behavior may be stubbed.
    Beta,
    /// Listed in picker but not lowered yet.
    Planned,
}

pub fn node_maturity(kind: &str) -> NodeMaturity {
    match kind {
        crate::NODE_START | crate::NODE_LOG | crate::NODE_ASSIGN | crate::NODE_IF => {
            NodeMaturity::Stable
        }
        crate::NODE_DB_READ | crate::NODE_SUBGRAPH => NodeMaturity::Beta,
        crate::NODE_UI_PAGE
        | crate::NODE_UI_BUTTON
        | crate::NODE_UI_LABEL
        | crate::NODE_UI_INPUT
        | crate::NODE_UI_EVENT => NodeMaturity::Beta,
        crate::NODE_API_ROUTE | crate::NODE_API_QUERY | crate::NODE_EMIT_UI => NodeMaturity::Beta,
        _ => NodeMaturity::Planned,
    }
}

pub fn maturity_label(m: NodeMaturity) -> &'static str {
    match m {
        NodeMaturity::Stable => "tayyor",
        NodeMaturity::Beta => "beta",
        NodeMaturity::Planned => "reja",
    }
}

pub fn node_support_hint(kind: &str) -> &'static str {
    match kind {
        crate::NODE_START => "Start faqat kirish nuqtasi; kod generatsiya qilinmaydi.",
        crate::NODE_LOG => "Log → IR print.",
        crate::NODE_ASSIGN => "Assign → IR o‘zgaruvchi.",
        crate::NODE_IF => "If → IR branch (true/false/done portlari).",
        crate::NODE_DB_READ => "DB Read → IR db_read (mock ma’lumot).",
        crate::NODE_SUBGRAPH => "Subgraph → manifestdagi `subgraphs[]` modulini chaqiradi.",
        crate::NODE_UI_PAGE
        | crate::NODE_UI_BUTTON
        | crate::NODE_UI_LABEL
        | crate::NODE_UI_INPUT
        | crate::NODE_UI_EVENT => "View nodlari View qatlamida emit-view orqali spec beradi.",
        crate::NODE_API_ROUTE | crate::NODE_API_QUERY | crate::NODE_EMIT_UI => {
            "Bridge nodlari Bridge qatlamida emit-bridge orqali stub beradi."
        }
        _ => "Bu nod turi hali qo‘llab-quvvatlanmaydi.",
    }
}
