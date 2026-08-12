use crate::{Declaration, Diagnostic, Document, SoftBehaviorKind, Span};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NoticeKind {
    CompatibilityContainer {
        kind: String,
        name: String,
    },
    ActionSoftBehavior {
        action: String,
        behavior: SoftBehaviorKind,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notice {
    pub code: &'static str,
    pub message: String,
    pub span: Span,
    pub kind: NoticeKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnalysisReport {
    pub errors: Vec<Diagnostic>,
    pub notices: Vec<Notice>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnalysisFinding<'a> {
    Error(&'a Diagnostic),
    Notice(&'a Notice),
}

impl AnalysisFinding<'_> {
    pub fn span(&self) -> Span {
        match self {
            Self::Error(error) => error.span,
            Self::Notice(notice) => notice.span,
        }
    }
}

impl AnalysisReport {
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn findings(&self) -> Vec<AnalysisFinding<'_>> {
        let mut findings = self
            .errors
            .iter()
            .map(AnalysisFinding::Error)
            .chain(self.notices.iter().map(AnalysisFinding::Notice))
            .collect::<Vec<_>>();
        findings.sort_by_key(|finding| {
            let span = finding.span();
            let severity = match finding {
                AnalysisFinding::Error(_) => 0u8,
                AnalysisFinding::Notice(_) => 1u8,
            };
            (span.start, span.end, severity)
        });
        findings
    }
}

pub fn analyze(document: &Document) -> AnalysisReport {
    let errors = crate::semantic::check(document);
    let mut notices = Vec::new();
    collect_notices(&document.declarations, &mut notices);
    notices.sort_by_key(|notice| (notice.span.start, notice.span.end));
    AnalysisReport { errors, notices }
}

fn collect_notices(declarations: &[Declaration], notices: &mut Vec<Notice>) {
    for declaration in declarations {
        if let Declaration::Container(container) = declaration {
            notices.push(Notice {
                code: "MORVA5001",
                message: format!(
                    "compatibility {} '{}' is parsed but not semantically validated",
                    container.kind, container.name.text
                ),
                span: container.name.span,
                kind: NoticeKind::CompatibilityContainer {
                    kind: container.kind.clone(),
                    name: container.name.text.clone(),
                },
            });
        }
        if let Declaration::Action(action) = declaration {
            for behavior in &action.soft_behaviors {
                notices.push(Notice {
                    code: "MORVA5002",
                    message: format!(
                        "action '{}' soft behavior '{}' is parsed but not semantically validated or executed by simulation",
                        action.name.text,
                        behavior.kind.as_str()
                    ),
                    span: behavior.span,
                    kind: NoticeKind::ActionSoftBehavior {
                        action: action.name.text.clone(),
                        behavior: behavior.kind,
                    },
                });
            }
        }
        collect_notices(declaration.declarations(), notices);
    }
}
