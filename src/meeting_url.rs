use regex::Regex;

/// Parse Zoom meeting URL
/// Returns (meeting_id_or_vanity_id, password)
/// For PMR URLs (with /my/), returns the vanity ID as the first element
/// For standard URLs, returns the numeric meeting ID as the first element
pub fn parse(meeting_url: &str) -> Option<(String, Option<String>)> {
    url::Url::parse(meeting_url)
        .map(|_url| {
            // Extract password first (works for both formats)
            let password = {
                let re =
                    Regex::new(r"(?i)(?:pwd=|pwd%3D|Passcode:\s?)([^\s&\)]+)").unwrap();
                re.captures(meeting_url)
                    .map(|captures| captures.get(1).map(|pwd| pwd.as_str().to_string()))
                    .flatten()
            };

            meeting_url
                .split_once(&"zoom.us".to_lowercase())
                .map(|(_, suffix)| {
                    let parts: Vec<&str> = suffix.split("/").collect();
                    
                    // Check if this is a Personal Meeting Room URL (contains /my/)
                    if parts.len() >= 3 && parts[1].to_lowercase() == "my" {
                        // Extract vanity ID (the part after /my/)
                        parts.get(2)
                            .map(|vanity_id| {
                                // Remove query parameters if present
                                vanity_id.split("?").next().unwrap_or(vanity_id).to_string()
                            })
                            .filter(|s| !s.is_empty())
                            .map(|vanity_id| (vanity_id, password))
                    } else {
                        // Standard meeting URL - extract numeric meeting ID
                        parts
                            .get(2)
                            .map(|second_group| {
                                second_group
                                    .chars()
                                    .take_while(|c| c.is_digit(10))
                                    .collect::<String>()
                            })
                            .filter(|s| !s.is_empty())
                            .map(|meeting_id| (meeting_id, password))
                    }
                })
                .flatten()
        })
        .ok()
        .flatten()
}

#[cfg(test)]
mod test_parsing {
    use super::parse;

    #[test]
    fn should_parse_standard_zoom_url() {
        let url = "https://zoom.us/j/1234567890?pwd=abcdef";
        assert_eq!(
            parse(url),
            Some(("1234567890".into(), Some("abcdef".into())))
        );
    }
    #[test]
    fn should_parse_url_with_dot_in_password() {
        let url = "https://app.zoom.us/wc/81561946371/start?fromPWA=1&pwd=aBMNeaTbJOStQrPCRT2fBrviRTp15D.1";
        assert_eq!(
            parse(url),
            Some((
                "81561946371".into(),
                Some("aBMNeaTbJOStQrPCRT2fBrviRTp15D.1".into())
            ))
        );
    }
    #[test]
    fn should_parse_url_with_passcode_format() {
        let url = "https://us06web.zoom.us/j/88240852079 (Passcode: 584706)";
        assert_eq!(
            parse(url),
            Some(("88240852079".into(), Some("584706".into())))
        );
    }
    #[test]
    fn should_parse_url_with_passcode_format_without_spaces() {
        let url = "https://us06web.zoom.us/j/88240852079 (Passcode:584706)";
        assert_eq!(
            parse(url),
            Some(("88240852079".into(), Some("584706".into())))
        );
    }
    #[test]
    fn should_parse_url_without_spaces_before_passcode() {
        let url = "https://us06web.zoom.us/j/88240852079(passcode: 584706)";
        assert_eq!(
            parse(url),
            Some(("88240852079".into(), Some("584706".into())))
        );
    }

    #[test]
    fn subdomain_standard_password() {
        let url = "https://turing.zoom.us/j/85001833920?pwd=cWY6TnhHRkdKZXcwSVk5aGE1VXpqUT09";
        assert_eq!(
            parse(url),
            Some((
                "85001833920".into(),
                Some("cWY6TnhHRkdKZXcwSVk5aGE1VXpqUT09".into())
            ))
        );
    }
    #[test]
    fn subdomain_with_passcode_format() {
        let url = "https://us06web.zoom.us/j/84617432243 (Passcode: 584706)";
        assert_eq!(
            parse(url),
            Some(("84617432243".into(), Some("584706".into())))
        );
    }
    #[test]
    fn subdomain_with_dot_in_password() {
        let url = "https://acrons-team.zoom.us/j/98832106351?pwd=LED1nQDsZvuIED3ccBTlw04Gzi0MOw.1";
        assert_eq!(
            parse(url),
            Some((
                "98832106351".into(),
                Some("LED1nQDsZvuIED3ccBTlw04Gzi0MOw.1".into())
            ))
        );
    }

