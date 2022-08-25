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

pub fn user_tweets(
    username: &str,
    tweets: Vec<twitter_v2::Tweet>,
    referenced_tweets: Vec<twitter_v2::Tweet>,
    media_objects: Vec<twitter_v2::Media>,
) -> Markup {
    let title = format!("Tweets of @{}", username);
    html! {
        (header(&title))
            body class="max-w-3xl mx-auto" {
                h1 {
                    "Tweets of "
                    a href={(format!("https://twitter.com/{}", username))} { "@"(username) }
                }
                @for (tweet_idx, tweet) in tweets.iter().enumerate() {
                    (tweet_to_html(&tweet, &referenced_tweets, &media_objects))

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

struct Image {
    url: String,
    width: usize,
    height: usize,
}

type TweetText = Vec<TweetTextPart>;

fn tweet_to_html(
    tweet: &twitter_v2::Tweet,
    referenced_tweets: &[twitter_v2::Tweet],
    media_objects: &[twitter_v2::Media],
) -> Markup {
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

    let mut media_keys = Vec::new();
    if let Some(attachments) = &tweet.attachments {
        if let Some(attachment_media_keys) = &attachments.media_keys {
            media_keys = attachment_media_keys.clone();
        }
    }
    let mut images = Vec::new();
    for media_object in media_objects {
        if media_keys.contains(&media_object.media_key)
            && media_object.kind == twitter_v2::data::MediaType::Photo
        {
            let url: String;
            let width: usize;
            let height: usize;
            if let Some(media_url) = &media_object.url {
                url = media_url.to_string();
            } else {
                log::warn!("No url for media object");
                continue;
            }
            if let Some(media_width) = &media_object.width {
                width = *media_width;
            } else {
                log::warn!("No width for media object");
                continue;
            }
            if let Some(media_height) = &media_object.height {
                height = *media_height
            } else {
                log::warn!("No height for media object");
                continue;
            }
            images.push(Image {
                url: url,
                width: width,
                height: height,
            });
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

    let mut output = html! {
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
            @for image in images {
                img src=(image.url) width=(image.width) height=(image.height);
            }
        }
    };

    if let Some(tweet_referenced_tweets) = &tweet.referenced_tweets {
        for referenced_tweet in tweet_referenced_tweets {
            if referenced_tweet.kind == twitter_v2::data::ReferencedTweetKind::Quoted {
                for ref_tweet in referenced_tweets {
                    if ref_tweet.id == referenced_tweet.id {
                        output = html! {
                            (output)
                                blockquote {
                                    (tweet_to_html(ref_tweet, &Vec::new(), media_objects))
                                }
                        };
                    }
                }
            }
        }
    }

    output
}
