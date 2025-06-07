/*!
unofficial [keyauth](https://keyauth.cc) library that uses 1.2 api version

basic usage:
```rust
let mut auth = keyauth::v1_2::KeyauthApi::new("application name", "ownerid", "application secret", "application version", "api url"); // if you dont have a custom domain for api use "https://keyauth.win/api/1.2/"
auth.init().unwrap();
auth.login("username", "password", Some("hwid".to_string()).unwrap()); // if you want to automaticly generate hwid use None insted.
```

also if you want to use an obfuscator for rust i recommend using [obfstr](https://crates.io/crates/obfstr) and [llvm obfuscator](https://github.com/eshard/obfuscator-llvm/wiki/Rust-obfuscation-guide)
*/

use serde::Deserialize;
use uuid::Uuid;
use std::collections::HashMap;
use reqwest::blocking::Client;
use reqwest::blocking::Response;
use hmac_sha256::HMAC;
use base16::decode;
use machineid_rs::{IdBuilder, Encryption};
use machineid_rs::HWIDComponent;


/// every function in this struct (accept log) returns a Result and Err("Request was tampered with") will be returned if the request signature doesnt mathc the sha256 hmac of the message
#[derive(Debug, Clone)]
pub struct KeyauthApi {
    name: String,
    owner_id: String,
    secret: String,
    version: String,
    enckey: String,
    enckey_s: String,
    session_id: String,
    pub api_url: String,
    pub num_keys: String,
    pub num_online_users: String,
    pub num_users: String,
    pub app_version: String,
    pub customer_panel_link: String,
    pub username: String,
    pub ip: String,
    pub hwid: Option<String>,
    pub create_date: String,
    pub last_login: String,
    pub subscription: String,
    pub sub_time_left: i64,
    pub expiry: String,
    pub message: String,
    pub success: bool,
    pub blacklisted: bool,
    pub response: String,
}

fn get_hwid() -> String {
    let mut builder = IdBuilder::new(Encryption::SHA256);
    builder
        .add_component(HWIDComponent::SystemID)
        .add_component(HWIDComponent::CPUCores);

    builder.build("mykey").unwrap()
}

impl KeyauthApi {
    /// creats a new KeyauthApi and its defaults, api_url has to be api version 1.2 example: "https://keyauth.win/api/1.2/" or if you have a custom api domain: "https://api.example.com/1.2/"
    pub fn new(name: &str, owner_id: &str, secret: &str, version: &str, api_url: &str) -> Self {
        Self {
            name: name.to_string(),
            owner_id: owner_id.to_string(),
            secret: secret.to_string(),
            version: version.to_string(),
            enckey: String::new(),
            enckey_s: String::new(),
            session_id: String::new(),
            num_keys: String::new(),
            api_url: api_url.to_string(),
            num_online_users: String::new(),
            num_users: String::new(),
            app_version: version.to_string(),
            customer_panel_link: String::new(),
            username: String::new(),
            ip: String::new(),
            hwid: None,
            create_date: String::new(),
            last_login: String::new(),
            subscription: String::new(),
            sub_time_left: 0,
            expiry: String::new(),
            message: String::new(),
            success: false,
            blacklisted: false,
            response: String::new(),
        }
    }

