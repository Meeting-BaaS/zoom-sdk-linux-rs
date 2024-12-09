use std::collections::HashMap;

/// Parse Zoom meeting URL
pub fn parse(meeting_url: &str) -> Option<(String, String)> {
    match url::Url::parse(meeting_url) {
        // https://app.zoom.us/wc/81561946371/start?fromPWA=1&pwd=aBMNeaTbJOStQrPCRT2fBrviRTp15D.1
        Ok(parsed_url) => {
            let path_segments: Vec<&str> = parsed_url
                .path_segments()
                .map(|c| c.collect())
                .unwrap_or_default();
            let query_params: HashMap<_, _> = parsed_url.query_pairs().into_owned().collect();
            let id = path_segments.get(1).map(|s| s.to_string());
            let pwd = query_params.get("pwd");
            id.and_then(|id| pwd.and_then(|pwd| Some((id, pwd.clone()))))
                .or({
                    // https://us06web.zoom.us/j/85636588041 (Passcode: 718843)
                    let parts: Option<(&str, &str)> = meeting_url.split_once(' ');
                    let id = parts.and_then(|inner| inner.0.split('/').last());
                    let pwd = parts.and_then(|inner| {
                        inner
                            .1
                            .strip_prefix("(Passcode:")
                            .map(|s| s.trim_end_matches(')'))
                    });
                    id.and_then(|id| pwd.and_then(|pwd| Some((id.to_owned(), pwd.to_owned()))))
                })
        }
        Err(err) => {
            tracing::error!("Cannot parse url : {}", err);
            None
        }
    }
}