    #[test]
    fn urls_without_password() {
        let url = "https://zoom.us/j/92648182477";
        assert_eq!(parse(url), Some(("92648182477".into(), None)));
    }
    #[test]
    fn urls_without_password_with_special_zoom_prefix() {
        let url = "https://us02web.zoom.us/j/74495491647";
        assert_eq!(parse(url), Some(("74495491647".into(), None)));
    }
    #[test]
    fn urls_without_password_with_special_zoom() {
        let url = "https://us05web.zoom.us/j/6298382741";
        assert_eq!(parse(url), Some(("6298382741".into(), None)));
    }

    #[test]
    fn google_redirect_url() {
        let url = "https://www.google.com/url?q=https://zoom.us/j/1122334455?pwd=abc123";
        assert_eq!(
            parse(url),
            Some(("1122334455".into(), Some("abc123".into())))
        );
    }

    #[test]
    fn error_invalid_url() {
        let url = "https://invalid-url.com";
        assert_eq!(parse(url), None);
    }
    #[test]
    fn error_non_numeric_id() {
        let url = "https://zoom.us/j/abcdefg";
        assert_eq!(parse(url), None);
    }

    #[test]
    fn web_client_pma_url() {
        let url = "https://app.zoom.us/wc/79642156509/";
        assert_eq!(parse(url), Some(("79642156509".into(), None)));
    }
    #[test]
    fn web_client_pma_url_with_password() {
        let url = "https://app.zoom.us/wc/79642156509/start?fromPWA=1&pwd=tJO3lY9HeH80y1mQw354RMsXzFilgW.1";
        assert_eq!(
            parse(url),
            Some((
                "79642156509".into(),
                Some("tJO3lY9HeH80y1mQw354RMsXzFilgW.1".into())
            ))
        );
    }
    #[test]
    fn web_client_pma_url_without_password() {
        let url = "https://app.zoom.us/wc/98110585089/start?fromPWA=1";
        assert_eq!(parse(url), Some(("98110585089".into(), None)));
    }

    #[test]
    fn should_parse_url_encoded_password_params() {
        let url = "https://zoom.us/j/5165671036?pwd%3DaHkyUy9xcjBDczlDY3NOSCtXMlhMQT09&sa=D&source=calendar";
        assert_eq!(
            parse(url),
            Some((
                "5165671036".into(),
                Some("aHkyUy9xcjBDczlDY3NOSCtXMlhMQT09".into())
            ))
        );
    }
    #[test]
    fn personal_meeting_room_url() {
        let url = "https://zoom.us/my/voelker.ai";
        assert_eq!(parse(url), Some(("voelker.ai".into(), None)));
    }
    #[test]
    fn personal_meeting_room_url_with_subdomain() {
        let url = "https://turing.zoom.us/my/marco.santos.turing";
        assert_eq!(parse(url), Some(("marco.santos.turing".into(), None)));
    }
    #[test]
    fn personal_meeting_room_url_with_password() {
        let url = "https://us06web.zoom.us/my/audiencelab?pwd=YMTT1l9sJNYChkBfhBnuST2nSJQsD6.1";
        assert_eq!(
            parse(url),
            Some(("audiencelab".into(), Some("YMTT1l9sJNYChkBfhBnuST2nSJQsD6.1".into())))
        );
    }
    // #[test]
    // fn should_parse_password_appended_without_parentheses() {
    //     let url = "https://us06web.zoom.us/j/3290230144?pwd=esnQHAW0JYGE3jUbNQjkTjZmeNs6FQ.1Passcode: 497810";
    //     assert_eq!(
    //         parse(url),
    //         Some((
    //             "3290230144".into(),
    //             Some("497810".into())
    //         ))
    //     );
    // }
}
