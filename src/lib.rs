//! This crate contains tools you can use to get gleam giveaways links.  
//!   
//! You can search google for every youtube video mentionning gleam.io in the last hour with google::search().  
//! After you got this links to youtube, you can load the pages and parse the description to get gleam.io links with youtube::resolve().  
//! You can parse a gleam.io page with the Giveaway struct.
//! 
//! # Examples
//! 
//! ```no_run
//! use gleam_finder::*;
//! 
//! for page in 0..4 {
//!     for link in google::search(page) {
//!         println!("resolving {}", link);
//!         for gleam_link in intermediary::resolve(&link) {
//!             println!("gleam link found: {}", gleam_link);
//!         }
//!     }
//! }
//! ```

#[derive(Debug)]
pub enum Error {
    Timeout,
    InvalidResponse,
}

/// Contains functions related to google pages parsing.
pub mod google {
    use string_tools::{get_all_between_strict, get_all_after};
    use super::Error;

    fn get_full_url(page: usize) -> String {
        format!(
            "https://www.google.com/search?q=\"gleam.io\"&tbs=qdr:h&filter=0&start={}",
            page * 10
        )
    }

    /// Search google for a something and returns result urls.  
    /// See [Google Advanced Search](https://www.google.com/advanced_search) for more information about request syntax.  
    /// Only one page is loaded.  
    /// # Examples
    /// ```
    /// use gleam_finder::google;
    /// 
    /// // note that we only test the first page of google results and that there can be more
    /// let links = google::search(0);
    /// ```
    pub fn search(page: usize) -> Result<Vec<String>, Error> {
        if let Ok(response) = minreq::get(get_full_url(page))
            .with_header("Accept", "text/plain")
            .with_header("Host", "www.google.com")
            .with_header(
                "User-Agent",
                "Mozilla/5.0 (X11; Linux x86_64; rv:71.0) Gecko/20100101 Firefox/71.0",
            )
            .send()
        {
            if let Ok(mut body) = response.as_str() {
                let mut rep = Vec::new();
                while let Some(url) = get_all_between_strict(body, "\"r\"><a href=\"", "\" onmousedown=\"return rwt(") {
                    rep.push(url.to_string());
                    body = get_all_after(body, url);
                }
                Ok(rep)
            } else {
                Err(Error::InvalidResponse)
            }
        } else {
            Err(Error::Timeout)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn get_full_url_test() {
            assert_eq!(
                "https://www.google.com/search?q=\"gleam.io\"&tbs=qdr:h&filter=0&start=10",
                get_full_url(1)
            );
        }

        #[test]
        fn resolve_google_request() {
            let result = search(0).unwrap();
            assert!(!result.is_empty());
        }
    }
}

pub mod intermediary {
    use string_tools::get_all_after;
    use crate::gleam::get_gleam_id;
    use super::Error;

    /// put an url+noise, get url (without http://domain.something/)
    fn get_url(url: &str) -> &str {
        let mut i = 0;
        for c in url.chars() {
            if !c.is_ascii_alphanumeric() && c != '-' && c != '/' && c != '_' {
                break;
            }
            i += 1;
        }
        &url[..i]
    }

    pub fn resolve(url: &str) -> Result<Vec<String>, Error> {
        if let Ok(response) = minreq::get(url)
            .with_header("Accept", "text/plain")
            .with_header(
                "User-Agent",
                "Mozilla/5.0 (X11; Linux x86_64; rv:71.0) Gecko/20100101 Firefox/71.0",
            )
            .send()
        {
            if let Ok(mut body) = response.as_str() {
                let mut rep = Vec::new();
                while get_all_after(&body, "https://gleam.io/") != "" {
                    let url = get_url(get_all_after(&body, "https://gleam.io/"));
                    body = get_all_after(&body, "https://gleam.io/");
                    let url = if url.len() >= 20 {
                        format!("https://gleam.io/{}", &url[..20])
                    } else if !url.is_empty() {
                        format!("https://gleam.io/{}", url)
                    } else {
                        continue;
                    };
                    if !rep.contains(&url) {
                        rep.push(url);
                    }
                }
                let mut final_rep = Vec::new();
                for url in rep {
                    if let Some(id) = get_gleam_id(&url) {
                        final_rep.push(format!("https://gleam.io/{}/-", id));
                    }
                };
                Ok(final_rep)
            } else {
                Err(Error::InvalidResponse)
            }
        } else {
            Err(Error::Timeout)
        }
    }
}

/// Contains giveaways fetcher
pub mod gleam {
    use string_tools::{get_all_between_strict, get_idx_between_strict};
    use std::time::{SystemTime, UNIX_EPOCH, Duration};
    use std::thread::sleep;
    use serde_json::{from_str, Value};
    use super::Error;

