use askama::Template;

#[derive(Template)]
#[template(source = "{% block foo %}{% endblock not_foo %}", ext = "html")]
struct NameMismatchEndBlock;

fn main() {
}
