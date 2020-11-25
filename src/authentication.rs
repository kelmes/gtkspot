// extern crate secret_service;
// use secret_service::SecretService;
// use secret_service::EncryptionType;
extern crate keyring;


use librespot_core::authentication::Credentials as RespotCredentials;
use librespot_protocol::authentication::AuthenticationType;

use std::io::{self, Read};

pub fn create_credentials() -> Result<RespotCredentials, String> {
    // initialize secret service (dbus connection and encryption session)
    // let ss = SecretService::new(EncryptionType::Dh).unwrap();

    // get default collection
    // let collection = ss.get_default_collection().unwrap();



    // search items by properties
    // let mut search_items = ss.search_items(
    //     vec![("spotify_user", "atheris84")]
    // ).unwrap();


    // println!("found {} secrets", search_items.len());

    // if search_items.len() == 0 {
        //create new item
    //     collection.create_item(
    //         "gtkspot", // label
    //         vec![("spotify_user", "atheris84")], // properties
    //         b"st0rmS#@", //secret
    //         true, // replace item with same attributes
    //         "text/plain" // secret content type
    //     ).unwrap();
    //     search_items = ss.search_items(
    //         vec![("spotify_user", "atheris84")]
    //     ).unwrap();
    // }

    // let item = search_items.get(0).unwrap();

    // retrieve secret from item
    // let secret = item.get_secret().unwrap();
    // let secret_string = match std::str::from_utf8(&secret) {
    //     Ok(x) => x,
    //     Err(x) => panic!("failed to read secret password")
    // };
    // println!("secret is: {}", secret_string);
    // assert_eq!(secret, b"test_secret");

    // delete item (deletes the dbus object, not the struct instance)
    // item.delete().unwrap();

    let service = "com.github.kelmes.gtkspot";
    let username = "atheris84";

    let keyring = keyring::Keyring::new(&service, &username);

    let password = match keyring.get_password() {
        Ok(x) => x,
        Err(e) => {
            println!("error retrieving password: {}", e);
            println!("adding new password from stdin");

            let mut input = String::new();
            match io::stdin().read_line(&mut input) {
                Ok(n) => {
                    println!("{} bytes read", n);
                    println!("{}", input);
                }
                Err(error) => println!("error: {}", error),
            }
            if input.ends_with("\n") {
                input.pop();
            }
            println!("storing password in keyring");
            match keyring.set_password(&input) {
                Ok(x) => {},
                Err(x) => {println!("error storing password: {}", x)},
            };
            input
        }
    };
    println!("The password is '{}'", password);

    let username = String::from(username);
    let auth_data = String::from(password).as_bytes().to_vec();
    Ok(RespotCredentials {
        username,
        auth_type: AuthenticationType::AUTHENTICATION_USER_PASS,
        auth_data,
    })
}

// TODO: better with futures?
fn auth_poller(url: &str) {
    //let app_sink = app_sink.clone();
    let url = url.to_string();
    std::thread::spawn(move || {
        let timeout = std::time::Duration::from_secs(5 * 60);
        let start_time = std::time::SystemTime::now();
        while std::time::SystemTime::now()
            .duration_since(start_time)
            .unwrap_or(timeout)
            < timeout
        {
            if let Ok(mut response) = reqwest::get(&url) {
                if response.status() != reqwest::StatusCode::ACCEPTED {
                    let result = match response.status() {
                        reqwest::StatusCode::OK => {
                            let creds = response
                                .json::<AuthResponse>()
                                .expect("Unable to parse")
                                .credentials;
                            Ok(creds)
                        }

                        _ => Err(format!(
                            "Facebook auth failed with code {}: {}",
                            response.status(),
                            response.text().unwrap()
                        )),
                    };
                    //app_sink
                    //    .send(Box::new(|s: &mut Cursive| {
                    //        s.set_user_data(result);
                    //        s.quit();
                    //    }))
                    //    .unwrap();
                    return;
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(1000));
        }

        //app_sink
        //    .send(Box::new(|s: &mut Cursive| {
        //        s.set_user_data::<Result<RespotCredentials, String>>(Err(
        //            "Timed out authenticating".to_string(),
        //        ));
        //        s.quit();
        //    }))
        //    .unwrap();
    });
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthResponse {
    pub credentials: RespotCredentials,
    pub error: Option<String>,
}
