use askama::Template;

struct Post {
    id: u32,
}

struct Client<'a> {
    can_post_ids: &'a [u32],
    can_update_ids: &'a [u32],
}

impl Client<'_> {
    fn can_post(&self, post: &Post) -> bool {
        self.can_post_ids.contains(&post.id)
    }

    fn can_update(&self, post: &Post) -> bool {
        self.can_update_ids.contains(&post.id)
    }
}

#[derive(Template)]
#[template(
    source = r#"
{%- match (client.can_post(post), client.can_update(post)) -%}
    {%- when (false, false) -%}
        No!
    {%- when (can_post, can_update) -%}
        <ul>
        {%- if can_post -%}<li>post</li>{%- endif -%}
        {%- if can_update -%}<li>update</li>{%- endif -%}
        </ul>
{%- endmatch -%}
"#,
    ext = "txt"
)]
struct TupleTemplate<'a> {
    client: &'a Client<'a>,
    post: &'a Post,
}

#[test]
fn test_tuple() {
    let template = TupleTemplate {
        client: &Client {
            can_post_ids: &[1, 2],
            can_update_ids: &[2, 3],
        },
        post: &Post { id: 1 },
    };
    assert_eq!(template.render().unwrap(), "<ul><li>post</li></ul>");

    let template = TupleTemplate {
        client: &Client {
            can_post_ids: &[1, 2],
            can_update_ids: &[2, 3],
        },
        post: &Post { id: 2 },
    };
    assert_eq!(
        template.render().unwrap(),
        "<ul><li>post</li><li>update</li></ul>"
    );

    let template = TupleTemplate {
        client: &Client {
            can_post_ids: &[1, 2],
            can_update_ids: &[2, 3],
        },
        post: &Post { id: 3 },
    };
    assert_eq!(template.render().unwrap(), "<ul><li>update</li></ul>");

    let template = TupleTemplate {
        client: &Client {
            can_post_ids: &[1, 2],
            can_update_ids: &[2, 3],
        },
        post: &Post { id: 4 },
    };
    assert_eq!(template.render().unwrap(), "No!");
}