    /// initializes a session, **required to run before any other function in this struct!!!** accept new
    pub fn init(&mut self, hash: Option<&str>) -> Result<(), String> {
        self.enckey = Uuid::new_v4().simple().to_string();
        self.enckey_s = format!("{}-{}", self.enckey, self.secret);
        let mut req_data = HashMap::new();
        req_data.insert("type", "init");
        if hash.is_some() {
            req_data.insert("hash", hash.unwrap());
        }
        req_data.insert("ver", &self.version);
        req_data.insert("name", &self.name);
        req_data.insert("ownerid", &self.owner_id);
        req_data.insert("enckey", &self.enckey);

        let req = Self::request(req_data, &self.api_url);
        let head = req.headers().clone();
        let resp = req.text().unwrap();

        if resp == "KeyAuth_Invalid" {
            return Err("The application doesn't exist".to_string());
        }
        if !head.contains_key("signature") {
            #[cfg(feature = "panic")]
            {
                panic!("response was tampered with");
            }
            #[cfg(not(feature = "panic"))]
            {
                return Err("response was tampered with".to_string());
            }
            #[cfg(feature = "panic")]
            {
                panic!("response was tampered with");
            }
            #[cfg(not(feature = "panic"))]
            {
                return Err("response was tampered with".to_string());
            }
        }
        let sig = head.get("signature").unwrap().to_str().unwrap();
        if sig != Self::make_hmac(&resp, &self.secret) {
            #[cfg(feature = "panic")]
            {
                panic!("response was tampered with");
            }
            #[cfg(not(feature = "panic"))]
            {
                return Err("response was tampered with".to_string());
            }
        }
        let json_rep: serde_json::Value = serde_json::from_str(&resp).unwrap();
        if json_rep["success"].as_bool().unwrap() {
            self.session_id = json_rep["sessionid"].as_str().unwrap().to_string();
            self.num_keys = json_rep["appinfo"]["numKeys"].as_str().unwrap().to_string();
            self.num_online_users = json_rep["appinfo"]["numOnlineUsers"].as_str().unwrap().to_string();
            self.num_users = json_rep["appinfo"]["numUsers"].as_str().unwrap().to_string();
            self.customer_panel_link = json_rep["appinfo"]["customerPanelLink"].as_str().unwrap_or("").to_string();
            Ok(())
        } else {
            if json_rep["message"].as_str().unwrap() == "invalidver" {
                let download_url = json_rep["download"].as_str().unwrap();
                if !download_url.is_empty() {
                    webbrowser::open(download_url).unwrap();
                }
            }
            Err(json_rep["message"].as_str().unwrap().to_string())
        }
    }

    /// registeres a new user
    pub fn register(&mut self, username: String, password: String, license: String, hwid: Option<String>) -> Result<(), String> {
        let hwidd = match hwid {
            Some(hwid) => hwid,
            None => get_hwid(),
        };
        let mut req_data = HashMap::new();
        req_data.insert("type", "register");
        req_data.insert("username", &username);
        req_data.insert("pass", &password);
        req_data.insert("key", &license);
        req_data.insert("sessionid", &self.session_id);
        req_data.insert("name", &self.name);
        req_data.insert("ownerid", &self.owner_id);
        req_data.insert("hwid", &hwidd);

        let req = Self::request(req_data, &self.api_url);
        let head = req.headers().clone();
        let resp = req.text().unwrap();

        if !head.contains_key("signature") {
            #[cfg(feature = "panic")]
            {
                panic!("response was tampered with");
            }
            #[cfg(not(feature = "panic"))]
            {
                return Err("response was tampered with".to_string());
            }
        }
        let sig = head.get("signature").unwrap().to_str().unwrap();
        if sig != Self::make_hmac(&resp, &self.enckey_s) {
            #[cfg(feature = "panic")]
            {
                panic!("Response was tampered with");
            }
            #[cfg(not(feature = "panic"))]
            {
                return Err("Response was tampered with".to_string());
            }
        }
        let json_rep: serde_json::Value = serde_json::from_str(&resp).unwrap();
        if json_rep["success"].as_bool().unwrap() {
            self.username = username;
            self.ip = json_rep["info"]["ip"].as_str().unwrap().to_string();
            self.create_date = json_rep["info"]["createdate"].as_str().unwrap().to_string();
            self.last_login = json_rep["info"]["lastlogin"].as_str().unwrap().to_string();
            self.subscription = json_rep["info"]["subscriptions"][0]["subscription"].as_str().unwrap().to_string();
            self.sub_time_left = json_rep["info"]["subscriptions"][0]["timeleft"].as_i64().unwrap();
            self.expiry = json_rep["info"]["subscriptions"][0]["expiry"].as_str().unwrap().to_string();
            Ok(())
        } else {
            Err(json_rep["message"].as_str().unwrap().to_string())
        }
    }

    /// upgrades a user license level or extends a license
    pub fn upgrade(&mut self, username: String, license: String) -> Result<(), String> {
        let mut req_data = HashMap::new();
        req_data.insert("type", "upgrade");
        req_data.insert("username", &username);
        req_data.insert("key", &license);
        req_data.insert("sessionid", &self.session_id);
        req_data.insert("name", &self.name);
        req_data.insert("ownerid", &self.owner_id);

        let req = Self::request(req_data, &self.api_url);
        let head = req.headers().clone();
        let resp = req.text().unwrap();

        if !head.contains_key("signature") {
            #[cfg(feature = "panic")]
            {
                panic!("response was tampered with");
            }
            #[cfg(not(feature = "panic"))]
            {
                return Err("response was tampered with".to_string());
            }
        }
        let sig = head.get("signature").unwrap().to_str().unwrap();
        if sig != Self::make_hmac(&resp, &self.enckey_s) {
            #[cfg(feature = "panic")]
            {
                panic!("Response was tampered with");
            }
            #[cfg(not(feature = "panic"))]
            {
                return Err("Response was tampered with".to_string());
            }
        }
        let json_rep: serde_json::Value = serde_json::from_str(&resp).unwrap();
        if json_rep["success"].as_bool().unwrap() {
            Ok(())
        } else {
            Err(json_rep["message"].as_str().unwrap().to_string())
        }
    }

