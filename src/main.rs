use dotenv::dotenv;

use std::{collections::HashMap, env, error::Error};

use serenity::{
    async_trait,
    builder::{CreateEmbed, CreateMessage},
    http::Http,
    model::{channel::Message, gateway::Ready, id::ChannelId},
    prelude::*,
};

use reqwest;

use serde::{de::DeserializeOwned, Deserialize};

use chrono::{Duration as ChronoDuration, Local, NaiveDate, Timelike};

use tokio::time::sleep;

struct Handler {}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!ping" {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                println!("Error sending message: {why:?}");
            }
        }

        /* if msg.author.id == KATO_ID {
            if let Err(why) = msg.channel_id.say(&ctx.http, "!!Cállese Kato gei!!").await {
                println!("Error enviando mensaje: {:?}", why);
            }
        } */

        /* if msg.author.id == LEX_ID {
            if let Err(why) = msg
                .channel_id
                .say(&ctx.http, "Man Lexan, cálmese, es solo un ARAM! 🙏")
                .await
            {
                println!("Error enviando mensaje: {:?}", why);
            }
        } */
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

async fn fetch<T: DeserializeOwned>(url: &str) -> Result<T, Box<dyn Error>> {
    let response = reqwest::get(url).await?.json::<T>().await?;

    Ok(response)
}

#[derive(Deserialize, Debug)]
struct Response {
    MRData: MRData,
}

#[derive(Deserialize, Debug)]
struct MRData {
    RaceTable: RaceTable,
}

#[derive(Deserialize, Debug)]
struct RaceTable {
    Races: Vec<Race>,
}

#[derive(Deserialize, Debug)]
struct Location {
    lat: String,
    long: String,
    locality: String,
    country: String,
}

#[derive(Deserialize, Debug)]
struct Circuit {
    circuitId: String,
    url: String,
    circuitName: String,
    Location: Location,
}

#[derive(Deserialize, Debug)]
struct Race {
    season: String,
    round: String,
    url: String,
    raceName: String,
    Circuit: Circuit,
    date: String,
}

async fn fetch_races(url: &str) -> Result<Vec<Race>, Box<dyn Error>> {
    let response: Response = fetch(url).await?;

    Ok(response.MRData.RaceTable.Races)
}

fn filter_next_race(races: Vec<Race>) -> Race {
    let today = Local::now().date_naive();

    let next_race = races
        .into_iter()
        .filter_map(|race| {
            let race_date = NaiveDate::parse_from_str(&race.date, "%Y-%m-%d").unwrap();

            if race_date > today {
                Some(race)
            } else {
                None
            }
        })
        .min_by_key(|race| NaiveDate::parse_from_str(&race.date, "%Y-%m-%d").unwrap())
        .unwrap();

    next_race
}

fn generate_google_maps_url(lat: &str, long: &str) -> String {
    format!("https://www.google.com/maps/?q={},{}", lat, long)
}

fn get_country_code(country_name: &str) -> Option<String> {
    let mut map = HashMap::new();
    map.insert("Afghanistan".to_string(), "af".to_string());
    map.insert("Albania".to_string(), "al".to_string());
    map.insert("Algeria".to_string(), "dz".to_string());
    map.insert("Bahrain".to_string(), "bh".to_string());
    map.insert("Saudi Arabia".to_string(), "sa".to_string());
    map.insert("Australia".to_string(), "au".to_string());
    map.insert("Japan".to_string(), "jp".to_string());
    map.insert("China".to_string(), "cn".to_string());
    map.insert("USA".to_string(), "us".to_string());
    map.insert("Italy".to_string(), "it".to_string());
    map.insert("Monaco".to_string(), "mc".to_string());
    map.insert("Canada".to_string(), "ca".to_string());
    map.insert("Spain".to_string(), "es".to_string());
    map.insert("Austria".to_string(), "at".to_string());
    map.insert("UK".to_string(), "gb".to_string());
    map.insert("Hungary".to_string(), "hu".to_string());
    map.insert("Belgium".to_string(), "be".to_string());
    map.insert("Netherlands".to_string(), "nl".to_string());
    map.insert("Azerbaijan".to_string(), "az".to_string());
    map.insert("Singapore".to_string(), "sg".to_string());
    map.insert("Mexico".to_string(), "mx".to_string());
    map.insert("Brazil".to_string(), "br".to_string());
    map.insert("United States".to_string(), "us".to_string());
    map.insert("Qatar".to_string(), "qa".to_string());
    map.insert("UAE".to_string(), "ae".to_string());

    map.get(country_name).cloned()
}

