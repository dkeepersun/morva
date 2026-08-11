use crate::{
    Declaration, Diagnostic, Document, Notice, RebaseSpans, Span, System,
    analyze as analyze_document, check, parse,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourceId(pub usize);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectSource {
    pub id: SourceId,
    pub name: String,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectDiagnostic {
    Project {
        diagnostic: Diagnostic,
    },
    Source {
        source_id: SourceId,
        local_diagnostic: Diagnostic,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectNotice {
    pub source_id: SourceId,
    pub local_notice: Notice,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectAnalysisReport {
    pub errors: Vec<ProjectDiagnostic>,
    pub notices: Vec<ProjectNotice>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectFinding<'a> {
    Error(&'a ProjectDiagnostic),
    Notice(&'a ProjectNotice),
}

impl ProjectAnalysisReport {
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn findings(&self) -> Vec<ProjectFinding<'_>> {
        let mut findings = self
            .errors
            .iter()
            .map(ProjectFinding::Error)
            .chain(self.notices.iter().map(ProjectFinding::Notice))
            .collect::<Vec<_>>();
        findings.sort_by_key(|finding| match finding {
            ProjectFinding::Error(ProjectDiagnostic::Project { diagnostic }) => (
                0usize,
                0usize,
                diagnostic.span.start,
                diagnostic.span.end,
                0u8,
            ),
            ProjectFinding::Error(ProjectDiagnostic::Source {
                source_id,
                local_diagnostic,
            }) => (
                1,
                source_id.0,
                local_diagnostic.span.start,
                local_diagnostic.span.end,
                0,
            ),
            ProjectFinding::Notice(notice) => (
                1,
                notice.source_id.0,
                notice.local_notice.span.start,
                notice.local_notice.span.end,
                1,
            ),
        });
        findings
    }
}

impl ProjectDiagnostic {
    pub fn source_id(&self) -> Option<SourceId> {
        match self {
            Self::Project { .. } => None,
            Self::Source { source_id, .. } => Some(*source_id),
        }
    }

    pub fn diagnostic(&self) -> &Diagnostic {
        match self {
            Self::Project { diagnostic } => diagnostic,
            Self::Source {
                local_diagnostic, ..
            } => local_diagnostic,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalSourceSpan {
    pub source_id: SourceId,
    pub local_span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SourceMapEntry {
    source_id: SourceId,
    base: usize,
    length: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceMap {
    entries: Vec<SourceMapEntry>,
}

impl SourceMap {
    /// Maps a merged-document virtual span to a source-local span.
    /// Source-local diagnostic spans must not be passed to this method.
    pub fn locate_virtual_span(&self, span: Span) -> Option<LocalSourceSpan> {
        if span.start > span.end {
            return None;
        }
        self.entries.iter().find_map(|entry| {
            let end = entry.base.checked_add(entry.length)?;
            (span.start >= entry.base && span.end >= entry.base && span.end <= end).then_some(
                LocalSourceSpan {
                    source_id: entry.source_id,
                    local_span: Span {
                        start: span.start - entry.base,
                        end: span.end - entry.base,
                    },
                },
            )
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Project {
    sources: Vec<ProjectSource>,
    document: Document,
    source_map: SourceMap,
}

impl Project {
    pub fn parse<I, N, S>(sources: I) -> Result<Self, Vec<ProjectDiagnostic>>
    where
        I: IntoIterator<Item = (N, S)>,
        N: Into<String>,
        S: Into<String>,
    {
        let sources = sources
            .into_iter()
            .enumerate()
            .map(|(index, (name, source))| ProjectSource {
                id: SourceId(index),
                name: name.into(),
                source: source.into(),
            })
            .collect::<Vec<_>>();
        if sources.is_empty() {
            return Err(vec![ProjectDiagnostic::Project {
                diagnostic: Diagnostic::new(
                    "MORVA2023",
                    "project must contain at least one source",
                    Span::default(),
                ),
            }]);
        }

        let mut parsed = Vec::with_capacity(sources.len());
        let mut diagnostics = Vec::new();
        for source in &sources {
            match parse(&source.source) {
                Ok(document) => parsed.push((source.id, document)),
                Err(items) => diagnostics.extend(items.into_iter().map(|diagnostic| {
                    ProjectDiagnostic::Source {
                        source_id: source.id,
                        local_diagnostic: diagnostic,
                    }
                })),
            }
        }
        if !diagnostics.is_empty() {
            return Err(diagnostics);
        }

        let mut system_name = None::<String>;
        for (source_id, document) in &parsed {
            let systems = document
                .declarations
                .iter()
                .filter_map(|declaration| match declaration {
                    Declaration::System(system) => Some(system),
                    _ => None,
                })
                .collect::<Vec<_>>();
            if systems.len() != 1 || document.declarations.len() != 1 {
                let (message, span) = invalid_shell_diagnostic(document, &systems);
                diagnostics.push(ProjectDiagnostic::Source {
                    source_id: *source_id,
                    local_diagnostic: Diagnostic::new("MORVA2020", message, span),
                });
                continue;
            }
            let current = &systems[0].name;
            if let Some(expected) = &system_name {
                if current.text != *expected {
                    diagnostics.push(ProjectDiagnostic::Source {
                        source_id: *source_id,
                        local_diagnostic: Diagnostic::new(
                            "MORVA2021",
                            format!(
                                "project system '{}' does not match expected system '{expected}'",
                                current.text
                            ),
                            current.span,
                        ),
                    });
                }
            } else {
                system_name = Some(current.text.clone());
            }
        }
        if !diagnostics.is_empty() {
            return Err(diagnostics);
        }

        let mut entries = Vec::with_capacity(sources.len());
        let mut children = Vec::new();
        let mut merged_name = None;
        let mut merged_system_span = None;
        let mut merged_document_span = None;
        let mut base = 0usize;
        let source_count = sources.len();
        for (index, ((source_id, mut document), source)) in
            parsed.into_iter().zip(&sources).enumerate()
        {
            document
                .rebase_spans(base)
                .ok_or_else(|| vec![overflow_diagnostic(source_id)])?;
            if merged_document_span.is_none() {
                merged_document_span = Some(document.span);
            }
            entries.push(SourceMapEntry {
                source_id,
                base,
                length: source.source.len(),
            });
            let Declaration::System(system) = document.declarations.remove(0) else {
                unreachable!("project system shape was validated")
            };
            if merged_name.is_none() {
                merged_name = Some(system.name.clone());
                merged_system_span = Some(system.span);
            }
            children.extend(system.declarations);
            if index + 1 < source_count {
                base = base
                    .checked_add(source.source.len())
                    .and_then(|value| value.checked_add(1))
                    .ok_or_else(|| vec![overflow_diagnostic(source_id)])?;
            }
        }
        let system = System {
            name: merged_name.expect("non-empty project has a system name"),
            declarations: children,
            span: merged_system_span.expect("non-empty project has a system span"),
        };
        let document = Document {
            span: merged_document_span.expect("non-empty project has a document span"),
            declarations: vec![Declaration::System(system)],
        };
        Ok(Self {
            sources,
            document,
            source_map: SourceMap { entries },
        })
    }

    pub fn sources(&self) -> &[ProjectSource] {
        &self.sources
    }

    pub fn document(&self) -> &Document {
        &self.document
    }

    pub fn source_map(&self) -> &SourceMap {
        &self.source_map
    }

    pub fn check(&self) -> Vec<ProjectDiagnostic> {
        check(&self.document)
            .into_iter()
            .map(|diagnostic| {
                let location = self
                    .source_map
                    .locate_virtual_span(diagnostic.span)
                    .expect("all project diagnostics originate from a project source");
                ProjectDiagnostic::Source {
                    source_id: location.source_id,
                    local_diagnostic: Diagnostic {
                        span: location.local_span,
                        ..diagnostic
                    },
                }
            })
            .collect()
    }

    pub fn analyze(&self) -> ProjectAnalysisReport {
        let report = analyze_document(&self.document);
        let errors = report
            .errors
            .into_iter()
            .map(|diagnostic| {
                let location = self
                    .source_map
                    .locate_virtual_span(diagnostic.span)
                    .expect("all project diagnostics originate from a project source");
                ProjectDiagnostic::Source {
                    source_id: location.source_id,
                    local_diagnostic: Diagnostic {
                        span: location.local_span,
                        ..diagnostic
                    },
                }
            })
            .collect();
        let notices = report
            .notices
            .into_iter()
            .map(|notice| {
                let location = self
                    .source_map
                    .locate_virtual_span(notice.span)
                    .expect("all project notices originate from a project source");
                ProjectNotice {
                    source_id: location.source_id,
                    local_notice: Notice {
                        span: location.local_span,
                        ..notice
                    },
                }
            })
            .collect();
        ProjectAnalysisReport { errors, notices }
    }

    /// Maps a span from the merged `document()` virtual offset space to a local source span.
    pub fn locate_virtual_span(&self, span: Span) -> Option<LocalSourceSpan> {
        self.source_map.locate_virtual_span(span)
    }
}

fn invalid_shell_diagnostic(document: &Document, systems: &[&System]) -> (&'static str, Span) {
    if systems.len() > 1 {
        return (
            "project source contains multiple top-level systems",
            systems[1].name.span,
        );
    }
    if systems.len() == 1 {
        let span = document
            .declarations
            .iter()
            .find(|declaration| !matches!(declaration, Declaration::System(_)))
            .map_or(document.span, Declaration::span);
        return (
            "project source contains a declaration outside its system",
            span,
        );
    }
    (
        "project source must contain one top-level system",
        document
            .declarations
            .first()
            .map_or(document.span, Declaration::span),
    )
}

fn overflow_diagnostic(source_id: SourceId) -> ProjectDiagnostic {
    ProjectDiagnostic::Source {
        source_id,
        local_diagnostic: Diagnostic::new(
            "MORVA2022",
            "project source offsets exceed the supported address space",
            Span::default(),
        ),
    }
}