    /// login self explanatory
    pub fn login(&mut self, username: String, password: String, hwid: Option<String>) -> Result<(), String> {
        let hwidd = match hwid {
            Some(hwid) => hwid,
            None => get_hwid(),
        };

        let mut req_data = HashMap::new();
        req_data.insert("type", "login");
        req_data.insert("username", &username);
        req_data.insert("pass", &password);
        req_data.insert("hwid", &hwidd);
        req_data.insert("sessionid", &self.session_id);
        req_data.insert("name", &self.name);
        req_data.insert("ownerid", &self.owner_id);

        let req = Self::request(req_data, &self.api_url);
        let head = req.headers().clone();
        let resp = req.text().unwrap();

        if !head.contains_key("signature") {
            #[cfg(feature = "panic")]
            {
                panic!("response was tampered with");
            }
            #[cfg(not(feature = "panic"))]
            {
                return Err("response was tampered with".to_string());
            }
        }
        let sig = head.get("signature").unwrap().to_str().unwrap();
        if sig != Self::make_hmac(&resp, &self.enckey_s) {
            #[cfg(feature = "panic")]
            {
                panic!("Response was tampered with");
            }
            #[cfg(not(feature = "panic"))]
            {
                return Err("Response was tampered with".to_string());
            }
        }
        let json_rep: serde_json::Value = serde_json::from_str(&resp).unwrap();

        if json_rep["success"].as_bool().unwrap() {
            self.username = username;
            self.ip = json_rep["info"]["ip"].as_str().unwrap().to_string();
            self.hwid = Some(hwidd);
            self.create_date = json_rep["info"]["createdate"].as_str().unwrap().to_string();
            self.last_login = json_rep["info"]["lastlogin"].as_str().unwrap().to_string();
            self.subscription = json_rep["info"]["subscriptions"][0]["subscription"].as_str().unwrap().to_string();
            self.sub_time_left = json_rep["info"]["subscriptions"][0]["timeleft"].as_i64().unwrap();
            self.expiry = json_rep["info"]["subscriptions"][0]["expiry"].as_str().unwrap().to_string();
            Ok(())
        } else {
            Err(json_rep["message"].as_str().unwrap().to_string())
        }
    }

    /// <https://docs.keyauth.cc/api/license>
    pub fn license(&mut self, license: String, hwid: Option<String>) -> Result<(), String> {
        let hwidd = match hwid {
            Some(hwid) => hwid,
            None => get_hwid(),
        };

        let mut req_data = HashMap::new();
        req_data.insert("type", "license");
        req_data.insert("key", &license);
        req_data.insert("hwid", &hwidd);
        req_data.insert("sessionid", &self.session_id);
        req_data.insert("name", &self.name);
        req_data.insert("ownerid", &self.owner_id);

        let req = Self::request(req_data, &self.api_url);
        let head = req.headers().clone();
        let resp = req.text().unwrap();

        if !head.contains_key("signature") {
            #[cfg(feature = "panic")]
            {
                panic!("response was tampered with");
            }
            #[cfg(not(feature = "panic"))]
            {
                return Err("response was tampered with".to_string());
            }
        }
        let sig = head.get("signature").unwrap().to_str().unwrap();
        if sig != Self::make_hmac(&resp, &self.enckey_s) {
            #[cfg(feature = "panic")]
            {
                panic!("Response was tampered with");
            }
            #[cfg(not(feature = "panic"))]
            {
                return Err("Response was tampered with".to_string());
            }
        }
        let json_rep: serde_json::Value = serde_json::from_str(&resp).unwrap();

        if json_rep["success"].as_bool().unwrap() {
            self.username = json_rep["info"]["username"].as_str().unwrap().to_string();
            self.ip = json_rep["info"]["ip"].as_str().unwrap().to_string();
            self.hwid = Some(hwidd);
            self.create_date = json_rep["info"]["createdate"].as_str().unwrap().to_string();
            self.last_login = json_rep["info"]["lastlogin"].as_str().unwrap().to_string();
            self.subscription = json_rep["info"]["subscriptions"][0]["subscription"].as_str().unwrap().to_string();
            self.sub_time_left = json_rep["info"]["subscriptions"][0]["timeleft"].as_i64().unwrap();
            self.expiry = json_rep["info"]["subscriptions"][0]["expiry"].as_str().unwrap().to_string();
            Ok(())
        } else {
            Err(json_rep["message"].as_str().unwrap().to_string())
        }
    }

