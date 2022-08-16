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

pub fn user_tweets(tweets: Vec<twitter_v2::Tweet>) -> Markup {
    let title = "User tweets";
    html! {
        (header(title))
            body class="max-w-3xl mx-auto" {
                @for (tweet_idx, tweet) in tweets.iter().enumerate() {
                    (tweet_to_html(&tweet))

                        @if tweet_idx < tweets.len() - 1 {
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

fn utf8_idx(s: &String, idx: usize) -> usize {
    match s.char_indices().map(|(i, _)| i).nth(idx) {
        Some(idx) => idx,
        None => idx,
    }
}

fn tweet_to_html(tweet: &twitter_v2::Tweet) -> Markup {
    let par_separator = "\n\n";
    let par_starts: Vec<usize> = tweet.text.match_indices("\n\n").map(|(i, _)| i).collect();

    let mut par_limits = vec![0];
    for par_start in par_starts {
        par_limits.push(utf8_idx(&tweet.text, par_start));
        par_limits.push(utf8_idx(&tweet.text, par_start + par_separator.len()));
    }
    par_limits.push(utf8_idx(&tweet.text, tweet.text.len()));

    let par_limits: Vec<(usize, usize)> = par_limits
        .chunks(2)
        .map(|chunk| (chunk[0], chunk[1]))
        .collect();

    let mut urls = vec![];

    if let Some(entities) = &tweet.entities {
        if let Some(tweet_urls) = &entities.urls {
            for url in tweet_urls {
                urls.push(url);
            }
        }
    }

    let mut tweet_html = html! {};

    for (par_start, par_end) in par_limits {
        let mut start_idx = par_start;
        let mut par_html = html! {};

        for url in urls.iter() {
            let url_start = utf8_idx(&tweet.text, url.start);
            let url_end = utf8_idx(&tweet.text, url.end);
            if url_start > start_idx && url_start < par_end {
                let text = &tweet.text[start_idx..url_start];
                par_html = html! {
                    (par_html)
                    (newlines_to_br(text))
                    a href=(url.expanded_url) {
                        (url.display_url)
                    }
                };
                start_idx = url_end;
            }
        }

        if start_idx < par_end {
            let text = &tweet.text[start_idx..par_end];
            par_html = html! {
                (par_html)
                (newlines_to_br(text))
            };
        }

        tweet_html = html! {
            (tweet_html)
            p { (par_html) }
        };
    }

    tweet_html
}

fn newlines_to_br(par: &str) -> Markup {
    let mut par_html = html! {};
    for (idx, line) in par.split('\n').enumerate() {
        par_html = html! {
            (par_html)
            (line)

            @if idx < par.split('\n').count() - 1 {
                br;
            }
        };
    }
    par_html
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
