mod string_tools {
    pub fn get_all_before_strict<'a>(text: &'a str, begin: &str) -> Option<&'a str> {
        let begin = text.find(begin)?;
        Some(&text[..begin])
    }

    pub fn get_all_after_strict<'a>(text: &'a str, end: &str) -> Option<&'a str> {
        let end = text.find(end)? + end.len();
        Some(&text[end..])
    }

    pub fn get_all_between_strict<'a>(text: &'a str, begin: &str, end: &str) -> Option<&'a str> {
        let text = get_all_after_strict(text, begin)?;
        let text = get_all_before_strict(text, end)?;
        Some(text)
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

    #[test]
    fn string_tools_test() {
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

/// Contains functions related to google pages parsing.
mod google {
    use crate::string_tools::*;

    fn get_full_url(request: &str, page: usize) -> String {
        format!(
            "https://www.google.com/search?q={}&start={}",
            request,
            page * 10
        )
    }

    pub fn resolve(request: &str, page: usize) -> Vec<String> {
        if let Ok(response) = minreq::get(get_full_url(request, page))
            .with_header("Accept", "text/plain")
            .with_header("Host", "www.google.com")
            .with_header(
                "User-Agent",
                "Mozilla/5.0 (X11; Linux x86_64; rv:71.0) Gecko/20100101 Firefox/71.0",
            )
            .send()
        {
            let mut body = response.body.as_str();
            let mut rep = Vec::new();
            while let Some(url) = get_all_between_strict(body, "\"r\"><a href=\"", "\" onmousedown=\"return rwt(") {
                rep.push(url.to_string());
                body = get_all_after(body, url);
            }
            rep
        } else {
            eprintln!(
                "Warning: can't get response from google for {}",
                get_full_url(request, page)
            );
            Vec::new()
        }
    }

    #[test]
    fn get_full_url_test() {
        assert_eq!(
            "https://www.google.com/search?q=\"gleam.io\"+site:youtube.com&tbs=qdr:h&start=10",
            get_full_url("\"gleam.io\"+site:youtube.com&tbs=qdr:h", 1)
        );
    }

    #[test]
    fn resolve_google_request() {
        let result = resolve("\"gleam.io\"+site:youtube.com&tbs=qdr:h", 0);
        assert!(result.len() > 0);
    }
}

/// Contains functions related to youtube pages parsing
mod youtube {
    use crate::string_tools::*;

    /// Load a youtube page and return any gleam url located in the description of the video.
    pub fn resolve(url: &str) -> Vec<String> {
        if let Ok(response) = minreq::get(url)
            .with_header("Accept", "text/plain")
            .with_header("Host", "www.youtube.com")
            .with_header(
                "User-Agent",
                "Mozilla/5.0 (X11; Linux x86_64; rv:71.0) Gecko/20100101 Firefox/71.0",
            )
            .send()
        {
            let mut description = get_all_between_strict(&response.body, ",\"shortDescription\":\"", "\",\"isCrawlable\":").unwrap_or("");
            let mut rep = Vec::new();
            while get_all_between(description, "https://gleam.io/competitions/", "\\") != "" {
                rep.push(format!("https://gleam.io/competitions/{}", get_all_between(description, "https://gleam.io/competitions/", "\\")));
                description = get_all_after(description, url);
            }
            rep
        } else {
            eprintln!(
                "Warning: can't get response for {}",
                url
            );
            Vec::new()
        }
    }

    #[test]
    fn find_in_youtube() {
        let result = resolve("https://www.youtube.com/watch?v=yy9tGgHMIE8");
        assert_eq!(result, vec!["https://gleam.io/competitions/KgwYi-giveaway-5x-invitatii-bucharest-gaming-week"]);
        let result = resolve("https://www.youtube.com/watch?v=d1QzAvTmCZs");
        assert_eq!(result, vec!["https://gleam.io/competitions/4t6vD-ardagamertv7"]);
        let result = resolve("https://www.youtube.com/watch?v=Am7v1Fp92I0");
        assert_eq!(result, vec!["https://gleam.io/competitions/6mqZ0-7-gnlk-awp-ekilii"]);
    }
}