    /// this will get a global variable (not user) and return it
    pub fn var(&mut self, varid: String) -> Result<String, String> {
        let mut req_data = HashMap::new();
        req_data.insert("type", "var");
        req_data.insert("varid", &varid);
        req_data.insert("sessionid", &self.session_id);
        req_data.insert("name", &self.name);
        req_data.insert("ownerid", &self.owner_id);

        let req = Self::request(req_data, &self.api_url);
        let head = req.headers().clone();
        let resp = req.text().unwrap();

        if !head.contains_key("signature") {
            #[cfg(feature = "panic")]
            {
                panic!("response was tampered with");
            }
            #[cfg(not(feature = "panic"))]
            {
                return Err("response was tampered with".to_string());
            }
        }
        let sig = head.get("signature").unwrap().to_str().unwrap();
        if sig != Self::make_hmac(&resp, &self.enckey_s) {
            #[cfg(feature = "panic")]
            {
                panic!("Response was tampered with");
            }
            #[cfg(not(feature = "panic"))]
            {
                return Err("Response was tampered with".to_string());
            }
        }
        let json_rep: serde_json::Value = serde_json::from_str(&resp).unwrap();

        if json_rep["success"].as_bool().unwrap() {
            Ok(json_rep["message"].as_str().unwrap().to_string())
        } else {
            Err(json_rep["message"].as_str().unwrap().to_string())
        }
    }

    /// downloads a file, and decodes using base16::decode
    pub fn file(&mut self, fileid: String) -> Result<Vec<u8>, String> {
        let mut req_data = HashMap::new();
        req_data.insert("type", "file");
        req_data.insert("fileid", &fileid);
        req_data.insert("sessionid", &self.session_id);
        req_data.insert("name", &self.name);
        req_data.insert("ownerid", &self.owner_id);

        let req = Self::request(req_data, &self.api_url);
        let head = req.headers().clone();
        let resp = req.text().unwrap();

        if !head.contains_key("signature") {
            #[cfg(feature = "panic")]
            {
                panic!("response was tampered with");
            }
            #[cfg(not(feature = "panic"))]
            {
                return Err("response was tampered with".to_string());
            }
        }
        let sig = head.get("signature").unwrap().to_str().unwrap();
        if sig != Self::make_hmac(&resp, &self.enckey_s) {
            #[cfg(feature = "panic")]
            {
                panic!("Response was tampered with");
            }
            #[cfg(not(feature = "panic"))]
            {
                return Err("Response was tampered with".to_string());
            }
        }
        let json_rep: serde_json::Value = serde_json::from_str(&resp).unwrap();

        if json_rep["success"].as_bool().unwrap() {
            Ok(decode(json_rep["contents"].as_str().unwrap()).unwrap())
        } else {
            Err(json_rep["message"].as_str().unwrap().to_string())
        }
    }

    /// sends a webhook from keyauth's servers so the url isnt exposed
    pub fn webhook(&mut self, webid: String, params: String) -> Result<String, String> {
        let mut req_data = HashMap::new();
        req_data.insert("type", "webhook");
        req_data.insert("webid", &webid);
        req_data.insert("params", &params);
        req_data.insert("sessionid", &self.session_id);
        req_data.insert("name", &self.name);
        req_data.insert("ownerid", &self.owner_id);

        let req = Self::request(req_data, &self.api_url);
        let head = req.headers().clone();
        let resp = req.text().unwrap();

        if !head.contains_key("signature") {
            #[cfg(feature = "panic")]
            {
                panic!("response was tampered with");
            }
            #[cfg(not(feature = "panic"))]
            {
                return Err("response was tampered with".to_string());
            }
        }
        let sig = head.get("signature").unwrap().to_str().unwrap();
        if sig != Self::make_hmac(&resp, &self.enckey_s) {
            #[cfg(feature = "panic")]
            {
                panic!("Response was tampered with");
            }
            #[cfg(not(feature = "panic"))]
            {
                return Err("Response was tampered with".to_string());
            }
        }
        let json_rep: serde_json::Value = serde_json::from_str(&resp).unwrap();

        if json_rep["success"].as_bool().unwrap() {
            Ok(json_rep["message"].as_str().unwrap().to_string())
        } else {
            Err(json_rep["message"].as_str().unwrap().to_string())
        }
    }

