pub(crate) enum Screen {
    Launcher,
    Studio { id: String },
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub(crate) enum Template {
    Empty,
    Hello,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub(crate) enum PipelineJobKind {
    Idle,
    Check,
    Build,
    BuildRun,
}

impl PipelineJobKind {
    pub(crate) fn is_busy(self) -> bool {
        !matches!(self, PipelineJobKind::Idle)
    }
}

/// View qatlamida markaziy panel: graf muharriri yoki jonli UI runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum ViewStudioMode {
    #[default]
    GraphEditor,
    ViewRuntime,
}
