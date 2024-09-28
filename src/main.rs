use std::fs::File;
use std::io::Write;
use reqwest::blocking::*;
use scraper::{Html, Selector};
use reqwest::header::USER_AGENT;
use isocountry::CountryCode;
use serde::Serialize;

const CUSTOM_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36";

#[derive(Serialize, Default)]
struct ChannelInfo {
    title: String,
    about: String,
    allowed_counties: Vec<String>,
    url: String
}

fn get_input(query: &str) -> std::io::Result<String>{
    print!("{}", query);
    std::io::stdout().flush()?;

    let mut answer = String::new();
    std::io::stdin().read_line(&mut answer)?;

    Ok(answer.trim().to_owned())
}

fn is_country_allowed(country: &String, con_vec: &Vec<String>) -> bool{
    if con_vec.contains(country) {true} else {false}
}

fn next_action() -> i32{
    let mut action = 1;

    if let Ok(next) = get_input("If you want to check if country is in allowed countries list press 0,\nif you want to check other channels info press 1,\nif you want to exit press 2: "){
        action = next.parse::<i32>().unwrap();
    };
    action
}


fn main() -> std::io::Result<()>{

    loop {
        let mut id = String::new();

        if let Ok( idd) = get_input("Enter chanel id ( or press F to exit ): "){
            id = idd;
        }

        if id == "F"{
            break;
        }
        println!();
        let url = format!("https://www.youtube.com/{}", id);

        let title = Selector::parse("meta[property=\"og:title\"]").unwrap();
        let about = Selector::parse("meta[property='og:description']").unwrap();

        let smart_response = Client::new().get(&url).header(USER_AGENT, CUSTOM_USER_AGENT).send().unwrap().text().unwrap();

        let document = Html::parse_document(&smart_response);

        let mut channel_info = ChannelInfo::default();


        if let Some(element) = document.select(&title).next() {
            if let Some(content) = element.value().attr("content") {
                println!("Title: {}\n", content);
                channel_info.title = content.to_string();
            }
        }

        if let Some(element) = document.select(&about).next() {
            if let Some(content) = element.value().attr("content") {
                println!("About : {}\n", content);
                channel_info.about = content.to_string();
            }
        }

        let selector = Selector::parse("meta[itemprop=\"regionsAllowed\"]").unwrap();

        if let Some(element) = document.select(&selector).next() {
            if let Some(content) = element.value().attr("content") {
                let vec: Vec<&str> = content.split(',').collect();
                let mut allowed_countries = Vec::new();

                for el in vec {
                    match CountryCode::for_alpha2(el.trim()) {
                        Ok(country) => {
                            let full_name = country.name();
                            let common_name = full_name.split('(').next().unwrap().trim().to_string();
                            allowed_countries.push(common_name)
                        }
                        Err(_) => {}
                    }
                }
                println!("Allowed countries: {:?}\n", allowed_countries);
                channel_info.allowed_counties = allowed_countries;
            }
        }

        let link = Selector::parse("link[itemprop=\"url\"]").unwrap();

        if let Some(element) = document.select(&link).next() {
            if let Some(href) = element.value().attr("href") {
                println!("Channel URL: {}\n", href);
                channel_info.url = href.to_string();
            }
        }

        let serialized_data = serde_json::to_string_pretty(&channel_info)?;

        let mut file = File::create("channel_info.json")?;
        file.write_all(serialized_data.as_bytes())?;

        let next_move = next_action();

        println!();


        if next_move == 0{
            loop {
                if let Ok(country) = get_input("Enter country to check or F to exit: ") {
                    println!();
                    if country == "F"{
                        break;
                    }
                    let result = is_country_allowed(&country, &channel_info.allowed_counties);
                    if result == true{
                        println!("{} is allowed\n", country);
                    } else{
                        println!("{} is not allowed\n", country);
                    }
                }
            }
        } else if next_move == 1{
            continue;
        } else if next_move == 2{
            break;
        }

    }
    Ok(())
}