    /// checks if the user is blacklisted and sets self.blacklisted acordingly
    pub fn checkblacklist(&mut self) -> Result<(), String> {
        let mut req_data = HashMap::new();
        req_data.insert("type", "checkblacklist");
        req_data.insert("sessionid", &self.session_id);
        req_data.insert("name", &self.name);
        req_data.insert("ownerid", &self.owner_id);

        let req = Self::request(req_data, &self.api_url);
        let head = req.headers().clone();
        let resp = req.text().unwrap();

        if !head.contains_key("signature") {
            #[cfg(feature = "panic")]
            {
                panic!("response was tampered with");
            }
            #[cfg(not(feature = "panic"))]
            {
                return Err("response was tampered with".to_string());
            }
        }
        let sig = head.get("signature").unwrap().to_str().unwrap();
        if sig != Self::make_hmac(&resp, &self.enckey_s) {
            #[cfg(feature = "panic")]
            {
                panic!("Response was tampered with");
            }
            #[cfg(not(feature = "panic"))]
            {
                return Err("Response was tampered with".to_string());
            }
        }
        let json_rep: serde_json::Value = serde_json::from_str(&resp).unwrap();

        if json_rep["success"].as_bool().unwrap() {
            self.blacklisted = true;
            Ok(())
        } else {
            self.blacklisted = false;
            Ok(())
        }
    }

    /// checks if the session is still active or if it expired
    pub fn check_session(&mut self) -> Result<bool, String> {
        let mut req_data = HashMap::new();
        req_data.insert("type", "check");
        req_data.insert("sessionid", &self.session_id);
        req_data.insert("name", &self.name);
        req_data.insert("ownerid", &self.owner_id);

        let req = Self::request(req_data, &self.api_url);
        let head = req.headers().clone();
        let resp = req.text().unwrap();

        if !head.contains_key("signature") {
            #[cfg(feature = "panic")]
            {
                panic!("response was tampered with");
            }
            #[cfg(not(feature = "panic"))]
            {
                return Err("response was tampered with".to_string());
            }
        }
        let sig = head.get("signature").unwrap().to_str().unwrap();
        if sig != Self::make_hmac(&resp, &self.enckey_s) {
            #[cfg(feature = "panic")]
            {
                panic!("Response was tampered with");
            }
            #[cfg(not(feature = "panic"))]
            {
                return Err("Response was tampered with".to_string());
            }
        }
        let json_rep: serde_json::Value = serde_json::from_str(&resp).unwrap();

        Ok(json_rep["success"].as_bool().unwrap())
    }

    /// gets json of online users
    pub fn fetch_online(&mut self) -> Result<serde_json::Value, String> {
        let mut req_data = HashMap::new();
        req_data.insert("type", "fetchOnline");
        req_data.insert("sessionid", &self.session_id);
        req_data.insert("name", &self.name);
        req_data.insert("ownerid", &self.owner_id);

        let req = Self::request(req_data, &self.api_url);
        let head = req.headers().clone();
        let resp = req.text().unwrap();

        if !head.contains_key("signature") {
            #[cfg(feature = "panic")]
            {
                panic!("response was tampered with");
            }
            #[cfg(not(feature = "panic"))]
            {
                return Err("response was tampered with".to_string());
            }
        }
        let sig = head.get("signature").unwrap().to_str().unwrap();
        if sig != Self::make_hmac(&resp, &self.enckey_s) {
            #[cfg(feature = "panic")]
            {
                panic!("Response was tampered with");
            }
            #[cfg(not(feature = "panic"))]
            {
                return Err("Response was tampered with".to_string());
            }
        }
        let json_rep: serde_json::Value = serde_json::from_str(&resp).unwrap();

        if json_rep["success"].as_bool().unwrap() {
            Ok(json_rep["users"].clone())
        } else {
            Err(json_rep["message"].as_str().unwrap().to_string())
        }
    }