fn create_message_next_race(race: &Race, remaining_days: i64, f1_role: u64) -> CreateMessage {
    let country_code = get_country_code(&race.Circuit.Location.country).unwrap();
    let flag_url = format!("https://flagcdn.com/h120/{}.png", country_code);

    let embed = CreateEmbed::new()
        .title("🏎🏁 Proxima carrera F1 🏎🏁")
        .description("¡Prepárate!, porque falta poco")
        .color(0xff0000)
        .field("Carrera", &race.raceName, true)
        .field("Circuito", &race.Circuit.circuitName, true)
        .field("Nombre del circuito", &race.Circuit.circuitName, false)
        //.field("Fecha", &race.date, true)
        .field(
            "Días restantes",
            format!("{} dia(s)", &remaining_days.to_string()),
            true,
        )
        .image(&flag_url)
        .thumbnail(&flag_url);

    let message = CreateMessage::new()
        .content(format!(
            "Oigan <@&{}> pendejos, ahi les aviso, que viene el FIUUUMMMMM!!!",
            f1_role
        ))
        .embed(embed);

    message
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN not found");
    let f1_api = env::var("F1_RACES_API").expect("Race api endpoint");
    let kato_id = env::var("KATO_ID").unwrap().parse::<u64>().unwrap();
    let lex_id = env::var("LEX_ID").unwrap().parse::<u64>().unwrap();

    let main_channel = env::var("MAIN_CHANNEL").unwrap().parse::<u64>().unwrap();
    let f1_role = env::var("F1_ROLE").unwrap().parse::<u64>().unwrap();

    let channel_id = ChannelId::from(main_channel);

    let discord_http_client = Http::new(&token);

    /* let intents =
    GatewayIntents::MESSAGE_CONTENT | GatewayIntents::GUILD_MESSAGES | GatewayIntents::GUILDS; */

    // DEBUG
    /* let now = Local::now();
    let races = fetch_races(&f1_api.as_str()).await.unwrap();
    let next_race = filter_next_race(races);
    let next_race_date = NaiveDate::parse_from_str(&next_race.date, "%Y-%m-%d").unwrap();
    let days_until_next_race = (next_race_date - now.date_naive()).num_days();
    let _ = channel_id
        .send_message(
            &discord_http_client,
            create_message_next_race(&next_race, days_until_next_race, 1),
        )
        .await; */

    let loop_task = tokio::spawn(async move {
        loop {
            let now = Local::now();

            println!("Start SKG BOT current time: {}", now.format("%H:%M:%S"));

            let until_two_pm = if now.hour() < 14 {
                ChronoDuration::hours(14 - now.hour() as i64)
            } else {
                ChronoDuration::hours(24 - now.hour() as i64 + 14)
            };

            let sleep_duration = std::time::Duration::from_secs(until_two_pm.num_seconds() as u64);
            sleep(sleep_duration).await;

            let races = fetch_races(&f1_api.as_str()).await.unwrap();
            let next_race = filter_next_race(races);
            let next_race_date = NaiveDate::parse_from_str(&next_race.date, "%Y-%m-%d").unwrap();
            let days_until_next_race = (next_race_date - now.date_naive()).num_days();

            println!("faltan {} para la proxima carrera", &days_until_next_race);

            if [7, 5, 3, 1].contains(&days_until_next_race) {
                // Send a message
                let _ = channel_id
                    .send_message(
                        &discord_http_client,
                        create_message_next_race(&next_race, days_until_next_race, f1_role),
                    )
                    .await;
            }
        }
    });

    // Wait for the loop task to finish (it never does)
    println!("The loop will start now!");
    loop_task.await.unwrap();
}