    #[cfg(feature = "serde-support")]
    use serde::{Serialize, Deserialize};

    /// Extract the id of the giveaway from an url.
    pub fn get_gleam_id(url: &str) -> Option<&str> {
        if url.len() == 37 && &url[0..30] == "https://gleam.io/competitions/" {
            return Some(&url[30..35]);
        } else if url.len() >= 23 && &url[0..17] == "https://gleam.io/" && &url[22..23] == "/"{
            return Some(&url[17..22]);
        }
        None
    }

    /// A simple struct used to store informations about a gleam.io giveaway.
    /// Can be serialized by activing the feature "serde-support"
    #[derive(Debug)]
    #[cfg_attr(feature = "serde-support", derive(Serialize, Deserialize))]
    pub struct Giveaway {
        gleam_id: String,
        entry_count: Option<u64>,
        entry_methods: Vec<(String, u64)>,
        start_date: u64,
        end_date: u64,
        update_date: u64,
        name: String,
        description: String,
    }

    impl Giveaway {
        /// Load a gleam.io page and produce a giveaway struct.
        /// The url stored in this struct will be reformatted (ex: https://gleam.io/2zAsX/bitforex-speci => https://gleam.io/2zAsX/-) in order to make duplication inpossible.
        /// Return None if something does not work.
        pub fn fetch(url: &str) -> Result<Giveaway, Error> {
            let giveaway_id = match get_gleam_id(url) {
                Some(id) => id,
                None => return Err(Error::InvalidResponse)
            };
            let url = format!("https://gleam.io/{}/-", giveaway_id);

            if let Ok(response) = minreq::get(&url)
                .with_header("Host", "gleam.io")
                .with_header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64; rv:72.0) Gecko/20100101 Firefox/72.0")
                .with_header("Accept", "text/html")
                .with_header("DNT", "1")
                .with_header("Connection", "keep-alive")
                .with_header("Upgrade-Insecure-Requests", "1")
                .with_header("TE", "Trailers")
                .send()
            {
                if let Ok(body) = response.as_str() {
                    if let Some(json) = get_all_between_strict(body, "<div class='popup-blocks-container' ng-init='initCampaign(", ")'>") {
                        let json = json.replace("&quot;", "\"");
                        if let Ok(json) = from_str::<Value>(&json) {
                            if let (Some(campaign), Some(Some(incentives)), Some(entry_methods_json)) = (json["campaign"].as_object(), json["incentives"].as_array().map(|a| a[0].as_object()), json["entry_methods"].as_array()) {
                                let entry_count: Option<u64> = if let Some(entry_count) = get_all_between_strict(body, "initEntryCount(", ")") {
                                    if let Ok(entry_count) = entry_count.parse() {
                                        Some(entry_count)
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                };

                                let mut entry_methods = Vec::new();
                                for entry_method in entry_methods_json {
                                    entry_methods.push((entry_method["entry_type"].as_str().ok_or(Error::InvalidResponse)?.to_string(), entry_method["worth"].as_u64().ok_or(Error::InvalidResponse)?))
                                }

                                let mut description = incentives["description"].as_str().ok_or(Error::InvalidResponse)?.to_string();
                                while let Some((begin, end)) = get_idx_between_strict(&description, "<", ">") {
                                    description.replace_range(begin-1..end+1, "");
                                }

                                description = description.replace("\u{a0}", "\n");
                                description = description.replace("&#39;", "'");
                            
                                return Ok(Giveaway {
                                    gleam_id: giveaway_id.to_string(),
                                    name: campaign["name"].as_str().map(|s| s.to_string()).ok_or(Error::InvalidResponse)?,
                                    description,
                                    entry_methods,
                                    start_date: campaign["starts_at"].as_u64().ok_or(Error::InvalidResponse)?,
                                    end_date: campaign["ends_at"].as_u64().ok_or(Error::InvalidResponse)?,
                                    update_date: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                                    entry_count
                                })
                            }
                        }
                    }
                }
                Err(Error::InvalidResponse)
            } else {
                Err(Error::Timeout)
            }
        }