    /// gets the arry of messages in a channel
    pub fn get_chat(&mut self, channel: String) -> Result<serde_json::Value, String> {
        let mut req_data = HashMap::new();
        req_data.insert("type", "chatget");
        req_data.insert("channel", &channel);
        req_data.insert("sessionid", &self.session_id);
        req_data.insert("name", &self.name);
        req_data.insert("ownerid", &self.owner_id);

        let req = Self::request(req_data, &self.api_url);
        let head = req.headers().clone();
        let resp = req.text().unwrap();

        if !head.contains_key("signature") {
            #[cfg(feature = "panic")]
            {
                panic!("response was tampered with");
            }
            #[cfg(not(feature = "panic"))]
            {
                return Err("response was tampered with".to_string());
            }
        }
        let sig = head.get("signature").unwrap().to_str().unwrap();
        if sig != Self::make_hmac(&resp, &self.enckey_s) {
            #[cfg(feature = "panic")]
            {
                panic!("Response was tampered with");
            }
            #[cfg(not(feature = "panic"))]
            {
                return Err("Response was tampered with".to_string());
            }
        }
        let json_rep: serde_json::Value = serde_json::from_str(&resp).unwrap();

        if json_rep["success"].as_bool().unwrap() {
            Ok(json_rep["messages"].clone())
        } else {
            Err(json_rep["message"].as_str().unwrap().to_string())
        }
    }

    /// sends a chat message in a channel
    pub fn send_chat_message(&mut self, channel: String, message: String) -> Result<(), String> {
        let mut req_data = HashMap::new();
        req_data.insert("type", "chatsend");
        req_data.insert("channel", &channel);
        req_data.insert("message", &message);
        req_data.insert("sessionid", &self.session_id);
        req_data.insert("name", &self.name);
        req_data.insert("ownerid", &self.owner_id);

        let req = Self::request(req_data, &self.api_url);
        let head = req.headers().clone();
        let resp = req.text().unwrap();

        if !head.contains_key("signature") {
            #[cfg(feature = "panic")]
            {
                panic!("response was tampered with");
            }
            #[cfg(not(feature = "panic"))]
            {
                return Err("response was tampered with".to_string());
            }
        }
        let sig = head.get("signature").unwrap().to_str().unwrap();
        if sig != Self::make_hmac(&resp, &self.enckey_s) {
            #[cfg(feature = "panic")]
            {
                panic!("Response was tampered with");
            }
            #[cfg(not(feature = "panic"))]
            {
                return Err("Response was tampered with".to_string());
            }
        }
        let json_rep: serde_json::Value = serde_json::from_str(&resp).unwrap();

        if json_rep["success"].as_bool().unwrap() {
            Ok(())
        } else {
            Err(json_rep["message"].as_str().unwrap().to_string())
        }
    }

    /// self explanatory
    pub fn ban(&mut self) {
        let mut req_data = HashMap::new();
        req_data.insert("type", "ban");
        req_data.insert("sessionid", &self.session_id);
        req_data.insert("name", &self.name);
        req_data.insert("ownerid", &self.owner_id);

        Self::request(req_data, &self.api_url);
    }

    /// sets a user variable to varvalue
    pub fn setvar(&mut self, varname: String, varvalue: String) -> Result<(), String> {
        let mut req_data = HashMap::new();
        req_data.insert("type", "setvar");
        req_data.insert("var", &varname);
        req_data.insert("data", &varvalue);
        req_data.insert("sessionid", &self.session_id);
        req_data.insert("name", &self.name);
        req_data.insert("ownerid", &self.owner_id);

        let req = Self::request(req_data, &self.api_url);
        let head = req.headers().clone();
        let resp = req.text().unwrap();

        if !head.contains_key("signature") {
            #[cfg(feature = "panic")]
            {
                panic!("response was tampered with");
            }
            #[cfg(not(feature = "panic"))]
            {
                return Err("response was tampered with".to_string());
            }
        }
        let sig = head.get("signature").unwrap().to_str().unwrap();
        if sig != Self::make_hmac(&resp, &self.enckey_s) {
            #[cfg(feature = "panic")]
            {
                panic!("Response was tampered with");
            }
            #[cfg(not(feature = "panic"))]
            {
                return Err("Response was tampered with".to_string());
            }
        }
        let json_rep: serde_json::Value = serde_json::from_str(&resp).unwrap();

        self.message = json_rep["message"].as_str().unwrap().to_string();
        self.success = json_rep["success"].as_bool().unwrap();
        Ok(())
    }

