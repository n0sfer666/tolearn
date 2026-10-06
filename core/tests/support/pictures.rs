pub const UNSAFE: [(&str, &str); 17] = [
    (
        "svg-script",
        r#"<svg xmlns="http://www.w3.org/2000/svg"><script>alert(1)</script></svg>"#,
    ),
    (
        "svg-namespaced-script",
        r#"<s:svg xmlns:s="http://www.w3.org/2000/svg"><s:script/></s:svg>"#,
    ),
    (
        "svg-onload",
        r#"<svg xmlns="http://www.w3.org/2000/svg" onload="alert(1)"/>"#,
    ),
    (
        "svg-external-href",
        r#"<svg xmlns="http://www.w3.org/2000/svg"><image href="https://example.com/a.png"/></svg>"#,
    ),
    (
        "svg-external-xlink",
        r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink"><a xlink:href="javascript:alert(1)"><text>x</text></a></svg>"#,
    ),
    (
        "svg-entity",
        r#"<!DOCTYPE svg [<!ENTITY s "&#60;script&#62;">]><svg xmlns="http://www.w3.org/2000/svg"><text>&s;</text></svg>"#,
    ),
    (
        "svg-stylesheet",
        r#"<?xml-stylesheet href="https://example.com/a.css"?><svg xmlns="http://www.w3.org/2000/svg"/>"#,
    ),
    (
        "svg-not-xml",
        r#"<svg xmlns="http://www.w3.org/2000/svg"><g></svg>"#,
    ),
    (
        "svg-animated-href",
        r##"<svg xmlns="http://www.w3.org/2000/svg"><a href="#x"><animate attributeName="href" to="https://example.com/"/><text>x</text></a></svg>"##,
    ),
    (
        "svg-set-xlink",
        r##"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink"><a xlink:href="#x"><set attributeName="xlink:href" to="javascript:alert(1)"/><text>x</text></a></svg>"##,
    ),
    (
        "svg-escaped-attribute-name",
        r##"<svg xmlns="http://www.w3.org/2000/svg"><a href="#x"><set attributeName="h&#114;ef" to="javascript:alert(1)"/><text>x</text></a></svg>"##,
    ),
    (
        "svg-foreign-object",
        r#"<svg xmlns="http://www.w3.org/2000/svg"><foreignObject><div xmlns="http://www.w3.org/1999/xhtml">x</div></foreignObject></svg>"#,
    ),
    (
        "svg-iframe",
        r#"<svg xmlns="http://www.w3.org/2000/svg"><iframe src="https://example.com/"/></svg>"#,
    ),
    (
        "svg-embed",
        r#"<svg xmlns="http://www.w3.org/2000/svg"><embed src="https://example.com/a.swf"/></svg>"#,
    ),
    (
        "svg-object",
        r#"<svg xmlns="http://www.w3.org/2000/svg"><object data="https://example.com/a.svg"/></svg>"#,
    ),
    (
        "svg-declared-encoding",
        r#"<?xml version="1.0" encoding="ISO-8859-1"?><svg xmlns="http://www.w3.org/2000/svg"/>"#,
    ),
    (
        "svg-control-byte",
        "<svg xmlns=\"http://www.w3.org/2000/svg\"><text>\u{1b}[31m</text></svg>",
    ),
];
