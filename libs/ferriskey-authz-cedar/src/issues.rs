use ferriskey_domain::common::app_errors::PolicyIssue;
use miette::Diagnostic;

pub(crate) fn issues_from(
    owner: Option<&str>,
    diagnostic: &dyn Diagnostic,
    source: &str,
) -> Vec<PolicyIssue> {
    let related: Vec<PolicyIssue> = diagnostic
        .related()
        .into_iter()
        .flatten()
        .map(|inner| issue(owner, inner, source))
        .collect();

    if related.is_empty() {
        vec![issue(owner, diagnostic, source)]
    } else {
        related
    }
}

pub(crate) fn issue(owner: Option<&str>, diagnostic: &dyn Diagnostic, source: &str) -> PolicyIssue {
    let offset = diagnostic
        .labels()
        .and_then(|mut labels| labels.next())
        .map(|label| label.offset());
    let (line, column) = offset.map_or((None, None), |offset| {
        let (line, column) = line_and_column(source, offset);
        (Some(line), Some(column))
    });

    PolicyIssue {
        policy: owner.map(str::to_string),
        message: message(diagnostic),
        line,
        column,
    }
}

pub(crate) fn plain(owner: Option<&str>, message: String) -> PolicyIssue {
    PolicyIssue {
        policy: owner.map(str::to_string),
        message,
        line: None,
        column: None,
    }
}

fn message(diagnostic: &dyn Diagnostic) -> String {
    match diagnostic.help() {
        Some(help) => format!("{diagnostic}: {help}"),
        None => diagnostic.to_string(),
    }
}

fn line_and_column(source: &str, offset: usize) -> (u32, u32) {
    let before = source.get(..offset).unwrap_or(source);
    let line = before.matches('\n').count() + 1;
    let column = before.rsplit('\n').next().map_or(0, |l| l.chars().count()) + 1;
    (to_u32(line), to_u32(column))
}

fn to_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_offset_maps_to_a_one_based_line_and_column() {
        let source = "ab\ncdé\nf";
        assert_eq!(line_and_column(source, 0), (1, 1));
        assert_eq!(line_and_column(source, 4), (2, 2));
        assert_eq!(line_and_column(source, 8), (3, 1));
    }
}