        /// Fetch some urls and wait a cooldown between each request
        pub fn fetch_vec(urls: Vec<&str>, cooldown: u64) -> Vec<Giveaway> {
            let mut giveaways = Vec::new();

            for url in &urls {
                if let Ok(giveaway) = Giveaway::fetch(url) {
                    giveaways.push(giveaway)
                }
                if urls.len() > 1 {
                    sleep(Duration::from_secs(cooldown));
                }
            }

            giveaways
        }

        /// Return the url
        pub fn get_url(&self) -> String {
            format!("https://gleam.io/{}/-", self.gleam_id)
        }

        /// Return the gleam giveaway id
        pub fn get_gleam_id(&self) -> &str {
            &self.gleam_id
        }

        /// Return the name
        pub fn get_name(&self) -> &str {
            &self.name
        }

        /// Return the description
        pub fn get_description(&self) -> &str {
            &self.description
        }

        /// Return the number of entries
        pub fn get_entry_count(&self) -> Option<u64> {
            self.entry_count
        }

        /// Return the creation date in seconds
        pub fn get_start_date(&self) -> u64 {
            self.start_date
        }

        /// Return the end date in seconds
        pub fn get_end_date(&self) -> u64 {
            self.end_date
        }

        /// Return the last update date in seconds
        pub fn get_update_date(&self) -> u64 {
            self.update_date
        }

        /// Check if the giveaway is ended
        pub fn is_active(&self) -> bool {
            if self.end_date < SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() {
                return true;
            }
            false
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_giveaway_struct() {
            let giveaway = Giveaway::fetch("https://gleam.io/29CPn/-2-alok-gveaway-and-12000-diamonds-").unwrap();
            println!("{:?}", giveaway);

            sleep(Duration::from_secs(5));

            let giveaway = Giveaway::fetch("https://gleam.io/8nTqy/amd-5700xt-gpu").unwrap();
            println!("{:?}", giveaway);

            sleep(Duration::from_secs(5));

            let giveaway = Giveaway::fetch("https://gleam.io/ff3QT/win-an-ipad-pro-with-canstar").unwrap();
            println!("{:?}", giveaway);
        }

        #[test]
        fn get_gleam_urls() {
            assert_eq!(get_gleam_id("https://gleam.io/competitions/lSq1Q-s"), Some("lSq1Q"));
            assert_eq!(get_gleam_id("https://gleam.io/2zAsX/bitforex-speci"), Some("2zAsX"));
            assert_eq!(get_gleam_id("https://gleam.io/7qHd6/sorteo"),         Some("7qHd6"));
            assert_eq!(get_gleam_id("https://gleam.io/3uSs9/taylor-moon"),    Some("3uSs9"));
            assert_eq!(get_gleam_id("https://gleam.io/OWMw8/sorteo-de-1850"), Some("OWMw8"));
            assert_eq!(get_gleam_id("https://gleam.io/competitions/CEoiZ-h"), Some("CEoiZ"));
            assert_eq!(get_gleam_id("https://gleam.io/7qHd6/-"),              Some("7qHd6"));
        }
    }
}