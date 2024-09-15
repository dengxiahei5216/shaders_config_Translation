use std:: fs::File;
use rfd::FileDialog;
use std::io::{BufReader, BufRead, Write};
use regex::Regex;
use serde::{Serialize,Deserialize};
use reqwest::blocking::{Client,Response};
use serde_json::Value;
static URL:&str = "https://aip.baidubce.com/rpc/2.0/mt/texttrans/v1?access_token=";
static AUTH_URL:&str = "https://aip.baidubce.com/oauth/2.0/token?grant_type=client_credentials";
static APP_KEY: &str = "uOsxTTUQIknuNx03HzpNkjOe";
static SECRET_KEY:&str = "keU8Y4PVO47FiWqIWAee10m58ryJYH5Z";

#[derive(Debug)]
struct Dict {
    option: String,
    en_us: String,
    zh_cn: String,
}
#[derive(Serialize,Deserialize)]
#[derive(Debug)]
struct QueryData {
    q:String,
    from:String,
    to:String,
    //termlds:String
}


fn open_file_reload_2() -> Vec<Dict>  {
    let mut dict_vec:Vec<Dict> = Vec::new();
    let file_path = match FileDialog::new()
        .add_filter("text", &["lang"])
        .set_directory("/")
        .pick_file() {
        Some(file) => file.display().to_string(),
        None => String::from("")
    };
    let options: Regex = match Regex::new(r".*\b=") {
        Ok(s) => s,
        Err(err) => {
            println!("Error creating regex: {}", err);
            return Vec::new();
        }
    };
    let en_us = match Regex::new(r"\b=.*") {
        Ok(s) => s,
        Err(err) => {
            println!("Error creating regex: {}", err);
            return Vec::new();
        }
    
    };
    

    let _ = match File::open(file_path) {
        Ok(file) => {
            let read_data = BufReader::new(file);
            for read_line in read_data.lines() {
                match read_line {
                    Ok(line) => {
                         if options.is_match(&line) && en_us.is_match(&line) {
                            dict_vec.push(
                                Dict {
                                    option: options.find_iter(&line)
                                        .map(|cap| &cap.as_str()[0..cap.as_str().to_string().len()-1])
                                        .filter(|s| !s.is_empty())
                                        .collect::<String>(),
                                    en_us: en_us.find_iter(&line)
                                        .map(|cap| &cap.as_str()[1..cap.as_str().to_string().len()])
                                        .filter(|s| !s.is_empty())
                                        .collect::<String>(),
                                    zh_cn: String::from("")
                                }
                            )
                        }
                        
                    },
                    Err(err) => {
                        
                        panic!("读取行出错：{}", err);
                    }
                }
            }

        },
        Err(err) => {
            panic!("打开文件出错：{}", err);
            
        }
    };
    //println!("{:#?}",dict_vec);
    return dict_vec;

}





/* fn print_type_of<T>(_:&T) {
    println!("{}",std::any::type_name::<T>());
} */


fn send_post_request(client: &Client, url: &str, body: String) -> Result<Response, reqwest::Error> {
    client.post(url)
        .header("Content-Type", "application/json;charset=utf-8")
        .body(body)
        .send()
}

fn get_access_token(client: &Client) -> Result<String, reqwest::Error> {
    let auth_url = format!("{}&client_id={}&client_secret={}", AUTH_URL, APP_KEY, SECRET_KEY);
    let response = send_post_request(client, &auth_url, "".to_string())?;
    let json: Value = match  serde_json::from_str(&response.text()?) {
        Ok(json) => {json},
        Err(e) => {panic!("{}",e)}
    };
    Ok(json["access_token"].to_string())
}

fn translate(client: &Client, url: &str, body: String) -> Result<String, reqwest::Error> {
    
    let response = send_post_request(&client, &url, body)?;
    let json: Value = match serde_json::from_str(&response.text()?) {
        Ok(json) => {json},
        Err(e) => {panic!("{}",e)}
    };
    //println!("String: {}\nJson:\ndst: {}\tsrc: {}",json, json["result"]["trans_result"][0]["dst"], json["result"]["trans_result"][0]["src"]);
    Ok(json["result"]["trans_result"][0]["dst"].to_string())
}



fn main(){
    
    let mut dict_vec = open_file_reload_2();
    let client = Client::new();
    let access_token = match get_access_token(&client) {
        Ok(token) => {token},
        Err(e) => {panic!("{}",e)}
    };
    //println!("access_token: {}", access_token);
    let query_url = format!("{}{}", URL, access_token);
/*     println!("query_url: {}", query_url);
    let query_dat = QueryData {
        q: dict_vec[0].en_us.to_string(),
        from: "en".to_string(),
        to: "zh".to_string(),
    };
    let request_body = serde_json::json!(query_dat).to_string();
    println!("request_body: {}", request_body);
    let s =match translate(&client,&query_url,request_body) {
        Ok(text) => {text},
        Err(e) => {panic!("{}",e)}
    };
    println!("s: {}", s); */
    
    //let query_url = format!("{}{}", URL, access_token);
/*     for dict in dict_vec.iter_mut() {
        let query_data = QueryData {
            q: dict.en_us.clone(),
            from: "en".to_string(),
            to: "zh".to_string(),
        };
        let request_body = serde_json::json!(query_data).to_string();
        dict.zh_cn = match translate(&client,&query_url,request_body) {
            Ok(text) => {text},
            Err(e) => {panic!("{}",e)}
        };
    } */
    let dict_vec_translate = dict_vec.iter_mut().map(
        |dict| {
            Dict {
                option: dict.option.clone(),
                en_us: dict.en_us.clone(),
                zh_cn: match translate(&client,&query_url,serde_json::json!(QueryData {
                    q: dict.en_us.clone(),
                    from: "en".to_string(),
                    to: "zh".to_string(),
                }).to_string()) {
                    Ok(text) => {text},
                    Err(e) => {panic!("{}",e)}
                }
            }
            
            
        }
    ).collect::<Vec<Dict>>();
    //println!("{:#?}",dict_vec_translate);
    //println!("{:#?}",dict_vec);
    let mut file = match File::create("zh_CN.lang") {
        Ok(file) => file,
        Err(err) => panic!("{}", err),
    };
    for i in 0..dict_vec_translate.len() {
        let _ = write!(file, "{}={}\n", dict_vec_translate[i].option, dict_vec_translate[i].zh_cn);
    }
        
    
    
   
}
