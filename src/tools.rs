// use anyhow::Result;
// use url::Url;

pub fn split_table_name(table_name: &str) -> (&str, &str) {
    let res: Vec<&str> = table_name.split('.').collect();
    let mut table = table_name;
    let mut schema = "public";

    if res.len() != 1 {
        schema = res[0];
        table = res[1];
    }
    (schema, table)
}

#[test]
fn test_split_table_name() {
    assert_eq!(split_table_name(r#"test"#), (r#"public"#, r#"test"#));
    assert_eq!(split_table_name(r#"scheam.test"#), (r#"scheam"#, r#"test"#));
}

pub fn enshure_it_has_schema(table_name: &str) -> String {
    let (schema, table) = split_table_name(table_name);
    format!(r#"{schema}.{table}"#)
}

#[test]
fn test_enshure_it_has_schema() {
    assert_eq!(
        enshure_it_has_schema(r#"test"#),
        r#"public.test"#.to_string()
    );

    assert_eq!(
        enshure_it_has_schema(r#"scheam.test"#),
        r#"scheam.test"#.to_string()
    );
}

pub fn wrap_tables_names(table_name: &str) -> String {
    let (schema, table) = split_table_name(table_name);
    format!(r#""{schema}"."{table}""#)
}

#[test]
fn test_wrap_tables_names() {
    assert_eq!(
        wrap_tables_names(r#"test"#),
        r#""public"."test""#.to_string()
    );

    assert_eq!(
        wrap_tables_names(r#"scheam.test"#),
        r#""scheam"."test""#.to_string()
    );
}
