#[allow(dead_code)]
#[test]
fn tests_pass() {
    let t = trybuild::TestCases::new();
    t.pass("tests/simple_web_framework/*-pass.rs");
}

#[allow(dead_code)]
#[rustversion::attr(stable(1.75), test)]
fn tests_fail() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/simple_web_framework/*-fail.rs");
}
