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

fn plain_text_to_html(text: &str) -> Markup {
    let lines = text.split("\n\n");

    html! {
        @for line in lines {
            p {
                @let els: Vec<String> = line.split('\n').map(str::to_string).collect();
                @for (idx, el) in els.iter().enumerate() {
                    (el)
                    @if idx + 1 < els.len() {
                        br;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plain_text_to_html() {
        let test_cases = [
            (
                "The new #TwitterAPI includes some improvements to the Tweet payload. You’re probably wondering — what are the main differences? 🧐\n\nIn this video, @SuhemParack compares the v1.1 Tweet payload with what you’ll find using our v2 endpoints. https://t.co/CjneyMpgCq",
                "<p>The new #TwitterAPI includes some improvements to the Tweet payload. You’re probably wondering — what are the main differences? 🧐</p><p>In this video, @SuhemParack compares the v1.1 Tweet payload with what you’ll find using our v2 endpoints. https://t.co/CjneyMpgCq</p>",
                ),
            (
                "The new #TwitterAPI includes some improvements to the Tweet payload. You’re probably wondering — what are the main differences? 🧐\nIn this video, @SuhemParack compares the v1.1 Tweet payload with what you’ll find using our v2 endpoints. https://t.co/CjneyMpgCq",
                "<p>The new #TwitterAPI includes some improvements to the Tweet payload. You’re probably wondering — what are the main differences? 🧐<br>In this video, @SuhemParack compares the v1.1 Tweet payload with what you’ll find using our v2 endpoints. https://t.co/CjneyMpgCq</p>",
                ),
        ];

        for (plain_text, expected_html) in test_cases {
            let actual_html = plain_text_to_html(plain_text);
            assert_eq!(actual_html.into_string(), expected_html);
        }
    }
}
