use crate::secrets;

use twitter_v2::query::MediaField;
use twitter_v2::query::TweetExpansion;
use twitter_v2::query::TweetField;
use twitter_v2::{authorization, TwitterApi};

pub async fn user_by_username(username: &str) -> Result<twitter_v2::User, String> {
    let secrets = secrets::extract()?;

    let auth = authorization::BearerToken::new(secrets.twitter.bearer_token);

    let user = match TwitterApi::new(auth)
        .get_user_by_username(username)
        .send()
        .await
    {
        Ok(user) => user,
        Err(err) => return Err(err.to_string()),
    };

    let user = match user.into_data() {
        Some(user) => user,
        None => return Err(String::from("User not found.")),
    };

    Ok(user)
}

pub async fn tweets_by_user(
    user: &twitter_v2::User,
    max_results: usize,
) -> Result<
    (
        Vec<twitter_v2::Tweet>, // Original tweets
        Vec<twitter_v2::Tweet>, // Referenced tweets
        Vec<twitter_v2::Media>, // Media from both types of tweets
    ),
    String,
> {
    let secrets = secrets::extract()?;
    let auth = authorization::BearerToken::new(secrets.twitter.bearer_token);

    let tweets_data = match TwitterApi::new(auth)
        .get_user_tweets(user.id)
        .tweet_fields([
            TweetField::Entities,
            TweetField::Attachments,
            TweetField::ReferencedTweets,
        ])
        .media_fields([
            MediaField::MediaKey,
            MediaField::Url,
            MediaField::Type,
            MediaField::Width,
            MediaField::Height,
        ])
        .expansions([TweetExpansion::AttachmentsMediaKeys])
        .max_results(max_results)
        .send()
        .await
    {
        Ok(data) => data,
        Err(err) => return Err(err.to_string()),
    };

    let tweets = match tweets_data.clone().into_data() {
        Some(data) => data,
        None => return Err(String::from("Tweets not found.")),
    };

    let mut media_objects = match tweets_data.clone().into_includes() {
        Some(includes) => match includes.media {
            Some(media_objects) => media_objects,
            None => Vec::new(),
        },
        None => Vec::new(),
    };

    let mut referenced_tweets: Vec<twitter_v2::Tweet> = Vec::new();
    for tweet in tweets.clone() {
        if let Some(tweet_referended_tweets) = tweet.referenced_tweets {
            let referenced_tweet_ids = tweet_referended_tweets.iter().map(|x| x.id).collect();
            match tweets_by_ids(referenced_tweet_ids).await {
                Ok((ref_tweets, ref_media)) => {
                    referenced_tweets.extend(ref_tweets);
                    media_objects.extend(ref_media);
                }
                Err(e) => return Err(e),
            }
        }
    }

    Ok((tweets, referenced_tweets, media_objects))
}

pub async fn tweets_by_ids(
    ids: Vec<twitter_v2::id::NumericId>,
) -> Result<(Vec<twitter_v2::Tweet>, Vec<twitter_v2::Media>), String> {
    let secrets = secrets::extract()?;

    let auth = authorization::BearerToken::new(secrets.twitter.bearer_token);

    let tweet_data = match TwitterApi::new(auth)
        .get_tweets(ids)
        .tweet_fields([TweetField::Entities, TweetField::Attachments])
        .media_fields([
            MediaField::MediaKey,
            MediaField::Url,
            MediaField::Type,
            MediaField::Width,
            MediaField::Height,
        ])
        .expansions([TweetExpansion::AttachmentsMediaKeys])
        .send()
        .await
    {
        Ok(data) => data,
        Err(err) => return Err(err.to_string()),
    };

    let tweets = match tweet_data.clone().into_data() {
        Some(data) => data,
        None => return Err(String::from("Tweet not found.")),
    };

    let media_objects = match tweet_data.clone().into_includes() {
        Some(includes) => match includes.media {
            Some(media_objects) => media_objects,
            None => Vec::new(),
        },
        None => Vec::new(),
    };

    Ok((tweets, media_objects))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_user_by_username() {
        let username = "jack";

        let user = user_by_username(username).await.unwrap();
        // See <https://web.archive.org/web/20220611133626/https://twitter.com/jack/status/49923786786615296>.
        assert_eq!(user.id, 12);
    }
}
