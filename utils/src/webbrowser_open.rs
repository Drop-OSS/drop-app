use log::warn;

pub fn webbrowser_open<T: AsRef<str>>(url: T) {
    if let Err(e) = webbrowser::open(url.as_ref()) {
        warn!(
            "Could not open web browser to url {} with error {}",
            url.as_ref(),
            e
        );
    };
}
