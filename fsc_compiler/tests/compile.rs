#![allow(clippy::expect_used)]

use fsc_compiler::{CompileRequest, CompileStage, Severity, compile};

const VALID_SOURCE: &str = "void main() { return; }";

#[test]
fn compiles_source_to_an_fsb_artifact() {
    let artifact =
        compile(CompileRequest::new(VALID_SOURCE, "main")).expect("valid source should compile");

    assert!(artifact.bytes().len() > 32);
    assert_eq!(&artifact.bytes()[0..4], &32_u32.to_be_bytes());
    assert!(artifact.diagnostics().is_empty());
}

#[test]
fn parse_failure_retains_source_location() {
    let source = "void main() { return; }\n}\n";
    let failure = compile(CompileRequest::new(source, "broken"))
        .expect_err("an unmatched closing brace should fail");
    let diagnostic = &failure.diagnostics()[0];

    assert_eq!(diagnostic.stage(), CompileStage::Parse);
    assert_eq!(diagnostic.severity(), Severity::Error);
    assert_eq!(diagnostic.labels().len(), 1);

    let rendered = failure.render("broken.fsc", source);
    assert!(rendered.contains("broken.fsc:2:1"));
    assert!(rendered.contains('^'));
}

#[test]
fn semantic_failure_is_normalized() {
    let source = "void main() { missing(); return; }";
    let failure =
        compile(CompileRequest::new(source, "main")).expect_err("undeclared call should fail");
    let diagnostic = &failure.diagnostics()[0];

    assert_eq!(diagnostic.stage(), CompileStage::Semantic);
    assert!(diagnostic.message().contains("undeclared name `missing`"));
    assert!(diagnostic.labels().is_empty());
}

#[test]
fn invalid_script_name_is_an_assembly_failure() {
    let failure = compile(CompileRequest::new(VALID_SOURCE, "main!"))
        .expect_err("invalid B40 name should fail during serialization");
    let diagnostic = &failure.diagnostics()[0];

    assert_eq!(diagnostic.stage(), CompileStage::Assembly);
    assert!(diagnostic.message().contains("invalid character"));
}
