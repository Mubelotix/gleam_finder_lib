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

mod string_tools {
    pub fn get_all_before_strict<'a>(text: &'a str, begin: &str) -> Option<&'a str> {
        let begin = text.find(begin)?;
        Some(&text[..begin])
    }

    pub fn get_all_after_strict<'a>(text: &'a str, end: &str) -> Option<&'a str> {
        let end = text.find(end)? + end.len();
        Some(&text[end..])
    }

    pub fn get_idx_before_strict<'a>(text: &'a str, begin: &str) -> Option<usize> {
        let begin = text.find(begin)?;
        Some(begin)
    }

    pub fn get_idx_before(text: &str, begin: &str) -> usize {
        if let Some(idx) = text.find(begin) {
            return idx
        } else {
            return text.len();
        }
    }

    pub fn get_idx_after_strict<'a>(text: &'a str, end: &str) -> Option<usize> {
        let end = text.find(end)? + end.len();
        Some(end)
    }

    pub fn get_all_between_strict<'a>(text: &'a str, begin: &str, end: &str) -> Option<&'a str> {
        let text = get_all_after_strict(text, begin)?;
        let text = get_all_before_strict(text, end)?;
        Some(text)
    }

    pub fn get_idx_between_strict<'a>(text: &'a str, begin: &str, end: &str) -> Option<(usize, usize)> {
        let after = get_idx_after_strict(text, begin)?;
        let before = get_idx_before_strict(&text[after..], end)?;
        Some((after, after + before))
    }

    pub fn get_all_before<'a>(text: &'a str, begin: &str) -> &'a str {
        let begin = text.find(begin).unwrap_or(text.len());
        &text[..begin]
    }

    pub fn get_all_after<'a>(text: &'a str, end: &str) -> &'a str {
        if let Some(mut end_index) = text.find(end) {
            end_index += end.len();
            return &text[end_index..];
        } else {
            return "";
        }
    }

    pub fn get_all_between<'a>(text: &'a str, begin: &str, end: &str) -> &'a str {
        let text = get_all_after(text, begin);
        let text = get_all_before(text, end);
        text
    }

    /// put an url+noise, get url (without http://domain.something/)
    pub fn get_url(url: &str) -> &str {
        let mut i = 0;
        for c in url.chars() {
            if !c.is_ascii_alphanumeric() {
                if c != '-' && c != '/' && c != '_' {
                    break;
                }
            }
            i += 1;
        }
        &url[..i]
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn string_tools_test() {
            assert_eq!("/search", get_url("/search?q=\"gleam.io\"&tbs=qdr:h&filter=0&start={}"));
            assert_eq!("/competition/something-wtf-giveaway", get_url("/competition/something-wtf-giveaway and more"));
            assert_eq!(Some("test"), get_all_before_strict("testlol", "lol"));
            assert_eq!(Some("test"), get_all_before_strict("testloltestlol", "lol"));
            assert_eq!(Some("lol"), get_all_after_strict("testlol", "test"));
            assert_eq!(Some("testlol"), get_all_after_strict("testloltestlol", "lol"));
            assert_eq!(Some("str3str4"), get_all_between_strict("str1str2str3str4str5", "str2", "str5"));
            assert_eq!(Some("str3str4"), get_all_between_strict("str5str1str2str3str4str5str2str3str5", "str2", "str5"));
            assert_eq!(None, get_all_before_strict("str1str2", "str3"));
            assert_eq!("str1str2", get_all_before("str1str2", "str3"));
            assert_eq!(None, get_all_after_strict("str1str2", "str3"));
            assert_eq!("", get_all_after("str1str2", "str3"));
            assert_eq!("str2str3", get_all_between("str1str2str3str4", "str1", "str4"));
            assert_eq!("", get_all_between("str1str2str3str4", "str0", "str4"));
            assert_eq!("str2str3str4", get_all_between("str1str2str3str4", "str1", "str6"));
        }
    }
}

/// Contains functions related to google pages parsing.
pub mod google {
    use crate::string_tools::*;

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
    pub fn search(page: usize) -> Vec<String> {
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
                rep
            } else {
                Vec::new()
            }
        } else {
            eprintln!(
                "Warning: can't get response from google for {}",
                get_full_url(page)
            );
            Vec::new()
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
            let result = search(0);
            assert!(result.len() > 0);
        }
    }
}

pub mod intermediary {
    use crate::string_tools::*;
    use crate::gleam::get_gleam_id;

