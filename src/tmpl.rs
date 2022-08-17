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

pub fn user_tweets(username: &str, tweets: Vec<twitter_v2::Tweet>) -> Markup {
    let title = format!("Tweets of @{}", username);
    html! {
        (header(&title))
            body class="max-w-3xl mx-auto" {
                h1 {
                    "Tweets of "
                    a href={(format!("https://twitter.com/{}", username))} { "@"(username) }
                }
                @for (tweet_idx, tweet) in tweets.iter().enumerate() {
                    (tweet_to_html(&tweet))

                        @if tweet_idx < tweets.len() - 1 {
                            hr;
                        }
                }
            }
    }
}

#[derive(Debug)]
enum TweetTextPart {
    Text(String),
    Link(String, String),
    Newline,
}

struct Link {
    url: String,
    text: String,
    start: usize,
    end: usize,
}

type TweetText = Vec<TweetTextPart>;

fn tweet_to_html(tweet: &twitter_v2::Tweet) -> Markup {
    let chars: Vec<char> = tweet.text.chars().collect();

    let mut urls: Vec<Link> = Vec::new();
    if let Some(entities) = &tweet.entities {
        if let Some(tweet_urls) = &entities.urls {
            for url in tweet_urls {
                urls.push(Link {
                    url: url.url.clone(),
                    text: url.display_url.clone(),
                    start: url.start,
                    end: url.end,
                });
            }
        }
        if let Some(tweet_hashtags) = &entities.hashtags {
            for hashtag in tweet_hashtags {
                urls.push(Link {
                    url: format!("https://twitter.com/hashtag/{}", hashtag.tag),
                    text: format!("#{}", hashtag.tag),
                    start: hashtag.start,
                    end: hashtag.end,
                });
            }
        }
        if let Some(tweet_user_mentions) = &entities.mentions {
            for user_mention in tweet_user_mentions {
                urls.push(Link {
                    url: format!("https://twitter.com/{}", user_mention.username),
                    text: format!("@{}", user_mention.username),
                    start: user_mention.start,
                    end: user_mention.end,
                });
            }
        }
    }

    let mut tweet_text = TweetText::new();
    let mut skip_to_idx = 0;

    for (idx, ch) in chars.iter().enumerate() {
        if idx < skip_to_idx {
            continue;
        }

        if *ch == '\n' {
            tweet_text.push(TweetTextPart::Newline);
            continue;
        }
        for url in urls.iter() {
            if url.start == idx {
                tweet_text.push(TweetTextPart::Link(url.url.clone(), url.text.clone()));
                skip_to_idx = url.end;
            }
        }
        if idx < skip_to_idx {
            continue;
        }

        match tweet_text.last_mut() {
            Some(TweetTextPart::Text(ref mut s)) => {
                s.push(*ch);
            }
            Some(TweetTextPart::Newline) => {
                let new_part = TweetTextPart::Text(ch.to_string());
                tweet_text.push(new_part);
            }
            Some(TweetTextPart::Link(ref mut s, ref mut url)) => {
                let new_part = TweetTextPart::Text(ch.to_string());
                tweet_text.push(new_part);
            }
            None => {
                let new_part = TweetTextPart::Text(ch.to_string());
                tweet_text.push(new_part);
            }
        }
    }

    html! {
        p {
            @for part in tweet_text {
                @match part {
                    TweetTextPart::Text(text) => {
                        (text)
                    }
                    TweetTextPart::Link(url, text) => {
                        a href=(url) {
                            (text)
                        }
                    }
                    TweetTextPart::Newline => {
                        br;
                    }
                }
            }
        }
    }
}