    /// gets a user variable
    pub fn getvar(&mut self, varname: String) -> Result<String, String> {
        let mut req_data = HashMap::new();
        req_data.insert("type", "getvar");
        req_data.insert("var", &varname);
        req_data.insert("sessionid", &self.session_id);
        req_data.insert("name", &self.name);
        req_data.insert("ownerid", &self.owner_id);

        let req = Self::request(req_data, &self.api_url);
        let head = req.headers().clone();
        let resp = req.text().unwrap();

        if !head.contains_key("signature") {
            #[cfg(feature = "panic")]
            {
                panic!("response was tampered with");
            }
            #[cfg(not(feature = "panic"))]
            {
                return Err("response was tampered with".to_string());
            }
        }
        let sig = head.get("signature").unwrap().to_str().unwrap();
        if sig != Self::make_hmac(&resp, &self.enckey_s) {
            #[cfg(feature = "panic")]
            {
                panic!("Response was tampered with");
            }
            #[cfg(not(feature = "panic"))]
            {
                return Err("Response was tampered with".to_string());
            }
        }
        let json_rep: serde_json::Value = serde_json::from_str(&resp).unwrap();

        if json_rep["success"].as_bool().unwrap() {
            Ok(json_rep["response"].as_str().unwrap().to_string())
        } else {
            Err(json_rep["message"].as_str().unwrap().to_string())
        }
    }

    /// logs somethink to keyauth
    pub fn log(&mut self, message: String, pcuser: Option<String>) {
        let usr = match pcuser {
            Some(pcuser) => pcuser,
            None => self.username.clone(),
        };

        let mut req_data = HashMap::new();
        req_data.insert("type", "log");
        req_data.insert("message", &message);
        req_data.insert("pcuser", &usr);
        req_data.insert("sessionid", &self.session_id);
        req_data.insert("name", &self.name);
        req_data.insert("ownerid", &self.owner_id);

        Self::request(req_data, &self.api_url);
    }

    /// changes Username, 
    pub fn change_username(&mut self, new_username: String) -> Result<String, String> {
        let mut req_data = HashMap::new();
        req_data.insert("type", "changeUsername");
        req_data.insert("newUsername", &new_username);
        req_data.insert("sessionid", &self.session_id);
        req_data.insert("name", &self.name);
        req_data.insert("ownerid", &self.owner_id);

        let req = Self::request(req_data, &self.api_url);
        let head = req.headers().clone();
        let resp = req.text().unwrap();

        if !head.contains_key("signature") {
            #[cfg(feature = "panic")]
            {
                panic!("response was tampered with");
            }
            #[cfg(not(feature = "panic"))]
            {
                return Err("response was tampered with".to_string());
            }
        }
        let sig = head.get("signature").unwrap().to_str().unwrap();
        if sig != Self::make_hmac(&resp, &self.enckey_s) {
            #[cfg(feature = "panic")]
            {
                panic!("Response was tampered with");
            }
            #[cfg(not(feature = "panic"))]
            {
                return Err("Response was tampered with".to_string());
            }
        }
        let json_rep: serde_json::Value = serde_json::from_str(&resp).unwrap();

        if json_rep["success"].as_bool().unwrap() {
            Ok(json_rep["message"].as_str().unwrap().to_string())
        } else {
            Err(json_rep["message"].as_str().unwrap().to_string())
        }
    }