    pub fn resolve(url: &str) -> Vec<String> {
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
                    body = get_all_after(&body, &url);
                    let url = if url.len() >= 20 {
                        format!("https://gleam.io/{}", &url[..20])
                    } else {
                        format!("https://gleam.io/{}", url)
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
                final_rep
            } else {
                Vec::new()
            }
        } else {
            eprintln!(
                "Warning: can't get response for {}",
                url
            );
            Vec::new()
        }
    }
}

/// Contains giveaways fetcher
pub mod gleam {
    use super::string_tools::get_all_between_strict;
    use super::string_tools::get_idx_between_strict;
    use super::string_tools::get_idx_before;
    use std::time::{SystemTime, UNIX_EPOCH, Duration};
    use std::thread::sleep;

    #[cfg(feature = "serde-support")]
    use serde::{Serialize, Deserialize};

    fn clear_description(description: &mut String) {
        while let Some((x, x2)) = get_idx_between_strict(&description, "\\u003c", "\\u003e") {
            let mut before = description[..x-6].to_string();
            let after = &description[x2+6..];
            before.push_str(after);
            *description = String::from(before);
        }
        *description = description[..get_idx_before(description, "\\u003")].to_string();
        while let Some(idx) = description.find("&#39;") {
            description.remove(idx);
            description.remove(idx);
            description.remove(idx);
            description.remove(idx);
            description.remove(idx);
            description.insert(idx, '\'');
        }
        while let Some(value) = get_all_between_strict(description, "\\u0026#", ";") {
            if let Ok(charcode) = value.parse::<u32>() {
                if let Some(character) = std::char::from_u32(charcode) {
                    let range = get_idx_between_strict(description, "\\u0026#", ";").unwrap();
                    let range = range.0-7..range.1+1;
                    description.replace_range(range, &character.to_string())
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }

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
        pub fn fetch(url: &str) -> Option<Giveaway> {
            let giveaway_id = match get_gleam_id(url) {
                Some(id) => id,
                None => return None
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
                    let start_date: u64 = if let Ok(start_date) = get_all_between_strict(body, "starts_at&quot;:", ",&quot;")?.parse() {
                        start_date
                    } else {
                        return None;
                    };
                    let end_date: u64 = if let Ok(end_date) = get_all_between_strict(body, "ends_at&quot;:", ",&quot;")?.parse() {
                        end_date
                    } else {
                        return None;
                    };
                    let entry_count: Option<u64> = if let Some(entry_count) = get_all_between_strict(body, "initEntryCount(", ")") {
                        if let Ok(entry_count) = entry_count.parse() {
                            Some(entry_count)
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    let name = get_all_between_strict(body, "name&quot;:&quot;", "&quot;")?.to_string();
                    let mut description = get_all_between_strict(body, "description&quot;:&quot;", "&quot;")?.to_string();
                    
                    clear_description(&mut description);
    
                    return Some(Giveaway {
                        gleam_id: giveaway_id.to_string(),
                        description,
                        entry_count,
                        start_date,
                        end_date,
                        update_date: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                        name
                    });
                }
            }
            None
        }

        /// Fetch some urls and wait a cooldown between each request
        pub fn fetch_vec(urls: Vec<&str>, cooldown: u64) -> Vec<Giveaway> {
            let mut giveaways = Vec::new();

            for url in &urls {
                if let Some(giveaway) = Giveaway::fetch(url) {
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
            return false;
        }

        /// Reload the giveaway and update informations. 
        pub fn update(&mut self) {
            if let Some(giveaway) = Giveaway::fetch(&self.get_url()) {
                *self = giveaway;
            } else {
                eprintln!("this giveaways seems to be inexistant now...");
            }
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

        #[test]
        fn test_description() {
            let mut description = String::from("\\u003ch2\\u003eRetweet to Win Giveaway of $1000 TROY Tokens! \\u0026#128420;\\u003c/h2\\u003e");
            clear_description(&mut description);
            assert_eq!(&description, "Retweet to Win Giveaway of $1000 TROY Tokens! 🖤");

            let mut description = String::from("\\u003cp\\u003eGet a chance to win one of the 10 prizes worth $50 each equivalent in Matic Tokens ($500 = 12773.35)! It&#39;s a fun game... but you know that.\\u003c/p\\u003e\\u003cp\\u003e\\u003c/p\\u003e\\u003cdiv style=\\");
            clear_description(&mut description);
            assert_eq!(&description, "Get a chance to win one of the 10 prizes worth $50 each equivalent in Matic Tokens ($500 = 12773.35)! It's a fun game... but you know that.");
        }
    }
}