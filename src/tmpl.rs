use maud::{html, Markup, DOCTYPE};

/// A basic header with a dynamic `page_title`.
fn header(page_title: &str) -> Markup {
    html! {
        (DOCTYPE)
        meta charset="utf-8";
        meta http-equiv="X-UA-Compatible" content="IE=edge";
        meta name="viewport" content="width=device-width, initial-scale=1";
        link href="/style.css" rel="stylesheet";
        title { (page_title) }
    }
}

pub fn page(title: &str, content: Markup) -> Markup {
    html! {
        (header(title))
            body {
                main class="max-w-3xl mx-auto prose" {
                    h1 { (title) }
                    (content)
                }
            }
    }
}

pub fn error_page(code: u16, message: &str) -> Markup {
    let title = format!("{}—{}", code, message);
    html! {
        (header(title.as_str()))
            body {
                main class="max-w-3xl mx-auto prose" {
                    h1 class="text-red-700" { (title) }
                }
            }
    }
}
