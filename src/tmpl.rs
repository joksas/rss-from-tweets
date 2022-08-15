use maud::{html, Markup, DOCTYPE};
use twitter_v2;

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

pub fn user_tweets(tweets: Vec<twitter_v2::Tweet>) -> Markup {
    let title = "User tweets";
    html! {
        (header(title))
            body {
                @for (idx, tweet) in tweets.iter().enumerate() {
                    @let paragraphs = tweet_to_parts(tweet);

                    @for paragraph in paragraphs {
                        p {
                            @for (line_idx, line) in paragraph.iter().enumerate() {
                                @for part in line {
                                    @match part {
                                        TweetPart::Text(text) => {
                                            (text)
                                        }
                                        TweetPart::Link(url, text) => {
                                            a href=(url) { (text) }
                                        }
                                    }
                                }
                                @if line_idx < paragraph.len() - 1 {
                                    br;
                                }
                            }
                        }
                    }

                    @if idx < tweets.len() - 1 {
                        hr;
                    }
                }
            }
    }
}

pub enum TweetPart {
    Text(String),
    Link(String, String),
}

fn tweet_to_parts(tweet: &twitter_v2::Tweet) -> Vec<Vec<Vec<TweetPart>>> {
    let mut paragraphs: Vec<Vec<Vec<TweetPart>>> = Vec::new();
    let mut current_idx = 0;
    let mut line_start_idx = 0;
    let mut newline_adder = 0;

    let paragraph_texts = tweet.text.split("\n\n");
    for paragraph_text in paragraph_texts {
        let line_texts = paragraph_text.split("\n");
        let mut lines: Vec<Vec<TweetPart>> = Vec::new();
        for line_text in line_texts {
            let mut parts: Vec<TweetPart> = Vec::new();
            if let Some(entities) = &tweet.entities {
                if let Some(urls) = &entities.urls {
                    for url in urls {
                        if url.start >= line_start_idx
                            && url.end <= line_start_idx + line_text.len()
                        {
                            parts.push(TweetPart::Text(
                                tweet.text[current_idx..url.start + newline_adder].to_string(),
                            ));
                            parts.push(TweetPart::Link(
                                url.expanded_url.clone(),
                                url.display_url.clone(),
                            ));
                            current_idx = url.end + newline_adder;
                        }
                    }
                }
            }
            parts.push(TweetPart::Text(
                tweet.text[current_idx..line_start_idx + line_text.len()].to_string(),
            ));
            lines.push(parts);
            current_idx = line_start_idx + line_text.len() + 1;
            line_start_idx += line_text.len() + 1;
        }
        paragraphs.push(lines);
        newline_adder += 2 * paragraph_text.matches('\n').count() + 2;
    }

    paragraphs
}

fn parts_to_html(parts: Vec<TweetPart>) -> Markup {
    html! {
        @for part in parts {
            @match part {
                TweetPart::Text(text) => {
                    (text)
                }
                TweetPart::Link(url, display_url) => {
                    a href=(url) { (display_url) }
                }
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
