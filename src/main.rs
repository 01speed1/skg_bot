use dotenv::dotenv;
use serenity::all::Embed;
use std::{collections::HashMap, env, error::Error};

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

use serenity::{
    builder::{CreateEmbed, CreateMessage},
    model::{id::ChannelId, Timestamp},
    prelude::*,
};

use reqwest;

use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::Value;

use chrono::{DateTime, NaiveDate, Utc};

struct Handler;

const KATO_ID: u64 = 344959298205384716;
const LEX_ID: u64 = 901606283814256690;
pub const CHANNEL_TEST: u64 = 1206401965450338345;

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

        if msg.author.id == LEX_ID {
            if let Err(why) = msg
                .channel_id
                .say(&ctx.http, "Man Lexan, cálmese, es solo un ARAM! 🙏")
                .await
            {
                println!("Error enviando mensaje: {:?}", why);
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

type JSONResponse = HashMap<String, Value>;

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

fn filter_next_race(races: Vec<Race>) -> Result<Option<Race>, Box<dyn Error>> {
    let today = Utc::today().naive_utc();

    let next_race = races
        .into_iter()
        .filter_map(|race| {
            let race_date = NaiveDate::parse_from_str(&race.date, "%Y-%m-%d").ok()?;
            (race_date > today).then(|| race)
        })
        .min_by_key(|race| NaiveDate::parse_from_str(&race.date, "%Y-%m-%d").unwrap());

    Ok(next_race)
}

fn generate_google_maps_url(lat: &str, long: &str) -> String {
    format!("https://www.google.com/maps/?q={},{}", lat, long)
}

fn generate_announcement(race: &Race) -> String {
    format!(
        "
    **Temporada:** {}\n
    **Ronda:** {}\n
    **Nombre de la carrera:** {}\n
    **Circuito:** {}\n
    **Localidad:** {}\n
    **País:** {}\n
    **Fecha:** {}\n
    **URL de la carrera:** [Link]({})\n
    **URL del circuito:** [Link]({})",
        race.season,
        race.round,
        race.raceName,
        race.Circuit.circuitName,
        race.Circuit.Location.locality,
        race.Circuit.Location.country,
        race.date,
        race.url,
        race.Circuit.url
    )
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

fn create_message_next_race(race: &Race) -> CreateMessage {
    let country_code = get_country_code(&race.Circuit.Location.country).unwrap();
    let flag_url = format!("https://flagcdn.com/h120/{}.png", country_code);
    let date = match race.date.parse::<DateTime<Utc>>() {
        Ok(date) => date,
        Err(_) => Utc::now(),
    };

    let embed = CreateEmbed::new()
        .title("🏎🏁 Proxima carrera F1 🏎🏁")
        .description("¡Preparate!, porque falta poco")
        .color(0xff0000)
        .field("Carrera", &race.raceName, true)
        .field("Circuito", &race.Circuit.circuitName, true)
        .field("Nombre del circuito", &race.Circuit.circuitName, false)
        .field("Fecha", &race.date, true)
        .image(&flag_url)
        .thumbnail(&flag_url);

    let message = CreateMessage::new().embed(embed);

    message
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN not found");
    let f1_api = env::var("F1_RACES_API").expect("F1_API not found");

    let intents =
        GatewayIntents::MESSAGE_CONTENT | GatewayIntents::GUILD_MESSAGES | GatewayIntents::GUILDS;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    let races = fetch_races(f1_api.as_str()).await.unwrap();

    let next_race = filter_next_race(races).unwrap().unwrap();
    println!("{next_race:?}");

    /* let _ = ChannelId::from(CHANNEL_TEST)
        .send_message(&client.http, create_message_next_race(&next_race))
        .await; */

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {why:?}");
    }
}