    #[cfg(feature = "web_loader")]
    pub fn web_login(&mut self, hwid: Option<String>) -> Result<(), String> {
        use std::io::Write;

        let hwidd = match hwid {
            Some(hwid) => hwid,
            None => self.hwid.clone(),
        };

        let listener = TcpListener::bind("127.0.0.1:1337");
        if listener.is_err() {
            return Err("Couldnt bind to port 1337".to_string());
        }
        let listener = listener.unwrap();

        for stream in listener.incoming() {
            if stream.is_err() {
                continue;
            }
            let mut stream = stream.unwrap();
            let mut buf = [0u8; 4096];
            stream.read(&mut buf).unwrap();
            let mut headers = [httparse::EMPTY_HEADER; 16];
            let mut req = httparse::Request::new(&mut headers);
            req.parse(&buf).unwrap();
            if req.path.unwrap().starts_with("/handshake") {
                let s = req.path.unwrap();
                let start = s.find("?user=").unwrap_or(0) + 6;
                let end = s.rfind("&token=").unwrap_or(s.len());
                let user = &s[start..end];
                let start = s.find("&token=").unwrap_or(0) + 7;
                let token = &s[start..];
                let mut req_data = HashMap::new();
                req_data.insert("type", "login");
                req_data.insert("username", &user);
                req_data.insert("token", &token);
                req_data.insert("name", &self.name);
                req_data.insert("ownerid", &self.owner_id);
                req_data.insert("hwid", &self.hwid);
                req_data.insert("sessionid", &self.session_id);

                let req = Self::request(req_data, &self.api_url);
                let head = req.headers().clone();
                let resp = req.text().unwrap();

                if !head.contains_key("signature") {
                    #[cfg(feature = "panic")]
                    {
                        panic!("response was tampered with");
                    }
                    #[cfg(not(feature = "panic"))]
                    {
                        return Err("response was tampered with".to_string());
                    }
                }
                let sig = head.get("signature").unwrap().to_str().unwrap();
                if sig != Self::make_hmac(&resp, &self.enckey_s) {
                    #[cfg(feature = "panic")]
                    {
                        panic!("Response was tampered with");
                    }
                    #[cfg(not(feature = "panic"))]
                    {
                        return Err("Response was tampered with".to_string());
                    }
                }
                let json_rep: serde_json::Value = serde_json::from_str(&resp).unwrap();
                let (status, body) = if json_rep["success"].as_bool().unwrap() {
                    self.username = user.to_string();
                    self.ip = json_rep["info"]["ip"].as_str().unwrap().to_string();
                    self.hwid = hwidd;
                    self.create_date = json_rep["info"]["createdate"].as_str().unwrap().to_string();
                    self.last_login = json_rep["info"]["lastlogin"].as_str().unwrap().to_string();
                    self.subscription = json_rep["info"]["subscriptions"][0]["subscription"].as_str().unwrap().to_string();

                    (420, "SHEESH")
                } else {
                    (200, json_rep["message"].as_str().unwrap())
                };
                let response = format!(r#"HTTP/1.1 {} OK
Access-Control-Allow-Methods: Get, Post
Access-Control-Allow-Origin: *
Via: hugzho's big brain
Location: your kernel ;)
Retry-After: never lmao
Server: \r\n\r\n

{}"#, status, body);
                stream.write_all(response.as_bytes()).unwrap();
                return Ok(());
            }
        }
        Ok(())
    }

    #[cfg(feature = "web_loader")]
    pub fn button(&self, button: &str) -> Result<(), String> {
         use std::io::Write;

        let listener = TcpListener::bind("127.0.0.1:1337");
        if listener.is_err() {
            return Err("Couldnt bind to port 1337".to_string());
        }
        let listener = listener.unwrap();

        for stream in listener.incoming() {
            if stream.is_err() {
                continue;
            }
            let mut stream = stream.unwrap();
            let mut buf = [0u8; 4096];
            stream.read(&mut buf).unwrap();
            let mut headers = [httparse::EMPTY_HEADER; 16];
            let mut req = httparse::Request::new(&mut headers);
            req.parse(&buf).unwrap();
            if req.path.unwrap().starts_with(format!("/{}", button).as_str()) {
                let response = format!(r#"HTTP/1.1 {} OK
Access-Control-Allow-Methods: Get, Post
Access-Control-Allow-Origin: *
Via: hugzho's big brain
Location: your kernel ;)
Retry-After: never lmao
Server: \r\n\r\n

{}"#, 420, "SHEESH");
                stream.write_all(response.as_bytes()).unwrap();
                return Ok(());
            }
        }
        Ok(())
    }

    fn request(req_data: HashMap<&str, &str>, url: &str) -> Response {
        let client = Client::new();
        let mut req_data_str = String::new();
        for d in req_data {
            req_data_str.push_str(&format!("{}={}&", d.0, d.1))
        }
        req_data_str = req_data_str.strip_suffix("&").unwrap().to_string();
        client.post(url.to_string())
            .body(req_data_str)
            .header("User-Agent", "KeyAuth")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .send().unwrap()
    }

    fn make_hmac(message: &str, key: &str) -> String {
        hex::encode(HMAC::mac(message, key)).to_string()
    }
}
