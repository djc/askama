use askama::Template;

#[derive(Template)]
#[template(
    ext = "txt",
    source = r#"
{%- match good -%}
    // Help, I forgot how to write comments!
    {%- when true %}
        good
    {%- when _ -%}
        bad
{%- endmatch -%}"#
)]
struct MatchWithExtra {
    good: bool,
}

fn main() {
}
