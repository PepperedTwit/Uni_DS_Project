use pyo3::prelude::*;
use util::{Script, Show};

const SCRAPPER: Script = Script::new(r#"
    import re
    import requests
    import json
    from playwright.sync_api import Playwright, sync_playwright, expect
    from datetime import datetime, timedelta

    def get_html(url: str) -> tuple[str, int | None]:
        with sync_playwright() as p:
            browser = p.chromium.launch(headless=True)
            context = browser.new_context(user_agent="Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
            page = context.new_page()
            page.goto(url, timeout=60000)  # 60 second timeout
            page.get_by_role("link", name="Financials & Documents").click()

            curr_year = datetime.now().year
            fnd_year = None

            for year in range(curr_year, curr_year - 5, -1):
                selector = f'[title="View Annual Information Statement {year}"]'
                if page.is_visible(selector):
                    page.click(selector)
                    fnd_year = year
                    break

            if fnd_year is None:
                browser.close()
                return "No data found", fnd_year

            # Wait for the content to load with a timeout
            page.wait_for_selector('article.reports-ais', timeout=30000)  # 30 second timeout

            html = page.content()
            browser.close()
            return html, fnd_year

"#);

fn main() -> PyResult<()> {

    println!("Scrapper script: {}", SCRAPPER);

    Python::with_gil(|py| {

        // Create a Python module from our SCRAPPER script
        let scraper_module = PyModule::from_code(py, SCRAPPER.get(), "scraper.py", "scraper")?;

        // URLs to scrape
        let initial_url = "https://example.com";  // Replace with your target URL
        let data_url = "https://example.com/data";  // Replace with your data URL

        // Get cookies
        let get_cookie_fn = scraper_module.getattr("get_cookie_playwright")?;
        let cookies: Vec<PyObject> = get_cookie_fn.call1((initial_url,))?.extract()?;

        // Make request with cookies
        let req_with_cookie_fn = scraper_module.getattr("req_with_cookie")?;
        let result: String = req_with_cookie_fn.call1((cookies, data_url))?.extract()?;

        result.print();

        Ok(())

    })

}